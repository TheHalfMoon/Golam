#![forbid(unsafe_code)]

use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectSemantics {
    ReadOnly,
    IdempotentAtLeastOnce,
    AtMostOnce,
    Compensatable,
    Irreversible,
}

impl EffectSemantics {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::IdempotentAtLeastOnce => "idempotent_at_least_once",
            Self::AtMostOnce => "at_most_once",
            Self::Compensatable => "compensatable",
            Self::Irreversible => "irreversible",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectStatus {
    Proposed,
    Denied,
    Authorized,
    ApprovalRequired,
    Executing,
    Succeeded,
    Failed,
    UnknownOutcome,
    Reconciling,
    ManualReview,
}

impl EffectStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Denied => "denied",
            Self::Authorized => "authorized",
            Self::ApprovalRequired => "approval_required",
            Self::Executing => "executing",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::UnknownOutcome => "unknown_outcome",
            Self::Reconciling => "reconciling",
            Self::ManualReview => "manual_review",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "proposed" => Some(Self::Proposed),
            "denied" => Some(Self::Denied),
            "authorized" => Some(Self::Authorized),
            "approval_required" => Some(Self::ApprovalRequired),
            "executing" => Some(Self::Executing),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            "unknown_outcome" => Some(Self::UnknownOutcome),
            "reconciling" => Some(Self::Reconciling),
            "manual_review" => Some(Self::ManualReview),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReconciliationClass {
    Unsupported,
    ReadOnlyLookup,
    QueryableStatus,
    CompensationRecord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HandlerMetadata {
    pub handler_id: &'static str,
    pub handler_version: &'static str,
    pub supported_actions: &'static [&'static str],
    pub supported_resource_types: &'static [&'static str],
    pub execution_semantics: EffectSemantics,
    pub idempotency_supported: bool,
    pub reconciliation_class: ReconciliationClass,
    pub execution_timeout: Duration,
    pub reconciliation_timeout: Duration,
    pub manual_review_possible: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HandlerIntent<'a> {
    pub action: &'a str,
    pub resource: &'a str,
    pub execution_semantics: EffectSemantics,
    pub idempotency_key: Option<&'a str>,
    pub payload_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandlerAttemptOutcome {
    Success,
    Failure,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PriorAttempt<'a> {
    pub started_global_seq: u64,
    pub handler_id: &'a str,
    pub handler_version: &'a str,
    pub dispatch_token: &'a [u8],
    pub outcome: HandlerAttemptOutcome,
    pub receipt: Option<&'a [u8]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HandlerOutcome {
    Succeeded {
        receipt: Vec<u8>,
    },
    Failed {
        reason_code: String,
        receipt: Option<Vec<u8>>,
    },
    Unknown {
        evidence: Option<Vec<u8>>,
    },
}

pub trait EffectHandler {
    fn metadata(&self) -> HandlerMetadata;

