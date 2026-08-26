#![forbid(unsafe_code)]

use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::thread;
use std::time::{Duration, Instant};

use golam_core::authority::AuthorityLayout;
use golam_core::paths::RuntimeLayout;
use golam_core::{ClientId, PROTOCOL_VERSION, ResourceLimits};
use golam_ipc::client_handshake::authenticate_client;
use golam_ipc::command::{Command, encode_command};
use golam_ipc::credentials::{ClientCredentialStore, GeneratedClientCredential};
use golam_ipc::lifecycle::{ClientKeyId, LifecycleMessage, LifecyclePhase, ShutdownReason};
use golam_ipc::request::{
    ClientAction, ReplyMessage, RequestId, ServerAction, ServerRequestTracker, encode_request,
};
use golam_ipc::wire::{read_frame, write_frame};
use golam_ipc::{FrameHeader, FrameKind};

const IPC_DEADLINE: Duration = Duration::from_secs(30);
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
                    "Golam CLI local IPC deadline exceeded",
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
                    self.wait_for_progress()?
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
                    self.wait_for_progress()?
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn enroll(runtime: &RuntimeLayout, client_id: ClientId) -> Result<String, Box<dyn Error>> {
    let authority = AuthorityLayout::initialize(runtime)?;
    let store = ClientCredentialStore::new(&authority);
    let credential = credential_for_enrollment(&authority, &store, client_id)?;
    let signing_key = store.load(credential.client_id, credential.key_id)?;
    let limits = ResourceLimits::default();

    #[cfg(unix)]
    let stream = golam_ipc::unix_transport::connect_same_user(runtime)?;
    #[cfg(windows)]
    let stream = golam_ipc::windows_transport::connect_current_user(runtime)?;
    #[cfg(not(any(unix, windows)))]
    return Err("Golam local IPC is unsupported on this platform".into());

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

    Ok(format!(
        "client_id={} key_id={} credential_path={} enrolled=true\n",
        credential.client_id.0,
        hex(&credential.key_id.0),
        credential.path.display()
    ))
}

pub fn execute(runtime: &RuntimeLayout, command: &Command) -> Result<ReplyMessage, Box<dyn Error>> {
    let authority = AuthorityLayout::initialize(runtime)?;
    let store = ClientCredentialStore::new(&authority);
    let credential = single_execution_credential(&authority, &store)?;
    let signing_key = store.load(credential.client_id, credential.key_id)?;
    let limits = ResourceLimits::default();

    #[cfg(unix)]
    let stream = golam_ipc::unix_transport::connect_same_user(runtime)?;
    #[cfg(windows)]
    let stream = golam_ipc::windows_transport::connect_current_user(runtime)?;
    #[cfg(not(any(unix, windows)))]
    return Err("Golam local IPC is unsupported on this platform".into());

    stream.set_nonblocking(true)?;
    let mut stream = DeadlineIo::new(stream, IPC_DEADLINE);
    let ready = authenticate_client(
        &mut stream,
        credential.client_id,
        credential.key_id,
        &signing_key,
        limits,
    )?;
    let mut tracker = ServerRequestTracker::new(LifecyclePhase::Ready, ready.limits)?;
    let request_id = RequestId(1);
    let message = encode_command(command)?;
    let payload = encode_request(&message)?;
    let header = FrameHeader {
        protocol_version: PROTOCOL_VERSION,
        kind: FrameKind::Request,
        request_id: Some(request_id.0),
        payload_len: u32::try_from(payload.len()).expect("request payload length fits u32"),
    };
    match tracker.receive_client_frame(header, &payload)? {
        ClientAction::Begin {
            request_id: tracked_id,
            method,
        } => {
            debug_assert_eq!(tracked_id, request_id);
            debug_assert_eq!(method, message.method);
        }
        ClientAction::Cancel { .. } | ClientAction::CancelAlreadyRequested { .. } => {
            unreachable!("request frame cannot decode as cancel")
        }
    }
    write_frame(&mut stream, header, &payload, ready.limits)?;

    let reply_frame = read_frame(&mut stream, ready.limits)?;
    let reply = match tracker.settle_server_frame(reply_frame.header, &reply_frame.payload)? {
        ServerAction::Reply {
            settlement,
            message,
        } => {
            if settlement.request_id != request_id {
                return Err("daemon reply settled a different request id".into());
            }
            message
        }
        ServerAction::Event => {
            return Err("unexpected daemon event while awaiting CLI reply".into());
        }
    };
    write_shutdown(&mut stream, ready.limits)?;
    Ok(reply)
}

