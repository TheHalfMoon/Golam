# Data Model — Spec 005

All identifiers are bounded, canonical and serializable. Durable identities use stable content/version semantics where required; user/model strings are never authority-bearing merely because they populate a field.

## Tool model

### `ToolId`

Stable Golam-owned identifier for one logical tool family.

### `ToolVersion`

Immutable implementation/contract version. Material execution or validation changes require a new version.

### `ToolDescriptor`

```text
tool_id
version
operation_class
input_bounds
output_bounds
required_capability_class
effect_semantics
network_posture
sandbox_requirement
target_identity_kind
verification_policy
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
requested_target
target_identity_ref
capability_context_ref
taint_set
provenance_refs
idempotency_material
created_at
```

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
explicit_env
secret_handle_bindings
filesystem_rights
network_rights
device_rights
resource_limits
inherited_handle_rules
timeout
cancellation_policy
```

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
candidate_ref
promotion_authority_ref
reason
initiating_principal
created_at
```

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
created_at
```

### `MemoryReconciliationState`

`IN_SYNC | USER_EDIT_DETECTED | CONFLICT | RECONCILED | BLOCKED`

Binds last known managed version/content digest to current observed Markdown identity.

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
instruction_ref
script_refs
requested_capability_classes
network_posture
provenance
admission_state
```

`script_refs` do not imply execution permission.

### `ProtocolAdapterId`

Identifies MCP/ACP adapter implementation and version.

### `McpServerBinding`

```text
binding_id
server_identity
transport
process_profile_ref_or_remote_endpoint
allowed_protocol_features
network_policy_ref
secret_policy_ref
taint_class
version_lock
```

Server-advertised tools/resources are descriptors, not Golam capabilities.

### `ExternalToolDescriptor`

Normalized MCP/provider descriptor mapped into Golam's untrusted `ToolDescriptor`/candidate layer. It cannot request an authority class broader than locally configured mapping.

## Key invariants

```text
TOOL_DESCRIPTOR != CAPABILITY
TOOL_CALL_CANDIDATE != EFFECT_AUTHORIZATION
PATH_STRING != TARGET_IDENTITY
SANDBOX_PROFILE_ADVERTISEMENT != RUN_PERMISSION
CONTEXT_RANK != AUTHORITY
CONTEXT_CAPSULE != CANONICAL_SOURCE
MEMORY_CANDIDATE != DURABLE_MEMORY
MEMORY_INDEX != CANONICAL_MEMORY
MODEL_VERIFICATION != PROMOTION_AUTHORITY
SECRET_DERIVED != CANONICAL_LONG_TERM_MEMORY
SKILL != AUTHORITY
MCP_CAPABILITY_ADVERTISEMENT != GOLAM_CAPABILITY
ACP_CONNECTION != AUTHENTICATED_AUTHORITY
```
