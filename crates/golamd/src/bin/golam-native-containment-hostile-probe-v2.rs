#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../native_containment_v2.rs"]
mod native_containment;
#[allow(dead_code)]
#[path = "../native_process_supervisor_v2.rs"]
mod native_process_supervisor;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod linux_x86_64 {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpStream;
    use std::os::unix::net::UnixStream;
    use std::os::unix::process::ExitStatusExt;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    use super::native_containment::{LinuxContainmentPlan, PROFILE_TOKEN, observe_native_object};
    use super::native_process_supervisor::{
        ProcessTreeReconciliation, RootContainmentBinding, RootProcessControl,
        RootProcessSupervisor, RootTerminalObservation, RootTerminationKind,
    };

    const NORMAL_WALL_TIME_MS: u64 = 10_000;
    const NORMAL_OUTPUT_BYTES: u64 = 1024 * 1024;
    const HOSTILE_WALL_TIME_MS: u64 = 750;
    const HOSTILE_OUTPUT_BYTES: u64 = 1024;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum ChildMode {
        Normal,
        CancelHold,
        WallTimeHold,
        OutputFlood,
    }

    struct ChildControl {
        child: Child,
    }

    impl RootProcessControl for ChildControl {
        fn request_termination(&mut self, root_pid: u32) -> Result<(), String> {
            if self.child.id() != root_pid {
                return Err(format!(
                    "owned child pid changed: expected {root_pid}, observed {}",
                    self.child.id()
                ));
            }
            self.child.kill().map_err(|error| error.to_string())
        }

        fn observe_terminal(
            &mut self,
            root_pid: u32,
        ) -> Result<Option<RootTerminalObservation>, String> {
            if self.child.id() != root_pid {
                return Err(format!(
                    "owned child pid changed: expected {root_pid}, observed {}",
                    self.child.id()
                ));
            }
            let status = self.child.try_wait().map_err(|error| error.to_string())?;
            Ok(status.map(|status| RootTerminalObservation {
                root_pid,
                termination: match status.code() {
                    Some(code) => RootTerminationKind::Exited(code),
                    None => RootTerminationKind::Signaled(status.signal().unwrap_or(0)),
                },
            }))
        }
    }

    pub fn run() {
        let arg = std::env::args_os().nth(1);
        if arg.as_deref() == Some(std::ffi::OsStr::new("--supervisor-wall-time")) {
            run_wall_time_supervisor_probe();
            return;
        }
        if arg.as_deref() == Some(std::ffi::OsStr::new("--supervisor-output-limit")) {
            run_output_supervisor_probe();
            return;
        }

        let mode = match arg.as_deref() {
            None => ChildMode::Normal,
            Some(value) if value == std::ffi::OsStr::new("--cancel-hold") => ChildMode::CancelHold,
            Some(value) if value == std::ffi::OsStr::new("--wall-time-hold") => {
                ChildMode::WallTimeHold
            }
            Some(value) if value == std::ffi::OsStr::new("--output-flood") => {
                ChildMode::OutputFlood
            }
            Some(value) => {
                eprintln!(
                    "unsupported hostile v2 probe mode: {}",
                    value.to_string_lossy()
                );
                std::process::exit(64);
            }
        };
        run_contained_child(mode);
    }

    fn child_limits(mode: ChildMode) -> (u64, u64) {
        match mode {
            ChildMode::WallTimeHold => (HOSTILE_WALL_TIME_MS, NORMAL_OUTPUT_BYTES),
            ChildMode::OutputFlood => (NORMAL_WALL_TIME_MS, HOSTILE_OUTPUT_BYTES),
            ChildMode::Normal | ChildMode::CancelHold => (NORMAL_WALL_TIME_MS, NORMAL_OUTPUT_BYTES),
        }
    }

