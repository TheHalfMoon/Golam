#![forbid(unsafe_code)]

use core::fmt;

use crate::memory::{
    MemoryItemId, MemoryOperation, MemoryValidationError, MemoryVersion, MemoryVersionId,
    MemoryVersionStatus, PreparedMemoryMutationIntent,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryTransitionError {
    Intent(MemoryValidationError),
    Version(MemoryValidationError),
    OutputItemNotBound,
    OperationItemArity,
    MissingExpectedVersion,
    UnexpectedExpectedVersion,
    StatusMismatch,
    LineageMismatch,
    ConflictMismatch,
    SelfPredecessor,
}

impl fmt::Display for MemoryTransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Intent(error) => write!(f, "memory transition intent is invalid: {error}"),
            Self::Version(error) => write!(f, "memory transition version is invalid: {error}"),
            Self::OutputItemNotBound => {
                f.write_str("memory transition output item is not bound by the prepared intent")
            }
            Self::OperationItemArity => {
                f.write_str("memory operation has an invalid affected-item cardinality")
            }
            Self::MissingExpectedVersion => {
                f.write_str("memory operation requires an existing expected version")
            }
            Self::UnexpectedExpectedVersion => {
                f.write_str("ADD requires an absent expected current version")
            }
            Self::StatusMismatch => {
                f.write_str("memory version status does not match the prepared operation")
            }
            Self::LineageMismatch => {
                f.write_str("memory version lineage does not match the prepared operation")
            }
            Self::ConflictMismatch => {
                f.write_str("memory contradiction does not surface the expected conflict")
            }
            Self::SelfPredecessor => {
                f.write_str("memory version cannot name itself as a predecessor")
            }
        }
    }
}

impl std::error::Error for MemoryTransitionError {}

/// Validates the immutable version shape for one exact PREPARED memory operation.
///
/// This is deliberately stricter than `MemoryVersion::validate`: the version-only
/// validator protects canonical memory admission and reference shape, while this
/// boundary binds operation semantics to the immutable mutation intent.
pub fn validate_memory_transition(
    prepared: &PreparedMemoryMutationIntent,
    version: &MemoryVersion,
) -> Result<(), MemoryTransitionError> {
    prepared
        .intent()
        .validate()
        .map_err(MemoryTransitionError::Intent)?;
    version.validate().map_err(MemoryTransitionError::Version)?;

    let intent = prepared.intent();
    if !intent.item_ids.contains(&version.item_id) {
        return Err(MemoryTransitionError::OutputItemNotBound);
    }
    if version.predecessor_versions.contains(&version.version_id) {
        return Err(MemoryTransitionError::SelfPredecessor);
    }

    let expected_for_output = expected_version(
        intent.item_ids.as_slice(),
        &intent.expected_current_versions,
        version.item_id,
    );
    match intent.operation {
        MemoryOperation::Add => {
            require_single_item(intent.item_ids.len())?;
            if expected_for_output.flatten().is_some() {
                return Err(MemoryTransitionError::UnexpectedExpectedVersion);
            }
            require_status(version, MemoryVersionStatus::Active)?;
            require_lineage(version, &[])?;
            require_no_conflicts(version)?;
        }
        MemoryOperation::Update | MemoryOperation::Supersede => {
            require_single_item(intent.item_ids.len())?;
            let expected = require_expected(expected_for_output)?;
            require_status(version, MemoryVersionStatus::Active)?;
            require_lineage(version, &[expected])?;
            require_no_conflicts(version)?;
        }
        MemoryOperation::Contradict => {
            require_single_item(intent.item_ids.len())?;
            let expected = require_expected(expected_for_output)?;
            require_status(version, MemoryVersionStatus::Contradicted)?;
            require_lineage(version, &[expected])?;
            if version.conflict_refs.binary_search(&expected.0).is_err() {
                return Err(MemoryTransitionError::ConflictMismatch);
            }
        }
        MemoryOperation::Merge => {
            if intent.item_ids.len() < 2 {
                return Err(MemoryTransitionError::OperationItemArity);
            }
            let mut expected = intent
                .expected_current_versions
                .iter()
                .map(|binding| {
                    binding
                        .expected_version
                        .ok_or(MemoryTransitionError::MissingExpectedVersion)
                })
                .collect::<Result<Vec<_>, _>>()?;
            expected.sort_unstable();
            expected.dedup();
            if expected.len() != intent.item_ids.len() {
                return Err(MemoryTransitionError::LineageMismatch);
            }
            require_status(version, MemoryVersionStatus::Active)?;
            require_lineage(version, &expected)?;
        }
        MemoryOperation::Expire => {
            require_single_item(intent.item_ids.len())?;
            let expected = require_expected(expected_for_output)?;
            require_status(version, MemoryVersionStatus::Expired)?;
            require_lineage(version, &[expected])?;
            require_no_conflicts(version)?;
        }
        MemoryOperation::Forget => {
            require_single_item(intent.item_ids.len())?;
            let expected = require_expected(expected_for_output)?;
            require_status(version, MemoryVersionStatus::Forgotten)?;
            require_lineage(version, &[expected])?;
            require_no_conflicts(version)?;
        }
        MemoryOperation::Redact => {
            require_single_item(intent.item_ids.len())?;
            let expected = require_expected(expected_for_output)?;
            require_status(version, MemoryVersionStatus::Redacted)?;
            require_lineage(version, &[expected])?;
            require_no_conflicts(version)?;
        }
    }
    Ok(())
}