    fn derive_idempotency_key(&self, intent: &HandlerIntent<'_>) -> Option<String>;

    fn execute(&mut self, intent: &HandlerIntent<'_>) -> HandlerOutcome;

    fn reconcile(
        &self,
        intent: &HandlerIntent<'_>,
        prior_attempt: &PriorAttempt<'_>,
    ) -> HandlerOutcome;
}

pub fn transition_allowed(from: EffectStatus, to: EffectStatus) -> bool {
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

pub fn blind_retry_allowed(semantics: EffectSemantics) -> bool {
    matches!(
        semantics,
        EffectSemantics::ReadOnly | EffectSemantics::IdempotentAtLeastOnce
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ContractHandler {
        execute_count: usize,
    }

    impl EffectHandler for ContractHandler {
        fn metadata(&self) -> HandlerMetadata {
            HandlerMetadata {
                handler_id: "contract-handler",
                handler_version: "1",
                supported_actions: &["sim.write"],
                supported_resource_types: &["sim"],
                execution_semantics: EffectSemantics::IdempotentAtLeastOnce,
                idempotency_supported: true,
                reconciliation_class: ReconciliationClass::ReadOnlyLookup,
                execution_timeout: Duration::from_secs(2),
                reconciliation_timeout: Duration::from_secs(1),
                manual_review_possible: false,
            }
        }

        fn derive_idempotency_key(&self, intent: &HandlerIntent<'_>) -> Option<String> {
            Some(format!("{}:{}", intent.action, intent.resource))
        }

        fn execute(&mut self, intent: &HandlerIntent<'_>) -> HandlerOutcome {
            self.execute_count += 1;
            HandlerOutcome::Succeeded {
                receipt: intent.payload_hash.to_vec(),
            }
        }

        fn reconcile(
            &self,
            _intent: &HandlerIntent<'_>,
            prior_attempt: &PriorAttempt<'_>,
        ) -> HandlerOutcome {
            HandlerOutcome::Succeeded {
                receipt: prior_attempt.receipt.unwrap_or_default().to_vec(),
            }
        }
    }

    #[test]
    fn planned_effect_fsm_accepts_only_declared_edges() {
        assert!(transition_allowed(
            EffectStatus::Proposed,
            EffectStatus::Denied
        ));
        assert!(transition_allowed(
            EffectStatus::Proposed,
            EffectStatus::Authorized
        ));
        assert!(transition_allowed(
            EffectStatus::Authorized,
            EffectStatus::ApprovalRequired
        ));
        assert!(transition_allowed(
            EffectStatus::ApprovalRequired,
            EffectStatus::Authorized
        ));
        assert!(transition_allowed(
            EffectStatus::Executing,
            EffectStatus::UnknownOutcome
        ));
        assert!(transition_allowed(
            EffectStatus::UnknownOutcome,
            EffectStatus::Reconciling
        ));
        assert!(transition_allowed(
            EffectStatus::Reconciling,
            EffectStatus::ManualReview
        ));

        assert!(!transition_allowed(
            EffectStatus::Proposed,
            EffectStatus::Executing
        ));
        assert!(!transition_allowed(
            EffectStatus::UnknownOutcome,
            EffectStatus::Succeeded
        ));
        assert!(!transition_allowed(
            EffectStatus::Succeeded,
            EffectStatus::Executing
        ));
    }

    #[test]
    fn status_strings_round_trip() {
        for status in [
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
        ] {
            assert_eq!(EffectStatus::parse(status.as_str()), Some(status));
        }
        assert_eq!(EffectStatus::parse("bogus"), None);
    }

    #[test]
    fn handler_contract_exposes_metadata_execute_and_read_only_reconcile() {
        let intent = HandlerIntent {
            action: "sim.write",
            resource: "sim:item",
            execution_semantics: EffectSemantics::IdempotentAtLeastOnce,
            idempotency_key: None,
            payload_hash: [7; 32],
        };
        let mut handler = ContractHandler { execute_count: 0 };
        let metadata = handler.metadata();
        assert_eq!(metadata.handler_id, "contract-handler");
        assert!(metadata.idempotency_supported);
        assert_eq!(
            handler.derive_idempotency_key(&intent).as_deref(),
            Some("sim.write:sim:item")
        );
        assert!(matches!(
            handler.execute(&intent),
            HandlerOutcome::Succeeded { .. }
        ));
        assert_eq!(handler.execute_count, 1);

        let prior = PriorAttempt {
            started_global_seq: 4,
            handler_id: metadata.handler_id,
            handler_version: metadata.handler_version,
            dispatch_token: b"dispatch-1",
            outcome: HandlerAttemptOutcome::Unknown,
            receipt: Some(b"receipt"),
        };
        assert_eq!(
            handler.reconcile(&intent, &prior),
            HandlerOutcome::Succeeded {
                receipt: b"receipt".to_vec()
            }
        );
        assert_eq!(handler.execute_count, 1);
    }

    #[test]
    fn dangerous_semantics_never_blind_retry() {
        assert!(!blind_retry_allowed(EffectSemantics::AtMostOnce));
        assert!(!blind_retry_allowed(EffectSemantics::Irreversible));
    }
}
