#![forbid(unsafe_code)]

use golam_effects::simulators::IdempotentWriteHandler;
use golam_effects::{
    EffectHandler, EffectSemantics, EffectStatus, HandlerAttemptOutcome, HandlerIntent,
    HandlerOutcome, PriorAttempt, transition_allowed,
};

fn statuses() -> [EffectStatus; 10] {
    [
        EffectStatus::Proposed,
        EffectStatus::Denied,
        EffectStatus::Authorized,
        EffectStatus::ApprovalRequired,
        EffectStatus::Executing,
        EffectStatus::Succeeded,
        EffectStatus::Failed,
        EffectStatus::UnknownOutcome,
        EffectStatus::Reconciling,
        EffectStatus::ManualReview,
    ]
}

fn expected_edge(from: EffectStatus, to: EffectStatus) -> bool {
    use EffectStatus::*;
    matches!(
        (from, to),
        (Proposed, Denied | Authorized)
            | (Authorized, ApprovalRequired | Executing)
            | (ApprovalRequired, Authorized | Denied)
            | (Executing, Succeeded | Failed | UnknownOutcome)
            | (UnknownOutcome, Reconciling | ManualReview)
            | (Reconciling, Succeeded | Failed | ManualReview)
    )
}

#[test]
fn effect_fsm_matches_declared_edge_set_exhaustively() {
    for from in statuses() {
        for to in statuses() {
            assert_eq!(
                transition_allowed(from, to),
                expected_edge(from, to),
                "unexpected FSM decision for {from:?} -> {to:?}"
            );
        }
    }
}

#[test]
fn idempotent_handler_is_stable_across_deterministic_corpus() {
    for seed in 0_u8..64 {
        let resource = format!("sim:item-{seed}");
        let idempotency_key = format!("stable-key-{seed}");
        let intent = HandlerIntent {
            action: "sim.write",
            resource: &resource,
            execution_semantics: EffectSemantics::IdempotentAtLeastOnce,
            idempotency_key: Some(&idempotency_key),
            payload_hash: [seed; 32],
        };
        let mut handler = IdempotentWriteHandler::default();

        assert_eq!(
            handler.derive_idempotency_key(&intent).as_deref(),
            Some(idempotency_key.as_str())
        );
        let first = handler.execute(&intent);
        let second = handler.execute(&intent);
        assert_eq!(first, second, "idempotent replay changed for seed {seed}");

        let receipt = match &first {
            HandlerOutcome::Succeeded { receipt } => receipt.as_slice(),
            other => panic!("idempotent simulator must succeed, got {other:?}"),
        };
        let prior = PriorAttempt {
            started_global_seq: u64::from(seed) + 1,
            handler_id: "sim-idempotent-write",
            handler_version: "1",
            dispatch_token: b"qualification-dispatch",
            outcome: HandlerAttemptOutcome::Unknown,
            receipt: Some(receipt),
        };
        assert_eq!(handler.reconcile(&intent, &prior), first);
    }
}
