#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::time::Duration;

use golam_core::target_identity::{ObservedFileKind, ResolvedTargetIdentity};
use golam_core::tool_request::{RequestedOperationId, RequestedTarget};
use miniz_oxide::inflate::stream::{InflateState, inflate};
use miniz_oxide::{DataFormat, MZError, MZFlush, MZStatus};

use crate::git_read_budget::{
    DECOMPRESSION_INPUT_QUANTUM_BYTES, DECOMPRESSION_OUTPUT_QUANTUM_BYTES,
    DecompressionBudgetError, DecompressionDeadline,
};
use crate::git_sha1::{GitObjectSha1, GitObjectSha1Error};
use crate::local_fs::{LocalFsResolutionError, LocalFsResolver};
use crate::local_read::{LocalFileReadBounds, LocalFileReadError, read_regular_file};

pub const MAX_LOOSE_COMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_SINGLE_OBJECT_DECOMPRESSED_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_OBJECT_HEADER_BYTES: usize = 256;
pub const MAX_HEAD_BYTES: u64 = 4 * 1024;
pub const MAX_REF_BYTES: u64 = 4 * 1024;
pub const MAX_PACKED_REFS_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_PACKED_REFS: usize = 200_000;
pub const MAX_SYMBOLIC_REF_DEPTH: usize = 16;
pub const DEFAULT_GIT_READ_TIME_BUDGET: Duration = Duration::from_secs(10);
pub const MAX_GIT_READ_TIME_BUDGET: Duration = Duration::from_secs(60);
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitReadBounds {
    pub max_loose_compressed_bytes: u64,
    pub max_single_object_decompressed_bytes: usize,
    pub max_packed_refs_bytes: u64,
    pub max_packed_refs: usize,
    pub max_duration: Duration,
}

impl Default for GitReadBounds {
    fn default() -> Self {
        Self {
            max_loose_compressed_bytes: MAX_LOOSE_COMPRESSED_BYTES,
            max_single_object_decompressed_bytes: MAX_SINGLE_OBJECT_DECOMPRESSED_BYTES,
            max_packed_refs_bytes: MAX_PACKED_REFS_BYTES,
            max_packed_refs: MAX_PACKED_REFS,
            max_duration: DEFAULT_GIT_READ_TIME_BUDGET,
        }
    }
}

