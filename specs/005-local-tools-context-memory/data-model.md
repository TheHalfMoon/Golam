# Data Model — Spec 005

All identifiers are bounded, canonical and serializable. Durable identities use stable content/version semantics where required; user/model strings are never authority-bearing merely because they populate a field.

## Tool model

### `ToolId`

Stable Golam-owned identifier for one logical tool family.

### `ToolVersion`

Immutable implementation/contract version. Material execution or validation changes require a new version.

### `ToolIoBounds`

Finite operation-appropriate I/O limits:

```text
max_bytes
max_items
max_nesting_depth
max_field_bytes
```

Every applicable dimension is explicit and finite. A dimension that is structurally not applicable is encoded as `NOT_APPLICABLE`; omission or a sentinel meaning “unbounded” is invalid.

### `ToolDurationBounds`

```text
max_total_duration_ms
max_idle_duration_ms
```

Both values are finite when the corresponding execution mode can block or stream. No tool obtains an implicit unlimited duration.

### `ToolReconciliationPolicy`

```text
reconcile_on[]
unknown_outcome_behavior
dependent_effect_behavior
observation_requirements
terminal_evidence_requirements
```

`UNKNOWN_OUTCOME` never means success. When the policy requires reconciliation, dependent consequential work remains blocked until attributable evidence resolves the outcome.

### `ToolVerificationPolicy`

```text
success_evidence_requirements
independent_readback_required
readback_source_class
failure_evidence_requirements
```

Verification requirements are immutable descriptor semantics, not tool-result claims.

### `ToolDescriptor`

```text
tool_id
version
operation_class
input_bounds: ToolIoBounds
output_bounds: ToolIoBounds
duration_bounds: ToolDurationBounds
required_capability_class
effect_semantics
network_posture
sandbox_requirement
target_identity_kind
target_identity_rules
reconciliation_policy: ToolReconciliationPolicy
verification_policy: ToolVerificationPolicy
```

Descriptor availability is mechanism evidence only.

### `ToolRequestId`

Unique request identity.

### `ToolRequest`

```text
request_id
initiating_principal
tool_id
tool_version
candidate_ref
requested_operation
requested_target
authorized_resource_class
target_identity_ref
target_resolution_plan_ref
capability_context_ref
taint_set
provenance_refs
idempotency_material
current_preconditions
created_at
```

`target_identity_ref` and `target_resolution_plan_ref` are operation-specific bindings: protected execution requires the exact resolved identity when already known, or an explicit bounded resolution plan whose result is re-authorized before action. `current_preconditions` binds stale-state guards such as file mutation expectations or Git HEAD/index/worktree expectations where required.

Once the protected request is durably prepared, all authority-relevant fields above are immutable. Retry, target/operation/precondition changes, authority-context changes, or a materially different resolution result require a new request/effect identity rather than rewriting the prepared request.

### `ToolResult`

```text
request_id
status
observed_target_identity
output_artifact_refs
stdout_or_text_ref
stderr_or_error_ref
external_effect_refs
verification_refs
taint_set
started_at
terminal_at
```

A result is evidence, not authorization.

## Filesystem model

### `AuthorizedRoot`

Binds a policy resource identity to a platform-resolved root identity and allowed operations.

### `ResolvedTargetIdentity`

```text
platform
requested_path
normalized_path
resolved_parent_identity
resolved_target_identity
file_kind
symlink_or_reparse_chain
observed_metadata_digest
observed_at
```

A lexical path is not sufficient for protected mutation.

### `FileMutationExpectation`

Optional optimistic precondition for writes/deletes/renames:

```text
expected_exists
expected_kind
expected_identity
expected_content_digest
expected_size
expected_parent_identity
```

Mismatch fails closed as stale evidence.

## Process model

### `ExecutionContainmentProfileId`

Exact admitted production containment profile. `native:unqualified` is a denial state, not a runnable profile.

### `ProcessLaunchPlan`

```text
profile_id
executable_identity
argv
cwd_identity
ambient_environment_policy
explicit_env
secret_handle_bindings
filesystem_rights
network_rights
device_rights
resource_limits
inherited_handle_rules
timeout
cancellation_policy
descendant_supervision_policy
process_tree_reconciliation_policy
```

`ambient_environment_policy` is fail-closed and requires a cleared ambient environment; only explicitly bound environment values and secret-handle bindings may be introduced under the launch contract.

`descendant_supervision_policy` binds process-tree ownership/discovery, inherited-handle rules, termination responsibility and descendant observation. `process_tree_reconciliation_policy` binds terminal evidence requirements for the root and descendants, unresolved-descendant behavior and restart reconciliation. Cancellation alone is not process-tree supervision or proof of terminal containment.