fn credential_for_enrollment(
    authority: &AuthorityLayout,
    store: &ClientCredentialStore<'_>,
    client_id: ClientId,
) -> Result<GeneratedClientCredential, Box<dyn Error>> {
    let matches = credentials_matching_client(authority, store, client_id)?;
    match matches.len() {
        0 => Ok(store.generate(client_id)?),
        1 => Ok(matches.into_iter().next().expect("one credential exists")),
        count => Err(format!(
            "found {count} protected credentials for client {}; refusing ambiguous enrollment",
            client_id.0
        )
        .into()),
    }
}

fn single_execution_credential(
    authority: &AuthorityLayout,
    store: &ClientCredentialStore<'_>,
) -> Result<GeneratedClientCredential, Box<dyn Error>> {
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
    credentials.sort_by(|left, right| left.path.cmp(&right.path));
    match credentials.len() {
        0 => Err("no protected CLI credential exists; run `golam client enroll <client-id>` first".into()),
        1 => Ok(credentials.remove(0)),
        count => Err(format!(
            "found {count} protected client credentials; the minimal Spec 002 CLI requires exactly one"
        )
        .into()),
    }
}

fn credentials_matching_client(
    authority: &AuthorityLayout,
    store: &ClientCredentialStore<'_>,
    client_id: ClientId,
) -> Result<Vec<GeneratedClientCredential>, Box<dyn Error>> {
    let mut credentials = Vec::new();
    for entry in fs::read_dir(authority.credential_dir())? {
        let entry = entry?;
        let name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };
        let Some((candidate_id, key_id)) = parse_credential_filename(&name)? else {
            continue;
        };
        if candidate_id == client_id {
            credentials.push(store.inspect(candidate_id, key_id)?);
        }
    }
    credentials.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(credentials)
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

fn decode_hex_32(value: &str) -> Result<[u8; 32], Box<dyn Error>> {
    if value.len() != 64 {
        return Err("client key id must contain exactly 64 hexadecimal characters".into());
    }
    let mut bytes = [0_u8; 32];
    let (chunks, remainder) = value.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return Err("client key id must contain an even number of hexadecimal characters".into());
    }
    for (index, chunk) in chunks.iter().enumerate() {
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

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[(byte >> 4) as usize]));
        out.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golam-cli-client-{}-{t}-{n}", std::process::id())),
        )
        .unwrap()
    }

    struct WouldBlockForever;

    impl Read for WouldBlockForever {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::WouldBlock))
        }
    }

    impl Write for WouldBlockForever {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::WouldBlock))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn stalled_local_peer_is_bounded() {
        let mut stream = DeadlineIo::new(WouldBlockForever, Duration::from_millis(10));
        let error = stream.read(&mut [0_u8; 1]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }

    #[test]
    fn enrollment_resumes_exactly_one_existing_protected_credential() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(3002)).unwrap();
        let resumed = credential_for_enrollment(&authority, &store, ClientId(3002)).unwrap();
        assert_eq!(resumed, generated);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn execution_discovers_exactly_one_verified_credential() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let store = ClientCredentialStore::new(&authority);
        let generated = store.generate(ClientId(3003)).unwrap();
        assert_eq!(
            single_execution_credential(&authority, &store).unwrap(),
            generated
        );
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn malformed_key_id_fails_closed() {
        assert!(decode_hex_32("00").is_err());
        assert!(decode_hex_32(&"g".repeat(64)).is_err());
        assert!(decode_hex_32(&"0".repeat(64)).is_err());
    }
}
