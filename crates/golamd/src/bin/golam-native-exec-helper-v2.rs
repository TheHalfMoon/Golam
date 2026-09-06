#![forbid(unsafe_code)]

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(dead_code)]
#[path = "../native_containment_v2.rs"]
mod native_containment_v2;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(dead_code)]
#[path = "../static_elf_v2.rs"]
mod static_elf_v2;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod linux_x86_64 {
    use std::ffi::{CString, OsString};
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::MetadataExt;
    use std::path::PathBuf;

    use golam_core::digest::sha256;
    use nix::errno::Errno;
    use nix::sys::prctl::{get_pdeathsig, set_pdeathsig};
    use nix::sys::signal::Signal;
    use nix::unistd::{close, fexecve, geteuid, getppid};

    use super::native_containment_v2::{
        LinuxContainmentPlan, NativeObjectIdentity, PROFILE_TOKEN, apply_child_side,
        observe_native_object,
    };
    use super::static_elf_v2::{MAX_STATIC_EXECUTABLE_BYTES, validate_static_elf_v2};

    const STAGED_PERMISSION_BITS: u32 = 0o500;
    const MAX_ARG_COUNT: usize = 256;
    const MAX_ARG_BYTES: usize = 64 * 1024;
    const MAX_ROOT_COUNT: usize = 64;
    const READY_PREFIX: &str = "GOLAM_NATIVE_EXEC_V2_READY:";

    #[derive(Debug)]
    struct ExpectedObject {
        path: PathBuf,
        device: u64,
        inode: u64,
        mode: u32,
    }

    #[derive(Debug)]
    struct Config {
        expected_parent_pid: u32,
        execution_binding_digest: [u8; 32],
        executable_path: PathBuf,
        executable_device: u64,
        executable_inode: u64,
        executable_mode: u32,
        executable_digest: [u8; 32],
        cwd: ExpectedObject,
        read_roots: Vec<ExpectedObject>,
        write_roots: Vec<ExpectedObject>,
        cpu_seconds: u64,
        address_space_bytes: u64,
        max_created_file_bytes: u64,
        max_open_files: u64,
        wall_time_ms: u64,
        max_output_bytes: u64,
        argv: Vec<Vec<u8>>,
    }

