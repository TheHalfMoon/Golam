#![forbid(unsafe_code)]

pub mod storage;

use golam_core::{CanonicalEncoder, CoreError, SessionId};

const EVENT_DOMAIN: &[u8] = b"golam:event:v1";

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
    const fn code(self) -> u8 {
        match self {
            Self::SessionCreated => 1,
            Self::GoalVersioned => 2,
            Self::EffectProposed => 3,
            Self::EffectTransitioned => 4,
            Self::CheckpointCreated => 5,
            Self::SessionForked => 6,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRecord {
    pub session_id: SessionId,
    pub global_seq: u64,
    pub session_seq: u64,
    pub schema_version: u16,
    pub kind: EventKind,
    pub previous_integrity: [u8; 32],
}

pub fn follows(previous: &EventRecord, next: &EventRecord) -> bool {
    next.global_seq == previous.global_seq + 1
        && (next.session_id != previous.session_id || next.session_seq == previous.session_seq + 1)
}

pub fn canonical_event_bytes(record: &EventRecord) -> Result<Vec<u8>, CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(EVENT_DOMAIN)?;
    encoder.push_u16(record.schema_version);
    encoder.push_u128(record.session_id.0);
    encoder.push_u64(record.global_seq);
    encoder.push_u64(record.session_seq);
    encoder.push_u8(record.kind.code());
    encoder.push_bytes(&record.previous_integrity)?;
    Ok(encoder.finish())
}

pub fn integrity_hash(record: &EventRecord) -> Result<[u8; 32], CoreError> {
    let bytes = canonical_event_bytes(record)?;
    Ok(*blake3::hash(&bytes).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event() -> EventRecord {
        EventRecord {
            session_id: SessionId(1),
            global_seq: 4,
            session_seq: 2,
            schema_version: 1,
            kind: EventKind::SessionCreated,
            previous_integrity: [0; 32],
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
    fn integrity_hash_is_deterministic_and_field_sensitive() {
        let first = event();
        let first_hash = integrity_hash(&first).unwrap();
        assert_eq!(first_hash, integrity_hash(&first).unwrap());

        let mut changed = first;
        changed.global_seq += 1;
        assert_ne!(first_hash, integrity_hash(&changed).unwrap());
    }
}
