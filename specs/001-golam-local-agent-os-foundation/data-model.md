# Data Model

This document defines logical entities; storage schemas are deferred to bounded implementation specs. The GLM-5.3 reconciliation makes security-critical integrity, IPC identity, protected authority state, memory governance, and replay/fork semantics explicit.

## Principal

Represents an authority-bearing identity.

Fields:
- `principal_id`
- `kind`: user | device | local_client | worker | skill | channel | service | mcp_server | external_agent
- `public_identity_material`
- `owner_or_parent`
- `trust_state`
- `created_at`
- `revoked_at?`

## LocalClientCredential

- `client_id`
- `principal_id`
- `transport_kind`: windows_named_pipe | unix_socket | approved_loopback
- `os_peer_identity`
- `credential_hash_or_public_key`
- `scopes[]`
- `issued_at`
- `expires_at?`
- `revoked_at?`
- `last_authenticated_at?`

Invariant: same-machine/localhost presence is not authentication.

## ProtectedResource

Kernel-owned authority state that generic capabilities cannot mutate.

- `resource_id`
- `class`: policy_store | principal_registry | lease_registry | approval_store | secret_vault | effect_journal | audit_chain | goal_security_state | connect_pairing_registry | egress_policy | skill_lock | schedule_authority
- `storage_ref`
- `integrity_state`
- `owner`: privileged_kernel

## CapabilityLease

- `lease_id`
- `subject_principal`
- `issuer_principal`
- `capabilities[]`
- `resource_scope`
- `constraints`
- `issued_at`
- `expires_at?`
- `parent_lease?`
- `generation?`
- `revoked_at?`

Invariant: child lease can only narrow parent authority; current expiry/revocation/generation is checked at protected action time.

## ApprovalRecord

- `approval_id`
- `class`: once | session_scoped | time_boxed | operation_pattern | run_preauthorization
- `approver_principal`
- `approver_device`
- `parent_authorization_context`
- `allowed_actions[]`
- `resource_scope`
- `quantitative_limits?`
- `risk_class`
- `taint_summary[]`
- `issued_at`
- `expires_at?`
- `remaining_uses?`
- `revoked_at?`

## Session

- `session_id`
- `owner`
- `created_at`
- `status`
- `active_goal_id?`
- `execution_profile_id?`
- `last_checkpoint`
- `canonical_event_seq`
- `parent_session_id?`
- `parent_event_sequence?`
- `parent_event_hash?`

## SessionEvent

- `event_id`
- `session_id`
- `sequence`
- `global_audit_order?`
- `causal_parent_refs[]`
- `event_type`
- `schema_version`
- `timestamp`
- `actor_principal`
- `payload_or_artifact_refs`
- `provenance`
- `taint_labels[]`
- `previous_integrity_hash?`
- `integrity_hash?`

Invariant: integrity chaining is mandatory for security-critical event families (authorization/approval, capability changes, effects, Connect control/pairing, secret-use metadata, memory promotion/governance, receipts). Durable model-visible facts are derivable from canonical events; compaction cannot destroy canonical history.

## SessionFork

- `child_session_id`
- `parent_session_id`
- `parent_event_sequence`
- `parent_event_hash`
- `initiating_principal`
- `reason`
- `created_at`

Invariant: retry/rewind/model-alternative creates a new branch; parent canonical history is immutable.

## GoalLedger

- `goal_id`
- `session_id`
- `goal`
- `acceptance_criteria[]`
- `constraints[]`
- `scope`
- `proven_facts[]`
- `authoritative_state_refs[]`
- `blockers[]`
- `completed_work[]`
- `next_safe_action?`
- `version`

## EffectHandlerDefinition

- `handler_id`
- `effect_family`
- `schema_version`
- `execution_semantics`
- `idempotency_derivation?`
- `reconcile_strategy`
- `timeout_policy`
- `compensation_strategy?`
- `verification_requirements[]`

## EffectIntent

- `effect_id`
- `session_id`
- `requested_by`
- `capability`
- `resource`
- `operation`
- `risk_level`
- `taint_context[]`
- `execution_semantics`
- `handler_id`
- `idempotency_key?`
- `preconditions`
- `approval_requirement?`
- `proposed_at`
- `durably_committed_at?`

## EffectRecord

- `effect_id`
- `authorization_decision`
- `approval_record?`
- `started_at?`
- `execution_status`
- `reconciliation_status?`
- `reconciliation_evidence?`
- `completed_at?`
- `receipt_id?`
- `error?`

Invariant: dependent effects cannot proceed while a prerequisite is `UNKNOWN_OUTCOME`.

## ArtifactBlob

- `content_hash`
- `kind`
- `size`
- `sensitivity`
- `taint_labels[]`
- `encryption_state`
- `created_by_event`
- `retention_class`
- `last_required_checkpoint?`
- `deleted_at?`
- `tombstone_reason?`

## ExecutionReceipt