fn expected_version(
    item_ids: &[MemoryItemId],
    bindings: &[crate::memory::ExpectedMemoryVersion],
    item_id: MemoryItemId,
) -> Option<Option<MemoryVersionId>> {
    item_ids
        .iter()
        .position(|item| *item == item_id)
        .and_then(|index| bindings.get(index))
        .map(|binding| binding.expected_version)
}

fn require_single_item(count: usize) -> Result<(), MemoryTransitionError> {
    if count == 1 {
        Ok(())
    } else {
        Err(MemoryTransitionError::OperationItemArity)
    }
}

fn require_expected(
    expected: Option<Option<MemoryVersionId>>,
) -> Result<MemoryVersionId, MemoryTransitionError> {
    expected
        .flatten()
        .ok_or(MemoryTransitionError::MissingExpectedVersion)
}

fn require_status(
    version: &MemoryVersion,
    expected: MemoryVersionStatus,
) -> Result<(), MemoryTransitionError> {
    if version.status == expected {
        Ok(())
    } else {
        Err(MemoryTransitionError::StatusMismatch)
    }
}

fn require_lineage(
    version: &MemoryVersion,
    expected: &[MemoryVersionId],
) -> Result<(), MemoryTransitionError> {
    if version.predecessor_versions == expected {
        Ok(())
    } else {
        Err(MemoryTransitionError::LineageMismatch)
    }
}

