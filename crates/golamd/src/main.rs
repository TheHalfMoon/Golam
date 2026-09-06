#![forbid(unsafe_code)]

extern crate self as miniz_oxide;

pub use miniz_oxide_read::{DataFormat, MZError, MZFlush, MZStatus};

pub mod inflate {
    pub mod stream {
        pub use miniz_oxide_read::inflate::stream::{InflateState, inflate};
    }
}

#[cfg(unix)]
pub mod deflate {
    const ADLER_MODULUS: u32 = 65_521;
    const STORED_BLOCK_MAX: usize = u16::MAX as usize;

    pub fn compress_to_vec_zlib(input: &[u8], _level: u8) -> Vec<u8> {
        let block_count = input.len().div_ceil(STORED_BLOCK_MAX).max(1);
        let capacity = input
            .len()
            .saturating_add(block_count.saturating_mul(5))
            .saturating_add(6);
        let mut output = Vec::with_capacity(capacity);
        output.extend_from_slice(&[0x78, 0x01]);

        if input.is_empty() {
            push_stored_block(&mut output, &[], true);
        } else {
            let chunks = input.chunks(STORED_BLOCK_MAX);
            let count = chunks.len();
            for (index, chunk) in chunks.enumerate() {
                push_stored_block(&mut output, chunk, index + 1 == count);
            }
        }

        output.extend_from_slice(&adler32(input).to_be_bytes());
        output
    }

    fn push_stored_block(output: &mut Vec<u8>, bytes: &[u8], final_block: bool) {
        let len = u16::try_from(bytes.len()).expect("stored DEFLATE chunk is bounded to u16");
        output.push(u8::from(final_block));
        output.extend_from_slice(&len.to_le_bytes());
        output.extend_from_slice(&(!len).to_le_bytes());
        output.extend_from_slice(bytes);
    }

    fn adler32(bytes: &[u8]) -> u32 {
        let mut a = 1_u32;
        let mut b = 0_u32;
        for byte in bytes {
            a = (a + u32::from(*byte)) % ADLER_MODULUS;
            b = (b + a) % ADLER_MODULUS;
        }
        b << 16 | a
    }
}

#[cfg(unix)]
extern crate self as nix;

#[cfg(unix)]
mod errno {
    pub use golam_core::unix_fs::Errno;
}
#[cfg(unix)]
mod fcntl {
    pub use golam_core::unix_fs::{AtFlags, OFlag, open, openat, renameat};
}
#[cfg(unix)]
mod sys {
    pub mod stat {
        pub use golam_core::unix_fs::{Mode, mkdirat};
    }
}
#[cfg(unix)]
mod unistd {
    pub use golam_core::unix_fs::{UnlinkatFlags, linkat, unlinkat};
}

pub mod benchmark;
mod connection;
mod deadline_io;
pub mod file_mutation;
#[cfg(all(test, unix))]
mod file_mutation_qualification_tests;
pub mod file_path_mutation;
#[cfg(all(test, unix))]
mod file_path_mutation_qualification_tests;
pub mod git_index;
#[cfg_attr(not(unix), allow(dead_code, unused_imports))]
pub mod git_mutation;
#[cfg(all(test, target_os = "linux"))]
mod git_mutation_phase_f_qualification_tests;
#[allow(
    clippy::too_many_arguments,
    reason = "sealed Git observation constructors keep every evidence-bound field explicit"
)]
pub mod git_observe;
pub mod git_pack;
#[cfg(test)]
mod git_pack_ref_delta_qualification_tests;
pub mod git_read;
pub mod git_read_budget;
pub mod git_sha1;
#[cfg_attr(all(test, not(unix)), allow(dead_code, unused_imports))]
pub mod git_status;
pub mod harness;
pub mod local_dir;
pub mod local_fs;
pub mod local_read;
pub mod local_search;
pub mod local_walk;
pub mod memory_commit;
mod memory_reconcile;
#[cfg(test)]
mod spec004_compaction_tests;
#[cfg(all(test, unix))]
mod spec005_core_alpha_tests;

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
    let mut kernel = match startup {
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

    let memory_restart =
        memory_reconcile::reconcile_managed_memory_on_startup(&runtime, &mut kernel)?;
    eprintln!(
        "golamd: memory_restart_scanned={} committed={} no_mutation={} blocked_unknown={}",
        memory_restart.scanned,
        memory_restart.committed,
        memory_restart.no_mutation,
        memory_restart.blocked_unknown
    );

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
