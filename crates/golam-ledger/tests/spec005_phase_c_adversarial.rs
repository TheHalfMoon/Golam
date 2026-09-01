#![forbid(unsafe_code)]

use golam_core::EffectId;
use golam_core::harness::ToolCallCandidateId;
use golam_core::memory::{
    ExpectedMemoryVersion, MemoryCandidateId, MemoryItemId, MemoryMutationIntent, MemoryOperation,
    MemoryStoreId, MemoryVersionId,
};
use golam_core::skills_protocol::{
    CurrentMcpDispatchState, CurrentSkillDispatchState, DispatchValidationError,
    McpDispatchBinding, McpLifecycleState, McpVersionLock, SkillAdmissionState,
    SkillDispatchBinding, SkillDispatchKind, SkillVersion,
};
use golam_core::taint::{TaintLabel, TaintSet};
use golam_core::tool_descriptor::{ToolId, ToolVersion};
use golam_core::tool_request::{
    BindingDigest, PrincipalId, RequestedOperationId, RequestedTarget, ResourceClassId, ToolRequest,
    ToolRequestId,
};
use golam_ledger::memory_evidence::{MemoryEvidenceError, MemoryEvidenceStore};
use golam_ledger::tool_context_evidence::{ToolContextEvidenceError, ToolContextEvidenceStore};

fn digest(value: u8) -> BindingDigest {
    BindingDigest::new([value; 32])
}

fn tool_request() -> ToolRequest {
    ToolRequest {
        request_id: ToolRequestId::from_u128(7),
        initiating_principal: PrincipalId::new("principal.local").unwrap(),
        tool_id: ToolId::new("fs.read").unwrap(),
        tool_version: ToolVersion::new("1.0.0").unwrap(),
        candidate_ref: ToolCallCandidateId::from_u128(9),
        requested_operation: RequestedOperationId::new("read").unwrap(),
        requested_target: Some(RequestedTarget::new("src/lib.rs").unwrap()),
        authorized_resource_class: ResourceClassId::new("workspace.read").unwrap(),
        target_identity_ref: Some(digest(1)),
        target_resolution_plan_ref: None,
        capability_context_ref: digest(2),
        taint_set: TaintSet::from_labels([TaintLabel::LocalTrusted]),
        provenance_refs: vec![digest(3)],
        idempotency_material: digest(4),
        current_preconditions: vec![digest(5)],
        created_at_unix_ms: 10,
    }
}

fn memory_intent() -> MemoryMutationIntent {
    MemoryMutationIntent {
        operation: MemoryOperation::Update,
        item_ids: vec![MemoryItemId(digest(10))],
        expected_current_versions: vec![ExpectedMemoryVersion {
            item_id: MemoryItemId(digest(10)),
            expected_version: Some(MemoryVersionId(digest(11))),
        }],
        expected_markdown_target_identity_ref: digest(12),
        expected_markdown_content_digest: digest(13),
        expected_markdown_version: MemoryVersionId(digest(14)),
        memory_operational_store_ref: MemoryStoreId(digest(15)),
        candidate_ref: Some(MemoryCandidateId(digest(16))),
        kernel_authorization_ref: digest(17),
        promotion_authority_ref: digest(18),
        effect_id: EffectId(19),
        reason_ref: digest(20),
        initiating_principal: PrincipalId::new("principal.local").unwrap(),
        created_at_unix_ms: 21,
    }
}

fn assert_memory_rebinding_rejected(
    store: &mut MemoryEvidenceStore,
    changed: MemoryMutationIntent,
) {
    let changed = changed.prepare().unwrap();
    assert!(matches!(
        store.persist_prepared_intent(&changed),
        Err(MemoryEvidenceError::ImmutableEvidenceMismatch(
            "PREPARED memory intent"
        ))
    ));
}

#[test]
fn durable_tool_request_rejects_authority_rebinding_under_the_same_request_id() {
    let mut store = ToolContextEvidenceStore::open_in_memory().unwrap();
    let original = tool_request();
    store.persist_tool_request(&original).unwrap();

    let mut changed_target = original.clone();
    changed_target.target_identity_ref = Some(digest(90));
    assert!(matches!(
        store.persist_tool_request(&changed_target),
        Err(ToolContextEvidenceError::ImmutableEvidenceMismatch(
            "tool request"
        ))
    ));

    let mut changed_precondition = original.clone();
    changed_precondition.current_preconditions = vec![digest(91)];
    assert!(matches!(
        store.persist_tool_request(&changed_precondition),
        Err(ToolContextEvidenceError::ImmutableEvidenceMismatch(
            "tool request"
        ))
    ));

    let mut changed_authority = original;
    changed_authority.capability_context_ref = digest(92);
    assert!(matches!(
        store.persist_tool_request(&changed_authority),
        Err(ToolContextEvidenceError::ImmutableEvidenceMismatch(
            "tool request"
        ))
    ));
}

