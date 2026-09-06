#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::desktop_control::DesktopControlError;
use golam_core::tool_request::BindingDigest;
use golam_core::{CanonicalEncoder, CoreError, EffectId, SessionId};

const EFFECT_EVIDENCE_DOMAIN: &[u8] = b"golam:desktop-effect-evidence:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopEvidenceOperation {
    SemanticAction,
    Focus,
    RawInputFallback,
    Capture,
    ClipboardRead,
    ClipboardWrite,
}

impl DesktopEvidenceOperation {
    pub(crate) const fn code(self) -> i64 {
        match self {
            Self::SemanticAction => 1,
            Self::Focus => 2,
            Self::RawInputFallback => 3,
            Self::Capture => 4,
            Self::ClipboardRead => 5,
            Self::ClipboardWrite => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopEvidenceStatus {
    Prepared,
    Succeeded,
    Failed,
    UnknownOutcome,
    Interrupted,
    Reconciling,
    ReconciledSucceeded,
    ReconciledFailed,
    ManualReview,
}

impl DesktopEvidenceStatus {
    pub(crate) const fn code(self) -> i64 {
        match self {
            Self::Prepared => 1,
            Self::Succeeded => 2,
            Self::Failed => 3,
            Self::UnknownOutcome => 4,
            Self::Interrupted => 5,
            Self::Reconciling => 6,
            Self::ReconciledSucceeded => 7,
            Self::ReconciledFailed => 8,
            Self::ManualReview => 9,
        }
    }

    pub(crate) fn from_code(value: i64) -> Result<Self, DesktopControlEvidenceError> {
        match value {
            1 => Ok(Self::Prepared),
            2 => Ok(Self::Succeeded),
            3 => Ok(Self::Failed),
            4 => Ok(Self::UnknownOutcome),
            5 => Ok(Self::Interrupted),
            6 => Ok(Self::Reconciling),
            7 => Ok(Self::ReconciledSucceeded),
            8 => Ok(Self::ReconciledFailed),
            9 => Ok(Self::ManualReview),
            _ => Err(DesktopControlEvidenceError::InvalidStoredRecord(
                "desktop evidence status",
            )),
        }
    }

    pub const fn unresolved(self) -> bool {
        matches!(self, Self::UnknownOutcome | Self::Reconciling)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopEffectEvidence {
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub operation: DesktopEvidenceOperation,
    pub request_digest: BindingDigest,
    pub effect_digest: BindingDigest,
    pub intent_digest: BindingDigest,
    pub fallback_eligibility_digest: Option<BindingDigest>,
    pub control_lease_digest: Option<BindingDigest>,
    pub visible_channel_digest: Option<BindingDigest>,
    pub permission_session_digest: BindingDigest,
    pub target_or_source_digest: BindingDigest,
    pub status: DesktopEvidenceStatus,
    pub reconciliation_ref: Option<BindingDigest>,
    pub recorded_at_unix_ms: u64,
}

impl DesktopEffectEvidence {
    pub fn validate(&self) -> Result<(), DesktopControlEvidenceError> {
        if self.effect_id.0 == 0 || self.session_id.0 == 0 || self.recorded_at_unix_ms == 0 {
            return Err(DesktopControlEvidenceError::InvalidEvidence);
        }
        for digest in [
            self.request_digest,
            self.effect_digest,
            self.intent_digest,
            self.permission_session_digest,
            self.target_or_source_digest,
        ] {
            require_digest(digest)?;
        }
        for digest in [
            self.fallback_eligibility_digest,
            self.control_lease_digest,
            self.visible_channel_digest,
            self.reconciliation_ref,
        ]
        .into_iter()
        .flatten()
        {
            require_digest(digest)?;
        }
        match self.operation {
            DesktopEvidenceOperation::SemanticAction | DesktopEvidenceOperation::Focus => {
                if self.fallback_eligibility_digest.is_some()
                    || self.control_lease_digest.is_none()
                    || self.visible_channel_digest.is_none()
                {
                    return Err(DesktopControlEvidenceError::InvalidEvidence);
                }
            }
            DesktopEvidenceOperation::RawInputFallback => {
                if self.fallback_eligibility_digest.is_none()
                    || self.control_lease_digest.is_none()
                    || self.visible_channel_digest.is_none()
                {
                    return Err(DesktopControlEvidenceError::InvalidEvidence);
                }
            }
            DesktopEvidenceOperation::Capture
            | DesktopEvidenceOperation::ClipboardRead
            | DesktopEvidenceOperation::ClipboardWrite => {
                if self.control_lease_digest.is_some() || self.visible_channel_digest.is_some() {
                    return Err(DesktopControlEvidenceError::InvalidEvidence);
                }
            }
        }
        let reconciliation_status = matches!(
            self.status,
            DesktopEvidenceStatus::Reconciling
                | DesktopEvidenceStatus::ReconciledSucceeded
                | DesktopEvidenceStatus::ReconciledFailed
                | DesktopEvidenceStatus::ManualReview
        );
        if reconciliation_status != self.reconciliation_ref.is_some() {
            return Err(DesktopControlEvidenceError::InvalidEvidence);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DesktopControlEvidenceError> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(EFFECT_EVIDENCE_DOMAIN)?;
        encoder.push_u128(self.effect_id.0);
        encoder.push_u128(self.session_id.0);
        encoder.push_u8(operation_code(self.operation));
        push_digest(&mut encoder, self.request_digest)?;
        push_digest(&mut encoder, self.effect_digest)?;
        push_digest(&mut encoder, self.intent_digest)?;
        push_optional_digest(&mut encoder, self.fallback_eligibility_digest)?;
        push_optional_digest(&mut encoder, self.control_lease_digest)?;
        push_optional_digest(&mut encoder, self.visible_channel_digest)?;
        push_digest(&mut encoder, self.permission_session_digest)?;
        push_digest(&mut encoder, self.target_or_source_digest)?;
        encoder.push_u8(status_code(self.status));
        push_optional_digest(&mut encoder, self.reconciliation_ref)?;
        encoder.push_u64(self.recorded_at_unix_ms);
        Ok(encoder.finish())
    }

    pub fn payload_hash(&self) -> Result<[u8; 32], DesktopControlEvidenceError> {
        Ok(crate::payload_hash(&self.canonical_bytes()?))
    }
}

fn operation_code(operation: DesktopEvidenceOperation) -> u8 {
    match operation {
        DesktopEvidenceOperation::SemanticAction => 1,
        DesktopEvidenceOperation::Focus => 2,
        DesktopEvidenceOperation::RawInputFallback => 3,
        DesktopEvidenceOperation::Capture => 4,
        DesktopEvidenceOperation::ClipboardRead => 5,
        DesktopEvidenceOperation::ClipboardWrite => 6,
    }
}

fn status_code(status: DesktopEvidenceStatus) -> u8 {
    match status {
        DesktopEvidenceStatus::Prepared => 1,
        DesktopEvidenceStatus::Succeeded => 2,
        DesktopEvidenceStatus::Failed => 3,
        DesktopEvidenceStatus::UnknownOutcome => 4,
        DesktopEvidenceStatus::Interrupted => 5,
        DesktopEvidenceStatus::Reconciling => 6,
        DesktopEvidenceStatus::ReconciledSucceeded => 7,
        DesktopEvidenceStatus::ReconciledFailed => 8,
        DesktopEvidenceStatus::ManualReview => 9,
    }
}

fn require_digest(digest: BindingDigest) -> Result<(), DesktopControlEvidenceError> {
    if digest.bytes() == [0; 32] {
        Err(DesktopControlEvidenceError::InvalidEvidence)
    } else {
        Ok(())
    }
}

fn push_digest(
    encoder: &mut CanonicalEncoder,
    digest: BindingDigest,
) -> Result<(), DesktopControlEvidenceError> {
    require_digest(digest)?;
    encoder.push_bytes(&digest.bytes())?;
    Ok(())
}

fn push_optional_digest(
    encoder: &mut CanonicalEncoder,
    digest: Option<BindingDigest>,
) -> Result<(), DesktopControlEvidenceError> {
    match digest {
        Some(value) => {
            encoder.push_u8(1);
            push_digest(encoder, value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

#[derive(Debug)]
pub enum DesktopControlEvidenceError {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Control(DesktopControlError),
    InvalidEvidence,
    InvalidEvidenceTransition,
    ImmutableEvidenceMismatch,
    StaleGeneration,
    NonMonotonicTime,
    IntegrityMismatch,
    IntegerOverflow,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for DesktopControlEvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "desktop control evidence sqlite error: {error}"),
            Self::Core(error) => write!(f, "desktop control evidence encoding error: {error}"),
            Self::Control(error) => write!(f, "desktop control evidence contract error: {error}"),
            Self::InvalidEvidence => f.write_str("desktop control evidence is invalid"),
            Self::InvalidEvidenceTransition => {
                f.write_str("desktop effect evidence transition is invalid")
            }
            Self::ImmutableEvidenceMismatch => {
                f.write_str("desktop evidence identity collision or immutable mismatch")
            }
            Self::StaleGeneration => {
                f.write_str("desktop authority state generation is stale or substituted")
            }
            Self::NonMonotonicTime => f.write_str("desktop evidence time moved backwards"),
            Self::IntegrityMismatch => f.write_str("desktop evidence integrity validation failed"),
            Self::IntegerOverflow => f.write_str("desktop evidence integer overflow"),
            Self::InvalidStoredRecord(field) => {
                write!(f, "desktop evidence stored record invalid: {field}")
            }
        }
    }
}

impl Error for DesktopControlEvidenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Control(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for DesktopControlEvidenceError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for DesktopControlEvidenceError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<DesktopControlError> for DesktopControlEvidenceError {
    fn from(value: DesktopControlError) -> Self {
        Self::Control(value)
    }
}
