#![forbid(unsafe_code)]

//! Hardened Linux x86_64 production-containment candidate for Spec 005 Phase G.
//!
//! The earlier `...-v1` candidate remains non-admitted evidence. This v2 identity closes
//! additional device/IPC/cross-process and filesystem-root binding gaps before T005-077. It is
//! still not a production executor and does not create or launch a process; T005-078 owns launch.

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod linux_x86_64 {
    use std::collections::{BTreeMap, BTreeSet};
    use std::error::Error;
    use std::fmt;
    use std::fs;
    use std::os::fd::AsFd;
    use std::path::{Path, PathBuf};

    use landlock::{
        ABI, Access, AccessFs, AccessNet, CompatLevel, Compatible, PathBeneath, PathFd,
        RestrictionStatus, Ruleset, RulesetAttr, RulesetCreatedAttr, RulesetStatus,
    };
    use nix::sys::resource::{Resource, setrlimit};
    use nix::sys::stat::fstat;
    use seccompiler::{
        BpfProgram, SeccompAction, SeccompFilter, SeccompRule, TargetArch, apply_filter_all_threads,
    };

    pub const PROFILE_TOKEN: &str = "platform:linux-x86_64-landlock-v4-seccomp-v2";
    const ABI_REQUIRED: ABI = ABI::V4;
    const MAX_ROOTS: usize = 64;
    const MAX_PATH_BYTES: usize = 4096;
    const MIN_MEMORY_BYTES: u64 = 16 * 1024 * 1024;
    const MAX_MEMORY_BYTES: u64 = 64 * 1024 * 1024 * 1024;
    const MAX_CPU_SECONDS: u64 = 24 * 60 * 60;
    const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024 * 1024;
    const MAX_OPEN_FILES: u64 = 4096;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct NativeObjectIdentity {
        pub canonical_path: PathBuf,
        pub device: u64,
        pub inode: u64,
        pub mode: u32,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct LinuxContainmentPlan {
        pub profile_token: String,
        pub executable: NativeObjectIdentity,
        pub cwd: NativeObjectIdentity,
        /// Exact identity-bound regular files readable by the payload.
        pub filesystem_read_roots: Vec<NativeObjectIdentity>,
        /// Exact identity-bound existing regular files writable by the payload.
        pub filesystem_write_roots: Vec<NativeObjectIdentity>,
        pub cpu_seconds: u64,
        pub address_space_bytes: u64,
        pub max_created_file_bytes: u64,
        pub max_open_files: u64,
        pub wall_time_ms: u64,
        pub max_stdout_stderr_bytes: u64,
        pub strict_local: bool,
        pub spawn_denied: bool,
        pub ambient_environment_cleared: bool,
        pub device_rules_empty: bool,
        pub ipc_rules_empty: bool,
        pub inherited_handle_rules_empty: bool,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct ChildContainmentReceipt {
        pub profile_token: &'static str,
        pub landlock_ruleset_fully_enforced: bool,
        pub no_new_privs: bool,
        pub seccomp_tsync_installed: bool,
        pub spawn_denied: bool,
        pub strict_local: bool,
        pub inherited_non_stdio_handles_observed: bool,
        pub linux_capability_sets_empty: bool,
        pub identity_bound_regular_file_roots: bool,
        pub ipc_creation_denied: bool,
        pub cross_process_control_denied: bool,
        pub credential_kernel_surfaces_denied: bool,
        pub wall_time_requires_parent_supervisor: bool,
        pub output_requires_parent_supervisor: bool,
    }

    #[derive(Debug)]
    pub enum NativeContainmentError {
        InvalidProfileToken,
        InvalidPlan(&'static str),
        InvalidPath(PathBuf),
        PathResolution {
            path: PathBuf,
            source: std::io::Error,
        },
        PathIdentityChanged(PathBuf),
        AmbientEnvironmentPresent,
        InheritedSocket(u32),
        InheritedHandle(u32),
        ProcFdInspection(std::io::Error),
        ProcStatusInspection(std::io::Error),
        InvalidCapabilityStatus(&'static str),
        Resource(nix::errno::Errno),
        Landlock(String),
        LandlockNotFullyEnforced,
        Seccomp(String),
    }

    impl fmt::Display for NativeContainmentError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidProfileToken => {
                    f.write_str("native containment v2 profile token mismatch")
                }
                Self::InvalidPlan(reason) => {
                    write!(f, "native containment v2 plan is invalid: {reason}")
                }
                Self::InvalidPath(path) => {
                    write!(
                        f,
                        "native containment v2 path is invalid: {}",
                        path.display()
                    )
                }
                Self::PathResolution { path, source } => write!(
                    f,
                    "native containment v2 path resolution failed for {}: {source}",
                    path.display()
                ),
                Self::PathIdentityChanged(path) => write!(
                    f,
                    "native containment v2 path identity changed before restriction: {}",
                    path.display()
                ),
                Self::AmbientEnvironmentPresent => f.write_str(
                    "native containment v2 helper requires a cleared ambient environment",
                ),
                Self::InheritedSocket(fd) => write!(
                    f,
                    "native containment v2 helper inherited a socket on standard descriptor {fd}"
                ),
                Self::InheritedHandle(fd) => write!(
                    f,
                    "native containment v2 helper inherited an undeclared descriptor {fd}"
                ),
                Self::ProcFdInspection(error) => write!(
                    f,
                    "native containment v2 descriptor inspection failed: {error}"
                ),
                Self::ProcStatusInspection(error) => write!(
                    f,
                    "native containment v2 capability status inspection failed: {error}"
                ),
                Self::InvalidCapabilityStatus(reason) => write!(
                    f,
                    "native containment v2 Linux capability posture is invalid: {reason}"
                ),
                Self::Resource(error) => {
                    write!(f, "native containment v2 resource limit failed: {error}")
                }
                Self::Landlock(error) => {
                    write!(f, "native containment v2 Landlock failure: {error}")
                }
                Self::LandlockNotFullyEnforced => {
                    f.write_str("native containment v2 Landlock ruleset was not fully enforced")
                }
                Self::Seccomp(error) => {
                    write!(f, "native containment v2 seccomp failure: {error}")
                }
            }
        }
    }

    impl Error for NativeContainmentError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::PathResolution { source, .. } => Some(source),
                Self::ProcFdInspection(error) | Self::ProcStatusInspection(error) => Some(error),
                _ => None,
            }
        }
    }

    pub fn observe_native_object(
        path: &Path,
    ) -> Result<NativeObjectIdentity, NativeContainmentError> {
        validate_absolute_path(path)?;
        let canonical_path =
            fs::canonicalize(path).map_err(|source| NativeContainmentError::PathResolution {
                path: path.to_owned(),
                source,
            })?;
        validate_absolute_path(&canonical_path)?;
        let descriptor = PathFd::new(&canonical_path).map_err(|error| {
            NativeContainmentError::Landlock(format!("open identity descriptor: {error}"))
        })?;
        let stat = fstat(descriptor.as_fd()).map_err(NativeContainmentError::Resource)?;
        Ok(NativeObjectIdentity {
            canonical_path,
            device: stat.st_dev,
            inode: stat.st_ino,
            mode: stat.st_mode,
        })
    }

    pub fn validate_plan(plan: &LinuxContainmentPlan) -> Result<(), NativeContainmentError> {
        if plan.profile_token != PROFILE_TOKEN {
            return Err(NativeContainmentError::InvalidProfileToken);
        }
        if !plan.strict_local {
            return Err(NativeContainmentError::InvalidPlan(
                "first admitted v2 profile is strict-local only",
            ));
        }
        if !plan.spawn_denied {
            return Err(NativeContainmentError::InvalidPlan(
                "first admitted v2 profile requires spawn_rule=DENY",
            ));
        }
        if !plan.ambient_environment_cleared {
            return Err(NativeContainmentError::InvalidPlan(
                "ambient environment must be cleared by the trusted launcher",
            ));
        }
        if !plan.device_rules_empty || !plan.ipc_rules_empty || !plan.inherited_handle_rules_empty {
            return Err(NativeContainmentError::InvalidPlan(
                "device, IPC and inherited-handle requests are unsupported by v2",
            ));
        }
        if plan.filesystem_read_roots.len() > MAX_ROOTS
            || plan.filesystem_write_roots.len() > MAX_ROOTS
        {
            return Err(NativeContainmentError::InvalidPlan(
                "filesystem root list exceeds the bounded v2 profile",
            ));
        }

        validate_identity(&plan.executable)?;
        require_kind(
            &plan.executable,
            libc::S_IFREG,
            "executable must be a regular file",
        )?;
        validate_identity(&plan.cwd)?;
        require_kind(&plan.cwd, libc::S_IFDIR, "cwd must be a directory")?;
        validate_regular_file_roots(&plan.filesystem_read_roots)?;
        validate_regular_file_roots(&plan.filesystem_write_roots)?;
        reject_read_write_overlap(plan)?;

        if plan.cpu_seconds == 0 || plan.cpu_seconds > MAX_CPU_SECONDS {
            return Err(NativeContainmentError::InvalidPlan("invalid CPU limit"));
        }
        if !(MIN_MEMORY_BYTES..=MAX_MEMORY_BYTES).contains(&plan.address_space_bytes) {
            return Err(NativeContainmentError::InvalidPlan(
                "invalid address-space limit",
            ));
        }
        if plan.max_created_file_bytes == 0 || plan.max_created_file_bytes > MAX_FILE_BYTES {
            return Err(NativeContainmentError::InvalidPlan(
                "invalid created-file limit",
            ));
        }
        if plan.max_open_files < 3 || plan.max_open_files > MAX_OPEN_FILES {
            return Err(NativeContainmentError::InvalidPlan(
                "invalid open-file limit",
            ));
        }
        if plan.wall_time_ms == 0 {
            return Err(NativeContainmentError::InvalidPlan(
                "wall-time limit must be finite",
            ));
        }
        if plan.max_stdout_stderr_bytes == 0 {
            return Err(NativeContainmentError::InvalidPlan(
                "stdout/stderr limit must be finite",
            ));
        }
        Ok(())
    }

    /// Apply child-side restrictions before the untrusted payload begins.
    ///
    /// The trusted launcher must already have called `env_clear()` and removed every inherited
    /// descriptor except admitted stdio. The caller remains responsible for parent-side wall/output
    /// supervision and for the later one-way payload exec transition at T005-078.
    pub fn apply_child_side(
        plan: &LinuxContainmentPlan,
    ) -> Result<ChildContainmentReceipt, NativeContainmentError> {
        validate_plan(plan)?;
        if std::env::vars_os().next().is_some() {
            return Err(NativeContainmentError::AmbientEnvironmentPresent);
        }
        verify_descriptor_hygiene()?;
        verify_capability_hygiene()?;

        revalidate_identity(&plan.executable)?;
        revalidate_identity(&plan.cwd)?;
        for root in &plan.filesystem_read_roots {
            revalidate_identity(root)?;
        }
        for root in &plan.filesystem_write_roots {
            revalidate_identity(root)?;
        }

        apply_resource_limits(plan)?;
        let status = apply_landlock(plan)?;
        if status.ruleset != RulesetStatus::FullyEnforced || !status.no_new_privs {
            return Err(NativeContainmentError::LandlockNotFullyEnforced);
        }
        install_seccomp_deny_filter()?;

        Ok(ChildContainmentReceipt {
            profile_token: PROFILE_TOKEN,
            landlock_ruleset_fully_enforced: true,
            no_new_privs: true,
            seccomp_tsync_installed: true,
            spawn_denied: true,
            strict_local: true,
            inherited_non_stdio_handles_observed: false,
            linux_capability_sets_empty: true,
            identity_bound_regular_file_roots: true,
            ipc_creation_denied: true,
            cross_process_control_denied: true,
            credential_kernel_surfaces_denied: true,
            wall_time_requires_parent_supervisor: true,
            output_requires_parent_supervisor: true,
        })
    }

    pub fn compile_seccomp_deny_filter() -> Result<BpfProgram, NativeContainmentError> {
        let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();
        for syscall in blocked_syscalls() {
            rules.insert(syscall, Vec::new());
        }
        let arch = TargetArch::try_from(std::env::consts::ARCH)
            .map_err(|error| NativeContainmentError::Seccomp(error.to_string()))?;
        let filter = SeccompFilter::new(
            rules,
            SeccompAction::Allow,
            SeccompAction::Errno(libc::EPERM as u32),
            arch,
        )
        .map_err(|error| NativeContainmentError::Seccomp(error.to_string()))?;
        filter
            .try_into()
            .map_err(|error: seccompiler::BackendError| {
                NativeContainmentError::Seccomp(error.to_string())
            })
    }

    fn install_seccomp_deny_filter() -> Result<(), NativeContainmentError> {
        let program = compile_seccomp_deny_filter()?;
        apply_filter_all_threads(&program)
            .map_err(|error| NativeContainmentError::Seccomp(error.to_string()))
    }

    fn blocked_syscalls() -> Vec<i64> {
        vec![
            // Network and local socket IPC.
            libc::SYS_socket,
            libc::SYS_socketpair,
            // Process/thread creation and cross-process control/observation.
            libc::SYS_clone,
            libc::SYS_clone3,
            libc::SYS_fork,
            libc::SYS_vfork,
            libc::SYS_ptrace,
            libc::SYS_process_vm_readv,
            libc::SYS_process_vm_writev,
            libc::SYS_pidfd_open,
            libc::SYS_pidfd_getfd,
            libc::SYS_pidfd_send_signal,
            libc::SYS_kill,
            libc::SYS_tkill,
            libc::SYS_tgkill,
            libc::SYS_kcmp,
            // SysV IPC.
            libc::SYS_msgget,
            libc::SYS_msgsnd,
            libc::SYS_msgrcv,
            libc::SYS_msgctl,
            libc::SYS_semget,
            libc::SYS_semop,
            libc::SYS_semtimedop,
            libc::SYS_semctl,
            libc::SYS_shmget,
            libc::SYS_shmat,
            libc::SYS_shmdt,
            libc::SYS_shmctl,
            // POSIX message queues and anonymous IPC primitives.
            libc::SYS_mq_open,
            libc::SYS_mq_unlink,
            libc::SYS_mq_timedsend,
            libc::SYS_mq_timedreceive,
            libc::SYS_mq_notify,
            libc::SYS_mq_getsetattr,
            libc::SYS_pipe,
            libc::SYS_pipe2,
            libc::SYS_eventfd,
            libc::SYS_eventfd2,
            libc::SYS_signalfd,
            libc::SYS_signalfd4,
            libc::SYS_memfd_create,
            // Namespace/mount escape and filesystem-monitor bypass surfaces.
            libc::SYS_mount,
            libc::SYS_umount2,
            libc::SYS_unshare,
            libc::SYS_setns,
            libc::SYS_open_by_handle_at,
            libc::SYS_inotify_init,
            libc::SYS_inotify_init1,
            libc::SYS_fanotify_init,
            libc::SYS_fanotify_mark,
            libc::SYS_mknod,
            libc::SYS_mknodat,
            // Ambient credential/kernel instrumentation surfaces not admitted by this profile.
            libc::SYS_keyctl,
            libc::SYS_add_key,
            libc::SYS_request_key,
            libc::SYS_bpf,
            libc::SYS_perf_event_open,
            libc::SYS_userfaultfd,
            libc::SYS_io_uring_setup,
            libc::SYS_io_uring_enter,
            libc::SYS_io_uring_register,
        ]
    }

    fn apply_resource_limits(plan: &LinuxContainmentPlan) -> Result<(), NativeContainmentError> {
        set_exact_limit(Resource::RLIMIT_CORE, 0)?;
        set_exact_limit(Resource::RLIMIT_CPU, plan.cpu_seconds)?;
        set_exact_limit(Resource::RLIMIT_AS, plan.address_space_bytes)?;
        set_exact_limit(Resource::RLIMIT_FSIZE, plan.max_created_file_bytes)?;
        set_exact_limit(Resource::RLIMIT_NOFILE, plan.max_open_files)?;
        Ok(())
    }

    fn set_exact_limit(resource: Resource, value: u64) -> Result<(), NativeContainmentError> {
        let value = libc::rlim_t::try_from(value).map_err(|_| {
            NativeContainmentError::InvalidPlan("resource limit does not fit platform rlim_t")
        })?;
        setrlimit(resource, value, value).map_err(NativeContainmentError::Resource)
    }

    fn apply_landlock(
        plan: &LinuxContainmentPlan,
    ) -> Result<RestrictionStatus, NativeContainmentError> {
        let fs_handled = AccessFs::from_all(ABI_REQUIRED);
        let net_handled = AccessNet::from_all(ABI_REQUIRED);
        let mut ruleset = Ruleset::default()
            .set_compatibility(CompatLevel::HardRequirement)
            .handle_access(fs_handled)
            .map_err(|error| NativeContainmentError::Landlock(error.to_string()))?
            .handle_access(net_handled)
            .map_err(|error| NativeContainmentError::Landlock(error.to_string()))?
            .create()
            .map_err(|error| NativeContainmentError::Landlock(error.to_string()))?
            .no_new_privs(true);

        let read_access = AccessFs::ReadFile;
        let write_access = AccessFs::ReadFile | AccessFs::WriteFile | AccessFs::Truncate;
        let executable_access = AccessFs::Execute | AccessFs::ReadFile;

        for root in &plan.filesystem_read_roots {
            ruleset = add_path_rule(ruleset, &root.canonical_path, read_access)?;
        }
        for root in &plan.filesystem_write_roots {
            ruleset = add_path_rule(ruleset, &root.canonical_path, write_access)?;
        }
        ruleset = add_path_rule(ruleset, &plan.executable.canonical_path, executable_access)?;
        ruleset = add_path_rule(ruleset, &plan.cwd.canonical_path, AccessFs::ReadDir)?;

        ruleset
            .restrict_self()
            .map_err(|error| NativeContainmentError::Landlock(error.to_string()))
    }

    fn add_path_rule<A>(
        ruleset: landlock::RulesetCreated,
        path: &Path,
        access: A,
    ) -> Result<landlock::RulesetCreated, NativeContainmentError>
    where
        A: Into<landlock::BitFlags<AccessFs>>,
    {
        let descriptor = PathFd::new(path)
            .map_err(|error| NativeContainmentError::Landlock(error.to_string()))?;
        ruleset
            .set_compatibility(CompatLevel::HardRequirement)
            .add_rule(
                PathBeneath::new(descriptor, access)
                    .set_compatibility(CompatLevel::HardRequirement),
            )
            .map_err(|error| NativeContainmentError::Landlock(error.to_string()))
    }

    fn revalidate_identity(identity: &NativeObjectIdentity) -> Result<(), NativeContainmentError> {
        let descriptor = PathFd::new(&identity.canonical_path)
            .map_err(|error| NativeContainmentError::Landlock(error.to_string()))?;
        let stat = fstat(descriptor.as_fd()).map_err(NativeContainmentError::Resource)?;
        if stat.st_dev != identity.device
            || stat.st_ino != identity.inode
            || stat.st_mode != identity.mode
        {
            return Err(NativeContainmentError::PathIdentityChanged(
                identity.canonical_path.clone(),
            ));
        }
        Ok(())
    }

    fn validate_identity(identity: &NativeObjectIdentity) -> Result<(), NativeContainmentError> {
        validate_absolute_path(&identity.canonical_path)?;
        if identity.device == 0 && identity.inode == 0 {
            return Err(NativeContainmentError::InvalidPlan(
                "native object identity is empty",
            ));
        }
        let canonical = fs::canonicalize(&identity.canonical_path).map_err(|source| {
            NativeContainmentError::PathResolution {
                path: identity.canonical_path.clone(),
                source,
            }
        })?;
        if canonical != identity.canonical_path {
            return Err(NativeContainmentError::InvalidPlan(
                "native object identity path must be pre-canonicalized",
            ));
        }
        Ok(())
    }

    fn require_kind(
        identity: &NativeObjectIdentity,
        kind: u32,
        reason: &'static str,
    ) -> Result<(), NativeContainmentError> {
        if identity.mode & libc::S_IFMT != kind {
            return Err(NativeContainmentError::InvalidPlan(reason));
        }
        Ok(())
    }

    fn validate_regular_file_roots(
        roots: &[NativeObjectIdentity],
    ) -> Result<(), NativeContainmentError> {
        let mut seen = BTreeSet::new();
        for root in roots {
            validate_identity(root)?;
            require_kind(
                root,
                libc::S_IFREG,
                "v2 filesystem roots must be exact existing regular files",
            )?;
            if !seen.insert(root.canonical_path.clone()) {
                return Err(NativeContainmentError::InvalidPlan(
                    "v2 filesystem roots contain a duplicate identity",
                ));
            }
        }
        Ok(())
    }

    fn reject_read_write_overlap(
        plan: &LinuxContainmentPlan,
    ) -> Result<(), NativeContainmentError> {
        let reads = plan
            .filesystem_read_roots
            .iter()
            .map(|root| &root.canonical_path)
            .collect::<BTreeSet<_>>();
        if plan
            .filesystem_write_roots
            .iter()
            .any(|root| reads.contains(&root.canonical_path))
        {
            return Err(NativeContainmentError::InvalidPlan(
                "v2 read/write roots must be disjoint to preserve exact rights",
            ));
        }
        Ok(())
    }

    fn validate_absolute_path(path: &Path) -> Result<(), NativeContainmentError> {
        if !path.is_absolute()
            || path.as_os_str().as_encoded_bytes().is_empty()
            || path.as_os_str().as_encoded_bytes().len() > MAX_PATH_BYTES
        {
            return Err(NativeContainmentError::InvalidPath(path.to_owned()));
        }
        Ok(())
    }

    fn verify_capability_hygiene() -> Result<(), NativeContainmentError> {
        let status = fs::read_to_string("/proc/self/status")
            .map_err(NativeContainmentError::ProcStatusInspection)?;
        validate_capability_status(&status)
    }

    fn validate_capability_status(status: &str) -> Result<(), NativeContainmentError> {
        for field in ["CapInh", "CapPrm", "CapEff", "CapAmb"] {
            let prefix = format!("{field}:");
            let value = status
                .lines()
                .find_map(|line| line.strip_prefix(&prefix))
                .ok_or(NativeContainmentError::InvalidCapabilityStatus(
                    "required capability field is missing",
                ))?
                .trim();
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(NativeContainmentError::InvalidCapabilityStatus(
                    "capability field is not canonical hexadecimal",
                ));
            }
            if value.bytes().any(|byte| byte != b'0') {
                return Err(NativeContainmentError::InvalidCapabilityStatus(
                    "inherited, permitted, effective and ambient capability sets must be empty",
                ));
            }
        }
        Ok(())
    }

    fn verify_descriptor_hygiene() -> Result<(), NativeContainmentError> {
        for fd in 0_u32..=2 {
            let target = fs::read_link(format!("/proc/self/fd/{fd}"))
                .map_err(NativeContainmentError::ProcFdInspection)?;
            if target.to_string_lossy().starts_with("socket:") {
                return Err(NativeContainmentError::InheritedSocket(fd));
            }
        }

        let entries = fs::read_dir("/proc/self/fd")
            .map_err(NativeContainmentError::ProcFdInspection)?
            .map(|entry| {
                entry
                    .map_err(NativeContainmentError::ProcFdInspection)
                    .and_then(|entry| {
                        entry
                            .file_name()
                            .to_string_lossy()
                            .parse::<u32>()
                            .map_err(|_| {
                                NativeContainmentError::InvalidPlan(
                                    "non-numeric /proc/self/fd entry",
                                )
                            })
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        for fd in entries.into_iter().filter(|fd| *fd > 2) {
            if fs::read_link(format!("/proc/self/fd/{fd}")).is_ok() {
                return Err(NativeContainmentError::InheritedHandle(fd));
            }
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn fixture_identity(path: &Path) -> NativeObjectIdentity {
            observe_native_object(path).unwrap()
        }

        fn fixture_plan() -> LinuxContainmentPlan {
            LinuxContainmentPlan {
                profile_token: PROFILE_TOKEN.to_owned(),
                executable: fixture_identity(Path::new("/bin/sh")),
                cwd: fixture_identity(Path::new("/tmp")),
                filesystem_read_roots: vec![],
                filesystem_write_roots: vec![],
                cpu_seconds: 30,
                address_space_bytes: 512 * 1024 * 1024,
                max_created_file_bytes: 64 * 1024 * 1024,
                max_open_files: 64,
                wall_time_ms: 30_000,
                max_stdout_stderr_bytes: 4 * 1024 * 1024,
                strict_local: true,
                spawn_denied: true,
                ambient_environment_cleared: true,
                device_rules_empty: true,
                ipc_rules_empty: true,
                inherited_handle_rules_empty: true,
            }
        }

        #[test]
        fn v2_profile_is_exact_and_fail_closed() {
            let plan = fixture_plan();
            validate_plan(&plan).unwrap();

            let mut widened = plan.clone();
            widened.spawn_denied = false;
            assert!(matches!(
                validate_plan(&widened),
                Err(NativeContainmentError::InvalidPlan(_))
            ));

            let mut remote = plan.clone();
            remote.strict_local = false;
            assert!(matches!(
                validate_plan(&remote),
                Err(NativeContainmentError::InvalidPlan(_))
            ));
        }

        #[test]
        fn capability_status_requires_empty_inherited_permitted_effective_and_ambient_sets() {
            let empty = "CapInh:\t0000000000000000\nCapPrm:\t0000000000000000\nCapEff:\t0000000000000000\nCapAmb:\t0000000000000000\n";
            validate_capability_status(empty).unwrap();

            let nonzero = "CapInh:\t0000000000000000\nCapPrm:\t0000000000000000\nCapEff:\t0000000000000400\nCapAmb:\t0000000000000000\n";
            assert!(matches!(
                validate_capability_status(nonzero),
                Err(NativeContainmentError::InvalidCapabilityStatus(_))
            ));

            let missing =
                "CapInh:\t0000000000000000\nCapPrm:\t0000000000000000\nCapEff:\t0000000000000000\n";
            assert!(matches!(
                validate_capability_status(missing),
                Err(NativeContainmentError::InvalidCapabilityStatus(_))
            ));
        }

        #[test]
        fn device_directory_and_other_special_roots_are_rejected() {
            let mut plan = fixture_plan();
            plan.filesystem_read_roots = vec![fixture_identity(Path::new("/dev/null"))];
            assert!(matches!(
                validate_plan(&plan),
                Err(NativeContainmentError::InvalidPlan(
                    "v2 filesystem roots must be exact existing regular files"
                ))
            ));

            let mut plan = fixture_plan();
            plan.filesystem_read_roots = vec![fixture_identity(Path::new("/tmp"))];
            assert!(matches!(
                validate_plan(&plan),
                Err(NativeContainmentError::InvalidPlan(
                    "v2 filesystem roots must be exact existing regular files"
                ))
            ));
        }

        #[test]
        fn root_identity_mode_and_path_are_bound() {
            let mut plan = fixture_plan();
            plan.executable.mode ^= 1;
            assert!(matches!(
                validate_plan(&plan),
                Err(NativeContainmentError::InvalidPlan(_))
            ));
        }

        #[test]
        fn seccomp_v2_binds_network_spawn_ipc_cross_process_and_credential_denials() {
            let program = compile_seccomp_deny_filter().unwrap();
            assert!(!program.is_empty());
            assert!(program.len() <= 4096);
            let blocked = blocked_syscalls();
            let unique = blocked.iter().copied().collect::<BTreeSet<_>>();
            assert_eq!(unique.len(), blocked.len());
            for syscall in [
                libc::SYS_socket,
                libc::SYS_socketpair,
                libc::SYS_clone,
                libc::SYS_fork,
                libc::SYS_msgget,
                libc::SYS_semget,
                libc::SYS_shmget,
                libc::SYS_mq_open,
                libc::SYS_process_vm_readv,
                libc::SYS_pidfd_getfd,
                libc::SYS_kill,
                libc::SYS_keyctl,
                libc::SYS_bpf,
                libc::SYS_open_by_handle_at,
                libc::SYS_mknodat,
            ] {
                assert!(blocked.contains(&syscall));
            }
        }

        #[test]
        fn nonempty_device_ipc_or_handle_authority_is_rejected() {
            for mutation in 0..3 {
                let mut plan = fixture_plan();
                match mutation {
                    0 => plan.device_rules_empty = false,
                    1 => plan.ipc_rules_empty = false,
                    _ => plan.inherited_handle_rules_empty = false,
                }
                assert!(matches!(
                    validate_plan(&plan),
                    Err(NativeContainmentError::InvalidPlan(_))
                ));
            }
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub use linux_x86_64::*;

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
mod unsupported {
    use std::error::Error;
    use std::fmt;

    pub const PROFILE_TOKEN: &str = "platform:linux-x86_64-landlock-v4-seccomp-v2";

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct NativeContainmentError;

    impl fmt::Display for NativeContainmentError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("native containment v2 is unsupported outside Linux x86_64")
        }
    }

    impl Error for NativeContainmentError {}

    pub const fn production_profile_available() -> bool {
        false
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
pub use unsupported::*;
