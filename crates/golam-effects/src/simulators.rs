#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use crate::{
    EffectHandler, EffectSemantics, HandlerIntent, HandlerMetadata, HandlerOutcome, PriorAttempt,
    ReconciliationClass,
};

const READ_ACTIONS: &[&str] = &["sim.read"];
const WRITE_ACTIONS: &[&str] = &["sim.write"];
const SIM_RESOURCE_TYPES: &[&str] = &["sim"];

#[derive(Default)]
pub struct PureReadHandler;

impl EffectHandler for PureReadHandler {
    fn metadata(&self) -> HandlerMetadata {
        metadata(
            "sim-pure-read",
            EffectSemantics::ReadOnly,
            READ_ACTIONS,
            false,
            ReconciliationClass::ReadOnlyLookup,
            false,
        )
    }

    fn derive_idempotency_key(&self, _intent: &HandlerIntent<'_>) -> Option<String> {
        None
    }

    fn execute(&mut self, intent: &HandlerIntent<'_>) -> HandlerOutcome {
        HandlerOutcome::Succeeded {
            receipt: receipt("read", &operation_key(intent)),
        }
    }

    fn reconcile(
        &self,
        intent: &HandlerIntent<'_>,
        _prior_attempt: &PriorAttempt<'_>,
    ) -> HandlerOutcome {
        HandlerOutcome::Succeeded {
            receipt: receipt("read", &operation_key(intent)),
        }
    }
}

#[derive(Default)]
pub struct IdempotentWriteHandler {
    receipts: BTreeMap<String, Vec<u8>>,
}

impl EffectHandler for IdempotentWriteHandler {
    fn metadata(&self) -> HandlerMetadata {
        metadata(
            "sim-idempotent-write",
            EffectSemantics::IdempotentAtLeastOnce,
            WRITE_ACTIONS,
            true,
            ReconciliationClass::ReadOnlyLookup,
            false,
        )
    }

    fn derive_idempotency_key(&self, intent: &HandlerIntent<'_>) -> Option<String> {
        Some(
            intent
                .idempotency_key
                .map(str::to_owned)
                .unwrap_or_else(|| operation_key(intent)),
        )
    }

    fn execute(&mut self, intent: &HandlerIntent<'_>) -> HandlerOutcome {
        let key = self
            .derive_idempotency_key(intent)
            .expect("idempotent simulator always derives a stable key");
        let stored = self
            .receipts
            .entry(key.clone())
            .or_insert_with(|| receipt("idempotent-write", &key));
        HandlerOutcome::Succeeded {
            receipt: stored.clone(),
        }
    }

    fn reconcile(
        &self,
        intent: &HandlerIntent<'_>,
        _prior_attempt: &PriorAttempt<'_>,
    ) -> HandlerOutcome {
        let key = self
            .derive_idempotency_key(intent)
            .expect("idempotent simulator always derives a stable key");
        match self.receipts.get(&key) {
            Some(stored) => HandlerOutcome::Succeeded {
                receipt: stored.clone(),
            },
            None => HandlerOutcome::Unknown {
                evidence: Some(receipt("idempotency-key-not-found", &key)),
            },
        }
    }
}

#[derive(Default)]
pub struct AtMostOnceWriteHandler {
    accepted: BTreeMap<String, Vec<u8>>,
}

impl EffectHandler for AtMostOnceWriteHandler {
    fn metadata(&self) -> HandlerMetadata {
        metadata(
            "sim-at-most-once-write",
            EffectSemantics::AtMostOnce,
            WRITE_ACTIONS,
            false,
            ReconciliationClass::QueryableStatus,
            true,
        )
    }

    fn derive_idempotency_key(&self, _intent: &HandlerIntent<'_>) -> Option<String> {
        None
    }

    fn execute(&mut self, intent: &HandlerIntent<'_>) -> HandlerOutcome {
        let key = operation_key(intent);
        if self.accepted.contains_key(&key) {
            return HandlerOutcome::Failed {
                reason_code: "at_most_once_redispatch_blocked".to_owned(),
                receipt: self.accepted.get(&key).cloned(),
            };
        }
        let stored = receipt("at-most-once-write", &key);
        self.accepted.insert(key, stored.clone());
        HandlerOutcome::Succeeded { receipt: stored }
    }

