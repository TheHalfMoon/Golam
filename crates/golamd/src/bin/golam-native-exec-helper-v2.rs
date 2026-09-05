#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../native_containment_v2.rs"]
mod native_containment_v2;
#[allow(dead_code)]
#[path = "../static_elf_v2.rs"]
mod static_elf_v2;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod linux_x86_64 {
    use std::ffi::{CString, OsString};
    use std::fs::File;
    use std::io::Read;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::MetadataExt;
    use std::path::PathBuf;

    use golam_core::digest::sha256;
    use nix::sys::prctl::{get_pdeathsig, set_pdeathsig};
    use nix::sys::signal::Signal;
    use nix::unistd::{fexecve, geteuid, getppid};

    use super::native_containment_v2::{
        LinuxContainmentPlan, NativeObjectIdentity, PROFILE_TOKEN, apply_child_side,
        observe_native_object,
    };
    use super::static_elf_v2::{MAX_STATIC_EXECUTABLE_BYTES, validate_static_elf_v2};

    const STAGED_PERMISSION_BITS: u32 = 0o500;
    const MAX_ARG_COUNT: usize = 256;
    const MAX_ARG_BYTES: usize = 64 * 1024;
    const MAX_ROOT_COUNT: usize = 64;

    #[derive(Debug)]
    struct Config {
        expected_parent_pid: u32,
        executable_path: PathBuf,
        executable_device: u64,
        executable_inode: u64,
        executable_mode: u32,
        executable_digest: [u8; 32],
        cwd_path: PathBuf,
        read_paths: Vec<PathBuf>,
        write_paths: Vec<PathBuf>,
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
        let config = parse_args()?;
        bind_parent_death(config.expected_parent_pid)?;

        let initial_file = File::open(&config.executable_path)
            .map_err(|error| format!("open staged executable before containment: {error}"))?;
        verify_staged_file(&initial_file, &config)?;
        drop(initial_file);

        let cwd = observe_native_object(&config.cwd_path)
            .map_err(|error| format!("observe cwd identity: {error}"))?;
        let executable = expected_executable_identity(&config);
        let filesystem_read_roots = config
            .read_paths
            .iter()
            .map(|path| observe_native_object(path).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let filesystem_write_roots = config
            .write_paths
            .iter()
            .map(|path| observe_native_object(path).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;

        std::env::set_current_dir(&config.cwd_path)
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

    fn expected_executable_identity(config: &Config) -> NativeObjectIdentity {
        NativeObjectIdentity {
            canonical_path: config.executable_path.clone(),
            device: config.executable_device,
            inode: config.executable_inode,
            mode: config.executable_mode,
        }
    }

    fn verify_staged_file(file: &File, config: &Config) -> Result<(), String> {
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
        reader
            .by_ref()
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

    fn parse_args() -> Result<Config, String> {
        let mut args = std::env::args_os().skip(1).peekable();
        let mut expected_parent_pid = None;
        let mut executable_path = None;
        let mut executable_device = None;
        let mut executable_inode = None;
        let mut executable_mode = None;
        let mut executable_digest = None;
        let mut cwd_path = None;
        let mut read_paths = Vec::new();
        let mut write_paths = Vec::new();
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
            let value = |
                args: &mut std::iter::Peekable<std::iter::Skip<std::env::ArgsOs>>,
            | {
                args.next()
                    .ok_or_else(|| format!("missing value for {flag}"))
            };
            match flag {
                "--expected-parent-pid" => {
                    expected_parent_pid = Some(parse_u32(value(&mut args)?)?)
                }
                "--executable-path-hex" => executable_path = Some(parse_path(value(&mut args)?)?),
                "--executable-device" => executable_device = Some(parse_u64(value(&mut args)?)?),
                "--executable-inode" => executable_inode = Some(parse_u64(value(&mut args)?)?),
                "--executable-mode" => executable_mode = Some(parse_u32(value(&mut args)?)?),
                "--executable-sha256" => executable_digest = Some(parse_digest(value(&mut args)?)?),
                "--cwd-path-hex" => cwd_path = Some(parse_path(value(&mut args)?)?),
                "--read-path-hex" => {
                    if read_paths.len() >= MAX_ROOT_COUNT {
                        return Err("too many read roots".to_owned());
                    }
                    read_paths.push(parse_path(value(&mut args)?)?);
                }
                "--write-path-hex" => {
                    if write_paths.len() >= MAX_ROOT_COUNT {
                        return Err("too many write roots".to_owned());
                    }
                    write_paths.push(parse_path(value(&mut args)?)?);
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
            executable_path: executable_path.ok_or("missing executable path")?,
            executable_device: executable_device.ok_or("missing executable device")?,
            executable_inode: executable_inode.ok_or("missing executable inode")?,
            executable_mode: executable_mode.ok_or("missing executable mode")?,
            executable_digest: executable_digest.ok_or("missing executable digest")?,
            cwd_path: cwd_path.ok_or("missing cwd path")?,
            read_paths,
            write_paths,
            cpu_seconds: cpu_seconds.ok_or("missing cpu limit")?,
            address_space_bytes: address_space_bytes.ok_or("missing address-space limit")?,
            max_created_file_bytes: max_created_file_bytes.ok_or("missing file-size limit")?,
            max_open_files: max_open_files.ok_or("missing open-file limit")?,
            wall_time_ms: wall_time_ms.ok_or("missing wall-time limit")?,
            max_output_bytes: max_output_bytes.ok_or("missing output limit")?,
            argv,
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
            .chunks_exact(2)
            .map(|pair| {
                let high =
                    hex_nibble(pair[0]).ok_or_else(|| "invalid hex helper argument".to_owned())?;
                let low =
                    hex_nibble(pair[1]).ok_or_else(|| "invalid hex helper argument".to_owned())?;
                Ok(high << 4 | low)
            })
            .collect()
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
