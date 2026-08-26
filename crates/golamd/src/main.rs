#![forbid(unsafe_code)]

mod connection;

use std::error::Error;
use std::io::{self, Write};
use std::process::ExitCode;

use connection::{BootstrapApprover, ConnectionMaterial, serve_connection};
use golam_core::paths::RuntimeLayout;
use golam_core::runtime_home::default_runtime_root;
use golam_core::{ClientId, ResourceLimits};
use golam_ipc::lifecycle::{ClientKeyId, ConnectionId};
use golam_kernel::{BootstrapPolicy, KernelStartup, start_kernel};
use golamd::CommandRouter;

struct ForegroundApproval;

impl BootstrapApprover for ForegroundApproval {
    fn approve(&mut self, client_id: ClientId, key_id: ClientKeyId) -> bool {
        eprint!(
            "Approve first local Golam CLI enrollment for client {} key {}? [y/N] ",
            client_id.0,
            hex(&key_id.0)
        );
        if io::stderr().flush().is_err() {
            return false;
        }
        let mut answer = String::new();
        if io::stdin().read_line(&mut answer).is_err() {
            return false;
        }
        matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
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
    let startup = start_kernel(&runtime, BootstrapPolicy::default())?;
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
    let mut approval = ForegroundApproval;
    let limits = ResourceLimits::default();
    let server_epoch = random_nonzero_u64()?;
    serve_local_loop(
        &runtime,
        &mut router,
        &mut approval,
        limits,
        server_epoch,
    )
}

#[cfg(unix)]
fn serve_local_loop(
    runtime: &RuntimeLayout,
    router: &mut CommandRouter<BootstrapPolicy>,
    approval: &mut ForegroundApproval,
    limits: ResourceLimits,
    server_epoch: u64,
) -> Result<(), Box<dyn Error>> {
    use golam_ipc::unix_transport::UnixTransportListener;

    let listener = UnixTransportListener::bind(runtime)?;
    eprintln!("golamd: listening on {}", listener.socket_path().display());
    loop {
        let mut peer = listener.accept_same_user()?;
        let material = connection_material(limits, server_epoch)?;
        if let Err(error) = serve_connection(
            &mut peer.stream,
            runtime,
            router,
            material,
            approval,
        ) {
            eprintln!("golamd: connection rejected: {error}");
        }
    }
}

#[cfg(windows)]
fn serve_local_loop(
    runtime: &RuntimeLayout,
    router: &mut CommandRouter<BootstrapPolicy>,
    approval: &mut ForegroundApproval,
    limits: ResourceLimits,
    server_epoch: u64,
) -> Result<(), Box<dyn Error>> {
    use golam_ipc::windows_transport::WindowsPipeListener;

    let listener = WindowsPipeListener::bind(runtime, limits)?;
    eprintln!("golamd: listening on {}", listener.pipe_path());
    loop {
        let mut peer = listener.accept()?;
        let material = connection_material(limits, server_epoch)?;
        if let Err(error) = serve_connection(
            &mut peer.stream,
            runtime,
            router,
            material,
            approval,
        ) {
            eprintln!("golamd: connection rejected: {error}");
        }
    }
}

#[cfg(not(any(unix, windows)))]
fn serve_local_loop(
    _runtime: &RuntimeLayout,
    _router: &mut CommandRouter<BootstrapPolicy>,
    _approval: &mut ForegroundApproval,
    _limits: ResourceLimits,
    _server_epoch: u64,
) -> Result<(), Box<dyn Error>> {
    Err("golamd local IPC is unsupported on this platform".into())
}

fn connection_material(
    limits: ResourceLimits,
    server_epoch: u64,
) -> Result<ConnectionMaterial, getrandom::Error> {
    Ok(ConnectionMaterial {
        server_epoch,
        server_nonce: random_nonzero_array()?,
        connection_id: ConnectionId(random_nonzero_u128()?),
        limits,
    })
}

fn random_nonzero_array<const N: usize>() -> Result<[u8; N], getrandom::Error> {
    loop {
        let mut bytes = [0_u8; N];
        getrandom::fill(&mut bytes)?;
        if bytes.iter().any(|byte| *byte != 0) {
            return Ok(bytes);
        }
    }
}

fn random_nonzero_u64() -> Result<u64, getrandom::Error> {
    loop {
        let value = u64::from_be_bytes(random_nonzero_array()?);
        if value != 0 {
            return Ok(value);
        }
    }
}

fn random_nonzero_u128() -> Result<u128, getrandom::Error> {
    loop {
        let value = u128::from_be_bytes(random_nonzero_array()?);
        if value != 0 {
            return Ok(value);
        }
    }
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
