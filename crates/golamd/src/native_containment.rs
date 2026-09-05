#![forbid(unsafe_code)]

//! Linux x86_64 production-containment candidate for Spec 005 Phase G.
//!
//! This module is deliberately not wired into the production executor manifest yet. T005-077 is
//! the admission gate. Until that gate passes, `native:unqualified` remains the only production
//! capability manifest and no caller can launch through this candidate.

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

    pub const PROFILE_TOKEN: &str = "platform:linux-x86_64-landlock-v4-seccomp-v1";
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
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct LinuxContainmentPlan {
        pub profile_token: String,
        pub executable: NativeObjectIdentity,
        pub cwd: NativeObjectIdentity,
        pub filesystem_read_roots: Vec<PathBuf>,
        pub filesystem_write_roots: Vec<PathBuf>,
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
        Resource(nix::errno::Errno),
        Landlock(String),
        LandlockNotFullyEnforced,
        Seccomp(String),
        UnsupportedPlatform,
    }

    impl fmt::Display for NativeContainmentError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidProfileToken => {
                    f.write_str("native containment profile token mismatch")
                }
                Self::InvalidPlan(reason) => {
                    write!(f, "native containment plan is invalid: {reason}")
                }
                Self::InvalidPath(path) => {
                    write!(f, "native containment path is invalid: {}", path.display())
                }
                Self::PathResolution { path, source } => write!(
                    f,
                    "native containment path resolution failed for {}: {source}",
                    path.display()
                ),
                Self::PathIdentityChanged(path) => write!(
                    f,
                    "native containment path identity changed before restriction: {}",
                    path.display()
                ),
                Self::AmbientEnvironmentPresent => {
                    f.write_str("native containment helper requires a cleared ambient environment")
                }
                Self::InheritedSocket(fd) => write!(
                    f,
                    "native containment helper inherited a socket on standard descriptor {fd}"
                ),
                Self::InheritedHandle(fd) => write!(
                    f,
                    "native containment helper inherited an undeclared descriptor {fd}"
                ),
                Self::ProcFdInspection(error) => {
                    write!(
                        f,
                        "native containment descriptor inspection failed: {error}"
                    )
                }
                Self::Resource(error) => {
                    write!(f, "native containment resource limit failed: {error}")
                }
                Self::Landlock(error) => write!(f, "native containment Landlock failure: {error}"),
                Self::LandlockNotFullyEnforced => {
                    f.write_str("native containment Landlock ruleset was not fully enforced")
                }
                Self::Seccomp(error) => write!(f, "native containment seccomp failure: {error}"),
                Self::UnsupportedPlatform => f.write_str(
                    "production native containment candidate is supported only on Linux x86_64",
                ),
            }
        }
    }

    impl Error for NativeContainmentError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::PathResolution { source, .. } => Some(source),
                Self::ProcFdInspection(error) => Some(error),
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
        })
    }

    pub fn validate_plan(plan: &LinuxContainmentPlan) -> Result<(), NativeContainmentError> {
        if plan.profile_token != PROFILE_TOKEN {
            return Err(NativeContainmentError::InvalidProfileToken);
        }
        if !plan.strict_local {
            return Err(NativeContainmentError::InvalidPlan(
                "first production profile is strict-local only",
            ));
        }
        if !plan.spawn_denied {
            return Err(NativeContainmentError::InvalidPlan(
                "first production profile requires spawn_rule=DENY",
            ));
        }
        if !plan.ambient_environment_cleared {
            return Err(NativeContainmentError::InvalidPlan(
                "ambient environment must be cleared by the trusted launcher",
            ));
        }
        if !plan.device_rules_empty || !plan.ipc_rules_empty || !plan.inherited_handle_rules_empty {
            return Err(NativeContainmentError::InvalidPlan(
                "device, IPC and inherited-handle requests are unsupported by the first profile",
            ));
        }
        if plan.filesystem_read_roots.len() > MAX_ROOTS
            || plan.filesystem_write_roots.len() > MAX_ROOTS
        {
            return Err(NativeContainmentError::InvalidPlan(
                "filesystem root list exceeds the bounded profile",
            ));
        }
        validate_identity(&plan.executable)?;
        validate_identity(&plan.cwd)?;
        validate_roots(&plan.filesystem_read_roots)?;
        validate_roots(&plan.filesystem_write_roots)?;
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

    /// Apply the child-side portion of the first production containment candidate.
    ///
    /// This function must run inside a trusted, single-purpose helper that was started with
    /// `Command::env_clear()` and no inherited descriptors other than admitted stdio. It does not
    /// launch the untrusted payload. T005-078 owns the governed launch integration.
    pub fn apply_child_side(
        plan: &LinuxContainmentPlan,
    ) -> Result<ChildContainmentReceipt, NativeContainmentError> {
        validate_plan(plan)?;
        if std::env::vars_os().next().is_some() {
            return Err(NativeContainmentError::AmbientEnvironmentPresent);
        }
        verify_descriptor_hygiene()?;
        revalidate_identity(&plan.executable)?;
        revalidate_identity(&plan.cwd)?;

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

    fn blocked_syscalls() -> [i64; 11] {
        [
            libc::SYS_socket,
            libc::SYS_socketpair,
            libc::SYS_clone,
            libc::SYS_clone3,
            libc::SYS_fork,
            libc::SYS_vfork,
            libc::SYS_ptrace,
            libc::SYS_mount,
            libc::SYS_umount2,
            libc::SYS_unshare,
            libc::SYS_setns,
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

        let read_access = AccessFs::ReadFile | AccessFs::ReadDir;
        let write_access = AccessFs::ReadFile
            | AccessFs::ReadDir
            | AccessFs::WriteFile
            | AccessFs::RemoveDir
            | AccessFs::RemoveFile
            | AccessFs::MakeDir
            | AccessFs::MakeReg
            | AccessFs::MakeSym
            | AccessFs::Refer
            | AccessFs::Truncate;
        let executable_access = AccessFs::Execute | AccessFs::ReadFile;

        for root in &plan.filesystem_read_roots {
            ruleset = add_path_rule(ruleset, root, read_access)?;
        }
        for root in &plan.filesystem_write_roots {
            ruleset = add_path_rule(ruleset, root, write_access)?;
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
        if stat.st_dev != identity.device || stat.st_ino != identity.inode {
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
        Ok(())
    }

    fn validate_roots(roots: &[PathBuf]) -> Result<(), NativeContainmentError> {
        let mut seen = BTreeSet::new();
        for root in roots {
            validate_absolute_path(root)?;
            let canonical = fs::canonicalize(root).map_err(|source| {
                NativeContainmentError::PathResolution {
                    path: root.clone(),
                    source,
                }
            })?;
            if canonical != *root {
                return Err(NativeContainmentError::InvalidPlan(
                    "filesystem roots must be pre-canonicalized",
                ));
            }
            if !seen.insert(root.clone()) {
                return Err(NativeContainmentError::InvalidPlan(
                    "filesystem roots contain a duplicate",
                ));
            }
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

        // The read_dir descriptor is closed before this second pass. Any descriptor that remains
        // visible is inherited authority and is rejected.
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
                filesystem_read_roots: vec![fs::canonicalize("/usr").unwrap()],
                filesystem_write_roots: vec![fs::canonicalize("/tmp").unwrap()],
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
        fn candidate_profile_is_exact_and_fail_closed() {
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
        fn nonempty_ambient_device_ipc_or_handle_authority_is_rejected() {
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

        #[test]
        fn seccomp_candidate_compiles_and_binds_spawn_and_socket_denials() {
            let program = compile_seccomp_deny_filter().unwrap();
            assert!(!program.is_empty());
            assert!(program.len() <= 4096);
            let blocked = blocked_syscalls();
            assert!(blocked.contains(&libc::SYS_socket));
            assert!(blocked.contains(&libc::SYS_socketpair));
            assert!(blocked.contains(&libc::SYS_clone));
            assert!(blocked.contains(&libc::SYS_clone3));
            assert!(blocked.contains(&libc::SYS_fork));
            assert!(blocked.contains(&libc::SYS_vfork));
        }

        #[test]
        fn resource_limits_must_be_finite_and_bounded() {
            let mut plan = fixture_plan();
            plan.cpu_seconds = 0;
            assert!(matches!(
                validate_plan(&plan),
                Err(NativeContainmentError::InvalidPlan("invalid CPU limit"))
            ));

            let mut plan = fixture_plan();
            plan.wall_time_ms = 0;
            assert!(matches!(
                validate_plan(&plan),
                Err(NativeContainmentError::InvalidPlan(
                    "wall-time limit must be finite"
                ))
            ));
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub use linux_x86_64::*;

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
mod unsupported {
    use std::error::Error;
    use std::fmt;

    pub const PROFILE_TOKEN: &str = "platform:linux-x86_64-landlock-v4-seccomp-v1";

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct NativeContainmentError;

    impl fmt::Display for NativeContainmentError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("production native containment candidate is unsupported on this platform")
        }
    }

    impl Error for NativeContainmentError {}

    pub fn production_profile_available() -> bool {
        false
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
pub use unsupported::*;