A plan requires Kernel authorization and an admitted executor before launch.

## Context model

### `EvidenceSourceId`

Stable source identity: file/repository/Git object/canonical ledger/memory item/protocol resource/etc.

### `ContextEvidence`

```text
evidence_id
source_id
source_kind
source_version_or_observation
content_ref
content_digest
authority_class
taint_set
permission_scope
freshness_policy
observed_at
supersedes_or_conflicts_with[]
```

### `EvidenceRequirement`

Describes what evidence is needed to satisfy an intent/criterion without prescribing a specific backend.

### `ContextCapsuleId`

Content/provenance-bound identity for one model-visible context projection.

### `ContextCapsule`

```text
capsule_id
intent_ref
requirement_refs
evidence_refs
memory_refs
ranking_evidence
sufficiency_state
missing_requirements
projection_policy_ref
created_at
```

`ContextCapsule` is a projection; canonical sources remain authoritative.

## Memory model

### `MemoryScope`

`USER | PROJECT`

Additional scopes require explicit later governance.

### `MemoryCandidateId`

Immutable candidate identity.

### `MemoryCandidate`

```text
candidate_id
scope
proposed_content_ref
provenance_refs
taint_set
authority_class
created_by_principal
created_at
promotion_requirement
```

### `MemoryItemId`

Stable logical identity for managed canonical knowledge.

### `MemoryVersionId`

Immutable content/provenance version identity.

### `MemoryOperation`

`ADD | UPDATE | SUPERSEDE | CONTRADICT | MERGE | EXPIRE | FORGET | REDACT`

### `MemoryMutationIntent`

```text
operation
item_ids
expected_current_versions
expected_markdown_target_identity_ref
expected_markdown_content_digest
expected_markdown_version
memory_operational_store_ref
candidate_ref
kernel_authorization_ref
promotion_authority_ref
effect_id
reason
initiating_principal
created_at
```

The immutable intent is bound into a durable Effect Gate PREPARED record before the first canonical Markdown or memory-operational-SQLite mutation. `kernel_authorization_ref` and the applicable approval/pre-registered-verifier evidence must be current for the requested scope/operation; free-form content is not approval.

`expected_markdown_target_identity_ref`, `expected_markdown_content_digest`, and `expected_markdown_version` bind the exact observed Markdown state that may be replaced. `memory_operational_store_ref` binds the exact dedicated memory operational SQLite store identity/schema frozen by T005-045 and MUST NOT alias the authority database. All of these fields participate in the mutation-intent digest and survive durable PREPARED state unchanged.

Commit-time Markdown replacement revalidates the expected target identity, digest, and version immediately before replacement and uses an identity-preserving conditional compare-and-replace primitive. A changed or unpreservable identity/content precondition fails closed as `USER_EDIT_DETECTED`/`CONFLICT` and MUST NOT silently overwrite the user-edited target. Markdown/front matter remains content only; reserved authority-bearing fields are quarantined for reconciliation rather than imported as authority.

### `MemoryMutationOutcome`

```text
effect_id
mutation_intent_digest
status: COMMITTED | REJECTED | FAILED | UNKNOWN_OUTCOME
canonical_version_refs
authority_journal_readback_ref
markdown_readback_ref
memory_sqlite_readback_ref
reconciliation_ref
verification_refs
integrity_evidence_refs
terminal_at
```

`COMMITTED` requires read-back/reconciliation evidence across the authority journal, canonical Markdown, and the exact `memory_operational_store_ref`. Cross-store disagreement, unreadable state, or an unprovable commit boundary cannot produce success. `UNKNOWN_OUTCOME` blocks dependent managed-memory mutations until restart/reconciliation resolves the exact canonical state. Outcome and verification evidence are integrity-chained according to the existing Effect Gate/ledger boundary.

### `MemoryVersion`

```text
item_id
version_id
scope
canonical_markdown_ref
content_digest
provenance_refs
taint_set
status
predecessor_versions
conflict_refs
promotion_evidence_ref
created_by_principal
committed_by_writer_identity
mutation_effect_ref
created_at
```

`created_by_principal` preserves the attributable initiating/creating principal. `committed_by_writer_identity` identifies the admitted governed writer implementation/instance responsible for committing the version, and `mutation_effect_ref` binds the resulting version to the exact protected Effect Gate lifecycle. Restart/reconciliation MUST preserve all three identities; recovering from SQLite/Markdown state may not collapse them into a generic system creator.

### `MemoryReconciliationState`

