#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use crate::git_sha1::{GitObjectSha1, GitObjectSha1Error};

pub const MAX_INDEX_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_INDEX_ENTRIES: usize = 1_000_000;
pub const MAX_INDEX_PATH_BYTES: usize = 4096;
pub const MAX_INDEX_EXTENSIONS: usize = 128;
const INDEX_HEADER_BYTES: usize = 12;
const SHA1_BYTES: usize = 20;
const ENTRY_FIXED_V2_BYTES: usize = 62;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitIndexBounds {
    pub max_bytes: usize,
    pub max_entries: usize,
    pub max_path_bytes: usize,
    pub max_extensions: usize,
}

impl Default for GitIndexBounds {
    fn default() -> Self {
        Self {
            max_bytes: MAX_INDEX_BYTES,
            max_entries: MAX_INDEX_ENTRIES,
            max_path_bytes: MAX_INDEX_PATH_BYTES,
            max_extensions: MAX_INDEX_EXTENSIONS,
        }
    }
}

impl GitIndexBounds {
    pub fn validate(self) -> Result<(), GitIndexError> {
        if self.max_bytes < INDEX_HEADER_BYTES + SHA1_BYTES
            || self.max_bytes > MAX_INDEX_BYTES
            || self.max_entries == 0
            || self.max_entries > MAX_INDEX_ENTRIES
            || self.max_path_bytes == 0
            || self.max_path_bytes > MAX_INDEX_PATH_BYTES
            || self.max_extensions == 0
            || self.max_extensions > MAX_INDEX_EXTENSIONS
        {
            return Err(GitIndexError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitIndexVersion {
    V2,
    V3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitIndexMode {
    RegularFile { executable: bool },
    SymbolicLink,
    Gitlink,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitIndexEntry {
    pub ctime_seconds: u32,
    pub ctime_nanoseconds: u32,
    pub mtime_seconds: u32,
    pub mtime_nanoseconds: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: GitIndexMode,
    pub uid: u32,
    pub gid: u32,
    pub file_size: u32,
    pub object_id: [u8; SHA1_BYTES],
    pub assume_valid: bool,
    pub stage: u8,
    pub skip_worktree: bool,
    pub intent_to_add: bool,
    pub path: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitIndexExtension {
    pub signature: [u8; 4],
    pub size: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitIndex {
    pub version: GitIndexVersion,
    pub entries: Vec<GitIndexEntry>,
    pub extensions: Vec<GitIndexExtension>,
    pub checksum: [u8; SHA1_BYTES],
}

pub fn parse_git_index(bytes: &[u8], bounds: GitIndexBounds) -> Result<GitIndex, GitIndexError> {
    bounds.validate()?;
    if bytes.len() > bounds.max_bytes {
        return Err(GitIndexError::ByteLimitExceeded);
    }
    if bytes.len() < INDEX_HEADER_BYTES + SHA1_BYTES {
        return Err(GitIndexError::Truncated);
    }

    let content_len = bytes
        .len()
        .checked_sub(SHA1_BYTES)
        .ok_or(GitIndexError::Truncated)?;
    let checksum: [u8; SHA1_BYTES] = bytes[content_len..]
        .try_into()
        .map_err(|_| GitIndexError::Truncated)?;
    let actual_checksum = GitObjectSha1::digest(&bytes[..content_len])?;
    if actual_checksum != checksum {
        return Err(GitIndexError::ChecksumMismatch);
    }

    let mut cursor = Cursor::new(&bytes[..content_len]);
    if cursor.take(4)? != b"DIRC" {
        return Err(GitIndexError::InvalidSignature);
    }
    let version = match cursor.read_u32()? {
        2 => GitIndexVersion::V2,
        3 => GitIndexVersion::V3,
        other => return Err(GitIndexError::UnsupportedVersion(other)),
    };

    let entry_count =
        usize::try_from(cursor.read_u32()?).map_err(|_| GitIndexError::EntryLimitExceeded)?;
    if entry_count > bounds.max_entries {
        return Err(GitIndexError::EntryLimitExceeded);
    }

    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let entry = parse_entry(&mut cursor, version, bounds.max_path_bytes)?;
        if let Some(previous) = entries.last() {
            validate_entry_order(previous, &entry)?;
        }
        entries.push(entry);
    }

    let mut extensions = Vec::new();
    while cursor.remaining() != 0 {
        if extensions.len() >= bounds.max_extensions {
            return Err(GitIndexError::ExtensionLimitExceeded);
        }
        let signature: [u8; 4] = cursor
            .take(4)?
            .try_into()
            .map_err(|_| GitIndexError::Truncated)?;
        let size = cursor.read_u32()?;
        let size_usize =
            usize::try_from(size).map_err(|_| GitIndexError::ExtensionSizeOverflow)?;
        let _data = cursor.take(size_usize)?;

        match &signature {
            b"link" => return Err(GitIndexError::SplitIndexUnsupported),
            b"sdir" => return Err(GitIndexError::SparseIndexUnsupported),
            _ if !signature[0].is_ascii_uppercase() => {
                return Err(GitIndexError::UnknownMandatoryExtension(signature));
            }
            _ => {}
        }

        extensions.push(GitIndexExtension { signature, size });
    }

    Ok(GitIndex {
        version,
        entries,
        extensions,
        checksum,
    })
}

fn parse_entry(
    cursor: &mut Cursor<'_>,
    version: GitIndexVersion,
    max_path_bytes: usize,
) -> Result<GitIndexEntry, GitIndexError> {
    let entry_start = cursor.position();
    if cursor.remaining() < ENTRY_FIXED_V2_BYTES {
        return Err(GitIndexError::TruncatedEntry);
    }

    let ctime_seconds = cursor.read_u32()?;
    let ctime_nanoseconds = cursor.read_u32()?;
    let mtime_seconds = cursor.read_u32()?;
    let mtime_nanoseconds = cursor.read_u32()?;
    let dev = cursor.read_u32()?;
    let ino = cursor.read_u32()?;
    let mode_raw = cursor.read_u32()?;
    let mode = parse_mode(mode_raw)?;
    let uid = cursor.read_u32()?;
    let gid = cursor.read_u32()?;
    let file_size = cursor.read_u32()?;
    let object_id: [u8; SHA1_BYTES] = cursor
        .take(SHA1_BYTES)?
        .try_into()
        .map_err(|_| GitIndexError::TruncatedEntry)?;
    let flags = cursor.read_u16()?;
    let assume_valid = flags & 0x8000 != 0;
    let extended = flags & 0x4000 != 0;
    let stage = ((flags >> 12) & 0x3) as u8;
    let stored_path_len = usize::from(flags & 0x0fff);

    let (skip_worktree, intent_to_add) = if extended {
        if version == GitIndexVersion::V2 {
            return Err(GitIndexError::ExtendedFlagsInV2);
        }
        let extended_flags = cursor.read_u16()?;
        if extended_flags & !0x6000 != 0 {
            return Err(GitIndexError::InvalidExtendedFlags);
        }
        (
            extended_flags & 0x4000 != 0,
            extended_flags & 0x2000 != 0,
        )
    } else {
        (false, false)
    };

    let path_start = cursor.position();
    let path_end = cursor
        .remaining_slice()
        .iter()
        .take(max_path_bytes.saturating_add(1))
        .position(|byte| *byte == 0)
        .map(|relative| path_start + relative)
        .ok_or(GitIndexError::PathLimitExceeded)?;
    let path_len = path_end
        .checked_sub(path_start)
        .ok_or(GitIndexError::TruncatedEntry)?;
    if path_len == 0 || path_len > max_path_bytes {
        return Err(GitIndexError::InvalidPath);
    }
    if stored_path_len < 0x0fff && stored_path_len != path_len {
        return Err(GitIndexError::NonCanonicalPathLength);
    }
    if stored_path_len == 0x0fff && path_len < 0x0fff {
        return Err(GitIndexError::NonCanonicalPathLength);
    }

    let path = cursor.take(path_len)?.to_vec();
    if cursor.take(1)? != b"\0" {
        return Err(GitIndexError::TruncatedEntry);
    }
    validate_path(&path)?;

    let consumed = cursor
        .position()
        .checked_sub(entry_start)
        .ok_or(GitIndexError::TruncatedEntry)?;
    let padding = (8 - (consumed % 8)) % 8;
    let padding_bytes = cursor.take(padding)?;
    if padding_bytes.iter().any(|byte| *byte != 0) {
        return Err(GitIndexError::NonZeroPadding);
    }

    Ok(GitIndexEntry {
        ctime_seconds,
        ctime_nanoseconds,
        mtime_seconds,
        mtime_nanoseconds,
        dev,
        ino,
        mode,
        uid,
        gid,
        file_size,
        object_id,
        assume_valid,
        stage,
        skip_worktree,
        intent_to_add,
        path,
    })
}

fn parse_mode(mode: u32) -> Result<GitIndexMode, GitIndexError> {
    match mode {
        0o100644 => Ok(GitIndexMode::RegularFile { executable: false }),
        0o100755 => Ok(GitIndexMode::RegularFile { executable: true }),
        0o120000 => Ok(GitIndexMode::SymbolicLink),
        0o160000 => Ok(GitIndexMode::Gitlink),
        _ => Err(GitIndexError::UnsupportedMode(mode)),
    }
}

fn validate_path(path: &[u8]) -> Result<(), GitIndexError> {
    if path.is_empty()
        || path.first() == Some(&b'/')
        || path.last() == Some(&b'/')
        || path.contains(&0)
    {
        return Err(GitIndexError::InvalidPath);
    }

    for component in path.split(|byte| *byte == b'/') {
        if component.is_empty()
            || component == b"."
            || component == b".."
            || component.eq_ignore_ascii_case(b".git")
        {
            return Err(GitIndexError::InvalidPath);
        }
    }
    Ok(())
}

fn validate_entry_order(
    previous: &GitIndexEntry,
    current: &GitIndexEntry,
) -> Result<(), GitIndexError> {
    match previous.path.cmp(&current.path) {
        std::cmp::Ordering::Less => Ok(()),
        std::cmp::Ordering::Equal if previous.stage < current.stage => Ok(()),
        _ => Err(GitIndexError::EntriesNotStrictlySorted),
    }
}

#[derive(Debug)]
pub enum GitIndexError {
    InvalidBounds,
    ByteLimitExceeded,
    Truncated,
    InvalidSignature,
    UnsupportedVersion(u32),
    EntryLimitExceeded,
    ExtensionLimitExceeded,
    ExtensionSizeOverflow,
    TruncatedEntry,
    ExtendedFlagsInV2,
    InvalidExtendedFlags,
    PathLimitExceeded,
    InvalidPath,
    NonCanonicalPathLength,
    NonZeroPadding,
    UnsupportedMode(u32),
    EntriesNotStrictlySorted,
    SplitIndexUnsupported,
    SparseIndexUnsupported,
    UnknownMandatoryExtension([u8; 4]),
    ChecksumMismatch,
    Sha1(GitObjectSha1Error),
}

impl fmt::Display for GitIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBounds => f.write_str("Git index bounds exceed the first-profile limits"),
            Self::ByteLimitExceeded => f.write_str("Git index byte limit exceeded"),
            Self::Truncated => f.write_str("Git index is truncated"),
            Self::InvalidSignature => f.write_str("Git index signature is not DIRC"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported Git index version: {version}")
            }
            Self::EntryLimitExceeded => f.write_str("Git index entry limit exceeded"),
            Self::ExtensionLimitExceeded => f.write_str("Git index extension limit exceeded"),
            Self::ExtensionSizeOverflow => f.write_str("Git index extension size overflowed"),
            Self::TruncatedEntry => f.write_str("Git index entry is truncated"),
            Self::ExtendedFlagsInV2 => {
                f.write_str("Git index v2 entry illegally sets the extended flag")
            }
            Self::InvalidExtendedFlags => {
                f.write_str("Git index v3 entry contains reserved extended flag bits")
            }
            Self::PathLimitExceeded => f.write_str("Git index path length limit exceeded"),
            Self::InvalidPath => f.write_str("Git index entry path is invalid for the first profile"),
            Self::NonCanonicalPathLength => {
                f.write_str("Git index entry path length flags are non-canonical")
            }
            Self::NonZeroPadding => f.write_str("Git index entry padding is non-zero"),
            Self::UnsupportedMode(mode) => write!(f, "unsupported Git index mode: {mode:#o}"),
            Self::EntriesNotStrictlySorted => {
                f.write_str("Git index entries are not strictly sorted by path and stage")
            }
            Self::SplitIndexUnsupported => {
                f.write_str("split Git indexes are unsupported by the first profile")
            }
            Self::SparseIndexUnsupported => {
                f.write_str("sparse Git indexes are unsupported by the first profile")
            }
            Self::UnknownMandatoryExtension(signature) => write!(
                f,
                "unknown mandatory Git index extension: {}",
                String::from_utf8_lossy(signature)
            ),
            Self::ChecksumMismatch => f.write_str("Git index SHA-1 checksum mismatch"),
            Self::Sha1(error) => write!(f, "Git index SHA-1 failed: {error}"),
        }
    }
}

impl Error for GitIndexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sha1(error) => Some(error),
            _ => None,
        }
    }
}

impl From<GitObjectSha1Error> for GitIndexError {
    fn from(value: GitObjectSha1Error) -> Self {
        Self::Sha1(value)
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    const fn position(&self) -> usize {
        self.position
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.position)
    }