fn require_no_conflicts(version: &MemoryVersion) -> Result<(), MemoryTransitionError> {
    if version.conflict_refs.is_empty() {
        Ok(())
    } else {
        Err(MemoryTransitionError::ConflictMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EffectId;
    use crate::memory::{
        ExpectedMemoryVersion, MemoryCandidateId, MemoryScope, MemoryStoreId, MemoryWriterId,
    };
    use crate::taint::{TaintLabel, TaintSet};
    use crate::tool_request::{BindingDigest, PrincipalId};

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    fn item(value: u8) -> MemoryItemId {
        MemoryItemId(digest(value))
    }

    fn version_id(value: u8) -> MemoryVersionId {
        MemoryVersionId(digest(value))
    }

    fn prepared(
        operation: MemoryOperation,
        items: Vec<MemoryItemId>,
        expected: Vec<Option<MemoryVersionId>>,
    ) -> PreparedMemoryMutationIntent {
        let candidate_ref = matches!(
            operation,
            MemoryOperation::Add
                | MemoryOperation::Update
                | MemoryOperation::Supersede
                | MemoryOperation::Contradict
                | MemoryOperation::Merge
        )
        .then_some(MemoryCandidateId(digest(90)));
        let expected_current_versions = items
            .iter()
            .copied()
            .zip(expected)
            .map(|(item_id, expected_version)| ExpectedMemoryVersion {
                item_id,
                expected_version,
            })
            .collect();
        crate::memory::MemoryMutationIntent {
            operation,
            item_ids: items,
            expected_current_versions,
            expected_markdown_target_identity_ref: digest(1),
            expected_markdown_content_digest: digest(2),
            expected_markdown_version: version_id(3),
            memory_operational_store_ref: MemoryStoreId(digest(4)),
            candidate_ref,
            kernel_authorization_ref: digest(5),
            promotion_authority_ref: digest(6),
            effect_id: EffectId(7),
            reason_ref: digest(8),
            initiating_principal: PrincipalId::new("principal.local").unwrap(),
            created_at_unix_ms: 9,
        }
        .prepare()
        .unwrap()
    }

    fn version(
        item_id: MemoryItemId,
        status: MemoryVersionStatus,
        predecessors: Vec<MemoryVersionId>,
        conflicts: Vec<BindingDigest>,
    ) -> MemoryVersion {
        MemoryVersion {
            item_id,
            version_id: version_id(40),
            scope: MemoryScope::Project,
            canonical_markdown_ref: digest(41),
            content_digest: digest(42),
            provenance_refs: vec![digest(43)],
            taint_set: TaintSet::from_labels([TaintLabel::UserTrusted]),
            status,
            predecessor_versions: predecessors,
            conflict_refs: conflicts,
            promotion_evidence_ref: digest(44),
            created_by_principal: PrincipalId::new("principal.creator").unwrap(),
            committed_by_writer_identity: MemoryWriterId(digest(45)),
            mutation_effect_ref: EffectId(7),
            created_at_unix_ms: 46,
        }
    }

    #[test]
    fn add_requires_absent_current_version_and_clean_active_lineage() {
        let prepared = prepared(MemoryOperation::Add, vec![item(1)], vec![None]);
        assert_eq!(
            validate_memory_transition(
                &prepared,
                &version(item(1), MemoryVersionStatus::Active, vec![], vec![])
            ),
            Ok(())
        );
    }

    #[test]
    fn update_and_supersede_require_exact_predecessor() {
        for operation in [MemoryOperation::Update, MemoryOperation::Supersede] {
            let previous = version_id(10);
            let prepared = prepared(operation, vec![item(1)], vec![Some(previous)]);
            assert_eq!(
                validate_memory_transition(
                    &prepared,
                    &version(item(1), MemoryVersionStatus::Active, vec![previous], vec![])
                ),
                Ok(())
            );
        }
    }

    #[test]
    fn contradiction_retains_lineage_and_surfaces_conflict() {
        let previous = version_id(10);
        let prepared = prepared(
            MemoryOperation::Contradict,
            vec![item(1)],
            vec![Some(previous)],
        );
        assert_eq!(
            validate_memory_transition(
                &prepared,
                &version(
                    item(1),
                    MemoryVersionStatus::Contradicted,
                    vec![previous],
                    vec![previous.0]
                )
            ),
            Ok(())
        );
    }

    #[test]
    fn merge_requires_every_source_version_as_immutable_lineage() {
        let first = version_id(10);
        let second = version_id(11);
        let prepared = prepared(
            MemoryOperation::Merge,
            vec![item(1), item(2)],
            vec![Some(first), Some(second)],
        );
        assert_eq!(
            validate_memory_transition(
                &prepared,
                &version(
                    item(1),
                    MemoryVersionStatus::Active,
                    vec![first, second],
                    vec![]
                )
            ),
            Ok(())
        );
    }

    #[test]
    fn expire_forget_and_redact_are_candidate_less_terminal_knowledge_states() {
        for (operation, status) in [
            (MemoryOperation::Expire, MemoryVersionStatus::Expired),
            (MemoryOperation::Forget, MemoryVersionStatus::Forgotten),
            (MemoryOperation::Redact, MemoryVersionStatus::Redacted),
        ] {
            let previous = version_id(10);
            let prepared = prepared(operation, vec![item(1)], vec![Some(previous)]);
            assert!(prepared.intent().candidate_ref.is_none());
            assert_eq!(
                validate_memory_transition(
                    &prepared,
                    &version(item(1), status, vec![previous], vec![])
                ),
                Ok(())
            );
        }
    }

    #[test]
    fn secret_derived_tombstone_cannot_smuggle_a_taint_downgrade() {
        let previous = version_id(10);
        let prepared = prepared(MemoryOperation::Redact, vec![item(1)], vec![Some(previous)]);
        let mut value = version(
            item(1),
            MemoryVersionStatus::Redacted,
            vec![previous],
            vec![],
        );
        value.taint_set = TaintSet::from_labels([TaintLabel::SecretDerived]);
        assert!(matches!(
            validate_memory_transition(&prepared, &value),
            Err(MemoryTransitionError::Version(
                MemoryValidationError::MemoryAdmission(_)
            ))
        ));
    }
}
