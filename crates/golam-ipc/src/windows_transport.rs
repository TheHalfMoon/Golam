use std::error::Error;
use std::fmt;
use std::io;
use std::num::NonZeroU8;

use golam_core::ResourceLimits;
use golam_core::paths::{ProtectedPathError, RuntimeLayout, windows_current_process_sid_string};
use interprocess::os::windows::named_pipe::{
    PipeListener, PipeListenerOptions, PipeStream, pipe_mode,
};
use interprocess::os::windows::security_descriptor::SecurityDescriptor;
use widestring::U16CString;

const PIPE_PREFIX: &str = r"\\.\pipe\golamd-";
const MAX_PIPE_PATH_UTF16: usize = 240;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowsPeerIdentity {
    pub process_id: u32,
    pub session_id: u32,
}

#[derive(Debug)]
pub enum WindowsTransportError {
    Io(io::Error),
    ProtectedPath(ProtectedPathError),
    InvalidInstanceLimit(u16),
    InvalidPipePath,
    PipePathTooLong {
        utf16_units: usize,
        maximum: usize,
    },
    InvalidPeerProcessId,
    PeerMetadataMismatch {
        client_pid: u32,
        peer_pid: u32,
    },
}

impl fmt::Display for WindowsTransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Windows IPC I/O error: {error}"),
            Self::ProtectedPath(error) => write!(f, "Windows IPC protected-path error: {error}"),
            Self::InvalidInstanceLimit(limit) => write!(
                f,
                "Windows named-pipe instance limit must be between 2 and 254; got {limit}"
            ),
            Self::InvalidPipePath => f.write_str("Windows named-pipe path contains an invalid NUL"),
            Self::PipePathTooLong {
                utf16_units,
                maximum,
            } => write!(
                f,
                "Windows named-pipe path is too long: {utf16_units} UTF-16 units; maximum is {maximum}"
            ),
            Self::InvalidPeerProcessId => {
                f.write_str("Windows named-pipe peer process id must be non-zero")
            }
            Self::PeerMetadataMismatch {
                client_pid,
                peer_pid,
            } => write!(
                f,
                "Windows named-pipe peer metadata mismatch: client PID {client_pid}, peer PID {peer_pid}"
            ),
        }
    }
}