    fn run_contained_child(mode: ChildMode) {
        let executable_path = std::env::current_exe().expect("qualification executable path");
        let executable_path = std::fs::canonicalize(executable_path).expect("canonical executable");
        let cwd_path = std::fs::canonicalize(std::env::current_dir().expect("qualification cwd"))
            .expect("canonical cwd");
        let (wall_time_ms, max_stdout_stderr_bytes) = child_limits(mode);

        let plan = LinuxContainmentPlan {
            profile_token: PROFILE_TOKEN.to_owned(),
            executable: observe_native_object(&executable_path).expect("executable identity"),
            cwd: observe_native_object(&cwd_path).expect("cwd identity"),
            filesystem_read_roots: vec![],
            filesystem_write_roots: vec![],
            cpu_seconds: 10,
            address_space_bytes: 512 * 1024 * 1024,
            max_created_file_bytes: 1024 * 1024,
            max_open_files: 64,
            wall_time_ms,
            max_stdout_stderr_bytes,
            strict_local: true,
            spawn_denied: true,
            ambient_environment_cleared: true,
            device_rules_empty: true,
            ipc_rules_empty: true,
            inherited_handle_rules_empty: true,
        };

        let receipt = super::native_containment::apply_child_side(&plan).unwrap_or_else(|error| {
            eprintln!("containment v2 application failed: {error}");
            std::process::exit(2);
        });

        if !receipt.landlock_ruleset_fully_enforced
            || !receipt.no_new_privs
            || !receipt.seccomp_tsync_installed
            || !receipt.spawn_denied
            || !receipt.strict_local
            || receipt.inherited_non_stdio_handles_observed
            || !receipt.identity_bound_regular_file_roots
            || !receipt.ipc_creation_denied
            || !receipt.cross_process_control_denied
            || !receipt.credential_kernel_surfaces_denied
            || !receipt.wall_time_requires_parent_supervisor
            || !receipt.output_requires_parent_supervisor
        {
            eprintln!("containment v2 receipt is incomplete");
            std::process::exit(3);
        }

        println!("CONTAINMENT_APPLIED=YES");
        println!("PROFILE={}", receipt.profile_token);
        println!("SPAWN_DENIED={}", receipt.spawn_denied);
        println!("STRICT_LOCAL={}", receipt.strict_local);
        println!("IDENTITY_BOUND_REGULAR_FILE_ROOTS=YES");
        println!("IPC_CREATION_DENIED=YES");
        println!("CROSS_PROCESS_CONTROL_DENIED=YES");
        println!("CREDENTIAL_KERNEL_SURFACES_DENIED=YES");

        match TcpStream::connect(("127.0.0.1", 9)) {
            Ok(_) => {
                eprintln!("network escape unexpectedly succeeded");
                std::process::exit(10);
            }
            Err(_) => println!("NETWORK_ESCAPE_DENIED=YES"),
        }

        match UnixStream::pair() {
            Ok(_) => {
                eprintln!("local socket IPC unexpectedly succeeded");
                std::process::exit(11);
            }
            Err(_) => println!("IPC_SOCKETPAIR_DENIED=YES"),
        }

        match Command::new("/bin/true").status() {
            Ok(_) => {
                eprintln!("process spawn unexpectedly succeeded");
                std::process::exit(12);
            }
            Err(_) => println!("PROCESS_SPAWN_DENIED=YES"),
        }

        let forbidden_write = PathBuf::from(format!(
            "/tmp/golam-native-containment-v2-forbidden-{}",
            std::process::id()
        ));
        match std::fs::write(&forbidden_write, b"forbidden") {
            Ok(()) => {
                let _ = std::fs::remove_file(&forbidden_write);
                eprintln!("filesystem escape unexpectedly succeeded");
                std::process::exit(13);
            }
            Err(_) => println!("FILESYSTEM_WRITE_ESCAPE_DENIED=YES"),
        }

        match File::open("/dev/null") {
            Ok(_) => {
                eprintln!("device access unexpectedly succeeded");
                std::process::exit(14);
            }
            Err(_) => println!("DEVICE_ACCESS_DENIED=YES"),
        }

        match mode {
            ChildMode::CancelHold => {
                println!("CANCEL_HOLD_READY=YES");
                thread::sleep(Duration::from_secs(30));
                eprintln!("cancel-hold payload was not cancelled within the qualification bound");
                std::process::exit(15);
            }
            ChildMode::WallTimeHold => {
                println!("WALL_TIME_HOLD_READY=YES");
                thread::sleep(Duration::from_secs(30));
                eprintln!("wall-time hostile payload escaped parent supervision");
                std::process::exit(16);
            }
            ChildMode::OutputFlood => {
                println!("OUTPUT_FLOOD_READY=YES");
                let flood = vec![b'X'; 8 * 1024];
                std::io::stdout()
                    .write_all(&flood)
                    .expect("hostile output flood");
                std::io::stdout().flush().expect("flush hostile output");
                thread::sleep(Duration::from_secs(30));
                eprintln!("output hostile payload escaped parent supervision");
                std::process::exit(17);
            }
            ChildMode::Normal => {
                println!("HOSTILE_PROBE_READY=YES");
                thread::sleep(Duration::from_secs(3));
                println!("HOSTILE_PROBE_COMPLETE=YES");
            }
        }
    }

    fn spawn_child(mode: &str) -> (ChildControl, std::process::ChildStdout, Instant) {
        let executable = std::fs::canonicalize(
            std::env::current_exe().expect("qualification supervisor executable path"),
        )
        .expect("canonical qualification supervisor executable");
        let mut child = Command::new(executable)
            .arg(mode)
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn qualification contained child");
        let started = Instant::now();
        let stdout = child.stdout.take().expect("qualification child stdout");
        (ChildControl { child }, stdout, started)
    }

