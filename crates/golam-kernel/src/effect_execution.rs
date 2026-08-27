#![forbid(unsafe_code)]

use golam_core::authority::AuthorityLayout;
use golam_core::{EffectAttemptId, EffectId};
use golam_ledger::dispatch::{
    EffectDispatchStore, EffectDispatchStoreError, PrepareEffectDispatch,
    PreparedEffectDispatchRecord,
};

/// Opaque proof that Golam durably recorded the attempt and moved the effect to
/// EXECUTING before any handler dispatch is allowed through the kernel path.
/// External callers can inspect identifiers but cannot construct this value.
///
/// ```compile_fail
/// use golam_kernel::PreparedEffectDispatch;
/// let _ = PreparedEffectDispatch {
///     effect_id: golam_core::EffectId(1),
///     attempt_id: golam_core::EffectAttemptId(2),
///     started_global_seq: 3,
///     executing_global_seq: 4,
/// };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedEffectDispatch {
    effect_id: EffectId,
    attempt_id: EffectAttemptId,
    started_global_seq: u64,
    executing_global_seq: u64,
}

impl PreparedEffectDispatch {
    pub const fn effect_id(self) -> EffectId {
        self.effect_id
    }

    pub const fn attempt_id(self) -> EffectAttemptId {
        self.attempt_id
    }

    pub const fn started_global_seq(self) -> u64 {
        self.started_global_seq
    }

    pub const fn executing_global_seq(self) -> u64 {
        self.executing_global_seq
    }
}

pub(crate) struct EffectExecutionAuthority {
    store: EffectDispatchStore,
}

impl EffectExecutionAuthority {
    pub(crate) fn open(layout: &AuthorityLayout) -> Result<Self, EffectDispatchStoreError> {
        Ok(Self {
            store: EffectDispatchStore::open(layout)?,
        })
    }

    pub(crate) fn prepare(
        &mut self,
        input: PrepareEffectDispatch<'_>,
    ) -> Result<PreparedEffectDispatch, EffectDispatchStoreError> {
        let record = self.store.prepare_dispatch(input)?;
        Ok(prepared_from_record(record))
    }
}

fn prepared_from_record(record: PreparedEffectDispatchRecord) -> PreparedEffectDispatch {
    PreparedEffectDispatch {
        effect_id: record.attempt.effect_id,
        attempt_id: record.attempt.attempt_id,
        started_global_seq: record.attempt.started_global_seq,
        executing_global_seq: record.transition.global_seq,
    }
}