- `receipt_id`
- `session_id`
- `goal_id?`
- `models[]`
- `model_locality`
- `tool_uses[]`
- `network_destinations[]`
- `files_read_summary`
- `files_changed[]`
- `secret_handles_used[]`
- `external_effects[]`
- `approvals[]`
- `verification_results[]`
- `trace_ref`
- `created_at`
- `previous_integrity_hash`
- `integrity_hash`
- `signature?`

## ExecutionProfile

- `profile_id`
- `model_id`
- `model_revision`
- `tokenizer_identity`
- `chat_template_identity`
- `inference_backend`
- `locality_class`
- `quantization_or_precision`
- `hardware_device_mapping`
- `harness_profile`
- `reasoning_mode`
- `tool_call_conformance_mode`: native | grammar_constrained | text_fallback
- `tool_schema_mode`
- `context_strategy`
- `prompt_prefix_cache_strategy`
- `kv_cache_policy`
- `warm_residency_policy`
- `sampling`
- `workload_class`: interactive | batch | background
- `multimodal_capabilities[]`
- `resource_limits`
- `latency_quality_budget`
- `privacy_policy`
- `network_policy`
- `fallback_policy`
- `benchmark_record_refs[]`

## HardwareProfile

- `device_id`
- `cpu`
- `ram`
- `gpu[]`
- `vram[]`
- `accelerators`
- `supported_backends`
- `disk_constraints`
- `measured_throughput`
- `calibrated_at`

## MemoryAsset

- `memory_id`
- `kind`: working | run | project | user | verified_repo_knowledge
- `canonical_markdown_ref?`
- `content_or_reference`
- `owner`
- `scope`
- `provenance[]`
- `authority`
- `confidence`
- `created_at`
- `valid_from?`
- `expires_at?`
- `supersedes[]`
- `contradicts[]`
- `taint_labels[]`
- `version`
- `content_hash`

## MemoryGovernanceEvent

- `operation`: add | update | supersede | contradict | merge | expire | forget | redact | promote
- `memory_ids[]`
- `actor_principal`
- `reason`
- `source_provenance[]`
- `approval_or_verifier?`
- `before_hashes[]`
- `after_hashes[]`
- `timestamp`
- `integrity_hash`

## ContextCapsule

- `capsule_id`
- `intent`
- `evidence_requirements[]`
- `evidence_items[]`
- `source_authority`
- `temporal_state`
- `permission_state`
- `taint_summary[]`
- `token_budget`
- `cache_plan`
- `sufficiency_status`

## SkillPackage

- `skill_id`
- `name`
- `version`
- `source`
- `source_commit?`
- `content_hash`
- `license`
- `verification_status`
- `manifest`
- `instructions_ref`
- `scripts[]`
- `resources[]`
- `requested_capabilities[]`
- `sandbox_profile?`
- `security_scan_record`
- `test_record`
- `signature?`
- `installed_at?`

## SandboxProfile

- `profile_id`
- `kind`
- `filesystem_roots[]`
- `writable_roots[]`
- `network_policy`
- `environment_allowlist[]`
- `process_spawn_policy`
- `cpu_memory_time_limits`
- `output_limits`
- `device_access[]`
- `ipc_endpoints[]`

## WorkerDefinition

- `worker_id`
- `version`
- `role`
- `behavior_contract`
- `capability_manifest`
- `memory_loadout`
- `execution_profile_policy`
- `tool_skill_loadout`
- `schedule_or_triggers[]`
- `evaluation_record`
- `signature_provenance`

## Device

- `device_id`
- `owner`
- `public_key`
- `platform`
- `capabilities`
- `trust_tier`
- `paired_at`
- `last_seen`
- `revoked_at?`

## ChannelBinding

- `binding_id`
- `provider`
- `provider_stable_sender_id`
- `principal_id`
- `conversation_scope?`
- `created_by_local_principal`
- `created_at`
- `version`
- `revoked_at?`

Invariant: usernames/display names are never authority keys.

## ConnectEvent

- `event_id`
- `sender_principal`
- `sender_device`
- `target_device`
- `channel_or_transport`
- `binding_id?`
- `timestamp`
- `nonce`
- `lease_id?`
- `lease_generation?`
- `payload_hash`
- `requested_capability?`
- `signature`
- `encrypted_payload`

## EgressDecision

- `decision_id`
- `principal_or_process`
- `destination`
- `purpose`
- `locality_mode`
- `taint_labels[]`
- `credential_handle?`
- `capability_lease?`
- `decision`
- `reason`
- `timestamp`

## ComputerStateSnapshot

- `snapshot_id`
- `device_id`
- `timestamp`
- `active_app`
- `windows[]`
- `monitors[]`
- `browser_state_refs[]`
- `process_summary`
- `clipboard_metadata?`
- `semantic_element_refs[]`
- `screenshot_ref?`
- `permission_state`

## DesktopElementRef

- `snapshot_id`
- `element_id`
- `platform_locator`
- `role`
- `name`
- `supported_actions[]`
- `bounds?`
- `staleness_token`

Invariant: stale refs must fail/reobserve rather than silently target another element.
