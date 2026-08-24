# Data Model — Spec 002

Logical model; exact SQL column types/migrations are implementation details constrained by these invariants.

## ClientIdentity

- `client_id`
- `key_id`
- `public_key`
- `kind`: cli | desktop_future | ide_future | test
- `owner_principal`
- `enrolled_at`
- `last_authenticated_at?`
- `revoked_at?`
- `assurance_class`

Invariant: revoked client cannot establish a new privileged IPC session.

## IpcConnection

- `connection_id`
- `client_id?`
- `peer_os_identity`
- `protocol_version`
- `server_epoch`
- `challenge_nonce`
- `client_nonce?`
- `authenticated_at?`
- `closed_at?`
- `close_reason?`

This is operational/audit state, not a bearer authority object exposed to the client.

## Session

- `session_id`
- `owner_principal`
- `created_global_seq`
- `status`
- `parent_session_id?`
- `parent_session_seq?`
- `parent_event_hash?`
- `latest_session_seq`
- `latest_event_hash`
- `latest_checkpoint_id?`

Fork invariant: parent anchor is immutable after child creation.

## SessionEvent

- `event_id`
- `global_seq`
- `session_id`
- `session_seq`
- `event_type`
- `schema_version`
- `actor_principal`
- `recorded_at`
- `payload_bytes`
- `payload_hash`
- `previous_session_event_hash?`
- `event_hash`
- `security_critical`
- `previous_audit_hash?`
- `audit_hash?`

Ordering invariant: `global_seq` defines total canonical audit order; timestamp does not.

## GoalVersion

- `goal_version_id`
- `goal_id`
- `session_id`
- `version`
- `goal`
- `acceptance_criteria[]`
- `constraints[]`
- `scope`
- `proven_facts[]`
- `blockers[]`
- `next_safe_action?`
- `created_event_id`
- `created_global_seq`

Invariant: versions append; prior goal state is never overwritten.

## ArtifactRef

- `artifact_hash`
- `size_bytes`
- `media_type`
- `created_global_seq`
- `retention_class`
- `path_relative_to_artifact_root`

Bytes are addressed by BLAKE3 hash. Authority is never inferred from artifact content.

## Checkpoint

- `checkpoint_id`
- `session_id`
- `through_session_seq`
- `through_global_seq`
- `through_event_hash`
- `projection_schema_version`
- `artifact_hash`
- `created_event_id`
- `verified_at?`

Invariant: invalid checkpoint can be discarded without losing canonical history.

## EffectIntent

- `effect_id`
- `session_id`
- `requested_by`
- `action`
- `resource`
- `risk_class`
- `execution_semantics`
- `idempotency_key?`
- `preconditions`
- `dependencies[]`
- `payload_hash`
- `proposed_event_id`

## EffectTransition

- `transition_id`
- `effect_id`
- `global_seq`
- `from_state?`
- `to_state`
- `attempt_id?`
- `reason_code?`
- `evidence_ref?`
- `event_id`

Invariant: state transition requires expected current state; conflicting/stale transition fails.

## EffectAttempt

- `attempt_id`
- `effect_id`
- `started_global_seq`
- `handler_id`
- `handler_version`
- `dispatch_token`
- `started_at`
- `finished_at?`
- `outcome`: success | failure | unknown
- `remote_or_simulated_receipt?`

## AuthorizationDecision

- `decision_id`
- `principal`
- `action`
- `resource`
- `context_hash`
- `decision`: allow | deny
- `reason_code`
- `global_seq`

Spec 002 policy is bootstrap-only; schema remains stable for Spec 003.

## AuditChainHead

- `chain_name`
- `last_global_seq`
- `last_hash`

## RecoveryIncident

- `incident_id`
- `detected_at`
- `kind`: db_integrity | hash_chain | checkpoint | disk_full | incomplete_effect | protocol
- `severity`
- `affected_refs[]`
- `recovery_mode`
- `resolution?`

Recovery incidents are append-only evidence; resolution does not erase the incident.