`IN_SYNC | USER_EDIT_DETECTED | CONFLICT | RECONCILED | BLOCKED`

Binds the last known managed version/content digest, exact Markdown target identity, exact memory-operational-store binding, and current observed Markdown identity/digest to the governing effect. Restart reconciliation compares all three canonical evidence surfaces: authority journal, Markdown, and memory operational SQLite.

### `DerivativeIndexGeneration`

```text
index_kind
generation_id
canonical_cut_digest
implementation_identity
status
built_at
```

A derivative generation is discardable/rebuildable and not authority.

## Skills/protocol model

### `SkillPackageId` / `SkillVersionId`

Bind exact package provenance and content digest.

### `SkillDescriptor`

```text
name
description
package_ref
version
content_digest
instruction_ref
script_refs
requested_capability_classes
network_posture
provenance
admission_state
```

`script_refs` do not imply execution permission. `admission_state` records the bounded instruction/executable lifecycle state and cannot itself grant execution authority.

### `SkillDispatchBinding`

```text
skill_package_ref
skill_version
reviewed_content_digest
reviewed_admission_state_ref
reviewed_capability_mapping_ref
queued_request_ref
capability_decision_ref
approval_decision_ref
```

Queued, prepared-but-not-dispatched, cached capability, cached approval, and dispatch-decision state is valid only for the exact reviewed package/version/content digest and reviewed capability mapping. Immediately before instruction activation or executable dispatch, Golam MUST re-read the current lifecycle state and revalidate this exact binding. `DEPRECATED`, `REVOKED`, replaced, unknown, version-mismatched, digest-mismatched, or mapping-mismatched state invalidates the queued/cached decision and requires fresh review/authority evaluation.

### `ProtocolAdapterId`

Identifies MCP/ACP adapter implementation and version.

### `McpServerBinding`

```text
binding_id
binding_digest
server_identity
transport
process_profile_ref_or_remote_endpoint
allowed_protocol_features
golam_local_mapping_ref
golam_local_mapping_digest
network_policy_ref
secret_policy_ref
taint_class
version_lock
lifecycle_state
```

Server-advertised tools/resources are descriptors, not Golam capabilities. `golam_local_mapping_ref` plus `golam_local_mapping_digest` binds the locally configured maximum mapping rather than accepting server-advertised authority. `lifecycle_state` must fail closed for unreviewed, replaced, deprecated or revoked bindings; version replacement requires a new reviewed binding identity/mapping rather than silently reusing prior authority.

### `McpDispatchBinding`

```text
binding_id
binding_digest
version_lock
golam_local_mapping_ref
golam_local_mapping_digest
lifecycle_state_ref
queued_request_ref
capability_decision_ref
approval_decision_ref
```

Every local or remote MCP dispatch revalidates the exact active reviewed `McpServerBinding`, version lock, binding digest, and Golam-local mapping identity/digest immediately before dispatch. `DEPRECATED`, `REVOKED`, replaced, unknown, version-mismatched, digest-mismatched, or mapping-mismatched state invalidates queued calls, prepared-but-not-dispatched calls, cached mapped descriptors, cached capabilities, cached approvals, and cached dispatch decisions. A superseded binding cannot donate authority to a replacement.

### `ExternalToolDescriptor`

Normalized MCP/provider descriptor mapped into Golam's untrusted `ToolDescriptor`/candidate layer. It cannot request an authority class broader than locally configured mapping.

## Key invariants

```text
TOOL_DESCRIPTOR != CAPABILITY
TOOL_CALL_CANDIDATE != EFFECT_AUTHORIZATION
PATH_STRING != TARGET_IDENTITY
SANDBOX_PROFILE_ADVERTISEMENT != RUN_PERMISSION
PROCESS_CANCEL_REQUEST != PROCESS_TREE_TERMINAL_PROOF
CONTEXT_RANK != AUTHORITY
CONTEXT_CAPSULE != CANONICAL_SOURCE
MEMORY_CANDIDATE != DURABLE_MEMORY
MEMORY_INDEX != CANONICAL_MEMORY
MODEL_VERIFICATION != PROMOTION_AUTHORITY
SECRET_DERIVED != CANONICAL_LONG_TERM_MEMORY
SKILL != AUTHORITY
STALE_SKILL_DISPATCH_BINDING != ACTIVE_AUTHORITY
MCP_CAPABILITY_ADVERTISEMENT != GOLAM_CAPABILITY
STALE_MCP_DISPATCH_BINDING != ACTIVE_AUTHORITY
ACP_CONNECTION != AUTHENTICATED_AUTHORITY
```
