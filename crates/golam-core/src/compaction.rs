use crate::harness::CompactionId;
use crate::harness_state::{
    CompactionArtifact, CompactionAttempt, CompactionState, ContextProjection, HarnessStateError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionSourceBinding {
    pub source_projection_ref: String,
    pub source_event_refs: Vec<String>,
    pub source_artifact_refs: Vec<String>,
    pub goal_refs: Vec<String>,
    pub source_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicCompactionTransaction {
    pub attempt: CompactionAttempt,
    pub source: CompactionSourceBinding,
}

pub fn begin_deterministic_compaction(
    compaction_id: CompactionId,
    source_projection: &ContextProjection,
    source_digest: [u8; 32],
    started_at_unix_ms: u64,
) -> Result<DeterministicCompactionTransaction, HarnessStateError> {
    source_projection.validate()?;

    let source = CompactionSourceBinding {
        source_projection_ref: source_projection.projection_ref.clone(),
        source_event_refs: source_projection.source_event_refs.clone(),
        source_artifact_refs: source_projection.source_artifact_refs.clone(),
        goal_refs: source_projection.goal_refs.clone(),
        source_digest,
    };
    let attempt = CompactionAttempt {
        compaction_id,
        session_id: source_projection.session_id,
        source_projection_ref: source_projection.projection_ref.clone(),
        state: CompactionState::Started,
        deterministic: true,
        producing_request_attempt_id: None,
        started_at_unix_ms,
        terminal_at_unix_ms: None,
        failure_class: None,
    };
    attempt.validate()?;

    Ok(DeterministicCompactionTransaction { attempt, source })
}

impl DeterministicCompactionTransaction {
    pub fn begin_derivation(&mut self) -> Result<(), HarnessStateError> {
        self.attempt.transition(CompactionState::Deriving)
    }

    pub fn commit(
        mut self,
        artifact_digest: [u8; 32],
        terminal_at_unix_ms: u64,
    ) -> Result<(CompactionAttempt, CompactionArtifact), HarnessStateError> {
        if terminal_at_unix_ms < self.attempt.started_at_unix_ms {
            return Err(HarnessStateError::InvalidBounds);
        }

        self.attempt.transition(CompactionState::Validating)?;
        let artifact = CompactionArtifact {
            compaction_id: self.attempt.compaction_id,
            source_projection_ref: self.source.source_projection_ref.clone(),
            source_event_refs: self.source.source_event_refs.clone(),
            goal_refs: self.source.goal_refs.clone(),
            deterministic: true,
            producing_request_attempt_id: None,
            accepted_output_ref: None,
            artifact_digest,
        };
        artifact.validate()?;

        self.attempt.transition(CompactionState::Committed)?;
        self.attempt.terminal_at_unix_ms = Some(terminal_at_unix_ms);
        self.attempt.validate()?;

        Ok((self.attempt, artifact))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SessionId;
    use crate::harness::ExecutionProfileId;

    fn projection() -> ContextProjection {
        ContextProjection {
            projection_ref: "projection:session-1:turn-4".into(),
            session_id: SessionId(1),
            execution_profile_id: ExecutionProfileId::from_u128(2),
            source_event_refs: vec!["event:10".into(), "event:11".into()],
            source_artifact_refs: vec!["artifact:7".into()],
            goal_refs: vec!["goal:3:version:2".into()],
            compaction_refs: Vec::new(),
            taint_refs: vec!["taint:model-generated".into()],
            max_tokens: 4096,
            render_policy_digest: [5; 32],
            rendered_digest: [6; 32],
            created_at_unix_ms: 7,
        }
    }

    #[test]
    fn transaction_binds_exact_sources_without_mutating_projection() {
        let source_projection = projection();
        let original = source_projection.clone();

        let transaction = begin_deterministic_compaction(
            CompactionId::from_u128(8),
            &source_projection,
            [9; 32],
            10,
        )
        .unwrap();

        assert_eq!(source_projection, original);
        assert_eq!(
            transaction.source.source_event_refs,
            source_projection.source_event_refs
        );
        assert_eq!(
            transaction.source.source_artifact_refs,
            source_projection.source_artifact_refs
        );
        assert_eq!(transaction.source.goal_refs, source_projection.goal_refs);
        assert_eq!(transaction.source.source_digest, [9; 32]);
    }

    #[test]
    fn commit_requires_derivation_and_preserves_source_identity() {
        let source_projection = projection();
        let transaction = begin_deterministic_compaction(
            CompactionId::from_u128(8),
            &source_projection,
            [9; 32],
            10,
        )
        .unwrap();

        assert_eq!(
            transaction.clone().commit([10; 32], 11),
            Err(HarnessStateError::InvalidTransition)
        );

        let mut transaction = transaction;
        transaction.begin_derivation().unwrap();
        let (attempt, artifact) = transaction.commit([10; 32], 11).unwrap();

        assert_eq!(attempt.state, CompactionState::Committed);
        assert_eq!(attempt.terminal_at_unix_ms, Some(11));
        assert_eq!(artifact.compaction_id, CompactionId::from_u128(8));
        assert_eq!(
            artifact.source_projection_ref,
            source_projection.projection_ref
        );
        assert_eq!(
            artifact.source_event_refs,
            source_projection.source_event_refs
        );
        assert_eq!(artifact.goal_refs, source_projection.goal_refs);
        assert_eq!(artifact.artifact_digest, [10; 32]);
    }

    #[test]
    fn commit_rejects_terminal_time_before_start() {
        let source_projection = projection();
        let mut transaction = begin_deterministic_compaction(
            CompactionId::from_u128(8),
            &source_projection,
            [9; 32],
            10,
        )
        .unwrap();
        transaction.begin_derivation().unwrap();

        assert_eq!(
            transaction.commit([10; 32], 9),
            Err(HarnessStateError::InvalidBounds)
        );
    }
}
