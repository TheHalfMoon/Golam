#![forbid(unsafe_code)]

use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::runtime_home::default_runtime_root;
use golam_core::{ClientId, PROTOCOL_VERSION, ResourceLimits};
use golam_ipc::client_handshake::{authenticate_client, random_connection_id};
use golam_ipc::credentials::{ClientCredentialStore, GeneratedClientCredential};
use golam_ipc::lifecycle::{ClientKeyId, LifecycleMessage, ShutdownReason};
use golam_ipc::wire::write_frame;
use golam_ipc::{FrameHeader, FrameKind};
use golam_ledger::clients::{ClientKind, ClientRegistry};

const IPC_DEADLINE: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(2);
const DESKTOP_BOOTSTRAP_MARKER: &str = ".desktop-bootstrap-client-id";
static DESKTOP_IDENTITY_LOCK: Mutex<()> = Mutex::new(());

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
    match authenticate_desktop_host_serialized() {
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

fn authenticate_desktop_host_serialized() -> Result<ClientId, Box<dyn Error>> {
    let _identity_guard = DESKTOP_IDENTITY_LOCK
        .lock()
        .map_err(|_| "Golam desktop identity lock is poisoned")?;
    authenticate_desktop_host()
}

fn authenticate_desktop_host() -> Result<ClientId, Box<dyn Error>> {
    let runtime = RuntimeLayout::initialize(default_runtime_root()?)?;
    let authority = AuthorityLayout::initialize(&runtime)?;
    let store = ClientCredentialStore::new(&authority);
    let registry = ClientRegistry::open(&authority)?;
    let (credential, bootstrap_candidate) = desktop_credential(&authority, &store, &registry)?;
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

    let registry = ClientRegistry::open(&authority)?;
    let record = registry.resolve_active(credential.client_id, credential.key_id.0)?;
    if record.kind != ClientKind::DesktopFuture {
        if bootstrap_candidate {
            clear_bootstrap_marker(&authority)?;
        }
        return Err(format!(
            "protected client {} was enrolled as {}; desktop requires explicit desktop bootstrap approval",
            credential.client_id.0,
            record.kind.as_str()
        )
        .into());
    }
    if bootstrap_candidate {
        clear_bootstrap_marker(&authority)?;
    }
    Ok(credential.client_id)
}

fn desktop_credential(
    authority: &AuthorityLayout,
    store: &ClientCredentialStore<'_>,
    registry: &ClientRegistry,
) -> Result<(GeneratedClientCredential, bool), Box<dyn Error>> {
    let mut active = Vec::new();
    for credential in all_credentials(authority, store)? {
        let Some(record) = registry.record_for_client(credential.client_id)? else {
            continue;
        };
        if record.revoked_at.is_none()
            && record.kind == ClientKind::DesktopFuture
            && record.key_id == credential.key_id.0
        {
            active.push(credential);
        }
    }
    active.sort_by(|left, right| left.path.cmp(&right.path));
    match active.len() {
        1 => {
            clear_bootstrap_marker_if_present(authority)?;
            return Ok((active.remove(0), false));
        }
        count if count > 1 => {
            return Err(format!(
                "found {count} active protected DesktopFuture credentials; refusing ambiguous desktop identity"
            )
            .into());
        }
        _ => {}
    }

    bootstrap_credential(authority, store, registry)
}

fn bootstrap_credential(
    authority: &AuthorityLayout,
    store: &ClientCredentialStore<'_>,
    registry: &ClientRegistry,
) -> Result<(GeneratedClientCredential, bool), Box<dyn Error>> {
    let marker = bootstrap_marker_path(authority);
    if marker.exists() {
        let client_id = read_bootstrap_client_id(&marker)?;
        if let Some(record) = registry.record_for_client(client_id)? {
            clear_bootstrap_marker(authority)?;
            return Err(format!(
                "desktop bootstrap client {} is already enrolled as {}; restart desktop status to create a fresh desktop candidate",
                client_id.0,
                record.kind.as_str()
            )
            .into());
        }
        let mut matches = credentials_for_client(authority, store, client_id)?;
        return match matches.len() {
            0 => Ok((store.generate(client_id)?, true)),
            1 => Ok((matches.remove(0), true)),
            count => Err(format!(
                "found {count} protected credentials for pending desktop bootstrap client {}; refusing ambiguity",
                client_id.0
            )
            .into()),
        };
    }

    loop {
        let client_id = ClientId(random_connection_id()?.0);
        if registry.record_for_client(client_id)?.is_some()
            || !credentials_for_client(authority, store, client_id)?.is_empty()
        {
            continue;
        }
        let generated = store.generate(client_id)?;
        match create_bootstrap_marker(&marker, client_id) {
            Ok(()) => return Ok((generated, true)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                store.remove(generated.client_id, generated.key_id)?;
                let existing = read_bootstrap_client_id(&marker)?;
                let mut matches = credentials_for_client(authority, store, existing)?;
                return match matches.len() {
                    0 => Ok((store.generate(existing)?, true)),
                    1 => Ok((matches.remove(0), true)),
                    count => Err(format!(
                        "found {count} protected credentials for concurrent desktop bootstrap client {}; refusing ambiguity",
                        existing.0
                    )
                    .into()),
                };
            }
            Err(error) => {
                store.remove(generated.client_id, generated.key_id)?;
                return Err(error.into());
            }
        }
    }
}

fn all_credentials(
    authority: &AuthorityLayout,
    store: &ClientCredentialStore<'_>,
) -> Result<Vec<GeneratedClientCredential>, Box<dyn Error>> {
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
        credentials.push(store.inspect(client_id, key_id)?);
    }
    Ok(credentials)
}