    pub fn run() -> Result<(), String> {
        if std::env::vars_os().next().is_some() {
            return Err(
                "trusted native exec helper requires an empty ambient environment".to_owned(),
            );
        }
        close_inherited_descriptors()?;
        let config = parse_args()?;
        bind_parent_death(config.expected_parent_pid)?;

        let initial_file = File::open(&config.executable_path)
            .map_err(|error| format!("open staged executable before containment: {error}"))?;
        verify_staged_file(&initial_file, &config)?;
        drop(initial_file);

        let cwd = observe_expected_object(&config.cwd, "cwd")?;
        let executable = expected_executable_identity(&config);
        let filesystem_read_roots = config
            .read_roots
            .iter()
            .map(|expected| observe_expected_object(expected, "read root"))
            .collect::<Result<Vec<_>, _>>()?;
        let filesystem_write_roots = config
            .write_roots
            .iter()
            .map(|expected| observe_expected_object(expected, "write root"))
            .collect::<Result<Vec<_>, _>>()?;

        std::env::set_current_dir(&config.cwd.path)
            .map_err(|error| format!("set exact contained cwd: {error}"))?;
        let plan = LinuxContainmentPlan {
            profile_token: PROFILE_TOKEN.to_owned(),
            executable,
            cwd,
            filesystem_read_roots,
            filesystem_write_roots,
            cpu_seconds: config.cpu_seconds,
            address_space_bytes: config.address_space_bytes,
            max_created_file_bytes: config.max_created_file_bytes,
            max_open_files: config.max_open_files,
            wall_time_ms: config.wall_time_ms,
            max_stdout_stderr_bytes: config.max_output_bytes,
            strict_local: true,
            spawn_denied: true,
            ambient_environment_cleared: true,
            device_rules_empty: true,
            ipc_rules_empty: true,
            inherited_handle_rules_empty: true,
        };
        let receipt = apply_child_side(&plan)
            .map_err(|error| format!("apply admitted containment profile: {error}"))?;
        if receipt.profile_token != PROFILE_TOKEN
            || !receipt.landlock_ruleset_fully_enforced
            || !receipt.no_new_privs
            || !receipt.seccomp_tsync_installed
            || !receipt.spawn_denied
            || !receipt.strict_local
            || receipt.inherited_non_stdio_handles_observed
            || !receipt.linux_capability_sets_empty
            || !receipt.identity_bound_regular_file_roots
            || !receipt.ipc_creation_denied
            || !receipt.cross_process_control_denied
            || !receipt.credential_kernel_surfaces_denied
        {
            return Err("containment receipt does not match the admitted v2 profile".to_owned());
        }

        // Open only after descriptor-hygiene qualification has run. Landlock now permits the
        // exact staged executable object; the same descriptor is verified and passed to fexecve.
        let executable_file = File::open(&config.executable_path)
            .map_err(|error| format!("reopen staged executable after containment: {error}"))?;
        verify_staged_file(&executable_file, &config)?;
        bind_parent_death(config.expected_parent_pid)?;
        emit_ready(config.execution_binding_digest)?;

        let argv = config
            .argv
            .iter()
            .map(|value| CString::new(value.as_slice()).map_err(|_| "argv contains NUL".to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        if argv.is_empty() {
            return Err("fexecve argv must contain argv[0]".to_owned());
        }
        let environment: [CString; 0] = [];
        match fexecve(&executable_file, &argv, &environment) {
            Ok(never) => match never {},
            Err(error) => Err(format!("descriptor-bound fexecve failed: {error}")),
        }
    }

    fn close_inherited_descriptors() -> Result<(), String> {
        let entries = fs::read_dir("/proc/self/fd").map_err(|error| {
            format!("inspect inherited descriptors before containment: {error}")
        })?;
        let mut descriptors = entries
            .map(|entry| {
                entry
                    .map_err(|error| format!("read inherited descriptor entry: {error}"))
                    .and_then(|entry| {
                        entry
                            .file_name()
                            .to_string_lossy()
                            .parse::<i32>()
                            .map_err(|_| "non-numeric /proc/self/fd entry".to_owned())
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        descriptors.sort_unstable();
        descriptors.dedup();
        for fd in descriptors.into_iter().filter(|fd| *fd > 2) {
            match close(fd) {
                Ok(()) | Err(Errno::EBADF) => {}
                Err(error) => {
                    return Err(format!("close inherited descriptor {fd}: {error}"));
                }
            }
        }
        Ok(())
    }

    fn bind_parent_death(expected_parent_pid: u32) -> Result<(), String> {
        if expected_parent_pid == 0 {
            return Err("expected launcher parent pid must be nonzero".to_owned());
        }
        set_pdeathsig(Signal::SIGKILL).map_err(|error| format!("set PR_SET_PDEATHSIG: {error}"))?;
        if get_pdeathsig().map_err(|error| format!("read PR_GET_PDEATHSIG: {error}"))?
            != Some(Signal::SIGKILL)
        {
            return Err("parent-death signal readback did not equal SIGKILL".to_owned());
        }
        let observed_parent = getppid().as_raw();
        if observed_parent <= 0 || u32::try_from(observed_parent).ok() != Some(expected_parent_pid)
        {
            return Err(format!(
                "trusted launcher parent changed: expected {expected_parent_pid}, observed {observed_parent}"
            ));
        }
        Ok(())
    }

    fn observe_expected_object(
        expected: &ExpectedObject,
        label: &str,
    ) -> Result<NativeObjectIdentity, String> {
        let observed = observe_native_object(&expected.path)
            .map_err(|error| format!("observe {label} identity: {error}"))?;
        if observed.canonical_path != expected.path
            || observed.device != expected.device
            || observed.inode != expected.inode
            || observed.mode != expected.mode
        {
            return Err(format!("{label} identity changed before containment"));
        }
        Ok(observed)
    }

    fn expected_executable_identity(config: &Config) -> NativeObjectIdentity {
        NativeObjectIdentity {
            canonical_path: config.executable_path.clone(),
            device: config.executable_device,
            inode: config.executable_inode,
            mode: config.executable_mode,
        }
    }

    fn verify_staged_file(file: &File, config: &Config) -> Result<(), String> {
        let path_metadata = std::fs::symlink_metadata(&config.executable_path)
            .map_err(|error| format!("read staged executable path metadata: {error}"))?;
        if path_metadata.file_type().is_symlink() {
            return Err("staged executable path must not be a symlink".to_owned());
        }
        let metadata = file
            .metadata()
            .map_err(|error| format!("read staged executable metadata: {error}"))?;
        if !metadata.is_file()
            || metadata.dev() != config.executable_device
            || metadata.ino() != config.executable_inode
            || metadata.mode() != config.executable_mode
        {
            return Err("staged executable descriptor identity changed".to_owned());
        }
        if metadata.mode() & 0o7777 != STAGED_PERMISSION_BITS {
            return Err("staged executable permissions must be exactly 0500".to_owned());
        }
        if metadata.uid() != geteuid().as_raw() {
            return Err("staged executable is not owned by the current effective uid".to_owned());
        }
        if usize::try_from(metadata.len())
            .ok()
            .is_none_or(|len| len > MAX_STATIC_EXECUTABLE_BYTES)
        {
            return Err("staged executable exceeds the admitted byte bound".to_owned());
        }
        let mut reader = file
            .try_clone()
            .map_err(|error| format!("clone staged executable descriptor: {error}"))?;
        let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
        std::io::Read::by_ref(&mut reader)
            .take((MAX_STATIC_EXECUTABLE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| format!("read staged executable: {error}"))?;
        if bytes.len() > MAX_STATIC_EXECUTABLE_BYTES || sha256(&bytes) != config.executable_digest {
            return Err("staged executable content digest changed".to_owned());
        }
        validate_static_elf_v2(&bytes)
            .map_err(|error| format!("staged executable class rejected: {error}"))?;
        Ok(())
    }

    fn emit_ready(binding_digest: [u8; 32]) -> Result<(), String> {
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{READY_PREFIX}{}", encode_hex(&binding_digest))
            .map_err(|error| format!("write containment-ready receipt: {error}"))?;
        stderr
            .flush()
            .map_err(|error| format!("flush containment-ready receipt: {error}"))
    }

    fn parse_args() -> Result<Config, String> {
        let mut args = std::env::args_os().skip(1).peekable();
        let mut expected_parent_pid = None;
        let mut execution_binding_digest = None;
        let mut executable_path = None;
        let mut executable_device = None;
        let mut executable_inode = None;
        let mut executable_mode = None;
        let mut executable_digest = None;
        let mut cwd_path = None;
        let mut cwd_device = None;
        let mut cwd_inode = None;
        let mut cwd_mode = None;
        let mut read_roots = Vec::new();
        let mut write_roots = Vec::new();
        let mut cpu_seconds = None;
        let mut address_space_bytes = None;
        let mut max_created_file_bytes = None;
        let mut max_open_files = None;
        let mut wall_time_ms = None;
        let mut max_output_bytes = None;
        let mut argv = Vec::new();

        while let Some(flag) = args.next() {
            let flag = flag
                .to_str()
                .ok_or_else(|| "helper flag is not canonical UTF-8".to_owned())?;
            let value = |args: &mut std::iter::Peekable<std::iter::Skip<std::env::ArgsOs>>| {
                args.next()
                    .ok_or_else(|| format!("missing value for {flag}"))
            };
            match flag {
                "--expected-parent-pid" => {
                    expected_parent_pid = Some(parse_u32(value(&mut args)?)?)
                }
                "--execution-binding-sha256" => {
                    execution_binding_digest = Some(parse_digest(value(&mut args)?)?)
                }
                "--executable-path-hex" => executable_path = Some(parse_path(value(&mut args)?)?),
                "--executable-device" => executable_device = Some(parse_u64(value(&mut args)?)?),
                "--executable-inode" => executable_inode = Some(parse_u64(value(&mut args)?)?),
                "--executable-mode" => executable_mode = Some(parse_u32(value(&mut args)?)?),
                "--executable-sha256" => executable_digest = Some(parse_digest(value(&mut args)?)?),
                "--cwd-path-hex" => cwd_path = Some(parse_path(value(&mut args)?)?),
                "--cwd-device" => cwd_device = Some(parse_u64(value(&mut args)?)?),
                "--cwd-inode" => cwd_inode = Some(parse_u64(value(&mut args)?)?),
                "--cwd-mode" => cwd_mode = Some(parse_u32(value(&mut args)?)?),
                "--read-root" => {
                    if read_roots.len() >= MAX_ROOT_COUNT {
                        return Err("too many read roots".to_owned());
                    }
                    read_roots.push(parse_expected_object(value(&mut args)?)?);
                }
                "--write-root" => {
                    if write_roots.len() >= MAX_ROOT_COUNT {
                        return Err("too many write roots".to_owned());
                    }
                    write_roots.push(parse_expected_object(value(&mut args)?)?);
                }
                "--cpu-seconds" => cpu_seconds = Some(parse_u64(value(&mut args)?)?),
                "--address-space-bytes" => {
                    address_space_bytes = Some(parse_u64(value(&mut args)?)?)
                }
                "--max-created-file-bytes" => {
                    max_created_file_bytes = Some(parse_u64(value(&mut args)?)?)
                }
                "--max-open-files" => max_open_files = Some(parse_u64(value(&mut args)?)?),
                "--wall-time-ms" => wall_time_ms = Some(parse_u64(value(&mut args)?)?),
                "--max-output-bytes" => max_output_bytes = Some(parse_u64(value(&mut args)?)?),
                "--arg-hex" => {
                    if argv.len() >= MAX_ARG_COUNT {
                        return Err("too many argv items".to_owned());
                    }
                    let bytes = decode_hex(value(&mut args)?)?;
                    if bytes.len() > MAX_ARG_BYTES {
                        return Err("argv item exceeds byte bound".to_owned());
                    }
                    argv.push(bytes);
                }
                _ => return Err(format!("unsupported helper flag: {flag}")),
            }
        }

        Ok(Config {
            expected_parent_pid: expected_parent_pid.ok_or("missing expected parent pid")?,
            execution_binding_digest: execution_binding_digest
                .ok_or("missing execution binding digest")?,
            executable_path: executable_path.ok_or("missing executable path")?,
            executable_device: executable_device.ok_or("missing executable device")?,
            executable_inode: executable_inode.ok_or("missing executable inode")?,
            executable_mode: executable_mode.ok_or("missing executable mode")?,
            executable_digest: executable_digest.ok_or("missing executable digest")?,
            cwd: ExpectedObject {
                path: cwd_path.ok_or("missing cwd path")?,
                device: cwd_device.ok_or("missing cwd device")?,
                inode: cwd_inode.ok_or("missing cwd inode")?,
                mode: cwd_mode.ok_or("missing cwd mode")?,
            },
            read_roots,
            write_roots,
            cpu_seconds: cpu_seconds.ok_or("missing cpu limit")?,
            address_space_bytes: address_space_bytes.ok_or("missing address-space limit")?,
            max_created_file_bytes: max_created_file_bytes.ok_or("missing file-size limit")?,
            max_open_files: max_open_files.ok_or("missing open-file limit")?,
            wall_time_ms: wall_time_ms.ok_or("missing wall-time limit")?,
            max_output_bytes: max_output_bytes.ok_or("missing output limit")?,
            argv,
        })
    }

    fn parse_expected_object(value: OsString) -> Result<ExpectedObject, String> {
        let value = value
            .to_str()
            .ok_or_else(|| "root helper argument is not canonical UTF-8".to_owned())?;
        let mut parts = value.split(':');
        let path_hex = parts.next().ok_or("missing root path")?;
        let device = parts.next().ok_or("missing root device")?;
        let inode = parts.next().ok_or("missing root inode")?;
        let mode = parts.next().ok_or("missing root mode")?;
        if parts.next().is_some() {
            return Err("root helper argument has extra fields".to_owned());
        }
        Ok(ExpectedObject {
            path: parse_path(OsString::from(path_hex))?,
            device: parse_u64(OsString::from(device))?,
            inode: parse_u64(OsString::from(inode))?,
            mode: parse_u32(OsString::from(mode))?,
        })
    }

    fn parse_u32(value: OsString) -> Result<u32, String> {
        value
            .to_str()
            .ok_or_else(|| "numeric helper argument is not UTF-8".to_owned())?
            .parse()
            .map_err(|_| "invalid u32 helper argument".to_owned())
    }

    fn parse_u64(value: OsString) -> Result<u64, String> {
        value
            .to_str()
            .ok_or_else(|| "numeric helper argument is not UTF-8".to_owned())?
            .parse()
            .map_err(|_| "invalid u64 helper argument".to_owned())
    }

    fn parse_path(value: OsString) -> Result<PathBuf, String> {
        Ok(PathBuf::from(OsString::from_vec(decode_hex(value)?)))
    }

    fn parse_digest(value: OsString) -> Result<[u8; 32], String> {
        let bytes = decode_hex(value)?;
        bytes
            .try_into()
            .map_err(|_| "digest helper argument must contain 32 bytes".to_owned())
    }

    fn decode_hex(value: OsString) -> Result<Vec<u8>, String> {
        let value = value
            .to_str()
            .ok_or_else(|| "hex helper argument is not UTF-8".to_owned())?
            .as_bytes();
        if value.len() % 2 != 0 {
            return Err("hex helper argument has odd length".to_owned());
        }
        value
            .chunks(2)
            .map(|pair| {
                let high =
                    hex_nibble(pair[0]).ok_or_else(|| "invalid hex helper argument".to_owned())?;
                let low =
                    hex_nibble(pair[1]).ok_or_else(|| "invalid hex helper argument".to_owned())?;
                Ok(high << 4 | low)
            })
            .collect()
    }

    fn encode_hex(value: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut encoded = String::with_capacity(value.len() * 2);
        for byte in value {
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0f) as usize] as char);
        }
        encoded
    }

    const fn hex_nibble(value: u8) -> Option<u8> {
        match value {
            b'0'..=b'9' => Some(value - b'0'),
            b'a'..=b'f' => Some(value - b'a' + 10),
            b'A'..=b'F' => Some(value - b'A' + 10),
            _ => None,
        }
    }
}

fn main() {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    if let Err(error) = linux_x86_64::run() {
        eprintln!("golam-native-exec-helper-v2: {error}");
        std::process::exit(70);
    }

    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        eprintln!("golam-native-exec-helper-v2: unsupported outside Linux x86_64");
        std::process::exit(64);
    }
}
