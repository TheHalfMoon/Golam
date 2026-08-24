#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectSemantics {
    ReadOnly,
    IdempotentAtLeastOnce,
    AtMostOnce,
    Compensatable,
    Irreversible,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectStatus {
    Proposed,
    Authorized,
    Executing,
    Succeeded,
    Failed,
    UnknownOutcome,
    Reconciling,
    ManualReview,
}

pub fn transition_allowed(from: EffectStatus, to: EffectStatus) -> bool {
    use EffectStatus::*;
    matches!(
        (from, to),
        (Proposed, Authorized)
            | (Authorized, Executing)
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
    fn dangerous_semantics_never_blind_retry() {
        assert!(!blind_retry_allowed(EffectSemantics::AtMostOnce));
        assert!(!blind_retry_allowed(EffectSemantics::Irreversible));
    }
}
