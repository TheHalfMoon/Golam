# Data Model

This document defines logical entities; storage schemas are deferred to bounded implementation specs.

## Principal

Represents an authority-bearing identity.

Fields:
- `principal_id`
- `kind`: user | device | worker | skill | channel | service | mcp_server | external_agent
- `public_identity_material`
- `owner_or_parent`
- `trust_state`
- `created_at`
- `revoked_at?`

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
- `revoked_at?`

Invariant: child lease can only narrow parent authority.

## Session

- `session_id`
- `owner`
- `created_at`
- `status`
- `active_goal_id?`
- `execution_profile_id?`
- `last_checkpoint`
- `canonical_event_seq`

## SessionEvent

- `event_id`
- `session_id`
- `sequence`
- `event_type`
- `schema_version`
- `timestamp`
- `actor_principal`
- `payload`
- `provenance`
- `integrity_hash?`

Invariant: durable model-visible facts are derivable from canonical events; compaction cannot destroy canonical history.

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

## EffectIntent

- `effect_id`
- `session_id`
- `requested_by`
- `capability`
- `resource`
- `operation`
- `risk_level`
- `execution_semantics`
- `idempotency_key?`
- `preconditions`
- `proposed_at`

## EffectRecord

- `effect_id`
- `authorization_decision`
- `approval_record?`
- `started_at?`
- `execution_status`
- `reconciliation_status?`
- `completed_at?`
- `receipt_id?`
- `error?`

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
- `signature?`

## ExecutionProfile

- `profile_id`
- `model_id`
- `inference_backend`
- `quantization`
- `harness_profile`
- `reasoning_mode`
- `context_strategy`
- `cache_strategy`
- `sampling`
- `tool_schema_mode`
- `resource_limits`
- `privacy_policy`
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
- `supersedes?`
- `taint_labels[]`

## ContextCapsule

- `capsule_id`
- `intent`
- `evidence_requirements[]`
- `evidence_items[]`
- `source_authority`
- `temporal_state`
- `permission_state`
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
- `manifest`
- `instructions_ref`
- `scripts[]`
- `resources[]`
- `requested_capabilities[]`
- `security_scan_record`
- `test_record`
- `signature?`
- `installed_at?`

## WorkerDefinition

- `worker_id`
- `version`
- `role`
- `behavior_contract`
- `capability_manifest`
- `memory_loadout`
- `execution_profile_policy`
- `tool/skill_loadout`
- `schedule_or_triggers[]`
- `evaluation_record`
- `signature/provenance`

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

## ConnectEvent

- `event_id`
- `sender_principal`
- `sender_device`
- `target_device`
- `channel`
- `timestamp`
- `nonce`
- `payload_hash`
- `requested_capability?`
- `signature`
- `encrypted_payload`

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
