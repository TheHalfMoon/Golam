#![forbid(unsafe_code)]

pub mod authority;
pub mod compaction;
pub mod context_authority;
pub mod context_compiler;
pub mod context_evidence;
pub mod context_projection;
pub mod desktop_control;
pub mod desktop_intent;
pub mod digest;
pub mod execution_profile;
pub mod git_authority;
pub mod harness;
pub mod harness_state;
pub mod memory;
pub mod memory_markdown;
pub mod memory_storage;
pub mod memory_transition;
pub mod model_backend;
pub mod paths;
pub mod routing;
pub mod runtime_home;
pub mod skills_protocol;
pub mod taint;
pub mod target_identity;
pub mod tool_call;
pub mod tool_descriptor;
pub mod tool_request;
#[cfg(unix)]
pub mod unix_fs;

use core::fmt;

pub const PROTOCOL_VERSION: u16 = 1;
pub const SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SessionId(pub u128);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EventId(pub u128);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CheckpointId(pub u128);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GoalId(pub u128);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GoalVersionId(pub u128);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ClientId(pub u128);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EffectId(pub u128);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EffectTransitionId(pub u128);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EffectAttemptId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolReconciliationResolution {
    Succeeded,
    Failed,
    UnknownOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolReconciliationContext {
    pub effect_id: EffectId,
    pub session_id: SessionId,
    pub action: String,
    pub resource: String,
    pub execution_semantics: String,
    pub idempotency_key: Option<String>,
    pub preconditions_hash: [u8; 32],
    pub payload_hash: [u8; 32],
    pub attempt_id: EffectAttemptId,
    pub started_global_seq: u64,
    pub handler_id: String,
    pub handler_version: String,
    pub dispatch_token: Vec<u8>,
    pub attempt_outcome: String,
    pub receipt: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolReconciliationResult {
    Resolved {
        effect_id: EffectId,
        state: String,
    },
    ManualReview {
        effect_id: EffectId,
        incident_id: [u8; 16],
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
    pub max_frame_bytes: u32,
    pub max_pending_requests: u32,
    pub max_concurrent_clients: u16,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_frame_bytes: 1024 * 1024,
            max_pending_requests: 128,
            max_concurrent_clients: 16,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreError {
    InvalidProtocolVersion,
    ResourceLimitExceeded,
    CanonicalLengthOverflow,
    InvalidCanonicalTaintSet,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProtocolVersion => f.write_str("invalid protocol version"),
            Self::ResourceLimitExceeded => f.write_str("resource limit exceeded"),
            Self::CanonicalLengthOverflow => f.write_str("canonical field length exceeds u32"),
            Self::InvalidCanonicalTaintSet => f.write_str("invalid canonical taint set"),
        }
    }
}

impl std::error::Error for CoreError {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CanonicalEncoder {
    bytes: Vec<u8>,
}

impl CanonicalEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn push_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub fn push_u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub fn push_u128(&mut self, value: u128) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub fn push_bytes(&mut self, value: &[u8]) -> Result<(), CoreError> {
        let len = u32::try_from(value.len()).map_err(|_| CoreError::CanonicalLengthOverflow)?;
        self.bytes.extend_from_slice(&len.to_be_bytes());
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    pub fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_bounded() {
        let limits = ResourceLimits::default();
        assert!(limits.max_frame_bytes <= 1024 * 1024);
        assert!(limits.max_pending_requests > 0);
        assert!(limits.max_concurrent_clients > 0);
    }

    #[test]
    fn canonical_encoding_is_explicit_and_big_endian() {
        let mut encoder = CanonicalEncoder::new();
        encoder.push_u8(0x7f);
        encoder.push_u16(0x0102);
        encoder.push_u64(3);
        encoder.push_bytes(b"ok").unwrap();

        assert_eq!(
            encoder.finish(),
            vec![
                0x7f, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
                0x02, b'o', b'k',
            ]
        );
    }
}
