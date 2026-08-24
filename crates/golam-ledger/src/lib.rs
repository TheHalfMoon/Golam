#![forbid(unsafe_code)]

use golam_core::SessionId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    SessionCreated,
    GoalVersioned,
    EffectProposed,
    EffectTransitioned,
    CheckpointCreated,
    SessionForked,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_validation_rejects_gaps() {
        let first = EventRecord {
            session_id: SessionId(1),
            global_seq: 4,
            session_seq: 2,
            schema_version: 1,
            kind: EventKind::SessionCreated,
            previous_integrity: [0; 32],
        };
        let mut next = first.clone();
        next.global_seq = 6;
        next.session_seq = 3;
        assert!(!follows(&first, &next));
    }
}