fn credentials_for_client(
    authority: &AuthorityLayout,
    store: &ClientCredentialStore<'_>,
    client_id: ClientId,
) -> Result<Vec<GeneratedClientCredential>, Box<dyn Error>> {
    let mut credentials = all_credentials(authority, store)?
        .into_iter()
        .filter(|credential| credential.client_id == client_id)
        .collect::<Vec<_>>();
    credentials.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(credentials)
}

fn bootstrap_marker_path(authority: &AuthorityLayout) -> PathBuf {
    authority.credential_dir().join(DESKTOP_BOOTSTRAP_MARKER)
}

fn create_bootstrap_marker(path: &Path, client_id: ClientId) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    writeln!(file, "{}", client_id.0)?;
    file.sync_all()
}

fn read_bootstrap_client_id(path: &Path) -> Result<ClientId, Box<dyn Error>> {
    let value = fs::read_to_string(path)?;
    let parsed = value.trim().parse::<u128>()?;
    if parsed == 0 {
        return Err("desktop bootstrap client id marker contains zero".into());
    }
    Ok(ClientId(parsed))
}

fn clear_bootstrap_marker(authority: &AuthorityLayout) -> Result<(), Box<dyn Error>> {
    let path = bootstrap_marker_path(authority);
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn clear_bootstrap_marker_if_present(authority: &AuthorityLayout) -> Result<(), Box<dyn Error>> {
    clear_bootstrap_marker(authority)
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
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golam-desktop-host-{}-{t}-{n}", std::process::id())),
        )
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    #[test]
    fn credential_filename_parser_rejects_zero_and_malformed_ids() {
        assert!(
            parse_credential_filename("not-a-credential")
                .unwrap()
                .is_none()
        );
        assert!(
            parse_credential_filename(&format!("{}-{}.gkey", "0".repeat(32), "1".repeat(64)))
                .is_err()
        );
        assert!(
            parse_credential_filename(&format!("{}-{}.gkey", "1".repeat(32), "0".repeat(64)))
                .is_err()
        );
    }

    #[test]
    fn pending_desktop_bootstrap_credential_is_resumed_without_duplication() {
        let (runtime, authority) = runtime();
        let store = ClientCredentialStore::new(&authority);
        let registry = ClientRegistry::open(&authority).unwrap();
        let (first, first_pending) = bootstrap_credential(&authority, &store, &registry).unwrap();
        let (second, second_pending) = bootstrap_credential(&authority, &store, &registry).unwrap();
        assert!(first_pending && second_pending);
        assert_eq!(first, second);
        assert_eq!(
            credentials_for_client(&authority, &store, first.client_id)
                .unwrap()
                .len(),
            1
        );
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn bootstrap_marker_rejects_zero_identity() {
        let (runtime, authority) = runtime();
        let marker = bootstrap_marker_path(&authority);
        fs::write(&marker, "0\n").unwrap();
        assert!(read_bootstrap_client_id(&marker).is_err());
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