    fn reconcile(
        &self,
        intent: &HandlerIntent<'_>,
        _prior_attempt: &PriorAttempt<'_>,
    ) -> HandlerOutcome {
        let key = operation_key(intent);
        match self.accepted.get(&key) {
            Some(stored) => HandlerOutcome::Succeeded {
                receipt: stored.clone(),
            },
            None => HandlerOutcome::Unknown {
                evidence: Some(receipt("at-most-once-status-missing", &key)),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CompensationState {
    Applied(Vec<u8>),
    Compensated(Vec<u8>),
}

#[derive(Default)]
pub struct CompensatableWriteHandler {
    records: BTreeMap<String, CompensationState>,
}

impl CompensatableWriteHandler {
    pub fn compensate(&mut self, intent: &HandlerIntent<'_>) -> HandlerOutcome {
        let key = operation_key(intent);
        match self.records.get(&key).cloned() {
            Some(CompensationState::Applied(_)) => {
                let compensation = receipt("compensated", &key);
                self.records
                    .insert(key, CompensationState::Compensated(compensation.clone()));
                HandlerOutcome::Succeeded {
                    receipt: compensation,
                }
            }
            Some(CompensationState::Compensated(existing)) => {
                HandlerOutcome::Succeeded { receipt: existing }
            }
            None => HandlerOutcome::Failed {
                reason_code: "nothing_to_compensate".to_owned(),
                receipt: None,
            },
        }
    }
}

impl EffectHandler for CompensatableWriteHandler {
    fn metadata(&self) -> HandlerMetadata {
        metadata(
            "sim-compensatable-write",
            EffectSemantics::Compensatable,
            WRITE_ACTIONS,
            false,
            ReconciliationClass::CompensationRecord,
            true,
        )
    }

    fn derive_idempotency_key(&self, _intent: &HandlerIntent<'_>) -> Option<String> {
        None
    }

    fn execute(&mut self, intent: &HandlerIntent<'_>) -> HandlerOutcome {
        let key = operation_key(intent);
        if self.records.contains_key(&key) {
            return HandlerOutcome::Failed {
                reason_code: "compensatable_redispatch_blocked".to_owned(),
                receipt: None,
            };
        }
        let stored = receipt("compensatable-write", &key);
        self.records
            .insert(key, CompensationState::Applied(stored.clone()));
        HandlerOutcome::Succeeded { receipt: stored }
    }

    fn reconcile(
        &self,
        intent: &HandlerIntent<'_>,
        _prior_attempt: &PriorAttempt<'_>,
    ) -> HandlerOutcome {
        let key = operation_key(intent);
        match self.records.get(&key) {
            Some(CompensationState::Applied(stored)) => HandlerOutcome::Succeeded {
                receipt: stored.clone(),
            },
            Some(CompensationState::Compensated(stored)) => HandlerOutcome::Failed {
                reason_code: "effect_compensated".to_owned(),
                receipt: Some(stored.clone()),
            },
            None => HandlerOutcome::Unknown {
                evidence: Some(receipt("compensation-record-missing", &key)),
            },
        }
    }
}

#[derive(Default)]
pub struct IrreversibleWriteHandler {
    accepted: BTreeSet<String>,
}

impl EffectHandler for IrreversibleWriteHandler {
    fn metadata(&self) -> HandlerMetadata {
        metadata(
            "sim-irreversible-write",
            EffectSemantics::Irreversible,
            WRITE_ACTIONS,
            false,
            ReconciliationClass::QueryableStatus,
            true,
        )
    }

    fn derive_idempotency_key(&self, _intent: &HandlerIntent<'_>) -> Option<String> {
        None
    }

    fn execute(&mut self, intent: &HandlerIntent<'_>) -> HandlerOutcome {
        let key = operation_key(intent);
        if !self.accepted.insert(key.clone()) {
            return HandlerOutcome::Failed {
                reason_code: "irreversible_redispatch_blocked".to_owned(),
                receipt: None,
            };
        }
        HandlerOutcome::Unknown {
            evidence: Some(receipt("accepted-without-ack", &key)),
        }
    }

    fn reconcile(
        &self,
        intent: &HandlerIntent<'_>,
        _prior_attempt: &PriorAttempt<'_>,
    ) -> HandlerOutcome {
        let key = operation_key(intent);
        if self.accepted.contains(&key) {
            HandlerOutcome::Succeeded {
                receipt: receipt("irreversible-accepted", &key),
            }
        } else {
            HandlerOutcome::Unknown {
                evidence: Some(receipt("irreversible-status-missing", &key)),
            }
        }
    }
}

fn metadata(
    handler_id: &'static str,
    execution_semantics: EffectSemantics,
    supported_actions: &'static [&'static str],
    idempotency_supported: bool,
    reconciliation_class: ReconciliationClass,
    manual_review_possible: bool,
) -> HandlerMetadata {
    HandlerMetadata {
        handler_id,
        handler_version: "1",
        supported_actions,
        supported_resource_types: SIM_RESOURCE_TYPES,
        execution_semantics,
        idempotency_supported,
        reconciliation_class,
        execution_timeout: Duration::from_secs(1),
        reconciliation_timeout: Duration::from_secs(1),
        manual_review_possible,
    }
}

fn operation_key(intent: &HandlerIntent<'_>) -> String {
    format!(
        "{}|{}|{}",
        intent.action,
        intent.resource,
        hex_hash(intent.payload_hash)
    )
}

fn receipt(prefix: &str, key: &str) -> Vec<u8> {
    format!("{prefix}:{key}").into_bytes()
}

fn hex_hash(hash: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(64);
    for byte in hash {
        value.push(char::from(HEX[(byte >> 4) as usize]));
        value.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HandlerAttemptOutcome;

    fn intent(semantics: EffectSemantics) -> HandlerIntent<'static> {
        HandlerIntent {
            action: if semantics == EffectSemantics::ReadOnly {
                "sim.read"
            } else {
                "sim.write"
            },
            resource: "sim:item-1",
            execution_semantics: semantics,
            idempotency_key: Some("stable-key-1"),
            payload_hash: [9; 32],
        }
    }

    fn prior() -> PriorAttempt<'static> {
        PriorAttempt {
            started_global_seq: 10,
            handler_id: "sim",
            handler_version: "1",
            dispatch_token: b"dispatch",
            outcome: HandlerAttemptOutcome::Unknown,
            receipt: None,
        }
    }

    #[test]
    fn simulator_metadata_covers_all_five_semantics() {
        let handlers = [
            PureReadHandler.metadata(),
            IdempotentWriteHandler::default().metadata(),
            AtMostOnceWriteHandler::default().metadata(),
            CompensatableWriteHandler::default().metadata(),
            IrreversibleWriteHandler::default().metadata(),
        ];
        assert_eq!(
            handlers.map(|handler| handler.execution_semantics),
            [
                EffectSemantics::ReadOnly,
                EffectSemantics::IdempotentAtLeastOnce,
                EffectSemantics::AtMostOnce,
                EffectSemantics::Compensatable,
                EffectSemantics::Irreversible,
            ]
        );
    }

    #[test]
    fn pure_read_is_deterministic_and_reconcilable() {
        let intent = intent(EffectSemantics::ReadOnly);
        let mut handler = PureReadHandler;
        let first = handler.execute(&intent);
        let second = handler.execute(&intent);
        assert_eq!(first, second);
        assert_eq!(handler.reconcile(&intent, &prior()), first);
    }

    #[test]
    fn idempotent_write_reuses_receipt_and_key_lookup_reconciles() {
        let intent = intent(EffectSemantics::IdempotentAtLeastOnce);
        let mut handler = IdempotentWriteHandler::default();
        let first = handler.execute(&intent);
        let second = handler.execute(&intent);
        assert_eq!(first, second);
        assert_eq!(handler.reconcile(&intent, &prior()), first);
        assert_eq!(
            handler.derive_idempotency_key(&intent).as_deref(),
            Some("stable-key-1")
        );
    }

    #[test]
    fn at_most_once_write_rejects_redispatch_and_queries_status() {
        let intent = intent(EffectSemantics::AtMostOnce);
        let mut handler = AtMostOnceWriteHandler::default();
        let first = handler.execute(&intent);
        assert!(matches!(first, HandlerOutcome::Succeeded { .. }));
        assert!(matches!(
            handler.execute(&intent),
            HandlerOutcome::Failed { ref reason_code, .. }
                if reason_code == "at_most_once_redispatch_blocked"
        ));
        assert_eq!(handler.reconcile(&intent, &prior()), first);
    }

    #[test]
    fn compensatable_write_records_compensation() {
        let intent = intent(EffectSemantics::Compensatable);
        let mut handler = CompensatableWriteHandler::default();
        assert!(matches!(
            handler.execute(&intent),
            HandlerOutcome::Succeeded { .. }
        ));
        let compensation = handler.compensate(&intent);
        assert!(matches!(compensation, HandlerOutcome::Succeeded { .. }));
        assert!(matches!(
            handler.reconcile(&intent, &prior()),
            HandlerOutcome::Failed { ref reason_code, .. } if reason_code == "effect_compensated"
        ));
        assert_eq!(handler.compensate(&intent), compensation);
    }

    #[test]
    fn irreversible_write_has_ambiguous_ack_and_never_accepts_twice() {
        let intent = intent(EffectSemantics::Irreversible);
        let mut handler = IrreversibleWriteHandler::default();
        assert!(matches!(
            handler.execute(&intent),
            HandlerOutcome::Unknown { .. }
        ));
        assert!(matches!(
            handler.execute(&intent),
            HandlerOutcome::Failed { ref reason_code, .. }
                if reason_code == "irreversible_redispatch_blocked"
        ));
        assert!(matches!(
            handler.reconcile(&intent, &prior()),
            HandlerOutcome::Succeeded { .. }
        ));
    }
}
