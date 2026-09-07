#![forbid(unsafe_code)]

use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::thread;
use std::time::{Duration, Instant};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::runtime_home::default_runtime_root;
use golam_core::{ClientId, PROTOCOL_VERSION, ResourceLimits};
use golam_ipc::client_handshake::authenticate_client;
use golam_ipc::credentials::{ClientCredentialStore, GeneratedClientCredential};
use golam_ipc::lifecycle::{ClientKeyId, LifecycleMessage, ShutdownReason};
use golam_ipc::wire::write_frame;
use golam_ipc::{FrameHeader, FrameKind};
use golam_ledger::clients::{ClientKind, ClientRegistry};

const IPC_DEADLINE: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(2);

struct DeadlineIo<S> {
    inner: S,
    deadline: Instant,
}

impl<S> DeadlineIo<S> {
    fn new(inner: S, lifetime: Duration) -> Self {
        Self {
            inner,
            deadline: Instant::now() + lifetime,
        }
    }

    fn wait_for_progress(&self) -> io::Result<()> {
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Golam desktop local IPC deadline exceeded",
                )
            })?;
        thread::sleep(remaining.min(POLL_INTERVAL));
        Ok(())
    }
}

impl<S: Read> Read for DeadlineIo<S> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        loop {
            match self.inner.read(buffer) {
                Ok(read) => return Ok(read),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    self.wait_for_progress()?;
                }
                Err(error) => return Err(error),
            }
        }
    }
}

impl<S: Write> Write for DeadlineIo<S> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        loop {
            match self.inner.write(buffer) {
                Ok(written) => return Ok(written),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    self.wait_for_progress()?;
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct DesktopHostStatus {
    pub connected: bool,
    pub client_id: Option<u128>,
    pub reason: Option<String>,
}

pub fn authenticate_status() -> DesktopHostStatus {
    match authenticate_desktop_host() {
        Ok(client_id) => DesktopHostStatus {
            connected: true,
            client_id: Some(client_id.0),
            reason: None,
        },
        Err(error) => DesktopHostStatus {
            connected: false,
            client_id: None,
            reason: Some(error.to_string()),
        },
    }
}

fn authenticate_desktop_host() -> Result<ClientId, Box<dyn Error>> {
    let runtime = RuntimeLayout::initialize(default_runtime_root()?)?;
    let authority = AuthorityLayout::initialize(&runtime)?;
    let store = ClientCredentialStore::new(&authority);
    let credential = single_desktop_credential(&authority, &store)?;
    let signing_key = store.load(credential.client_id, credential.key_id)?;
    let limits = ResourceLimits::default();

    #[cfg(unix)]
    let stream = golam_ipc::unix_transport::connect_same_user(&runtime)?;
    #[cfg(windows)]
    let stream = golam_ipc::windows_transport::connect_current_user(&runtime)?;
    #[cfg(not(any(unix, windows)))]
    return Err("Golam desktop local IPC is unsupported on this platform".into());

    stream.set_nonblocking(true)?;
    let mut stream = DeadlineIo::new(stream, IPC_DEADLINE);
    let ready = authenticate_client(
        &mut stream,
        credential.client_id,
        credential.key_id,
        &signing_key,
        limits,
    )?;
    write_shutdown(&mut stream, ready.limits)?;
    Ok(credential.client_id)
}

fn single_desktop_credential(
    authority: &AuthorityLayout,
    store: &ClientCredentialStore<'_>,
) -> Result<GeneratedClientCredential, Box<dyn Error>> {
    let registry = ClientRegistry::open(authority)?;
    let mut credentials = Vec::new();

    for entry in fs::read_dir(authority.credential_dir())? {
        let entry = entry?;
        let name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };
        let Some((client_id, key_id)) = parse_credential_filename(&name)? else {
            continue;
        };
        let Some(record) = registry.record_for_client(client_id)? else {
            continue;
        };
        if record.revoked_at.is_some()
            || record.kind != ClientKind::DesktopFuture
            || record.key_id != key_id.0
        {
            continue;
        }
        credentials.push(store.inspect(client_id, key_id)?);
    }

    credentials.sort_by(|left, right| left.path.cmp(&right.path));
    match credentials.len() {
        0 => Err(
            "no active protected DesktopFuture credential exists; desktop authentication is disconnected"
                .into(),
        ),
        1 => Ok(credentials.remove(0)),
        count => Err(format!(
            "found {count} active protected DesktopFuture credentials; refusing ambiguous desktop identity"
        )
        .into()),
    }
}

fn parse_credential_filename(
    name: &str,
) -> Result<Option<(ClientId, ClientKeyId)>, Box<dyn Error>> {
    let Some(stem) = name.strip_suffix(".gkey") else {
        return Ok(None);
    };
    let Some((client_hex, key_hex)) = stem.split_once('-') else {
        return Ok(None);
    };
    if client_hex.len() != 32 || key_hex.len() != 64 {
        return Ok(None);
    }
    let client_id = ClientId(u128::from_str_radix(client_hex, 16)?);
    if client_id.0 == 0 {
        return Err("protected credential filename contains a zero client id".into());
    }
    Ok(Some((client_id, ClientKeyId(decode_hex_32(key_hex)?))))
}

fn decode_hex_32(value: &str) -> Result<[u8; 32], Box<dyn Error>> {
    if value.len() != 64 {
        return Err("client key id must contain exactly 64 hexadecimal characters".into());
    }
    let mut bytes = [0_u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = hex_digit(chunk[0]).ok_or("client key id contains non-hexadecimal data")?;
        let low = hex_digit(chunk[1]).ok_or("client key id contains non-hexadecimal data")?;
        bytes[index] = (high << 4) | low;
    }
    if bytes.iter().all(|byte| *byte == 0) {
        return Err("client key id must not be all zero".into());
    }
    Ok(bytes)
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn write_shutdown<S: Write>(stream: &mut S, limits: ResourceLimits) -> Result<(), Box<dyn Error>> {
    let message = LifecycleMessage::Shutdown(ShutdownReason::Normal);
    let payload = message.encode_payload();
    let header = FrameHeader {
        protocol_version: PROTOCOL_VERSION,
        kind: FrameKind::Shutdown,
        request_id: None,
        payload_len: u32::try_from(payload.len()).expect("shutdown payload length fits u32"),
    };
    write_frame(stream, header, &payload, limits)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_filename_parser_rejects_zero_and_malformed_ids() {
        assert!(parse_credential_filename("not-a-credential").unwrap().is_none());
        assert!(parse_credential_filename(&format!("{}-{}.gkey", "0".repeat(32), "1".repeat(64))).is_err());
        assert!(parse_credential_filename(&format!("{}-{}.gkey", "1".repeat(32), "0".repeat(64))).is_err());
    }
}
