#![forbid(unsafe_code)]

use core::fmt;
use std::collections::BTreeMap;

use crate::digest::sha256;
use crate::memory::{MemoryItemId, MemoryScope, MemoryVersionId};
use crate::tool_request::BindingDigest;

const MAX_MARKDOWN_BYTES: usize = 1024 * 1024;
const MAX_FRONT_MATTER_ENTRIES: usize = 64;
const MAX_KEY_BYTES: usize = 64;
const MAX_VALUE_BYTES: usize = 4096;

const RESERVED_AUTHORITY_KEYS: &[&str] = &[
    "approval",
    "approval_id",
    "authorization",
    "effect_id",
    "item_id",
    "kernel_authorization",
    "memory_store_ref",
    "promotion",
    "promotion_authority",
    "provenance",
    "provenance_authority",
    "scope",
    "taint",
    "version_id",
    "writer_id",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedMarkdownDocument {
    metadata: BTreeMap<String, String>,
    body: String,
}

impl ManagedMarkdownDocument {
    pub fn new(
        metadata: BTreeMap<String, String>,
        body: impl Into<String>,
    ) -> Result<Self, ManagedMarkdownError> {
        let value = Self {
            metadata,
            body: body.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn serialize(&self) -> Result<Vec<u8>, ManagedMarkdownError> {
        self.validate()?;
        let mut output = String::new();
        if !self.metadata.is_empty() {
            output.push_str("---\n");
            for (key, value) in &self.metadata {
                output.push_str(key);
                output.push_str(": ");
                output.push_str(value);
                output.push('\n');
            }
            output.push_str("---\n");
        }
        output.push_str(&self.body);
        if output.len() > MAX_MARKDOWN_BYTES {
            return Err(ManagedMarkdownError::TooLarge);
        }
        Ok(output.into_bytes())
    }

    pub fn content_digest(&self) -> Result<BindingDigest, ManagedMarkdownError> {
        Ok(BindingDigest::new(sha256(&self.serialize()?)))
    }

    fn validate(&self) -> Result<(), ManagedMarkdownError> {
        if self.metadata.len() > MAX_FRONT_MATTER_ENTRIES {
            return Err(ManagedMarkdownError::TooManyFrontMatterEntries);
        }
        if self.body.as_bytes().contains(&0) || self.body.contains('\r') {
            return Err(ManagedMarkdownError::InvalidBody);
        }
        if self.metadata.is_empty() && self.body.starts_with("---\n") {
            return Err(ManagedMarkdownError::InvalidBody);
        }
        for (key, value) in &self.metadata {
            validate_key(key)?;
            validate_value(value)?;
        }
        let projected = self.body.len()
            + self
                .metadata
                .iter()
                .map(|(key, value)| key.len() + value.len() + 3)
                .sum::<usize>()
            + if self.metadata.is_empty() { 0 } else { 8 };
        if projected > MAX_MARKDOWN_BYTES {
            return Err(ManagedMarkdownError::TooLarge);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedMemoryEnvelope {
    pub item_id: MemoryItemId,
    pub version_id: MemoryVersionId,
    pub scope: MemoryScope,
    pub document: ManagedMarkdownDocument,
    pub content_digest: BindingDigest,
}

impl ManagedMemoryEnvelope {
    pub fn bind(
        item_id: MemoryItemId,
        version_id: MemoryVersionId,
        scope: MemoryScope,
        document: ManagedMarkdownDocument,
    ) -> Result<Self, ManagedMarkdownError> {
        let content_digest = document.content_digest()?;
        Ok(Self {
            item_id,
            version_id,
            scope,
            document,
            content_digest,
        })
    }

    pub fn verify_content(&self) -> Result<(), ManagedMarkdownError> {
        if self.document.content_digest()? != self.content_digest {
            return Err(ManagedMarkdownError::ContentDigestMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManagedMarkdownError {
    TooLarge,
    InvalidUtf8,
    InvalidBody,
    MissingFrontMatterTerminator,
    TooManyFrontMatterEntries,
    InvalidFrontMatterLine,
    InvalidKey(String),
    InvalidValue(String),
    DuplicateKey(String),
    AuthorityBearingFrontMatter(String),
    ContentDigestMismatch,
}

impl fmt::Display for ManagedMarkdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => f.write_str("managed Markdown exceeds the bounded byte limit"),
            Self::InvalidUtf8 => f.write_str("managed Markdown must be valid UTF-8"),
            Self::InvalidBody => {
                f.write_str("managed Markdown body contains a forbidden control sequence")
            }
            Self::MissingFrontMatterTerminator => {
                f.write_str("managed Markdown front matter is not terminated")
            }
            Self::TooManyFrontMatterEntries => {
                f.write_str("managed Markdown front matter entry bound exceeded")
            }
            Self::InvalidFrontMatterLine => {
                f.write_str("managed Markdown front matter line is not canonical key/value data")
            }
            Self::InvalidKey(key) => write!(f, "managed Markdown metadata key is invalid: {key}"),
            Self::InvalidValue(key) => {
                write!(f, "managed Markdown metadata value is invalid: {key}")
            }
            Self::DuplicateKey(key) => {
                write!(f, "managed Markdown metadata key is duplicated: {key}")
            }
            Self::AuthorityBearingFrontMatter(key) => write!(
                f,
                "managed Markdown content attempts to set protected authority metadata: {key}"
            ),
            Self::ContentDigestMismatch => {
                f.write_str("managed Markdown content no longer matches its protected digest")
            }
        }
    }
}

impl std::error::Error for ManagedMarkdownError {}

pub fn parse_managed_markdown(
    input: &[u8],
) -> Result<ManagedMarkdownDocument, ManagedMarkdownError> {
    if input.len() > MAX_MARKDOWN_BYTES {
        return Err(ManagedMarkdownError::TooLarge);
    }
    let text = std::str::from_utf8(input).map_err(|_| ManagedMarkdownError::InvalidUtf8)?;
    if text.as_bytes().contains(&0) || text.contains('\r') {
        return Err(ManagedMarkdownError::InvalidBody);
    }
    if !text.starts_with("---\n") {
        return ManagedMarkdownDocument::new(BTreeMap::new(), text);
    }

    let remainder = &text[4..];
    let Some(end) = remainder.find("\n---\n") else {
        return Err(ManagedMarkdownError::MissingFrontMatterTerminator);
    };
    let front_matter = &remainder[..end];
    let body = &remainder[end + 5..];
    let mut metadata = BTreeMap::new();
    if !front_matter.is_empty() {
        for line in front_matter.lines() {
            if metadata.len() >= MAX_FRONT_MATTER_ENTRIES {
                return Err(ManagedMarkdownError::TooManyFrontMatterEntries);
            }
            let Some((key, value)) = line.split_once(": ") else {
                return Err(ManagedMarkdownError::InvalidFrontMatterLine);
            };
            validate_key(key)?;
            validate_value(value)?;
            if metadata.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(ManagedMarkdownError::DuplicateKey(key.to_owned()));
            }
        }
    }
    ManagedMarkdownDocument::new(metadata, body)
}

fn validate_key(key: &str) -> Result<(), ManagedMarkdownError> {
    if key.is_empty()
        || key.len() > MAX_KEY_BYTES
        || key.trim() != key
        || !key.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
    {
        return Err(ManagedMarkdownError::InvalidKey(key.to_owned()));
    }
    if RESERVED_AUTHORITY_KEYS.binary_search(&key).is_ok() {
        return Err(ManagedMarkdownError::AuthorityBearingFrontMatter(
            key.to_owned(),
        ));
    }
    Ok(())
}

fn validate_value(value: &str) -> Result<(), ManagedMarkdownError> {
    if value.len() > MAX_VALUE_BYTES || value.contains(['\n', '\r', '\0']) || value.trim() != value
    {
        return Err(ManagedMarkdownError::InvalidValue("value".to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    #[test]
    fn parser_round_trips_safe_content_deterministically() {
        let input = b"---\ntags: rust,local\ntitle: Project note\n---\nbody\n";
        let parsed = parse_managed_markdown(input).unwrap();
        assert_eq!(parsed.serialize().unwrap(), input);
        let first = parsed.content_digest().unwrap();
        assert_eq!(first, parsed.content_digest().unwrap());
    }

    #[test]
    fn delimiter_prefixed_plain_body_is_rejected_as_ambiguous() {
        let body = "---\ntitle: forged-shape\n---\nbody\n";
        assert_eq!(
            ManagedMarkdownDocument::new(BTreeMap::new(), body),
            Err(ManagedMarkdownError::InvalidBody)
        );
        assert!(parse_managed_markdown(b"---\n---\nbody\n").is_err());
    }

    #[test]
    fn authority_bearing_front_matter_is_quarantined_not_imported() {
        for key in [
            "scope",
            "taint",
            "approval",
            "authorization",
            "item_id",
            "version_id",
            "writer_id",
            "effect_id",
            "promotion_authority",
            "memory_store_ref",
        ] {
            let text = format!("---\n{key}: forged\n---\nbody\n");
            assert!(matches!(
                parse_managed_markdown(text.as_bytes()),
                Err(ManagedMarkdownError::AuthorityBearingFrontMatter(found)) if found == key
            ));
        }
    }

    #[test]
    fn protected_identity_is_external_to_markdown_content() {
        let document = parse_managed_markdown(b"hello\n").unwrap();
        let envelope = ManagedMemoryEnvelope::bind(
            MemoryItemId(digest(1)),
            MemoryVersionId(digest(2)),
            MemoryScope::Project,
            document,
        )
        .unwrap();
        envelope.verify_content().unwrap();
        assert_eq!(envelope.item_id, MemoryItemId(digest(1)));
        assert_eq!(envelope.version_id, MemoryVersionId(digest(2)));
    }

    #[test]
    fn duplicate_or_noncanonical_metadata_fails_closed() {
        assert!(matches!(
            parse_managed_markdown(b"---\ntitle: one\ntitle: two\n---\nbody"),
            Err(ManagedMarkdownError::DuplicateKey(_))
        ));
        assert!(matches!(
            parse_managed_markdown(b"---\nTitle: bad\n---\nbody"),
            Err(ManagedMarkdownError::InvalidKey(_))
        ));
    }
}
