#![forbid(unsafe_code)]

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

    pub const fn from_str(value: &str) -> Option<Self> {
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
            assert_eq!(EffectStatus::from_str(status.as_str()), Some(status));
        }
        assert_eq!(EffectStatus::from_str("bogus"), None);
    }

    #[test]
    fn dangerous_semantics_never_blind_retry() {
        assert!(!blind_retry_allowed(EffectSemantics::AtMostOnce));
        assert!(!blind_retry_allowed(EffectSemantics::Irreversible));
    }
}
