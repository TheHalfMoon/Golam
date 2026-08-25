use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactReceipt {
    pub hash: [u8; 32],
    pub size_bytes: u64,
    pub relative_path: PathBuf,
}

#[derive(Debug)]
pub enum ArtifactError {
    Io(io::Error),
    Symlink(PathBuf),
    UnexpectedTempEntry(PathBuf),
    HashMismatch(PathBuf),
    SizeMismatch {
        path: PathBuf,
        expected: u64,
        actual: u64,
    },
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "artifact I/O error: {error}"),
            Self::Symlink(path) => write!(f, "artifact path is a symlink: {}", path.display()),
            Self::UnexpectedTempEntry(path) => {
                write!(f, "unexpected artifact temp entry: {}", path.display())
            }
            Self::HashMismatch(path) => {
                write!(f, "artifact hash mismatch: {}", path.display())
            }
            Self::SizeMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "artifact size mismatch at {}: expected {expected}, actual {actual}",
                path.display()
            ),
        }
    }
}

impl Error for ArtifactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Symlink(_)
            | Self::UnexpectedTempEntry(_)
            | Self::HashMismatch(_)
            | Self::SizeMismatch { .. } => None,
        }
    }
}

impl From<io::Error> for ArtifactError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactStore {
    root: PathBuf,
    temp_dir: PathBuf,
}

impl ArtifactStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ArtifactError> {
        let root = root.as_ref().to_path_buf();
        ensure_directory(&root)?;
        let temp_dir = root.join(".tmp");
        ensure_directory(&temp_dir)?;
        Ok(Self { root, temp_dir })
    }

    pub fn install_bytes(&self, bytes: &[u8]) -> Result<ArtifactReceipt, ArtifactError> {
        let hash = *blake3::hash(bytes).as_bytes();
        let hex = hash_hex(hash);
        let prefix_dir = self.root.join(&hex[..2]);
        ensure_directory(&prefix_dir)?;
        let final_path = prefix_dir.join(&hex);
        let relative_path = PathBuf::from(&hex[..2]).join(&hex);
        let size_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        let receipt = ArtifactReceipt {
            hash,
            size_bytes,
            relative_path,
        };

        if final_path.exists() {
            verify_path(&final_path, &receipt)?;
            return Ok(receipt);
        }

        let temp_path = self.next_temp_path(&hex);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        verify_path(&temp_path, &receipt)?;

        match fs::hard_link(&temp_path, &final_path) {
            Ok(()) => {
                set_installed_read_only(&final_path)?;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                verify_path(&final_path, &receipt)?;
            }
            Err(error) => {
                let _ = fs::remove_file(&temp_path);
                return Err(error.into());
            }
        }

        fs::remove_file(&temp_path)?;
        verify_path(&final_path, &receipt)?;
        Ok(receipt)
    }

    pub fn verify(&self, receipt: &ArtifactReceipt) -> Result<(), ArtifactError> {
        verify_path(&self.root.join(&receipt.relative_path), receipt)
    }

    pub fn read_verified(&self, receipt: &ArtifactReceipt) -> Result<Vec<u8>, ArtifactError> {
        let path = self.root.join(&receipt.relative_path);
        verify_path(&path, receipt)?;
        Ok(fs::read(path)?)
    }

    pub fn cleanup_temps(&self) -> Result<usize, ArtifactError> {
        let mut removed = 0_usize;
        for entry in fs::read_dir(&self.temp_dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink() || metadata.is_file() {
                fs::remove_file(path)?;
                removed += 1;
            } else {
                return Err(ArtifactError::UnexpectedTempEntry(path));
            }
        }
        Ok(removed)
    }

    fn next_temp_path(&self, hash_hex: &str) -> PathBuf {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        self.temp_dir.join(format!(
            "{hash_hex}.{}.{}.{counter}.tmp",
            std::process::id(),
            nanos
        ))
    }
}

fn ensure_directory(path: &Path) -> Result<(), ArtifactError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(ArtifactError::Symlink(path.to_path_buf()));
            }
            if !metadata.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::NotADirectory,
                    path.display().to_string(),
                )
                .into());
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(path)?,
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

fn verify_path(path: &Path, receipt: &ArtifactReceipt) -> Result<(), ArtifactError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(ArtifactError::Symlink(path.to_path_buf()));
    }
    if metadata.len() != receipt.size_bytes {
        return Err(ArtifactError::SizeMismatch {
            path: path.to_path_buf(),
            expected: receipt.size_bytes,
            actual: metadata.len(),
        });
    }

    let actual_hash = hash_file(path)?;
    if actual_hash != receipt.hash {
        return Err(ArtifactError::HashMismatch(path.to_path_buf()));
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<[u8; 32], ArtifactError> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(*hasher.finalize().as_bytes())
}

fn set_installed_read_only(path: &Path) -> Result<(), ArtifactError> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

fn hash_hex(hash: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in hash {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_root() -> PathBuf {
        let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "golam-artifacts-{}-{nanos}-{counter}",
            std::process::id()
        ))
    }

    #[test]
    fn install_is_content_addressed_verified_and_idempotent() {
        let root = unique_root();
        let store = ArtifactStore::open(&root).unwrap();
        let first = store.install_bytes(b"checkpoint bytes").unwrap();
        let second = store.install_bytes(b"checkpoint bytes").unwrap();
        assert_eq!(first, second);
        store.verify(&first).unwrap();
        assert_eq!(store.cleanup_temps().unwrap(), 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cleanup_removes_only_temp_files() {
        let root = unique_root();
        let store = ArtifactStore::open(&root).unwrap();
        let stale = store.temp_dir.join("stale.tmp");
        fs::write(&stale, b"partial").unwrap();
        assert_eq!(store.cleanup_temps().unwrap(), 1);
        assert!(!stale.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
