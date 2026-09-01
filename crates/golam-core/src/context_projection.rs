use std::collections::BTreeSet;

use crate::SessionId;
use crate::harness::ExecutionProfileId;
use crate::harness_state::{
    CompactionArtifact, CompactionAttempt, CompactionState, ContextProjection, HarnessStateError,
};
use crate::taint::{TaintLabel, TaintSet};

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
    /// Monotonic aggregate provenance derived from the exact canonical source
    /// references above. The builder never clears ordinary provenance and
    /// rejects SECRET_DERIVED at the model-visible projection boundary.
    pub source_taint: TaintSet,
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
    validate_model_visible_taint(input.source_taint)?;

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

/// Builds a model-visible projection after a committed compaction while
/// reintroducing Goal/non-negotiable constraints from independently durable
/// canonical GoalVersion evidence instead of trusting summary text.
///
/// `canonical_goal_refs` must be read from canonical Goal state by the caller.
/// Both the terminal compaction attempt and artifact are required so an
/// incomplete or failed compaction can never become active merely because an
/// artifact-shaped value exists. The artifact remains evidence, not authority
/// for replacing or rewriting canonical Goal state.
pub fn build_post_compaction_projection(
    mut input: ContextProjectionInput,
    attempt: &CompactionAttempt,
    artifact: &CompactionArtifact,
    canonical_goal_refs: &[String],
) -> Result<ContextProjection, HarnessStateError> {
    attempt.validate()?;
    artifact.validate()?;
    reject_duplicate_refs(canonical_goal_refs)?;
    if attempt.state != CompactionState::Committed
        || attempt.terminal_at_unix_ms.is_none()
        || attempt.compaction_id != artifact.compaction_id
        || attempt.deterministic != artifact.deterministic
        || attempt.producing_request_attempt_id != artifact.producing_request_attempt_id
        || canonical_goal_refs.is_empty()
        || artifact.goal_refs != canonical_goal_refs
    {
        return Err(HarnessStateError::InvalidBounds);
    }

    input.goal_refs = canonical_goal_refs.to_vec();
    if !input.compaction_refs.contains(&artifact.compaction_id) {
        input.compaction_refs.push(artifact.compaction_id);
    }
    build_context_projection(input)
}

