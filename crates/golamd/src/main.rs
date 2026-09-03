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
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let runtime = RuntimeLayout::initialize(default_runtime_root()?)?;
    let limits = ResourceLimits::default();
    let startup = KernelStartup {
        runtime: &runtime,
        policy: RuntimeAuthorityPolicy::default(),
        limits,
    };
    let mut kernel = start_kernel(startup)?;
    let listener = connection::bind_listener(&runtime)?;
    eprintln!("golamd: listening on {}", listener.description());
    let mut router = CommandRouter::new(kernel.api_mut());
    let mut approval = ForegroundApproval::new();
    loop {
        let connection = listener.accept()?;
        let material = ConnectionMaterial {
            connection_id: random_connection_id()?,
            server_epoch: random_server_epoch()?,
            server_nonce: random_server_nonce()?,
        };
        let mut deadline = DeadlineIo::new(connection, CONNECTION_DEADLINE);
        if let Err(error) = serve_connection(
            &mut deadline,
            material,
            &mut router,
            &mut approval,
            &runtime,
        ) {
            eprintln!("golamd: connection failed: {error}");
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