impl Error for WindowsTransportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::ProtectedPath(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for WindowsTransportError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ProtectedPathError> for WindowsTransportError {
    fn from(value: ProtectedPathError) -> Self {
        Self::ProtectedPath(value)
    }
}

pub struct AcceptedWindowsPeer {
    pub stream: PipeStream<pipe_mode::Bytes, pipe_mode::Bytes>,
    pub identity: WindowsPeerIdentity,
}

pub struct WindowsPipeListener {
    listener: PipeListener<pipe_mode::Bytes, pipe_mode::Bytes>,
    pipe_path: String,
    owner_sid: String,
}

impl WindowsPipeListener {
    pub fn bind(
        layout: &RuntimeLayout,
        limits: ResourceLimits,
    ) -> Result<Self, WindowsTransportError> {
        layout.require_authority_ready()?;
        let owner_sid = windows_current_process_sid_string()?;
        let pipe_path = format!("{PIPE_PREFIX}{owner_sid}");
        validate_pipe_path(&pipe_path)?;
        let instance_limit = instance_limit(limits.max_concurrent_clients)?;

        let sddl = format!("D:P(A;;GA;;;{owner_sid})");
        let wide_sddl =
            U16CString::from_str(&sddl).map_err(|_| WindowsTransportError::InvalidPipePath)?;
        let descriptor = SecurityDescriptor::deserialize(wide_sddl.as_ucstr())?;
        let listener = PipeListenerOptions::new()
            .path(pipe_path.as_str())
            .accept_remote(false)
            .inheritable(false)
            .instance_limit(Some(instance_limit))
            .security_descriptor(Some(descriptor))
            .create_duplex::<pipe_mode::Bytes>()?;

        Ok(Self {
            listener,
            pipe_path,
            owner_sid,
        })
    }

    pub fn pipe_path(&self) -> &str {
        &self.pipe_path
    }

    pub fn owner_sid(&self) -> &str {
        &self.owner_sid
    }

    pub fn accept(&self) -> Result<AcceptedWindowsPeer, WindowsTransportError> {
        let stream = self.listener.accept()?;
        let client_pid = stream.client_process_id()?;
        let peer_pid = stream.peer_process_id()?;
        if client_pid == 0 || peer_pid == 0 {
            return Err(WindowsTransportError::InvalidPeerProcessId);
        }
        if client_pid != peer_pid {
            return Err(WindowsTransportError::PeerMetadataMismatch {
                client_pid,
                peer_pid,
            });
        }
        let session_id = stream.client_session_id()?;
        Ok(AcceptedWindowsPeer {
            stream,
            identity: WindowsPeerIdentity {
                process_id: client_pid,
                session_id,
            },
        })
    }
}

fn instance_limit(limit: u16) -> Result<NonZeroU8, WindowsTransportError> {
    if !(2..=254).contains(&limit) {
        return Err(WindowsTransportError::InvalidInstanceLimit(limit));
    }
    NonZeroU8::new(u8::try_from(limit).expect("validated Windows pipe instance limit fits u8"))
        .ok_or(WindowsTransportError::InvalidInstanceLimit(limit))
}

fn validate_pipe_path(path: &str) -> Result<(), WindowsTransportError> {
    let utf16_units = path.encode_utf16().count();
    if utf16_units > MAX_PIPE_PATH_UTF16 {
        return Err(WindowsTransportError::PipePathTooLong {
            utf16_units,
            maximum: MAX_PIPE_PATH_UTF16,
        });
    }
    if path.contains('\0') {
        return Err(WindowsTransportError::InvalidPipePath);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_layout() -> RuntimeLayout {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "golam-win-pipe-{}-{nanos}-{counter}",
            std::process::id()
        ));
        RuntimeLayout::initialize(root).unwrap()
    }

    #[test]
    fn current_user_acl_pipe_accepts_local_client_and_reports_peer_metadata() {
        let layout = test_layout();
        layout.require_authority_ready().unwrap();
        let listener = WindowsPipeListener::bind(&layout, ResourceLimits::default()).unwrap();
        assert!(listener.pipe_path().starts_with(PIPE_PREFIX));
        assert!(listener.owner_sid().starts_with("S-1-"));

        let path = listener.pipe_path().to_string();
        let client = thread::spawn(move || {
            PipeStream::<pipe_mode::Bytes, pipe_mode::Bytes>::connect_by_path(path.as_str())
                .unwrap()
        });
        let accepted = listener.accept().unwrap();
        let client_stream = client.join().unwrap();

        assert_eq!(accepted.identity.process_id, std::process::id());
        assert_eq!(
            accepted.stream.peer_process_id().unwrap(),
            std::process::id()
        );
        assert_eq!(client_stream.peer_process_id().unwrap(), std::process::id());
        assert_eq!(
            accepted.identity.session_id,
            accepted.stream.peer_session_id().unwrap()
        );
        drop(client_stream);
        drop(accepted);
        drop(listener);
        fs::remove_dir_all(layout.root).unwrap();
    }

    #[test]
    fn invalid_instance_limits_fail_closed() {
        assert!(matches!(
            instance_limit(1),
            Err(WindowsTransportError::InvalidInstanceLimit(1))
        ));
        assert!(matches!(
            instance_limit(255),
            Err(WindowsTransportError::InvalidInstanceLimit(255))
        ));
    }

    #[test]
    fn overlong_pipe_names_fail_before_creation() {
        let path = format!("{PIPE_PREFIX}{}", "x".repeat(MAX_PIPE_PATH_UTF16));
        assert!(matches!(
            validate_pipe_path(&path),
            Err(WindowsTransportError::PipePathTooLong { .. })
        ));
    }
}
