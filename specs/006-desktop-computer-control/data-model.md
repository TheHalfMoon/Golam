# Data Model: Spec 006 Desktop Computer Control

All authority-relevant structures use deterministic versioned validation and canonical binding digests. Native handles are runtime-private and never serialized into frontend DTOs.

## DesktopCapabilitySet

Fields:
- `schema_version`
- `platform`
- `session_kind`
- supported observation kinds
- supported semantic action kinds
- supported capture source kinds
- raw-fallback support flag
- clipboard support flags
- permission/session evidence reference

Rules: capability discovery describes support only; it grants no authority.

## WorkSurfaceIdentity

Fields:
- platform/session identity
- application/process identity where available
- display/window/work-surface stable identifier where available
- creation/incarnation evidence when available
- bounds/geometry metadata digest
- observation generation/digest

Rules: human-readable title/name is metadata only; it is never the sole identity key.

## SemanticElementIdentity

Fields:
- parent work-surface identity digest
- platform element identifier/path/reference evidence
- role/control type
- supported semantic action set digest
- relevant state/geometry digest
- observation generation

Rules: inaccessible/stale/reused/substituted elements fail validation.

## DesktopObservation

Fields:
- observation id and timestamp
- capability/session evidence ref
- work-surface identities
- bounded semantic tree summaries
- focused surface/element ref where observable
- applied limits
- canonical binding digest

Rules: observation is read-only evidence and does not imply action authority.

## PreparedDesktopAction

Fields:
- effect id
- operation kind (`SEMANTIC_ACTION`, `RAW_INPUT_FALLBACK`, `FOCUS`, `CLIPBOARD_WRITE`, etc.)
- exact target identity digest
- action payload digest
- capability/policy/approval refs
- prepared permission/session evidence ref
- prepared observation digest
- expiration/deadline
- canonical intent digest

Rules: immutable after PREPARED; any relevant drift invalidates dispatch.

## CaptureIntent

Fields:
- effect/request id
- selected source identity digest
- dimensions/frame/byte/time limits
- cursor/audio policy
- system permission/session evidence ref
- retention policy (`EPHEMERAL_ONLY` in default Spec 006 path)
- canonical intent digest

Rules: audio policy is always disabled in Spec 006; capture cannot broaden source after preparation.

## CaptureObservation

Fields:
- source identity digest
- capture timestamp
- width/height/format
- payload byte count
- payload digest
- permission/session evidence ref
- terminal status

Rules: raw bytes are not included in ordinary durable record; payload digest is not action authority.

## ClipboardIntent

Fields:
- operation (`READ` or `WRITE`)
- capability/policy/approval/effect refs
- max byte limit
- content digest for writes
- expiration

Rules: no polling/background inspection; read payload ephemeral by default.

## DesktopActionOutcome

Fields:
- effect id + intent digest
- attempted target digest
- status (`DENIED`, `COMMITTED`, `FAILED_BEFORE_EFFECT`, `UNKNOWN_OUTCOME`, `STALE_TARGET`, `PERMISSION_REVOKED`, `NOT_SUPPORTED`)
- post-action observation/evidence ref when available
- platform error class sanitized
- timing/limit metadata

Rules: post-boundary uncertainty becomes `UNKNOWN_OUTCOME`; conflicting follow-up fails closed until reconciliation.

## SanitizedDesktopDto

Frontend-only projection:
- opaque observation/action refs
- human-readable labels/state
- sanitized capability/permission status
- bounded geometry and action menu
- no raw handles, pointers, access tokens, portal file descriptors or privileged session objects
