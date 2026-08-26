use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use golam_core::paths::{ProtectedPathError, RuntimeLayout};
use nix::sys::socket::{getsockopt, sockopt};
use nix::unistd::Uid;

const SOCKET_NAME: &str = "golamd.sock";

#[cfg(target_vendor = "apple")]
const MAX_UNIX_SOCKET_PATH_BYTES: usize = 103;

#[cfg(any(target_os = "linux", target_os = "android"))]
const MAX_UNIX_SOCKET_PATH_BYTES: usize = 107;

#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
const MAX_UNIX_SOCKET_PATH_BYTES: usize = 103;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PeerIdentity {
    pub uid: u32,
    pub gid: Option<u32>,
    pub pid: Option<i32>,
}

#[derive(Debug)]
pub enum UnixTransportError {
    Io(io::Error),
    ProtectedPath(ProtectedPathError),
    SocketPathExists(PathBuf),
    SocketPathTooLong {
        path: PathBuf,
        bytes: usize,
        maximum: usize,
    },
    SocketPathNotSocket(PathBuf),
    SocketPermissionsTooBroad {
        path: PathBuf,
        mode: u32,
    },
    RuntimePermissionsTooBroad {
        path: PathBuf,
        mode: u32,
    },
    PeerCredentialsUnavailable,
    InvalidPeerPid(i32),
    PeerUidMismatch {
        expected: u32,
        actual: u32,
    },
}

impl fmt::Display for UnixTransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Unix IPC I/O error: {error}"),
            Self::ProtectedPath(error) => write!(f, "Unix IPC protected-path error: {error}"),
            Self::SocketPathExists(path) => {
                write!(f, "Unix IPC socket path already exists: {}", path.display())
            }
            Self::SocketPathTooLong {
                path,
                bytes,
                maximum,
            } => write!(
                f,
                "Unix IPC socket path is too long at {}: {bytes} bytes; maximum is {maximum}",
                path.display()
            ),
            Self::SocketPathNotSocket(path) => {
                write!(f, "Unix IPC path is not a socket: {}", path.display())
            }
            Self::SocketPermissionsTooBroad { path, mode } => write!(
                f,
                "Unix IPC socket permissions are too broad at {}: {mode:o}",
                path.display()
            ),
            Self::RuntimePermissionsTooBroad { path, mode } => write!(
                f,
                "Unix IPC runtime directory permissions are too broad at {}: {mode:o}",
                path.display()
            ),
            Self::PeerCredentialsUnavailable => {
                f.write_str("Unix IPC peer credentials are unavailable on this platform")
            }
            Self::InvalidPeerPid(pid) => write!(f, "Unix IPC peer PID is invalid: {pid}"),
            Self::PeerUidMismatch { expected, actual } => write!(
                f,
                "Unix IPC peer UID mismatch: expected {expected}, got {actual}"
            ),
        }
    }
}

