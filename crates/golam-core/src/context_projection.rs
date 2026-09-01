use std::collections::BTreeSet;

use crate::SessionId;
use crate::harness::ExecutionProfileId;
use crate::harness_state::{ContextProjection, HarnessStateError};

const MAX_PROJECTION_ITEMS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextProjectionInput {
    pub projection_ref: String,
    pub session_id: SessionId,
    pub execution_profile_id: ExecutionProfileId,
    pub source_event_refs: Vec<String>,
    pub source_artifact_refs: Vec<String>,
    pub goal_refs: Vec<String>,
    pub compaction_refs: Vec<crate::harness::CompactionId>,
    pub taint_refs: Vec<String>,
    pub max_tokens: u32,
    pub render_policy_digest: [u8; 32],
    pub rendered_digest: [u8; 32],
    pub created_at_unix_ms: u64,
}

pub fn build_context_projection(
    input: ContextProjectionInput,
) -> Result<ContextProjection, HarnessStateError> {
    reject_duplicate_refs(&input.source_event_refs)?;
    reject_duplicate_refs(&input.source_artifact_refs)?;
    reject_duplicate_refs(&input.goal_refs)?;
    reject_duplicate_refs(&input.taint_refs)?;

    if input.compaction_refs.len() > MAX_PROJECTION_ITEMS {
        return Err(HarnessStateError::TooManyItems);
    }

    let projection = ContextProjection {
        projection_ref: input.projection_ref,
        session_id: input.session_id,
        execution_profile_id: input.execution_profile_id,
        source_event_refs: input.source_event_refs,
        source_artifact_refs: input.source_artifact_refs,
        goal_refs: input.goal_refs,
        compaction_refs: input.compaction_refs,
        taint_refs: input.taint_refs,
        max_tokens: input.max_tokens,
        render_policy_digest: input.render_policy_digest,
        rendered_digest: input.rendered_digest,
        created_at_unix_ms: input.created_at_unix_ms,
    };
    projection.validate()?;
    Ok(projection)
}

fn reject_duplicate_refs(refs: &[String]) -> Result<(), HarnessStateError> {
    if refs.len() > MAX_PROJECTION_ITEMS {
        return Err(HarnessStateError::TooManyItems);
    }
    let mut seen = BTreeSet::new();
    for reference in refs {
        if !seen.insert(reference.as_str()) {
            return Err(HarnessStateError::InvalidBounds);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::{CompactionId, ExecutionProfileId};

    fn input() -> ContextProjectionInput {
        ContextProjectionInput {
            projection_ref: "projection:session-1:turn-2".into(),
            session_id: SessionId(1),
            execution_profile_id: ExecutionProfileId::from_u128(2),
            source_event_refs: vec!["event:10".into(), "event:11".into()],
            source_artifact_refs: vec!["artifact:7".into()],
            goal_refs: vec!["goal:3:version:2".into()],
            compaction_refs: vec![CompactionId::from_u128(4)],
            taint_refs: vec!["taint:model-generated".into()],
            max_tokens: 4096,
            render_policy_digest: [5; 32],
            rendered_digest: [6; 32],
            created_at_unix_ms: 7,
        }
    }

    #[test]
    fn builder_preserves_exact_canonical_reference_order() {
        let projection = build_context_projection(input()).unwrap();
        assert_eq!(projection.source_event_refs, ["event:10", "event:11"]);
        assert_eq!(projection.source_artifact_refs, ["artifact:7"]);
        assert_eq!(projection.goal_refs, ["goal:3:version:2"]);
        assert_eq!(projection.max_tokens, 4096);
    }

    #[test]
    fn builder_rejects_duplicate_source_references() {
        let mut value = input();
        value.source_event_refs.push("event:10".into());
        assert_eq!(
            build_context_projection(value),
            Err(HarnessStateError::InvalidBounds)
        );
    }

    #[test]
    fn builder_rejects_empty_goal_reference() {
        let mut value = input();
        value.goal_refs = vec![String::new()];
        assert_eq!(
            build_context_projection(value),
            Err(HarnessStateError::EmptyReference)
        );
    }

    #[test]
    fn builder_rejects_zero_token_budget() {
        let mut value = input();
        value.max_tokens = 0;
        assert_eq!(
            build_context_projection(value),
            Err(HarnessStateError::InvalidBounds)
        );
    }
}