    fn remaining_slice(&self) -> &'a [u8] {
        &self.bytes[self.position..]
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], GitIndexError> {
        let end = self
            .position
            .checked_add(count)
            .ok_or(GitIndexError::Truncated)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(GitIndexError::Truncated)?;
        self.position = end;
        Ok(bytes)
    }

    fn read_u16(&mut self) -> Result<u16, GitIndexError> {
        let bytes: [u8; 2] = self
            .take(2)?
            .try_into()
            .map_err(|_| GitIndexError::Truncated)?;
        Ok(u16::from_be_bytes(bytes))
    }

    fn read_u32(&mut self) -> Result<u32, GitIndexError> {
        let bytes: [u8; 4] = self
            .take(4)?
            .try_into()
            .map_err(|_| GitIndexError::Truncated)?;
        Ok(u32::from_be_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestEntry<'a> {
        path: &'a [u8],
        stage: u8,
        mode: u32,
        extended_flags: Option<u16>,
    }

    #[test]
    fn parses_v2_entries_and_ignores_only_optional_extensions() {
        let bytes = build_index(
            2,
            &[
                TestEntry {
                    path: b"a.txt",
                    stage: 0,
                    mode: 0o100644,
                    extended_flags: None,
                },
                TestEntry {
                    path: b"bin/run",
                    stage: 0,
                    mode: 0o100755,
                    extended_flags: None,
                },
            ],
            &[(b"TREE", b"opaque")],
        );
        let index = parse_git_index(&bytes, GitIndexBounds::default()).unwrap();
        assert_eq!(index.version, GitIndexVersion::V2);
        assert_eq!(index.entries.len(), 2);
        assert_eq!(index.entries[0].path, b"a.txt");
        assert_eq!(
            index.entries[1].mode,
            GitIndexMode::RegularFile { executable: true }
        );
        assert_eq!(
            index.extensions,
            vec![GitIndexExtension {
                signature: *b"TREE",
                size: 6
            }]
        );
    }

    #[test]
    fn parses_v3_extended_flags_without_treating_them_as_authority() {
        let bytes = build_index(
            3,
            &[TestEntry {
                path: b"draft.txt",
                stage: 0,
                mode: 0o100644,
                extended_flags: Some(0x6000),
            }],
            &[],
        );
        let index = parse_git_index(&bytes, GitIndexBounds::default()).unwrap();
        assert!(index.entries[0].skip_worktree);
        assert!(index.entries[0].intent_to_add);
    }

    #[test]
    fn rejects_version_four_and_checksum_corruption() {
        let v4 = build_index(4, &[], &[]);
        assert!(matches!(
            parse_git_index(&v4, GitIndexBounds::default()),
            Err(GitIndexError::UnsupportedVersion(4))
        ));

        let mut corrupt = build_index(
            2,
            &[TestEntry {
                path: b"a",
                stage: 0,
                mode: 0o100644,
                extended_flags: None,
            }],
            &[],
        );
        corrupt[12] ^= 1;
        assert!(matches!(
            parse_git_index(&corrupt, GitIndexBounds::default()),
            Err(GitIndexError::ChecksumMismatch)
        ));
    }

    #[test]
    fn rejects_split_sparse_and_unknown_mandatory_extensions() {
        for signature in [b"link", b"sdir", b"abcd"] {
            let bytes = build_index(2, &[], &[(signature, b"x")]);
            let error = parse_git_index(&bytes, GitIndexBounds::default()).unwrap_err();
            match signature {
                b"link" => assert!(matches!(error, GitIndexError::SplitIndexUnsupported)),
                b"sdir" => assert!(matches!(error, GitIndexError::SparseIndexUnsupported)),
                _ => assert!(matches!(
                    error,
                    GitIndexError::UnknownMandatoryExtension(found) if found == *signature
                )),
            }
        }
    }

    #[test]
    fn rejects_unsorted_paths_invalid_components_modes_and_v2_extended_flags() {
        let unsorted = build_index(
            2,
            &[
                TestEntry {
                    path: b"z",
                    stage: 0,
                    mode: 0o100644,
                    extended_flags: None,
                },
                TestEntry {
                    path: b"a",
                    stage: 0,
                    mode: 0o100644,
                    extended_flags: None,
                },
            ],
            &[],
        );
        assert!(matches!(
            parse_git_index(&unsorted, GitIndexBounds::default()),
            Err(GitIndexError::EntriesNotStrictlySorted)
        ));

        for path in [&b"../x"[..], &b".git/config"[..], &b"a//b"[..], &b"a/"[..]] {
            let bytes = build_index(
                2,
                &[TestEntry {
                    path,
                    stage: 0,
                    mode: 0o100644,
                    extended_flags: None,
                }],
                &[],
            );
            assert!(matches!(
                parse_git_index(&bytes, GitIndexBounds::default()),
                Err(GitIndexError::InvalidPath)
            ));
        }

        let bad_mode = build_index(
            2,
            &[TestEntry {
                path: b"a",
                stage: 0,
                mode: 0o100600,
                extended_flags: None,
            }],
            &[],
        );
        assert!(matches!(
            parse_git_index(&bad_mode, GitIndexBounds::default()),
            Err(GitIndexError::UnsupportedMode(_))
        ));

        let v2_extended = build_index(
            2,
            &[TestEntry {
                path: b"a",
                stage: 0,
                mode: 0o100644,
                extended_flags: Some(0x2000),
            }],
            &[],
        );
        assert!(matches!(
            parse_git_index(&v2_extended, GitIndexBounds::default()),
            Err(GitIndexError::ExtendedFlagsInV2)
        ));
    }

    #[test]
    fn enforces_entry_path_and_extension_bounds() {
        let bytes = build_index(
            2,
            &[TestEntry {
                path: b"abc",
                stage: 0,
                mode: 0o100644,
                extended_flags: None,
            }],
            &[],
        );
        let mut bounds = GitIndexBounds::default();
        bounds.max_path_bytes = 2;
        assert!(matches!(
            parse_git_index(&bytes, bounds),
            Err(GitIndexError::PathLimitExceeded)
        ));

        let many_extensions = build_index(2, &[], &[(b"TREE", b""), (b"REUC", b"")]);
        let mut bounds = GitIndexBounds::default();
        bounds.max_extensions = 1;
        assert!(matches!(
            parse_git_index(&many_extensions, bounds),
            Err(GitIndexError::ExtensionLimitExceeded)
        ));
    }

    fn build_index(
        version: u32,
        entries: &[TestEntry<'_>],
        extensions: &[(&[u8; 4], &[u8])],
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"DIRC");
        bytes.extend_from_slice(&version.to_be_bytes());
        bytes.extend_from_slice(&(entries.len() as u32).to_be_bytes());

        for entry in entries {
            let start = bytes.len();
            for value in [1_u32, 2, 3, 4, 5, 6] {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            bytes.extend_from_slice(&entry.mode.to_be_bytes());
            for value in [7_u32, 8, 9] {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            bytes.extend_from_slice(&[0x11; SHA1_BYTES]);

            let stored_len = entry.path.len().min(0x0fff) as u16;
            let mut flags = (u16::from(entry.stage & 0x3)) << 12 | stored_len;
            if entry.extended_flags.is_some() {
                flags |= 0x4000;
            }
            bytes.extend_from_slice(&flags.to_be_bytes());
            if let Some(extended_flags) = entry.extended_flags {
                bytes.extend_from_slice(&extended_flags.to_be_bytes());
            }
            bytes.extend_from_slice(entry.path);
            bytes.push(0);
            while (bytes.len() - start) % 8 != 0 {
                bytes.push(0);
            }
        }

        for (signature, data) in extensions {
            bytes.extend_from_slice(*signature);
            bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
            bytes.extend_from_slice(data);
        }

        let checksum = GitObjectSha1::digest(&bytes).unwrap();
        bytes.extend_from_slice(&checksum);
        bytes
    }
}
