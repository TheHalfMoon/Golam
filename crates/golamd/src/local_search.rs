#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::str;
use std::time::{Duration, Instant};

use golam_core::target_identity::{ObservedFileKind, ResolvedTargetIdentity};
use golam_core::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget};

use crate::local_fs::LocalFsResolver;
use crate::local_read::{
    LocalFileReadBounds, LocalFileReadError, read_regular_file, stat_regular_file,
};
use crate::local_walk::{
    LocalDirectoryWalkBounds, LocalDirectoryWalkError, walk_directory,
};

const MAX_QUERY_BYTES: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalTextSearchBounds {
    pub max_walk_entries: u64,
    pub max_files: u64,
    pub max_matches: u64,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
    pub max_line_bytes: u64,
    pub max_depth: u32,
    pub max_duration: Duration,
}

impl LocalTextSearchBounds {
    pub fn validate(self) -> Result<(), LocalTextSearchError> {
        if self.max_walk_entries == 0
            || self.max_files == 0
            || self.max_matches == 0
            || self.max_file_bytes == 0
            || self.max_total_bytes == 0
            || self.max_line_bytes == 0
            || self.max_duration.is_zero()
            || self.max_file_bytes > self.max_total_bytes
            || usize::try_from(self.max_file_bytes).is_err()
            || usize::try_from(self.max_total_bytes).is_err()
            || usize::try_from(self.max_line_bytes).is_err()
        {
            return Err(LocalTextSearchError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkippedTextFileReason {
    NonUtf8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkippedTextFile {
    pub requested_path: RequestedTarget,
    pub identity: ResolvedTargetIdentity,
    pub content_digest: BindingDigest,
    pub reason: SkippedTextFileReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextMatchProvenance {
    pub requested_path: RequestedTarget,
    pub identity: ResolvedTargetIdentity,
    pub content_digest: BindingDigest,
    pub line_number: u64,
    pub byte_column_zero_based: u64,
    pub line_content: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedTextSearch {
    pub query: String,
    pub root: ResolvedTargetIdentity,
    pub matches: Vec<TextMatchProvenance>,
    pub skipped_files: Vec<SkippedTextFile>,
    pub files_observed: u64,
    pub bytes_observed: u64,
}

#[derive(Debug)]
pub enum LocalTextSearchError {
    Walk(LocalDirectoryWalkError),
    Read(LocalFileReadError),
    InvalidBounds,
    InvalidQuery,
    FileLimitExceeded { observed: u64, limit: u64 },
    TotalByteLimitExceeded { observed: u64, limit: u64 },
    MatchLimitExceeded { limit: u64 },
    LineLimitExceeded {
        path: RequestedTarget,
        line_number: u64,
        observed: u64,
        limit: u64,
    },
    DurationLimitExceeded,
    CounterOverflow,
}

impl fmt::Display for LocalTextSearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Walk(error) => write!(f, "bounded local text-search walk failed: {error}"),
            Self::Read(error) => write!(f, "bounded local text-search read failed: {error}"),
            Self::InvalidBounds => f.write_str(
                "bounded local text search requires positive finite walk/file/match/byte/line/time limits",
            ),
            Self::InvalidQuery => f.write_str(
                "bounded local text-search query must be non-empty, single-line, NUL-free and bounded",
            ),
            Self::FileLimitExceeded { observed, limit } => write!(
                f,
                "bounded local text-search file limit exceeded: observed={observed} limit={limit}"
            ),
            Self::TotalByteLimitExceeded { observed, limit } => write!(
                f,
                "bounded local text-search total byte limit exceeded: observed={observed} limit={limit}"
            ),
            Self::MatchLimitExceeded { limit } => write!(
                f,
                "bounded local text-search match limit exceeded: limit={limit}"
            ),
            Self::LineLimitExceeded {
                path,
                line_number,
                observed,
                limit,
            } => write!(
                f,
                "bounded local text-search line limit exceeded at {}:{}: observed={observed} limit={limit}",
                path.as_str(),
                line_number
            ),
            Self::DurationLimitExceeded => {
                f.write_str("bounded local text-search duration limit exceeded")
            }
            Self::CounterOverflow => f.write_str("bounded local text-search counter overflow"),
        }
    }
}

impl Error for LocalTextSearchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Walk(error) => Some(error),
            Self::Read(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LocalDirectoryWalkError> for LocalTextSearchError {
    fn from(value: LocalDirectoryWalkError) -> Self {
        Self::Walk(value)
    }
}

impl From<LocalFileReadError> for LocalTextSearchError {
    fn from(value: LocalFileReadError) -> Self {
        Self::Read(value)
    }
}

pub fn search_literal_text(
    resolver: &LocalFsResolver,
    requested_root: &RequestedTarget,
    list_operation: &RequestedOperationId,
    read_operation: &RequestedOperationId,
    query: &str,
    bounds: LocalTextSearchBounds,
    observed_at_unix_ms: u64,
) -> Result<BoundedTextSearch, LocalTextSearchError> {
    bounds.validate()?;
    validate_query(query)?;
    let started = Instant::now();

    let walk = walk_directory(
        resolver,
        requested_root,
        list_operation,
        LocalDirectoryWalkBounds {
            max_entries: bounds.max_walk_entries,
            max_depth: bounds.max_depth,
            max_duration: remaining_duration(started, bounds.max_duration)?,
        },
        observed_at_unix_ms,
    )?;

    let files = walk
        .entries
        .iter()
        .filter(|entry| entry.identity.file_kind == ObservedFileKind::RegularFile)
        .collect::<Vec<_>>();
    let file_count = u64::try_from(files.len()).map_err(|_| LocalTextSearchError::CounterOverflow)?;
    if file_count > bounds.max_files {
        return Err(LocalTextSearchError::FileLimitExceeded {
            observed: file_count,
            limit: bounds.max_files,
        });
    }

    let mut result = BoundedTextSearch {
        query: query.to_owned(),
        root: walk.root,
        matches: Vec::new(),
        skipped_files: Vec::new(),
        files_observed: 0,
        bytes_observed: 0,
    };

    for entry in files {
        let remaining = remaining_duration(started, bounds.max_duration)?;
        let stat = stat_regular_file(
            resolver,
            &entry.requested_path,
            read_operation,
            LocalFileReadBounds {
                max_bytes: bounds.max_file_bytes,
                max_duration: remaining,
            },
            observed_at_unix_ms,
        )?;
        let projected_total = result
            .bytes_observed
            .checked_add(stat.size_bytes)
            .ok_or(LocalTextSearchError::CounterOverflow)?;
        if projected_total > bounds.max_total_bytes {
            return Err(LocalTextSearchError::TotalByteLimitExceeded {
                observed: projected_total,
                limit: bounds.max_total_bytes,
            });
        }

        let read = read_regular_file(
            resolver,
            &entry.requested_path,
            read_operation,
            LocalFileReadBounds {
                max_bytes: bounds.max_file_bytes,
                max_duration: remaining_duration(started, bounds.max_duration)?,
            },
            observed_at_unix_ms,
            observed_at_unix_ms,
        )?;
        result.files_observed = result
            .files_observed
            .checked_add(1)
            .ok_or(LocalTextSearchError::CounterOverflow)?;
        result.bytes_observed = result
            .bytes_observed
            .checked_add(u64::try_from(read.bytes.len()).map_err(|_| LocalTextSearchError::CounterOverflow)?)
            .ok_or(LocalTextSearchError::CounterOverflow)?;
        if result.bytes_observed > bounds.max_total_bytes {
            return Err(LocalTextSearchError::TotalByteLimitExceeded {
                observed: result.bytes_observed,
                limit: bounds.max_total_bytes,
            });
        }

        let text = match str::from_utf8(&read.bytes) {
            Ok(text) => text,
            Err(_) => {
                result.skipped_files.push(SkippedTextFile {
                    requested_path: entry.requested_path.clone(),
                    identity: read.identity,
                    content_digest: read.content_digest,
                    reason: SkippedTextFileReason::NonUtf8,
                });
                continue;
            }
        };

        for (line_index, line) in text.lines().enumerate() {
            remaining_duration(started, bounds.max_duration)?;
            let line_number = u64::try_from(line_index + 1)
                .map_err(|_| LocalTextSearchError::CounterOverflow)?;
            let line_bytes = u64::try_from(line.len())
                .map_err(|_| LocalTextSearchError::CounterOverflow)?;
            if line_bytes > bounds.max_line_bytes {
                return Err(LocalTextSearchError::LineLimitExceeded {
                    path: entry.requested_path.clone(),
                    line_number,
                    observed: line_bytes,
                    limit: bounds.max_line_bytes,
                });
            }

            for (column, _) in line.match_indices(query) {
                let match_count = u64::try_from(result.matches.len())
                    .map_err(|_| LocalTextSearchError::CounterOverflow)?;
                if match_count >= bounds.max_matches {
                    return Err(LocalTextSearchError::MatchLimitExceeded {
                        limit: bounds.max_matches,
                    });
                }
                result.matches.push(TextMatchProvenance {
                    requested_path: entry.requested_path.clone(),
                    identity: read.identity.clone(),
                    content_digest: read.content_digest,
                    line_number,
                    byte_column_zero_based: u64::try_from(column)
                        .map_err(|_| LocalTextSearchError::CounterOverflow)?,
                    line_content: line.to_owned(),
                });
            }
        }
    }

    remaining_duration(started, bounds.max_duration)?;
    Ok(result)
}

fn validate_query(query: &str) -> Result<(), LocalTextSearchError> {
    if query.is_empty()
        || query.len() > MAX_QUERY_BYTES
        || query
            .chars()
            .any(|character| matches!(character, '\0' | '\r' | '\n'))
    {
        return Err(LocalTextSearchError::InvalidQuery);
    }
    Ok(())
}

fn remaining_duration(
    started: Instant,
    max_duration: Duration,
) -> Result<Duration, LocalTextSearchError> {
    max_duration
        .checked_sub(started.elapsed())
        .filter(|remaining| !remaining.is_zero())
        .ok_or(LocalTextSearchError::DurationLimitExceeded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::tool_request::ResourceClassId;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_root() -> PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "golam-local-search-{}-{nanos}-{counter}",
            std::process::id()
        ))
    }

    fn resolver(root: &Path) -> LocalFsResolver {
        LocalFsResolver::new(
            root,
            ResourceClassId::new("workspace.search").unwrap(),
            vec![
                RequestedOperationId::new("list").unwrap(),
                RequestedOperationId::new("read").unwrap(),
            ],
            [],
        )
        .unwrap()
    }

    fn bounds() -> LocalTextSearchBounds {
        LocalTextSearchBounds {
            max_walk_entries: 32,
            max_files: 16,
            max_matches: 16,
            max_file_bytes: 1024,
            max_total_bytes: 4096,
            max_line_bytes: 512,
            max_depth: 4,
            max_duration: Duration::from_secs(2),
        }
    }

    fn basename(path: &RequestedTarget) -> String {
        Path::new(path.as_str())
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn emits_deterministic_exact_match_provenance() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("b.txt"), b"other\nneedle two\n").unwrap();
        fs::write(root.join("a.txt"), b"needle one\n").unwrap();
        let resolver = resolver(&root);

        let result = search_literal_text(
            &resolver,
            &RequestedTarget::new(".").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            &RequestedOperationId::new("read").unwrap(),
            "needle",
            bounds(),
            10,
        )
        .unwrap();

        assert_eq!(result.matches.len(), 2);
        assert_eq!(basename(&result.matches[0].requested_path), "a.txt");
        assert_eq!(result.matches[0].line_number, 1);
        assert_eq!(result.matches[0].byte_column_zero_based, 0);
        assert_eq!(result.matches[0].line_content, "needle one");
        assert_eq!(basename(&result.matches[1].requested_path), "b.txt");
        assert_eq!(result.matches[1].line_number, 2);
        assert_eq!(result.matches[1].line_content, "needle two");
        assert!(result.matches.iter().all(|item| item.identity.resolved_target_identity.is_some()));
        assert!(result.skipped_files.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn match_limit_fails_closed() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("a.txt"), b"needle needle\n").unwrap();
        let resolver = resolver(&root);
        let mut limited = bounds();
        limited.max_matches = 1;
        let error = search_literal_text(
            &resolver,
            &RequestedTarget::new(".").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            &RequestedOperationId::new("read").unwrap(),
            "needle",
            limited,
            10,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            LocalTextSearchError::MatchLimitExceeded { limit: 1 }
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn total_byte_limit_fails_before_unbounded_accumulation() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("a.txt"), b"1234").unwrap();
        fs::write(root.join("b.txt"), b"5678").unwrap();
        let resolver = resolver(&root);
        let mut limited = bounds();
        limited.max_file_bytes = 4;
        limited.max_total_bytes = 5;
        let error = search_literal_text(
            &resolver,
            &RequestedTarget::new(".").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            &RequestedOperationId::new("read").unwrap(),
            "1",
            limited,
            10,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            LocalTextSearchError::TotalByteLimitExceeded {
                observed: 8,
                limit: 5
            }
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn non_utf8_file_is_explicitly_attested_as_skipped() {
        let root = unique_root();
        fs::create_dir(&root).unwrap();
        fs::write(root.join("binary.dat"), [0xff, b'n', b'e']).unwrap();
        let resolver = resolver(&root);
        let result = search_literal_text(
            &resolver,
            &RequestedTarget::new(".").unwrap(),
            &RequestedOperationId::new("list").unwrap(),
            &RequestedOperationId::new("read").unwrap(),
            "needle",
            bounds(),
            10,
        )
        .unwrap();
        assert!(result.matches.is_empty());
        assert_eq!(result.skipped_files.len(), 1);
        assert_eq!(
            result.skipped_files[0].reason,
            SkippedTextFileReason::NonUtf8
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn query_and_bounds_validation_fail_closed() {
        assert!(matches!(
            validate_query(""),
            Err(LocalTextSearchError::InvalidQuery)
        ));
        assert!(matches!(
            validate_query("two\nlines"),
            Err(LocalTextSearchError::InvalidQuery)
        ));
        let mut invalid = bounds();
        invalid.max_total_bytes = 0;
        assert!(matches!(
            invalid.validate(),
            Err(LocalTextSearchError::InvalidBounds)
        ));
    }
}