impl GitReadBounds {
    pub fn validate(self) -> Result<(), GitReadError> {
        if self.max_loose_compressed_bytes == 0
            || self.max_loose_compressed_bytes > MAX_LOOSE_COMPRESSED_BYTES
            || self.max_single_object_decompressed_bytes == 0
            || self.max_single_object_decompressed_bytes > MAX_SINGLE_OBJECT_DECOMPRESSED_BYTES
            || self.max_packed_refs_bytes == 0
            || self.max_packed_refs_bytes > MAX_PACKED_REFS_BYTES
            || self.max_packed_refs == 0
            || self.max_packed_refs > MAX_PACKED_REFS
            || self.max_duration.is_zero()
            || self.max_duration > MAX_GIT_READ_TIME_BUDGET
        {
            return Err(GitReadError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GitObjectId([u8; 20]);

impl GitObjectId {
    pub fn parse(value: &str) -> Result<Self, GitReadError> {
        if value.len() != 40
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        {
            return Err(GitReadError::InvalidObjectId);
        }
        let mut bytes = [0_u8; 20];
        for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            bytes[index] = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
        }
        Ok(Self(bytes))
    }

    pub const fn bytes(self) -> [u8; 20] {
        self.0
    }

    pub fn to_hex(self) -> String {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(40);
        for byte in self.0 {
            output.push(char::from(DIGITS[(byte >> 4) as usize]));
            output.push(char::from(DIGITS[(byte & 0x0f) as usize]));
        }
        output
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitObjectFormat {
    Sha1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitObjectKind {
    Blob,
    Tree,
    Commit,
    Tag,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitObject {
    pub id: GitObjectId,
    pub kind: GitObjectKind,
    pub declared_size: u64,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GitHeadRepresentation {
    Detached(GitObjectId),
    Symbolic(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitRefSource {
    Loose,
    Packed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitRefResolution {
    pub requested_ref: String,
    pub symbolic_chain: Vec<String>,
    pub object_id: GitObjectId,
    pub source: GitRefSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitHeadObservation {
    pub raw: Vec<u8>,
    pub representation: GitHeadRepresentation,
    pub resolved_ref: Option<GitRefResolution>,
    pub object_id: GitObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitRepositoryEvidence {
    pub repository_root: ResolvedTargetIdentity,
    pub git_directory: ResolvedTargetIdentity,
    pub object_store_directory: ResolvedTargetIdentity,
    pub object_format: GitObjectFormat,
    pub head: GitHeadObservation,
    pub observed_at_unix_ms: u64,
    pub bounds: GitReadBounds,
}

pub struct GitRepositoryReader<'a> {
    resolver: &'a LocalFsResolver,
    operation: RequestedOperationId,
    evidence: GitRepositoryEvidence,
    bounds: GitReadBounds,
}

impl<'a> GitRepositoryReader<'a> {
    pub fn open(
        resolver: &'a LocalFsResolver,
        operation: &RequestedOperationId,
        bounds: GitReadBounds,
        observed_at_unix_ms: u64,
    ) -> Result<Self, GitReadError> {
        bounds.validate()?;
        let repository_root = resolve_directory(resolver, operation, ".", observed_at_unix_ms)?;
        let git_directory = resolve_directory(resolver, operation, ".git", observed_at_unix_ms)
            .map_err(|error| match error {
                GitReadError::UnsupportedFileKind(ObservedFileKind::RegularFile) => {
                    GitReadError::GitFileWorktreeUnsupported
                }
                other => other,
            })?;
        let object_store_directory =
            resolve_directory(resolver, operation, ".git/objects", observed_at_unix_ms)?;

        let object_format = match read_optional_file(
            resolver,
            operation,
            ".git/config",
            MAX_CONFIG_BYTES,
            bounds.max_duration,
            observed_at_unix_ms,
        )? {
            Some(config) => parse_object_format(&config)?,
            None => GitObjectFormat::Sha1,
        };

        let raw_head = read_required_file(
            resolver,
            operation,
            ".git/HEAD",
            MAX_HEAD_BYTES,
            bounds.max_duration,
            observed_at_unix_ms,
        )?;
        let representation = parse_head(&raw_head)?;

        let mut reader = Self {
            resolver,
            operation: operation.clone(),
            evidence: GitRepositoryEvidence {
                repository_root,
                git_directory,
                object_store_directory,
                object_format,
                head: GitHeadObservation {
                    raw: raw_head,
                    representation: representation.clone(),
                    resolved_ref: None,
                    object_id: GitObjectId([0; 20]),
                },
                observed_at_unix_ms,
                bounds,
            },
            bounds,
        };

        let (resolved_ref, object_id) = match representation {
            GitHeadRepresentation::Detached(id) => (None, id),
            GitHeadRepresentation::Symbolic(reference) => {
                let resolution = reader.resolve_ref(&reference, observed_at_unix_ms)?;
                let id = resolution.object_id;
                (Some(resolution), id)
            }
        };
        reader.evidence.head.resolved_ref = resolved_ref;
        reader.evidence.head.object_id = object_id;
        Ok(reader)
    }

    pub fn evidence(&self) -> &GitRepositoryEvidence {
        &self.evidence
    }

    pub fn resolve_ref(
        &self,
        reference: &str,
        observed_at_unix_ms: u64,
    ) -> Result<GitRefResolution, GitReadError> {
        validate_ref_name(reference)?;
        let requested_ref = reference.to_owned();
        let mut current = requested_ref.clone();
        let mut chain = Vec::new();
        let mut seen = HashSet::new();

        for _ in 0..MAX_SYMBOLIC_REF_DEPTH {
            if !seen.insert(current.clone()) {
                return Err(GitReadError::SymbolicRefCycle);
            }
            chain.push(current.clone());
            let target = format!(".git/{current}");
            if let Some(bytes) = read_optional_file(
                self.resolver,
                &self.operation,
                &target,
                MAX_REF_BYTES,
                self.bounds.max_duration,
                observed_at_unix_ms,
            )? {
                let line = one_trimmed_line(&bytes)?;
                if let Some(next) = line.strip_prefix("ref: ") {
                    validate_ref_name(next)?;
                    current = next.to_owned();
                    continue;
                }
                let object_id = GitObjectId::parse(line)?;
                return Ok(GitRefResolution {
                    requested_ref,
                    symbolic_chain: chain,
                    object_id,
                    source: GitRefSource::Loose,
                });
            }

            if let Some(object_id) = self.resolve_packed_ref(&current, observed_at_unix_ms)? {
                return Ok(GitRefResolution {
                    requested_ref,
                    symbolic_chain: chain,
                    object_id,
                    source: GitRefSource::Packed,
                });
            }
            return Err(GitReadError::MissingRef(current));
        }

        Err(GitReadError::SymbolicRefDepthExceeded)
    }

    pub fn read_loose_object(
        &self,
        object_id: GitObjectId,
        observed_at_unix_ms: u64,
    ) -> Result<GitObject, GitReadError> {
        let hex = object_id.to_hex();
        let path = format!(".git/objects/{}/{}", &hex[..2], &hex[2..]);
        let compressed = read_optional_file(
            self.resolver,
            &self.operation,
            &path,
            self.bounds.max_loose_compressed_bytes,
            self.bounds.max_duration,
            observed_at_unix_ms,
        )?
        .ok_or(GitReadError::MissingObject(object_id))?;

        let decoded = decompress_zlib_bounded(
            &compressed,
            self.bounds.max_single_object_decompressed_bytes,
            self.bounds.max_duration,
        )?;
        let actual = GitObjectSha1::digest(&decoded)?;
        if actual != object_id.bytes() {
            return Err(GitReadError::ObjectHashMismatch);
        }
        parse_object_bytes(object_id, decoded)
    }

    fn resolve_packed_ref(
        &self,
        reference: &str,
        observed_at_unix_ms: u64,
    ) -> Result<Option<GitObjectId>, GitReadError> {
        let Some(bytes) = read_optional_file(
            self.resolver,
            &self.operation,
            ".git/packed-refs",
            self.bounds.max_packed_refs_bytes,
            self.bounds.max_duration,
            observed_at_unix_ms,
        )?
        else {
            return Ok(None);
        };
        parse_packed_ref(&bytes, reference, self.bounds.max_packed_refs)
    }
}

#[derive(Debug)]
pub enum GitReadError {
    InvalidBounds,
    InvalidInternalPath,
    Resolution(LocalFsResolutionError),
    LocalRead(LocalFileReadError),
    UnsupportedFileKind(ObservedFileKind),
    GitFileWorktreeUnsupported,
    UnsupportedObjectFormat(String),
    InvalidConfig,
    InvalidHead,
    InvalidRefName,
    InvalidRefValue,
    InvalidObjectId,
    MissingRef(String),
    DuplicatePackedRef(String),
    PackedRefsLimitExceeded,
    SymbolicRefCycle,
    SymbolicRefDepthExceeded,
    MissingObject(GitObjectId),
    Decompression(DecompressionBudgetError),
    DecompressionData,
    DecompressionTruncated,
    DecompressionStalled,
    DecompressionTrailingData,
    DecompressedSizeLimitExceeded,
    InvalidObjectHeader,
    UnsupportedObjectType(String),
    ObjectSizeMismatch,
    ObjectHashMismatch,
    Sha1(GitObjectSha1Error),
}

impl fmt::Display for GitReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBounds => {
                f.write_str("Git read bounds exceed the frozen first-profile limits")
            }
            Self::InvalidInternalPath => {
                f.write_str("Git reader constructed an invalid bounded internal path")
            }
            Self::Resolution(error) => write!(f, "Git repository path resolution failed: {error}"),
            Self::LocalRead(error) => write!(f, "Git bounded file read failed: {error}"),
            Self::UnsupportedFileKind(kind) => {
                write!(f, "Git reader requires a directory but observed {kind:?}")
            }
            Self::GitFileWorktreeUnsupported => f.write_str(
                "gitfile/worktree indirection is unsupported by the first Git read profile",
            ),
            Self::UnsupportedObjectFormat(format) => {
                write!(f, "unsupported Git object format: {format}")
            }
            Self::InvalidConfig => {
                f.write_str("Git repository config is malformed for the bounded format parser")
            }
            Self::InvalidHead => f.write_str("Git HEAD is malformed or non-canonical"),
            Self::InvalidRefName => {
                f.write_str("Git ref name is outside the bounded canonical first-profile grammar")
            }
            Self::InvalidRefValue => {
                f.write_str("Git ref file must contain exactly one bounded canonical value")
            }
            Self::InvalidObjectId => f.write_str(
                "Git SHA-1 object id must be exactly 40 lowercase hexadecimal characters",
            ),
            Self::MissingRef(reference) => write!(f, "Git ref is unavailable locally: {reference}"),
            Self::DuplicatePackedRef(reference) => {
                write!(f, "Git packed-refs contains duplicate ref: {reference}")
            }
            Self::PackedRefsLimitExceeded => f.write_str("Git packed-refs entry limit exceeded"),
            Self::SymbolicRefCycle => f.write_str("Git symbolic ref cycle detected"),
            Self::SymbolicRefDepthExceeded => f.write_str("Git symbolic ref depth limit exceeded"),
            Self::MissingObject(id) => write!(
                f,
                "Git object is unavailable in the local loose-object store: {}",
                id.to_hex()
            ),
            Self::Decompression(error) => {
                write!(f, "bounded Git zlib decompression failed: {error}")
            }
            Self::DecompressionData => f.write_str("Git loose object contains invalid zlib data"),
            Self::DecompressionTruncated => {
                f.write_str("Git loose object zlib stream is truncated")
            }
            Self::DecompressionStalled => {
                f.write_str("Git loose object decompression made no progress")
            }
            Self::DecompressionTrailingData => {
                f.write_str("Git loose object contains trailing bytes after the zlib stream")
            }
            Self::DecompressedSizeLimitExceeded => {
                f.write_str("Git loose object decompressed-size limit exceeded")
            }
            Self::InvalidObjectHeader => {
                f.write_str("Git loose object header is malformed or exceeds its bound")
            }
            Self::UnsupportedObjectType(kind) => write!(f, "unsupported Git object type: {kind}"),
            Self::ObjectSizeMismatch => {
                f.write_str("Git loose object declared size does not equal decoded body size")
            }
            Self::ObjectHashMismatch => {
                f.write_str("Git loose object SHA-1 does not match its requested object id")
            }
            Self::Sha1(error) => write!(f, "Git object SHA-1 failed: {error}"),
        }
    }
}

impl Error for GitReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resolution(error) => Some(error),
            Self::LocalRead(error) => Some(error),
            Self::Decompression(error) => Some(error),
            Self::Sha1(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LocalFsResolutionError> for GitReadError {
    fn from(value: LocalFsResolutionError) -> Self {
        Self::Resolution(value)
    }
}

impl From<LocalFileReadError> for GitReadError {
    fn from(value: LocalFileReadError) -> Self {
        Self::LocalRead(value)
    }
}

impl From<DecompressionBudgetError> for GitReadError {
    fn from(value: DecompressionBudgetError) -> Self {
        Self::Decompression(value)
    }
}

impl From<GitObjectSha1Error> for GitReadError {
    fn from(value: GitObjectSha1Error) -> Self {
        Self::Sha1(value)
    }
}

fn resolve_directory(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    path: &str,
    observed_at_unix_ms: u64,
) -> Result<ResolvedTargetIdentity, GitReadError> {
    let requested = requested(path)?;
    let identity = resolver.resolve_read_target(&requested, operation, observed_at_unix_ms)?;
    if identity.file_kind != ObservedFileKind::Directory {
        return Err(GitReadError::UnsupportedFileKind(identity.file_kind));
    }
    Ok(identity)
}

fn read_required_file(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    path: &str,
    max_bytes: u64,
    max_duration: Duration,
    observed_at_unix_ms: u64,
) -> Result<Vec<u8>, GitReadError> {
    read_optional_file(
        resolver,
        operation,
        path,
        max_bytes,
        max_duration,
        observed_at_unix_ms,
    )?
    .ok_or_else(|| GitReadError::MissingRef(path.to_owned()))
}

fn read_optional_file(
    resolver: &LocalFsResolver,
    operation: &RequestedOperationId,
    path: &str,
    max_bytes: u64,
    max_duration: Duration,
    observed_at_unix_ms: u64,
) -> Result<Option<Vec<u8>>, GitReadError> {
    let requested = requested(path)?;
    let identity = resolver.resolve_read_target(&requested, operation, observed_at_unix_ms)?;
    if identity.file_kind == ObservedFileKind::Missing {
        return Ok(None);
    }
    if identity.file_kind != ObservedFileKind::RegularFile {
        return Err(GitReadError::LocalRead(
            LocalFileReadError::UnsupportedFileKind(identity.file_kind),
        ));
    }
    let read = read_regular_file(
        resolver,
        &requested,
        operation,
        LocalFileReadBounds {
            max_bytes,
            max_duration,
        },
        observed_at_unix_ms,
        observed_at_unix_ms,
    )?;
    Ok(Some(read.bytes))
}

fn requested(path: &str) -> Result<RequestedTarget, GitReadError> {
    RequestedTarget::new(path).map_err(|_| GitReadError::InvalidInternalPath)
}

fn parse_object_format(config: &[u8]) -> Result<GitObjectFormat, GitReadError> {
    let text = std::str::from_utf8(config).map_err(|_| GitReadError::InvalidConfig)?;
    let mut section = String::new();
    let mut object_format = None;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') {
            if !line.ends_with(']') {
                return Err(GitReadError::InvalidConfig);
            }
            section = line[1..line.len() - 1].trim().to_ascii_lowercase();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if section == "extensions" && key.trim().eq_ignore_ascii_case("objectformat") {
            if object_format.is_some() {
                return Err(GitReadError::InvalidConfig);
            }
            object_format = Some(value.trim().to_ascii_lowercase());
        }
    }
    match object_format.as_deref() {
        None | Some("sha1") => Ok(GitObjectFormat::Sha1),
        Some(other) => Err(GitReadError::UnsupportedObjectFormat(other.to_owned())),
    }
}

fn parse_head(bytes: &[u8]) -> Result<GitHeadRepresentation, GitReadError> {
    let line = one_trimmed_line(bytes).map_err(|_| GitReadError::InvalidHead)?;
    if let Some(reference) = line.strip_prefix("ref: ") {
        validate_ref_name(reference).map_err(|_| GitReadError::InvalidHead)?;
        Ok(GitHeadRepresentation::Symbolic(reference.to_owned()))
    } else {
        GitObjectId::parse(line)
            .map(GitHeadRepresentation::Detached)
            .map_err(|_| GitReadError::InvalidHead)
    }
}

fn one_trimmed_line(bytes: &[u8]) -> Result<&str, GitReadError> {
    let text = std::str::from_utf8(bytes).map_err(|_| GitReadError::InvalidRefValue)?;
    let trimmed = text.trim_end_matches(['\n', '\r']);
    if trimmed.is_empty() || trimmed.contains(['\n', '\r']) || trimmed.trim() != trimmed {
        return Err(GitReadError::InvalidRefValue);
    }
    Ok(trimmed)
}

fn validate_ref_name(reference: &str) -> Result<(), GitReadError> {
    if !reference.starts_with("refs/")
        || reference.len() > 4096
        || reference.ends_with('/')
        || reference.contains("..")
        || reference.contains("//")
        || reference.contains("@{")
        || reference.bytes().any(|byte| {
            byte <= b' '
                || byte == 0x7f
                || matches!(byte, b'~' | b'^' | b':' | b'?' | b'*' | b'[' | b'\\')
        })
    {
        return Err(GitReadError::InvalidRefName);
    }
    for component in reference.split('/') {
        if component.is_empty()
            || component == "."
            || component == ".."
            || component.starts_with('.')
            || component.ends_with('.')
            || component.ends_with(".lock")
        {
            return Err(GitReadError::InvalidRefName);
        }
    }
    Ok(())
}

fn parse_packed_ref(
    bytes: &[u8],
    wanted: &str,
    max_refs: usize,
) -> Result<Option<GitObjectId>, GitReadError> {
    let text = std::str::from_utf8(bytes).map_err(|_| GitReadError::InvalidRefValue)?;
    let mut seen_refs = HashSet::new();
    let mut found = None;
    let mut count = 0_usize;
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() || line.starts_with('#') || line.starts_with('^') {
            continue;
        }
        count = count
            .checked_add(1)
            .ok_or(GitReadError::PackedRefsLimitExceeded)?;
        if count > max_refs {
            return Err(GitReadError::PackedRefsLimitExceeded);
        }
        let Some((id, reference)) = line.split_once(' ') else {
            return Err(GitReadError::InvalidRefValue);
        };
        if id.contains(' ') || reference.contains(' ') {
            return Err(GitReadError::InvalidRefValue);
        }
        validate_ref_name(reference)?;
        let object_id = GitObjectId::parse(id)?;
        if !seen_refs.insert(reference.to_owned()) {
            return Err(GitReadError::DuplicatePackedRef(reference.to_owned()));
        }
        if reference == wanted {
            found = Some(object_id);
        }
    }
    Ok(found)
}

fn decompress_zlib_bounded(
    compressed: &[u8],
    max_output_bytes: usize,
    max_duration: Duration,
) -> Result<Vec<u8>, GitReadError> {
    if max_output_bytes == 0 || max_output_bytes > MAX_SINGLE_OBJECT_DECOMPRESSED_BYTES {
        return Err(GitReadError::InvalidBounds);
    }
    let deadline = DecompressionDeadline::start(max_duration)?;
    let mut state = InflateState::new(DataFormat::Zlib);
    let mut input_offset = 0_usize;
    let mut output = Vec::with_capacity(max_output_bytes.min(DECOMPRESSION_OUTPUT_QUANTUM_BYTES));

    loop {
        if output.len() == max_output_bytes {
            return Err(GitReadError::DecompressedSizeLimitExceeded);
        }
        let input_end = input_offset
            .saturating_add(DECOMPRESSION_INPUT_QUANTUM_BYTES)
            .min(compressed.len());
        let input = &compressed[input_offset..input_end];
        let output_len = (max_output_bytes - output.len()).min(DECOMPRESSION_OUTPUT_QUANTUM_BYTES);
        let mut chunk = vec![0_u8; output_len];
        let result = deadline.run_quantum(input, &mut chunk, |input, output| {
            inflate(&mut state, input, output, MZFlush::None)
        })?;
        if result.bytes_consumed > input.len() || result.bytes_written > chunk.len() {
            return Err(GitReadError::DecompressionData);
        }
        input_offset += result.bytes_consumed;
        output.extend_from_slice(&chunk[..result.bytes_written]);

        match result.status {
            Ok(MZStatus::StreamEnd) => {
                if input_offset != compressed.len() {
                    return Err(GitReadError::DecompressionTrailingData);
                }
                return Ok(output);
            }
            Ok(MZStatus::Ok) => {
                if result.bytes_consumed == 0 && result.bytes_written == 0 {
                    return if input_offset == compressed.len() {
                        Err(GitReadError::DecompressionTruncated)
                    } else {
                        Err(GitReadError::DecompressionStalled)
                    };
                }
            }
            Ok(_) => return Err(GitReadError::DecompressionData),
            Err(MZError::Buf) if input_offset == compressed.len() => {
                return Err(GitReadError::DecompressionTruncated);
            }
            Err(MZError::Buf) => return Err(GitReadError::DecompressionStalled),
            Err(_) => return Err(GitReadError::DecompressionData),
        }
    }
}

fn parse_object_bytes(object_id: GitObjectId, decoded: Vec<u8>) -> Result<GitObject, GitReadError> {
    let header_end = decoded
        .iter()
        .take(MAX_OBJECT_HEADER_BYTES + 1)
        .position(|byte| *byte == 0)
        .ok_or(GitReadError::InvalidObjectHeader)?;
    if header_end == 0 || header_end > MAX_OBJECT_HEADER_BYTES {
        return Err(GitReadError::InvalidObjectHeader);
    }
    let header = std::str::from_utf8(&decoded[..header_end])
        .map_err(|_| GitReadError::InvalidObjectHeader)?;
    let Some((kind, size)) = header.split_once(' ') else {
        return Err(GitReadError::InvalidObjectHeader);
    };
    if size.is_empty() || !size.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(GitReadError::InvalidObjectHeader);
    }
    let declared_size = size
        .parse::<u64>()
        .map_err(|_| GitReadError::InvalidObjectHeader)?;
    let bytes = decoded[header_end + 1..].to_vec();
    if usize::try_from(declared_size).ok() != Some(bytes.len()) {
        return Err(GitReadError::ObjectSizeMismatch);
    }
    let kind = match kind {
        "blob" => GitObjectKind::Blob,
        "tree" => GitObjectKind::Tree,
        "commit" => GitObjectKind::Commit,
        "tag" => GitObjectKind::Tag,
        other => return Err(GitReadError::UnsupportedObjectType(other.to_owned())),
    };
    Ok(GitObject {
        id: object_id,
        kind,
        declared_size,
        bytes,
    })
}

fn hex_nibble(byte: u8) -> Result<u8, GitReadError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(GitReadError::InvalidObjectId),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::tool_request::ResourceClassId;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);
    const EMPTY_BLOB_ZLIB: &[u8] = &[120, 156, 75, 202, 201, 79, 82, 48, 96, 0, 0, 9, 176, 1, 240];
    const HELLO_BLOB_ZLIB: &[u8] = &[
        120, 156, 75, 202, 201, 79, 82, 48, 99, 200, 72, 205, 201, 201, 231, 2, 0, 29, 197, 4, 20,
    ];

    #[test]
    fn canonical_sha1_object_ids_round_trip_and_reject_noncanonical_text() {
        let id = GitObjectId::parse("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").unwrap();
        assert_eq!(id.to_hex(), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
        assert!(GitObjectId::parse("E69DE29BB2D1D6434B8B29AE775AD8C2E48C5391").is_err());
        assert!(GitObjectId::parse("../e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").is_err());
    }

    #[test]
    fn head_and_ref_grammars_fail_closed_on_escape_and_ambiguity() {
        assert_eq!(
            parse_head(b"ref: refs/heads/main\n").unwrap(),
            GitHeadRepresentation::Symbolic("refs/heads/main".into())
        );
        assert!(parse_head(b"ref: refs/heads/../outside\n").is_err());
        assert!(parse_head(b"ref: refs/heads/main\nsecond\n").is_err());
        assert!(validate_ref_name("refs/heads/a.lock").is_err());
        assert!(validate_ref_name("refs/heads/a@{1}").is_err());
    }

    #[test]
    fn sha256_repository_config_is_explicitly_unsupported() {
        assert_eq!(
            parse_object_format(b"[core]\nrepositoryformatversion = 0\n").unwrap(),
            GitObjectFormat::Sha1
        );
        assert!(matches!(
            parse_object_format(b"[extensions]\nobjectFormat = sha256\n"),
            Err(GitReadError::UnsupportedObjectFormat(format)) if format == "sha256"
        ));
    }

    #[test]
    fn packed_refs_are_bounded_canonical_and_duplicate_free() {
        let bytes = b"# pack-refs with: peeled fully-peeled sorted\ne69de29bb2d1d6434b8b29ae775ad8c2e48c5391 refs/heads/main\n^ce013625030ba8dba906f756967f9e9ca394464a\n";
        assert_eq!(
            parse_packed_ref(bytes, "refs/heads/main", 10)
                .unwrap()
                .unwrap()
                .to_hex(),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
        let duplicate = b"e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 refs/heads/main\nce013625030ba8dba906f756967f9e9ca394464a refs/heads/main\n";
        assert!(matches!(
            parse_packed_ref(duplicate, "refs/heads/main", 10),
            Err(GitReadError::DuplicatePackedRef(_))
        ));
        assert!(matches!(
            parse_packed_ref(bytes, "refs/heads/main", 0),
            Err(GitReadError::PackedRefsLimitExceeded)
        ));
    }

    #[test]
    fn bounded_zlib_decode_and_git_object_header_validation_match_known_blob() {
        let decoded = decompress_zlib_bounded(
            EMPTY_BLOB_ZLIB,
            MAX_SINGLE_OBJECT_DECOMPRESSED_BYTES,
            Duration::from_secs(1),
        )
        .unwrap();
        assert_eq!(decoded, b"blob 0\0");
        let id = GitObjectId::parse("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391").unwrap();
        let object = parse_object_bytes(id, decoded).unwrap();
        assert_eq!(object.kind, GitObjectKind::Blob);
        assert!(object.bytes.is_empty());

        assert!(
            decompress_zlib_bounded(&EMPTY_BLOB_ZLIB[..8], 1024, Duration::from_secs(1)).is_err()
        );
        assert!(matches!(
            decompress_zlib_bounded(HELLO_BLOB_ZLIB, 4, Duration::from_secs(1)),
            Err(GitReadError::DecompressedSizeLimitExceeded)
        ));
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    ))]
    #[test]
    fn repository_reader_resolves_symbolic_head_and_verifies_loose_object_without_git_process() {
        let root = fixture_root();
        fs::create_dir_all(root.join(".git/objects/ce")).unwrap();
        fs::create_dir_all(root.join(".git/refs/heads")).unwrap();
        fs::write(
            root.join(".git/config"),
            b"[core]\nrepositoryformatversion = 0\n",
        )
        .unwrap();
        fs::write(root.join(".git/HEAD"), b"ref: refs/heads/main\n").unwrap();
        fs::write(
            root.join(".git/refs/heads/main"),
            b"ce013625030ba8dba906f756967f9e9ca394464a\n",
        )
        .unwrap();
        fs::write(
            root.join(".git/objects/ce/013625030ba8dba906f756967f9e9ca394464a"),
            HELLO_BLOB_ZLIB,
        )
        .unwrap();

        let operation = RequestedOperationId::new("git-read").unwrap();
        let resolver = LocalFsResolver::new(
            &root,
            ResourceClassId::new("project").unwrap(),
            vec![operation.clone()],
            Vec::<PathBuf>::new(),
        )
        .unwrap();
        let reader =
            GitRepositoryReader::open(&resolver, &operation, GitReadBounds::default(), 1).unwrap();
        assert_eq!(
            reader.evidence().head.object_id.to_hex(),
            "ce013625030ba8dba906f756967f9e9ca394464a"
        );
        let object = reader
            .read_loose_object(reader.evidence().head.object_id, 2)
            .unwrap();
        assert_eq!(object.kind, GitObjectKind::Blob);
        assert_eq!(object.bytes, b"hello\n");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd"
    ))]
    #[test]
    fn packed_ref_is_used_only_when_loose_ref_is_absent() {
        let root = fixture_root();
        fs::create_dir_all(root.join(".git/objects")).unwrap();
        fs::write(root.join(".git/HEAD"), b"ref: refs/heads/main\n").unwrap();
        fs::write(
            root.join(".git/packed-refs"),
            b"e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 refs/heads/main\n",
        )
        .unwrap();
        let operation = RequestedOperationId::new("git-read").unwrap();
        let resolver = LocalFsResolver::new(
            &root,
            ResourceClassId::new("project").unwrap(),
            vec![operation.clone()],
            Vec::<PathBuf>::new(),
        )
        .unwrap();
        let reader =
            GitRepositoryReader::open(&resolver, &operation, GitReadBounds::default(), 1).unwrap();
        assert_eq!(
            reader.evidence().head.resolved_ref.as_ref().unwrap().source,
            GitRefSource::Packed
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn fixture_root() -> PathBuf {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("golam-git-read-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }
}
