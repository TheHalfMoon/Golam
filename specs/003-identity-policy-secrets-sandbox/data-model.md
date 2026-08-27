# Data Model — Spec 003

Logical protected authority model. Exact SQLite columns/indexes/canonical encodings are implementation details constrained by these invariants.

## PrincipalRecord

Extends authenticated identity with authority-facing metadata:
- `principal_id`
- `principal_kind`
- `owner_principal?`
- `status`
- `attributes_version`
- `created_global_seq`
- `revoked_at?`

Invariant: external/model data cannot instantiate or reactivate a principal.

## PolicyBundle

- `policy_bundle_id`
- `version`
- `schema_version`
- `canonical_policy_bytes`
- `bundle_hash`
- `created_by`
- `created_global_seq`
- `validation_status`

Invariant: immutable after creation; invalid bundle cannot become active.

## ActivePolicy

- singleton/protected scope
- `policy_bundle_id`
- `bundle_hash`
- `activated_by`
- `activation_effect_id`
- `activated_global_seq`

Invariant: pointer/hash must resolve to one validated immutable bundle; activation is atomic with security evidence.

## CapabilityLease

- `lease_id`
- `principal_id`
- `parent_lease_id?`
- `actions_scope`
- `resources_scope`
- `context_constraints`
- `issued_by`
- `issued_global_seq`
- `not_before?`
- `expires_at?`
- `generation`
- `status`
- `authority_digest`

Invariant: child authority is a strict/equal subset of parent; no self-widening.

## CapabilityRevocation

- `revocation_id`
- `lease_id`
- `revoked_by`
- `reason_code`
- `revoked_global_seq`
- `revoked_at`

Invariant: revocation is monotonic and checked before use.

## Approval

- `approval_id`
- `class`: ONCE | SESSION_SCOPED | TIME_BOXED | OPERATION_PATTERN | RUN_PREAUTHORIZATION
- `approver_principal`
- `scope_digest`
- `action_scope`
- `resource_scope`
- `effect_id?`
- `session_id?`
- `risk_class`
- `taint_digest`
- `parent_decision_id`
- `issued_at`
- `expires_at?`
- `max_uses?`
- `revoked_at?`

Invariant: approval does not widen lease/policy authority.

## ApprovalConsumption

- `consumption_id`
- `approval_id`
- `effect_or_operation_id`
- `reserved_global_seq`
- `consumed_global_seq?`
- `state`: reserved | consumed | released

Invariant: ONCE approval can have at most one successful protected consumption; crash/retry cannot duplicate authorization.

## AuthorizationDecisionV2

Extends Spec 002 decision evidence:
- `decision_id`
- `principal`
- `action`
- `resource`
- `context_hash`
- `hard_guard_result`
- `lease_id?`
- `lease_generation?`
- `policy_bundle_id?`
- `policy_bundle_hash?`
- `matched_rule_ids[]`
- `approval_id?`
- `decision`: allow | deny
- `reason_code`
- `global_seq`

Invariant: contains no secret plaintext; decision evidence identifies exact authority state used.

## TaintAttestation

- `attestation_id`
- `source_artifact_ids[]`
- `source_labels[]`
- `result_artifact_id`
- `result_labels[]`
- `mechanism`: human_approval | deterministic_verifier | secret_elimination_sanitizer
- `rule_id`
- `principal?`
- `evidence_hash`
- `created_global_seq`

Invariant: source labels are never mutated in place; downgrade creates auditable derived evidence.

## VerifierRule

- `rule_id`
- `kind`
- `version`
- `authority_source_binding`
- `allowed_downgrades`
- `registered_by`
- `status`
- `created_global_seq`

Invariant: tainted input cannot register or alter its own verifier rule.

## SecretRecord

- `secret_id`
- `classification`
- `owner_principal`
- `current_version`
- `status`
- `created_global_seq`
- `revoked_at?`

No plaintext value.

## SecretVersion

- `secret_id`
- `version`
- `ciphertext`
- `nonce_or_algorithm_metadata`
- `associated_data_hash`
- `created_global_seq`
- `rotated_from?`
- `retired_at?`

Invariant: durable canonical bytes must not contain the plaintext secret value.

## SecretHandle

Opaque authority reference returned to authorized callers:
- `handle_id`
- `secret_id`
- `version_constraint?`
- `purpose_scope`
- `expires_at?`

Invariant: handle is not the plaintext secret and is useless outside kernel validation.

## SecretUseRecord

- `use_id`
- `handle_id`
- `principal`
- `purpose`
- `destination_or_process`
- `mode`: brokered | isolated_fallback
- `approval_id?`
- `decision_id`
- `created_global_seq`

Invariant: records metadata only, not plaintext.

## EgressPermit

- `permit_id`
- `principal_or_process`
- `action`
- `purpose`
- `destination_scope`
- `protocol_port_scope`
- `taint_digest`
- `secret_handle_id?`
- `parent_lease_id`
- `issued_at`
- `expires_at?`
- `usage_limit?`
- `status`

Invariant: ineffective for external egress while strict-local hard guard is active.

## SandboxProfile

- `profile_id`
- `version`
- `class`
- `filesystem_read_roots[]`
- `filesystem_write_roots[]`
- `network_rule`
- `environment_allowlist[]`
- `spawn_rule`
- `cpu_limit?`
- `memory_limit?`
- `time_limit?`
- `output_limit?`
- `device_allowlist[]`
- `ipc_allowlist[]`
- `inherited_handle_rules[]`
- `platform_requirements[]`
- `status`

Invariant: profile declares desired limits; executable admission requires a platform executor proving all required controls are enforceable.

## SandboxAdmission

- `admission_id`
- `profile_id`
- `principal/process`
- `lease_id`
- `decision_id`
- `egress_permit_id?`
- `resolved_launch_plan_hash`
- `platform_executor`
- `created_global_seq`

Invariant: unsupported required enforcement denies before process launch.

## ProtectedResourceClass

Logical classes include:
- policy/schema/active pointer;
- principal/identity authority;
- capability leases/revocations;
- approvals/consumption;
- secret vault/keys/handles;
- taint verifier/sanitizer registry and attestations;
- egress policy/permits;
- sandbox profile definitions/admissions;
- inherited Spec 002 effect/audit/client/recovery authority.

All Spec 003 protected source records require complete `authority-security` coverage or an equivalently strong authenticated integrity mechanism. Missing coverage is corruption, not a rebuildable projection.