    fn binding(
        root_pid: u32,
        wall_time_limit_ms: u64,
        max_output_bytes: u64,
    ) -> RootContainmentBinding {
        RootContainmentBinding {
            profile_token: PROFILE_TOKEN.to_owned(),
            root_pid,
            landlock_ruleset_fully_enforced: true,
            no_new_privs: true,
            seccomp_tsync_installed: true,
            spawn_denied: true,
            strict_local: true,
            identity_bound_regular_file_roots: true,
            ipc_creation_denied: true,
            cross_process_control_denied: true,
            credential_kernel_surfaces_denied: true,
            wall_time_limit_ms,
            max_stdout_stderr_bytes: max_output_bytes,
        }
    }

    fn require_readiness_line(
        stdout: std::process::ChildStdout,
        supervisor: &mut RootProcessSupervisor<ChildControl>,
        expected: &str,
    ) -> BufReader<std::process::ChildStdout> {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        for _ in 0..48 {
            line.clear();
            let bytes = reader
                .read_line(&mut line)
                .expect("read qualification child marker");
            if bytes == 0 {
                eprintln!("qualification child exited before readiness marker: {expected}");
                std::process::exit(20);
            }
            let evidence = supervisor
                .account_output_bytes(bytes as u64)
                .expect("account qualification child marker output");
            if evidence.limit_exceeded {
                eprintln!("qualification child exceeded output limit before readiness marker");
                std::process::exit(21);
            }
            if line.trim_end() == expected {
                return reader;
            }
        }
        eprintln!("qualification child did not emit readiness marker: {expected}");
        std::process::exit(22);
    }

    fn require_terminal(supervisor: &mut RootProcessSupervisor<ChildControl>) {
        for _ in 0..200 {
            match supervisor
                .reconcile_terminal()
                .expect("reconcile qualification child terminal state")
            {
                ProcessTreeReconciliation::TerminalVerified(evidence) => {
                    if !evidence.terminal_verified
                        || !evidence.spawn_denial_bound
                        || evidence.observed_descendant_count != 0
                    {
                        eprintln!("qualification terminal evidence is incomplete");
                        std::process::exit(23);
                    }
                    return;
                }
                ProcessTreeReconciliation::Unresolved { .. } => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        eprintln!("qualification child did not reach exact terminal observation");
        std::process::exit(24);
    }

    fn run_wall_time_supervisor_probe() {
        let (control, stdout, started) = spawn_child("--wall-time-hold");
        let root_pid = control.child.id();
        let mut supervisor = RootProcessSupervisor::new(
            binding(root_pid, HOSTILE_WALL_TIME_MS, NORMAL_OUTPUT_BYTES),
            control,
        )
        .expect("bind wall-time qualification supervisor");
        let _reader = require_readiness_line(stdout, &mut supervisor, "WALL_TIME_HOLD_READY=YES");

        loop {
            let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
            let evidence = supervisor
                .observe_wall_time_ms(elapsed_ms)
                .expect("enforce wall-time qualification bound");
            if evidence.limit_exceeded {
                if !evidence.termination_request_dispatched {
                    eprintln!("wall-time limit did not dispatch root termination");
                    std::process::exit(25);
                }
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        require_terminal(&mut supervisor);
        println!("SUPERVISOR_WALL_TIME_ENFORCED=YES");
        println!("SUPERVISOR_WALL_TIME_TERMINAL_RECONCILED=YES");
    }

    fn run_output_supervisor_probe() {
        let (control, mut stdout, _started) = spawn_child("--output-flood");
        let root_pid = control.child.id();
        let mut supervisor = RootProcessSupervisor::new(
            binding(root_pid, NORMAL_WALL_TIME_MS, HOSTILE_OUTPUT_BYTES),
            control,
        )
        .expect("bind output qualification supervisor");

        let mut buffer = [0_u8; 128];
        let mut limit_observed = false;
        for _ in 0..128 {
            let bytes = stdout.read(&mut buffer).expect("read hostile output chunk");
            if bytes == 0 {
                break;
            }
            let evidence = supervisor
                .account_output_bytes(bytes as u64)
                .expect("enforce combined output qualification bound");
            if evidence.limit_exceeded {
                if !evidence.termination_request_dispatched
                    || evidence.accepted_output_bytes > HOSTILE_OUTPUT_BYTES
                {
                    eprintln!("output limit evidence is incomplete");
                    std::process::exit(26);
                }
                limit_observed = true;
                break;
            }
        }
        if !limit_observed {
            eprintln!("hostile output did not trigger the combined output bound");
            std::process::exit(27);
        }

        require_terminal(&mut supervisor);
        println!("SUPERVISOR_OUTPUT_LIMIT_ENFORCED=YES");
        println!(
            "SUPERVISOR_OUTPUT_ACCEPTED_BYTES={}",
            supervisor.accepted_output_bytes()
        );
        println!("SUPERVISOR_OUTPUT_TERMINAL_RECONCILED=YES");
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn main() {
    linux_x86_64::run();
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn main() {
    eprintln!(
        "native containment v2 hostile qualification is unsupported on this platform: profile={}",
        native_containment::PROFILE_TOKEN
    );
    std::process::exit(77);
}