fn validate_model_visible_taint(taint: TaintSet) -> Result<(), HarnessStateError> {
    if taint.contains(TaintLabel::SecretDerived) {
        return Err(HarnessStateError::InvalidBounds);
    }
    Ok(())
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
            source_taint: TaintSet::from_labels([TaintLabel::ModelGenerated]),
            max_tokens: 4096,
            render_policy_digest: [5; 32],
            rendered_digest: [6; 32],
            created_at_unix_ms: 7,
        }
    }

    fn committed_compaction() -> CompactionAttempt {
        CompactionAttempt {
            compaction_id: CompactionId::from_u128(8),
            session_id: SessionId(1),
            source_projection_ref: "projection:session-1:turn-1".into(),
            state: CompactionState::Committed,
            deterministic: true,
            producing_request_attempt_id: None,
            started_at_unix_ms: 8,
            terminal_at_unix_ms: Some(9),
            failure_class: None,
        }
    }

    fn compaction_artifact() -> CompactionArtifact {
        CompactionArtifact {
            compaction_id: CompactionId::from_u128(8),
            source_projection_ref: "projection:session-1:turn-1".into(),
            source_event_refs: vec!["event:1".into(), "event:2".into()],
            goal_refs: vec!["goal:3:version:2".into()],
            deterministic: true,
            producing_request_attempt_id: None,
            accepted_output_ref: None,
            artifact_digest: [9; 32],
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

    #[test]
    fn builder_rejects_secret_derived_model_visibility() {
        let mut value = input();
        value.source_taint = TaintSet::from_labels([
            TaintLabel::UserTrusted,
            TaintLabel::SecretDerived,
            TaintLabel::ModelGenerated,
        ]);
        value.taint_refs = vec!["taint:secret-derived".into()];

        assert_eq!(
            build_context_projection(value),
            Err(HarnessStateError::InvalidBounds)
        );
    }

    #[test]
    fn builder_preserves_non_secret_untrusted_provenance_refs() {
        let mut value = input();
        value.source_taint = TaintSet::from_labels([
            TaintLabel::WebUntrusted,
            TaintLabel::McpUntrusted,
            TaintLabel::ModelGenerated,
        ]);
        value.taint_refs = vec![
            "taint:web-untrusted".into(),
            "taint:mcp-untrusted".into(),
            "taint:model-generated".into(),
        ];

        let projection = build_context_projection(value).unwrap();
        assert_eq!(
            projection.taint_refs,
            [
                "taint:web-untrusted",
                "taint:mcp-untrusted",
                "taint:model-generated",
            ]
        );
    }

    #[test]
    fn post_compaction_projection_reinjects_canonical_goal_state_independently() {
        let mut value = input();
        value.goal_refs = vec!["summary:must-not-own-goal".into()];
        value.compaction_refs.clear();
        let attempt = committed_compaction();
        let artifact = compaction_artifact();
        let canonical_goal_refs = vec!["goal:3:version:2".into()];

        let projection = build_post_compaction_projection(
            value,
            &attempt,
            &artifact,
            &canonical_goal_refs,
        )
        .unwrap();

        assert_eq!(projection.goal_refs, canonical_goal_refs);
        assert_eq!(projection.compaction_refs, [CompactionId::from_u128(8)]);
    }

    #[test]
    fn post_compaction_projection_rejects_stale_goal_binding() {
        let value = input();
        let attempt = committed_compaction();
        let artifact = compaction_artifact();
        let changed_goal_refs = vec!["goal:3:version:3".into()];

        assert_eq!(
            build_post_compaction_projection(value, &attempt, &artifact, &changed_goal_refs),
            Err(HarnessStateError::InvalidBounds)
        );
    }

    #[test]
    fn failed_or_incomplete_compaction_never_activates_projection() {
        let artifact = compaction_artifact();
        let canonical_goal_refs = vec!["goal:3:version:2".into()];
        for state in [
            CompactionState::Started,
            CompactionState::Deriving,
            CompactionState::Validating,
            CompactionState::Cancelled,
            CompactionState::FailedChangedSource,
            CompactionState::FailedTransient,
            CompactionState::FailedDeterministic,
            CompactionState::FailedPersistence,
        ] {
            let mut attempt = committed_compaction();
            attempt.state = state;
            attempt.terminal_at_unix_ms = if state.is_terminal() { Some(9) } else { None };
            attempt.failure_class = if matches!(
                state,
                CompactionState::FailedChangedSource
                    | CompactionState::FailedTransient
                    | CompactionState::FailedDeterministic
                    | CompactionState::FailedPersistence
            ) {
                Some("fixture_failure".into())
            } else {
                None
            };

            assert_eq!(
                build_post_compaction_projection(
                    input(),
                    &attempt,
                    &artifact,
                    &canonical_goal_refs,
                ),
                Err(HarnessStateError::InvalidBounds),
                "state {state:?} must not activate compaction"
            );
        }
    }

    #[test]
    fn compaction_activation_preserves_canonical_source_reference_inputs() {
        let original = input();
        let original_event_refs = original.source_event_refs.clone();
        let original_artifact_refs = original.source_artifact_refs.clone();
        let attempt = committed_compaction();
        let artifact = compaction_artifact();
        let canonical_goal_refs = vec!["goal:3:version:2".into()];

        for _ in 0..16 {
            let projection = build_post_compaction_projection(
                original.clone(),
                &attempt,
                &artifact,
                &canonical_goal_refs,
            )
            .unwrap();
            assert_eq!(original.source_event_refs, original_event_refs);
            assert_eq!(original.source_artifact_refs, original_artifact_refs);
            assert_eq!(projection.source_event_refs, original_event_refs);
            assert_eq!(projection.source_artifact_refs, original_artifact_refs);
        }
    }
}