impl Error for UnixTransportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::ProtectedPath(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for UnixTransportError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ProtectedPathError> for UnixTransportError {
    fn from(value: ProtectedPathError) -> Self {
        Self::ProtectedPath(value)
    }
}

pub struct AcceptedUnixPeer {
    pub stream: UnixStream,
    pub identity: PeerIdentity,
}

pub fn connect_same_user(layout: &RuntimeLayout) -> Result<UnixStream, UnixTransportError> {
    layout.require_authority_ready()?;
    verify_runtime_directory(&layout.runtime_dir)?;
    let socket_path = layout.runtime_dir.join(SOCKET_NAME);
    validate_socket_path_length(&socket_path)?;
    verify_socket_permissions(&socket_path)?;

    let stream = UnixStream::connect(&socket_path)?;
    let identity = peer_identity(&stream)?;
    let expected_uid = Uid::effective().as_raw();
    if identity.uid != expected_uid {
        return Err(UnixTransportError::PeerUidMismatch {
            expected: expected_uid,
            actual: identity.uid,
        });
    }
    if let Some(pid) = identity.pid
        && pid <= 0
    {
        return Err(UnixTransportError::InvalidPeerPid(pid));
    }
    Ok(stream)
}

pub struct UnixTransportListener {
    listener: UnixListener,
    socket_path: PathBuf,
    expected_uid: u32,
}

impl UnixTransportListener {
    pub fn bind(layout: &RuntimeLayout) -> Result<Self, UnixTransportError> {
        layout.require_authority_ready()?;
        verify_runtime_directory(&layout.runtime_dir)?;

        let socket_path = layout.runtime_dir.join(SOCKET_NAME);
        validate_socket_path_length(&socket_path)?;
        match fs::symlink_metadata(&socket_path) {
            Ok(_) => return Err(UnixTransportError::SocketPathExists(socket_path)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        let listener = UnixListener::bind(&socket_path)?;
        if let Err(error) = set_and_verify_socket_permissions(&socket_path) {
            drop(listener);
            let _ = fs::remove_file(&socket_path);
            return Err(error);
        }

        Ok(Self {
            listener,
            socket_path,
            expected_uid: Uid::effective().as_raw(),
        })
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn expected_uid(&self) -> u32 {
        self.expected_uid
    }

    pub fn accept_same_user(&self) -> Result<AcceptedUnixPeer, UnixTransportError> {
        let (stream, _) = self.listener.accept()?;
        let identity = peer_identity(&stream)?;
        if identity.uid != self.expected_uid {
            return Err(UnixTransportError::PeerUidMismatch {
                expected: self.expected_uid,
                actual: identity.uid,
            });
        }
        if let Some(pid) = identity.pid
            && pid <= 0
        {
            return Err(UnixTransportError::InvalidPeerPid(pid));
        }
        Ok(AcceptedUnixPeer { stream, identity })
    }
}

impl Drop for UnixTransportListener {
    fn drop(&mut self) {
        if let Ok(metadata) = fs::symlink_metadata(&self.socket_path)
            && metadata.file_type().is_socket()
        {
            let _ = fs::remove_file(&self.socket_path);
        }
    }
}

fn verify_runtime_directory(path: &Path) -> Result<(), UnixTransportError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() {
        return Err(
            io::Error::new(io::ErrorKind::NotADirectory, path.display().to_string()).into(),
        );
    }
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(UnixTransportError::RuntimePermissionsTooBroad {
            path: path.to_path_buf(),
            mode,
        });
    }
    Ok(())
}

fn validate_socket_path_length(path: &Path) -> Result<(), UnixTransportError> {
    let bytes = path.as_os_str().as_bytes().len();
    if bytes > MAX_UNIX_SOCKET_PATH_BYTES {
        return Err(UnixTransportError::SocketPathTooLong {
            path: path.to_path_buf(),
            bytes,
            maximum: MAX_UNIX_SOCKET_PATH_BYTES,
        });
    }
    Ok(())
}

fn set_and_verify_socket_permissions(path: &Path) -> Result<(), UnixTransportError> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    verify_socket_permissions(path)
}

fn verify_socket_permissions(path: &Path) -> Result<(), UnixTransportError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_socket() {
        return Err(UnixTransportError::SocketPathNotSocket(path.to_path_buf()));
    }
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(UnixTransportError::SocketPermissionsTooBroad {
            path: path.to_path_buf(),
            mode,
        });
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn peer_identity(stream: &UnixStream) -> Result<PeerIdentity, UnixTransportError> {
    let credentials = getsockopt(stream, sockopt::PeerCredentials).map_err(io::Error::from)?;
    let pid = credentials.pid();
    if pid <= 0 {
        return Err(UnixTransportError::InvalidPeerPid(pid));
    }
    Ok(PeerIdentity {
        uid: credentials.uid(),
        gid: Some(credentials.gid()),
        pid: Some(pid),
    })
}

#[cfg(target_vendor = "apple")]
fn peer_identity(stream: &UnixStream) -> Result<PeerIdentity, UnixTransportError> {
    let credentials = getsockopt(stream, sockopt::LocalPeerCred).map_err(io::Error::from)?;
    let pid = getsockopt(stream, sockopt::LocalPeerPid).map_err(io::Error::from)?;
    if pid <= 0 {
        return Err(UnixTransportError::InvalidPeerPid(pid));
    }
    Ok(PeerIdentity {
        uid: credentials.uid(),
        gid: None,
        pid: Some(pid),
    })
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
fn peer_identity(_stream: &UnixStream) -> Result<PeerIdentity, UnixTransportError> {
    Err(UnixTransportError::PeerCredentialsUnavailable)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let root = PathBuf::from("/tmp").join(format!(
            "golam-uds-{}-{nanos}-{counter}",
            std::process::id()
        ));
        RuntimeLayout::initialize(root).unwrap()
    }

    fn remove_layout(layout: &RuntimeLayout) {
        let _ = fs::remove_dir_all(&layout.root);
    }

    #[test]
    fn listener_is_private_and_client_verifies_same_user_peer() {
        let layout = test_layout();
        let listener = UnixTransportListener::bind(&layout).unwrap();
        let path = listener.socket_path().to_path_buf();
        let metadata = fs::symlink_metadata(&path).unwrap();
        assert!(metadata.file_type().is_socket());
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        assert_eq!(
            fs::metadata(&layout.runtime_dir)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );

        let client_layout = layout.clone();
        let client = thread::spawn(move || connect_same_user(&client_layout).unwrap());
        let accepted = listener.accept_same_user().unwrap();
        let client_stream = client.join().unwrap();

        assert_eq!(accepted.identity.uid, Uid::effective().as_raw());
        assert_eq!(accepted.identity.pid, Some(std::process::id() as i32));
        assert_eq!(peer_identity(&client_stream).unwrap().uid, Uid::effective().as_raw());
        drop(client_stream);
        drop(accepted);
        drop(listener);
        assert!(!path.exists());
        remove_layout(&layout);
    }

    #[test]
    fn existing_socket_path_fails_closed_without_unlinking() {
        let layout = test_layout();
        let path = layout.runtime_dir.join(SOCKET_NAME);
        fs::write(&path, b"do-not-delete").unwrap();

        assert!(matches!(
            UnixTransportListener::bind(&layout),
            Err(UnixTransportError::SocketPathExists(existing)) if existing == path
        ));
        assert_eq!(fs::read(&path).unwrap(), b"do-not-delete");
        remove_layout(&layout);
    }

    #[test]
    fn client_rejects_non_socket_endpoint_before_connect() {
        let layout = test_layout();
        let path = layout.runtime_dir.join(SOCKET_NAME);
        fs::write(&path, b"not-a-socket").unwrap();
        assert!(matches!(
            connect_same_user(&layout),
            Err(UnixTransportError::SocketPathNotSocket(existing)) if existing == path
        ));
        remove_layout(&layout);
    }

    #[test]
    fn overlong_socket_path_fails_before_bind() {
        let path = PathBuf::from("/tmp").join("x".repeat(MAX_UNIX_SOCKET_PATH_BYTES));
        assert!(matches!(
            validate_socket_path_length(&path),
            Err(UnixTransportError::SocketPathTooLong {
                bytes,
                maximum,
                ..
            }) if bytes > maximum && maximum == MAX_UNIX_SOCKET_PATH_BYTES
        ));
    }
}
