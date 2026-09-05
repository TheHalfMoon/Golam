#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../native_containment.rs"]
mod native_containment;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn main() {
    use std::fs::File;
    use std::net::TcpStream;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::Duration;

    use native_containment::{LinuxContainmentPlan, PROFILE_TOKEN, observe_native_object};

    let cancel_hold = std::env::args_os().nth(1).is_some_and(|arg| arg == "--cancel-hold");
    let executable_path = std::env::current_exe().expect("qualification executable path");
    let executable_path = std::fs::canonicalize(executable_path).expect("canonical executable");
    let cwd_path = std::fs::canonicalize(std::env::current_dir().expect("qualification cwd"))
        .expect("canonical cwd");

    let plan = LinuxContainmentPlan {
        profile_token: PROFILE_TOKEN.to_owned(),
        executable: observe_native_object(&executable_path).expect("executable identity"),
        cwd: observe_native_object(&cwd_path).expect("cwd identity"),
        filesystem_read_roots: vec![cwd_path],
        filesystem_write_roots: vec![],
        cpu_seconds: 10,
        address_space_bytes: 512 * 1024 * 1024,
        max_created_file_bytes: 1024 * 1024,
        max_open_files: 64,
        wall_time_ms: 10_000,
        max_stdout_stderr_bytes: 1024 * 1024,
        strict_local: true,
        spawn_denied: true,
        ambient_environment_cleared: true,
        device_rules_empty: true,
        ipc_rules_empty: true,
        inherited_handle_rules_empty: true,
    };

    let receipt = native_containment::apply_child_side(&plan).unwrap_or_else(|error| {
        eprintln!("containment application failed: {error}");
        std::process::exit(2);
    });

    if !receipt.landlock_ruleset_fully_enforced
        || !receipt.no_new_privs
        || !receipt.seccomp_tsync_installed
        || !receipt.spawn_denied
        || !receipt.strict_local
        || receipt.inherited_non_stdio_handles_observed
    {
        eprintln!("containment receipt is incomplete");
        std::process::exit(3);
    }

    println!("CONTAINMENT_APPLIED=YES");
    println!("PROFILE={}", receipt.profile_token);
    println!("SPAWN_DENIED={}", receipt.spawn_denied);
    println!("STRICT_LOCAL={}", receipt.strict_local);

    match TcpStream::connect(("127.0.0.1", 9)) {
        Ok(_) => {
            eprintln!("network escape unexpectedly succeeded");
            std::process::exit(10);
        }
        Err(_) => println!("NETWORK_ESCAPE_DENIED=YES"),
    }

    match Command::new("/bin/true").status() {
        Ok(_) => {
            eprintln!("process spawn unexpectedly succeeded");
            std::process::exit(11);
        }
        Err(_) => println!("PROCESS_SPAWN_DENIED=YES"),
    }

    let forbidden_write = PathBuf::from(format!(
        "/tmp/golam-native-containment-forbidden-{}",
        std::process::id()
    ));
    match std::fs::write(&forbidden_write, b"forbidden") {
        Ok(()) => {
            let _ = std::fs::remove_file(&forbidden_write);
            eprintln!("filesystem escape unexpectedly succeeded");
            std::process::exit(12);
        }
        Err(_) => println!("FILESYSTEM_WRITE_ESCAPE_DENIED=YES"),
    }

    match File::open("/dev/null") {
        Ok(_) => {
            eprintln!("device access unexpectedly succeeded");
            std::process::exit(13);
        }
        Err(_) => println!("DEVICE_ACCESS_DENIED=YES"),
    }

    if cancel_hold {
        println!("CANCEL_HOLD_READY=YES");
        std::thread::sleep(Duration::from_secs(30));
        eprintln!("cancel-hold payload was not cancelled within the qualification bound");
        std::process::exit(14);
    }

    println!("HOSTILE_PROBE_READY=YES");
    std::thread::sleep(Duration::from_secs(3));
    println!("HOSTILE_PROBE_COMPLETE=YES");
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn main() {
    eprintln!("native containment hostile qualification is unsupported on this platform");
    std::process::exit(77);
}
