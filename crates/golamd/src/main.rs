#![forbid(unsafe_code)]

pub mod benchmark;
mod connection;
mod deadline_io;
pub mod git_index;
pub mod git_observe;
pub mod git_pack;
#[cfg(test)]
mod git_pack_ref_delta_qualification_tests;
pub mod git_read;
pub mod git_read_budget;
pub mod git_sha1;
pub mod git_status;
pub mod harness;
pub mod local_dir;
pub mod local_fs;
pub mod local_read;
pub mod local_search;
pub mod local_walk;
pub mod memory_commit;
#[cfg(test)]
mod spec004_compaction_tests;

use std::error::Error;
use std::io::{self, Write};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

use connection::{BootstrapApprover, ConnectionMaterial, serve_connection};
use deadline_io::DeadlineIo;
use golam_core::paths::RuntimeLayout;
use golam_core::runtime_home::default_runtime_root;
use golam_core::{ClientId, ResourceLimits};
use golam_ipc::client_handshake::{random_connection_id, random_server_epoch, random_server_nonce};
use golam_ipc::lifecycle::ClientKeyId;
use golam_kernel::{KernelStartup, RuntimeAuthorityPolicy, start_kernel};
use golamd::CommandRouter;

const CONNECTION_DEADLINE: Duration = Duration::from_secs(30);
const BOOTSTRAP_APPROVAL_DEADLINE: Duration = Duration::from_secs(30);

struct ForegroundApproval {
    stdin_pending: Arc<AtomicBool>,
}

impl ForegroundApproval {
    fn new() -> Self {
        Self {
            stdin_pending: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl BootstrapApprover for ForegroundApproval {
    fn approve(&mut self, client_id: ClientId, key_id: ClientKeyId) -> bool {
        if self
            .stdin_pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            eprintln!("golamd: bootstrap approval input is already pending; denying request");
            return false;
        }

        eprint!(
            "Approve first local Golam CLI enrollment for client {} key {}? [y/N] ",
            client_id.0,
            hex(&key_id.0)
        );
        if io::stderr().flush().is_err() {
            self.stdin_pending.store(false, Ordering::Release);
            return false;
        }

        let pending = Arc::clone(&self.stdin_pending);
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let mut answer = String::new();
            let approved = io::stdin().read_line(&mut answer).is_ok()
                && matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes");
            pending.store(false, Ordering::Release);
            let _ = sender.send(approved);
        });

        match receiver.recv_timeout(BOOTSTRAP_APPROVAL_DEADLINE) {
            Ok(approved) => approved,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                eprintln!("golamd: bootstrap approval timed out; denying request");
                false
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => false,
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("golamd: {error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.as_slice() != ["--foreground"] {
        return Err("usage: golamd --foreground".into());
    }

    let runtime = RuntimeLayout::initialize(default_runtime_root()?)?;
    let startup = start_kernel(&runtime, RuntimeAuthorityPolicy::for_runtime(&runtime)?)?;
    let kernel = match startup {
        KernelStartup::Serving { kernel, report } => {
            eprintln!(
                "golamd: recovery_mode={:?} issues={} runtime={}",
                report.mode,
                report.issues.len(),
                runtime.root.display()
            );
            *kernel
        }
        KernelStartup::RecoveryOnly(report) => {
            return Err(format!(
                "privileged service blocked: recovery_mode={:?} issues={}",
                report.mode,
                report.issues.len()
            )
            .into());
        }
        KernelStartup::Quarantined(report) => {
            return Err(format!(
                "privileged service blocked: recovery_mode={:?} issues={}",
                report.mode,
                report.issues.len()
            )
            .into());
        }
    };

    let mut router = CommandRouter::new(kernel);
    let mut approval = ForegroundApproval::new();
    let limits = ResourceLimits::default();
    let server_epoch = random_server_epoch()?;
    serve_local_loop(&runtime, &mut router, &mut approval, limits, server_epoch)
}

#[cfg(unix)]
fn serve_local_loop(
    runtime: &RuntimeLayout,
    router: &mut CommandRouter<RuntimeAuthorityPolicy>,
    approval: &mut ForegroundApproval,
    limits: ResourceLimits,
    server_epoch: u64,
) -> Result<(), Box<dyn Error>> {
    use golam_ipc::unix_transport::UnixTransportListener;

    let listener = UnixTransportListener::bind(runtime)?;
    eprintln!("golamd: listening on {}", listener.socket_path().display());
    loop {
        let peer = listener.accept_same_user()?;
        peer.stream.set_nonblocking(true)?;
        let mut stream = DeadlineIo::new(peer.stream, CONNECTION_DEADLINE);
        let material = connection_material(limits, server_epoch)?;
        if let Err(error) = serve_connection(&mut stream, runtime, router, material, approval) {
            eprintln!("golamd: connection rejected: {error}");
        }
    }
}

#[cfg(windows)]
fn serve_local_loop(
    runtime: &RuntimeLayout,
    router: &mut CommandRouter<RuntimeAuthorityPolicy>,
    approval: &mut ForegroundApproval,
    limits: ResourceLimits,
    server_epoch: u64,
) -> Result<(), Box<dyn Error>> {
    use golam_ipc::windows_transport::WindowsPipeListener;

    let listener = WindowsPipeListener::bind(runtime, limits)?;
    eprintln!("golamd: listening on {}", listener.pipe_path());
    loop {
        let peer = listener.accept()?;
        peer.stream.set_nonblocking(true)?;
        let mut stream = DeadlineIo::new(peer.stream, CONNECTION_DEADLINE);
        let material = connection_material(limits, server_epoch)?;
        if let Err(error) = serve_connection(&mut stream, runtime, router, material, approval) {
            eprintln!("golamd: connection rejected: {error}");
        }
    }
}

#[cfg(not(any(unix, windows)))]
fn serve_local_loop(
    _runtime: &RuntimeLayout,
    _router: &mut CommandRouter<RuntimeAuthorityPolicy>,
    _approval: &mut ForegroundApproval,
    _limits: ResourceLimits,
    _server_epoch: u64,
) -> Result<(), Box<dyn Error>> {
    Err("golamd local IPC is unsupported on this platform".into())
}

fn connection_material(
    limits: ResourceLimits,
    server_epoch: u64,
) -> Result<ConnectionMaterial, Box<dyn Error>> {
    Ok(ConnectionMaterial {
        server_epoch,
        server_nonce: random_server_nonce()?,
        connection_id: random_connection_id()?,
        limits,
    })
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[(byte >> 4) as usize]));
        out.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    out
}
