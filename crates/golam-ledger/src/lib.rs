#![forbid(unsafe_code)]

pub mod artifacts;
pub mod checkpoint;
pub mod clients;
pub mod fork;
pub mod goal;
pub mod integrity;
pub mod protocol_audit;
pub mod storage;

use golam_core::{CanonicalEncoder, CoreError, EventId, SessionId};

const EVENT_DOMAIN: &[u8] = b"golam:event:v1";
const AUDIT_DOMAIN: &[u8] = b"golam:audit:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    SessionCreated,
    GoalVersioned,
    EffectProposed,
    EffectTransitioned,
    CheckpointCreated,
    SessionForked,
}

impl EventKind {
    pub const fn code(self) -> u8 {
        match self {
            Self::SessionCreated => 1,
            Self::GoalVersioned => 2,
            Self::EffectProposed => 3,
            Self::EffectTransitioned => 4,
            Self::CheckpointCreated => 5,
            Self::SessionForked => 6,
        }
    }

    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::SessionCreated),
            2 => Some(Self::GoalVersioned),
            3 => Some(Self::EffectProposed),
            4 => Some(Self::EffectTransitioned),
            5 => Some(Self::CheckpointCreated),
            6 => Some(Self::SessionForked),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRecord {
    pub event_id: EventId,
    pub session_id: SessionId,
    pub global_seq: u64,
    pub session_seq: u64,
    pub schema_version: u16,
    pub kind: EventKind,
    pub actor_principal: String,
    pub recorded_at: String,
    pub payload_hash: [u8; 32],
    pub previous_session_event_hash: Option<[u8; 32]>,
    pub security_critical: bool,
    pub previous_audit_hash: Option<[u8; 32]>,
}

pub fn follows(previous: &EventRecord, next: &EventRecord) -> bool {
    next.global_seq == previous.global_seq + 1
        && (next.session_id != previous.session_id || next.session_seq == previous.session_seq + 1)
}

pub fn payload_hash(payload: &[u8]) -> [u8; 32] {
    *blake3::hash(payload).as_bytes()
}

pub fn canonical_event_bytes(record: &EventRecord) -> Result<Vec<u8>, CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(EVENT_DOMAIN)?;
    encoder.push_u16(record.schema_version);
    encoder.push_u128(record.event_id.0);
    encoder.push_u128(record.session_id.0);
    encoder.push_u64(record.global_seq);
    encoder.push_u64(record.session_seq);
    encoder.push_u8(record.kind.code());
    encoder.push_bytes(record.actor_principal.as_bytes())?;
    encoder.push_bytes(record.recorded_at.as_bytes())?;
    encoder.push_bytes(&record.payload_hash)?;
    encode_optional_hash(&mut encoder, record.previous_session_event_hash)?;
    encoder.push_u8(u8::from(record.security_critical));
    Ok(encoder.finish())
}

pub fn event_integrity_hash(record: &EventRecord) -> Result<[u8; 32], CoreError> {
    let bytes = canonical_event_bytes(record)?;
    Ok(*blake3::hash(&bytes).as_bytes())
}

pub fn audit_integrity_hash(
    record: &EventRecord,
    event_hash: [u8; 32],
) -> Result<[u8; 32], CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(AUDIT_DOMAIN)?;
    encoder.push_u64(record.global_seq);
    encoder.push_bytes(&event_hash)?;
    encode_optional_hash(&mut encoder, record.previous_audit_hash)?;
    Ok(*blake3::hash(&encoder.finish()).as_bytes())
}

fn encode_optional_hash(
    encoder: &mut CanonicalEncoder,
    value: Option<[u8; 32]>,
) -> Result<(), CoreError> {
    match value {
        Some(hash) => {
            encoder.push_u8(1);
            encoder.push_bytes(&hash)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event() -> EventRecord {
        EventRecord {
            event_id: EventId(10),
            session_id: SessionId(1),
            global_seq: 4,
            session_seq: 2,
            schema_version: 1,
            kind: EventKind::SessionCreated,
            actor_principal: "owner".to_owned(),
            recorded_at: "2026-08-24T00:00:00Z".to_owned(),
            payload_hash: payload_hash(b"payload"),
            previous_session_event_hash: None,
            security_critical: true,
            previous_audit_hash: None,
        }
    }

    #[test]
    fn sequence_validation_rejects_gaps() {
        let first = event();
        let mut next = first.clone();
        next.global_seq = 6;
        next.session_seq = 3;
        assert!(!follows(&first, &next));
    }

    #[test]
    fn event_hash_is_deterministic_and_payload_sensitive() {
        let first = event();
        let first_hash = event_integrity_hash(&first).unwrap();
        assert_eq!(first_hash, event_integrity_hash(&first).unwrap());
        let mut changed = first;
        changed.payload_hash = payload_hash(b"changed");
        assert_ne!(first_hash, event_integrity_hash(&changed).unwrap());
    }

    #[test]
    fn audit_hash_is_previous_head_sensitive() {
        let first = event();
        let event_hash = event_integrity_hash(&first).unwrap();
        let first_audit = audit_integrity_hash(&first, event_hash).unwrap();
        let mut changed = first;
        changed.previous_audit_hash = Some([7; 32]);
        assert_ne!(
            first_audit,
            audit_integrity_hash(&changed, event_hash).unwrap()
        );
    }
}