#[test]
fn durable_memory_effect_rejects_each_bound_markdown_and_store_rebinding() {
    let mut store = MemoryEvidenceStore::open_in_memory().unwrap();
    let original = memory_intent();
    store
        .persist_prepared_intent(&original.clone().prepare().unwrap())
        .unwrap();

    let mut changed_target = original.clone();
    changed_target.expected_markdown_target_identity_ref = digest(90);
    assert_memory_rebinding_rejected(&mut store, changed_target);

    let mut changed_digest = original.clone();
    changed_digest.expected_markdown_content_digest = digest(91);
    assert_memory_rebinding_rejected(&mut store, changed_digest);

    let mut changed_version = original.clone();
    changed_version.expected_markdown_version = MemoryVersionId(digest(92));
    assert_memory_rebinding_rejected(&mut store, changed_version);

    let mut changed_store = original;
    changed_store.memory_operational_store_ref = MemoryStoreId(digest(93));
    assert_memory_rebinding_rejected(&mut store, changed_store);
}

#[test]
fn stale_skill_and_mcp_dispatch_bindings_fail_closed() {
    let skill_binding = SkillDispatchBinding {
        skill_package_ref: digest(1),
        skill_version: SkillVersion::new("1.0.0").unwrap(),
        reviewed_content_digest: digest(2),
        reviewed_admission_state_ref: digest(3),
        reviewed_capability_mapping_ref: digest(4),
        queued_request_ref: digest(5),
        capability_decision_ref: digest(6),
        approval_decision_ref: digest(7),
        dispatch_kind: SkillDispatchKind::InstructionActivation,
    };
    let skill_state = CurrentSkillDispatchState {
        skill_package_ref: digest(1),
        skill_version: SkillVersion::new("1.0.0").unwrap(),
        content_digest: digest(2),
        admission_state: SkillAdmissionState::InstructionAdmitted,
        admission_state_ref: digest(3),
        capability_mapping_ref: digest(4),
    };
    assert_eq!(skill_binding.revalidate(&skill_state), Ok(()));

    let mut revoked_skill = skill_state.clone();
    revoked_skill.admission_state = SkillAdmissionState::Revoked;
    assert_eq!(
        skill_binding.revalidate(&revoked_skill),
        Err(DispatchValidationError::SkillLifecycleNotDispatchable)
    );

    let mut remapped_skill = skill_state;
    remapped_skill.capability_mapping_ref = digest(99);
    assert_eq!(
        skill_binding.revalidate(&remapped_skill),
        Err(DispatchValidationError::SkillCapabilityMappingMismatch)
    );

    let mcp_binding = McpDispatchBinding {
        binding_id: digest(30),
        binding_digest: digest(31),
        version_lock: McpVersionLock::new("2025-06-18").unwrap(),
        golam_local_mapping_ref: digest(32),
        golam_local_mapping_digest: digest(33),
        lifecycle_state_ref: digest(34),
        queued_request_ref: digest(35),
        capability_decision_ref: digest(36),
        approval_decision_ref: digest(37),
    };
    let mcp_state = CurrentMcpDispatchState {
        binding_id: digest(30),
        binding_digest: digest(31),
        version_lock: McpVersionLock::new("2025-06-18").unwrap(),
        golam_local_mapping_ref: digest(32),
        golam_local_mapping_digest: digest(33),
        lifecycle_state: McpLifecycleState::Active,
        lifecycle_state_ref: digest(34),
    };
    assert_eq!(mcp_binding.revalidate(&mcp_state), Ok(()));

    let mut replaced_mcp = mcp_state.clone();
    replaced_mcp.lifecycle_state = McpLifecycleState::Replaced;
    assert_eq!(
        mcp_binding.revalidate(&replaced_mcp),
        Err(DispatchValidationError::McpLifecycleNotDispatchable)
    );

    let mut remapped_mcp = mcp_state;
    remapped_mcp.golam_local_mapping_digest = digest(98);
    assert_eq!(
        mcp_binding.revalidate(&remapped_mcp),
        Err(DispatchValidationError::McpMappingMismatch)
    );
}
