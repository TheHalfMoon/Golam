# Data Model: Spec 006 Desktop Computer Control

All authority-relevant structures use deterministic versioned validation and canonical binding digests. Native handles and local-client authentication material are runtime-private and never serialized into frontend DTOs.

## DesktopCapabilitySet

Fields:
- `schema_version`
- `platform`
- `session_kind`
- supported observation kinds
- supported semantic action kinds
- supported capture source kinds
- raw-fallback support flag
- bounded pixel-hint support flag
- clipboard support flags
- human interrupt/takeover support state
- visible-control-channel support state
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

## DesktopControlLeaseState

Fields:
- protected lease id
- lease generation
- controlling principal/ref
- mode (`AGENT_ALLOWED`, `PAUSED`, `HUMAN_EXCLUSIVE`, `REVOKED`)
- issued/updated/expiry timestamps
- parent capability/policy refs
- interrupt/takeover cause and attributable evidence ref
- canonical state digest

Rules: only the privileged control/authority path may mutate this state. Human pause/stop/takeover advances, suspends or revokes the conflicting agent input generation. Stale model, renderer, worker or adapter state cannot restore a prior generation. Every prepared input action binds the exact generation that was current when prepared and must revalidate it immediately before dispatch.

## VisibleControlChannelState

Fields:
- channel id + generation
- channel kind (`TAURI_NATIVE_WINDOW`, `SYSTEM_TRAY`, `PLATFORM_INDICATOR`, or another separately qualified local surface)
- trusted host/client ref
- visibility/liveness state
- supported immediate controls (`PAUSE`, `STOP`, `TAKEOVER`)
- last trusted observation timestamp
- expiration/heartbeat deadline where applicable
- canonical state digest

Rules: at least one qualified channel must be trusted-visible and capable of immediate local interrupt before new autonomous actuation dispatch. Renderer-only DOM state is not sufficient. Loss/expiry of all qualified channels moves the protected control state into an actuation-suspended fail-closed condition until a qualified visible channel is restored. A stale UI message cannot fabricate channel liveness.

## PreparedDesktopAction

Fields:
- request id + canonical request digest
- effect id + immutable effect binding digest
- operation kind (`SEMANTIC_ACTION`, `RAW_INPUT_FALLBACK`, `FOCUS`, `CLIPBOARD_WRITE`, etc.)
- exact target identity digest
- optional bounded pixel-hint digest for raw fallback only
- action payload digest
- capability/policy/approval refs
- control lease id + generation for side-effecting input/focus operations
- qualified visible-control-channel state digest/generation for autonomous interactive actuation
- prepared permission/session evidence ref
- prepared observation digest
- expiration/deadline
- canonical intent digest

Rules: request and effect bindings are distinct, immutable after `Effect PREPARED`, and revalidated immediately before dispatch. Missing, mismatched, stale or substituted bindings, including a superseded/revoked control-lease generation or loss of every qualified visible-control channel for autonomous actuation, fail closed.

## CaptureIntent

Fields:
- request id + canonical request digest
- effect id + immutable effect binding digest
- selected source identity digest
- capture capability/policy/approval refs
- dimensions/frame/byte/time limits
- cursor/audio policy
- prepared system permission/session evidence ref
- retention policy (`EPHEMERAL_ONLY` in default Spec 006 path)
- canonical intent digest

Rules: audio policy is always disabled in Spec 006; capture cannot broaden source after preparation. The request, effect, intent and authority bindings are immutable after `Effect PREPARED`; missing, mismatched or stale authority fails before native capture dispatch.

## CaptureObservation

Fields:
- request digest + effect binding digest + intent digest
- source identity digest
- capture timestamp
- width/height/format
- payload byte count
- payload digest
- permission/session evidence ref
- terminal status (`COMMITTED`, `FAILED_BEFORE_EFFECT`, `UNKNOWN_OUTCOME`, `PERMISSION_REVOKED`, `STALE_TARGET`, `NOT_SUPPORTED`)
- reconciliation evidence ref when terminal truth required reconciliation

Rules: raw bytes are not included in ordinary durable record; payload digest is not action authority. If capture may have crossed the effect boundary but terminal truth is uncertain, durable status is `UNKNOWN_OUTCOME` and conflicting retry/reuse is blocked until reconciliation.

## PixelTargetHint

Fields:
- schema version
- originating capture/source identity digest
- capture observation/payload digest
- bounded region or coordinate in the captured source coordinate space
- coordinate-space/bounds metadata digest
- producer/provenance reference
- optional confidence/score as non-authoritative metadata
- creation timestamp + expiration
- canonical hint digest

Rules: a pixel hint is untrusted evidence only. It cannot contain a native handle, capability, policy decision, approval or semantic identity; it cannot authorize input; it cannot be used after its capture/source/work-surface generation is stale; it does not permit OCR/text extraction in Spec 006. A raw fallback using a hint must independently bind a fresh exact work-surface/focus/session identity plus a dedicated raw-input ToolRequest/effect/capability/policy/approval and current control-lease generation.

## ClipboardIntent

Fields:
- request id + canonical request digest
- effect id + immutable effect binding digest
- operation (`READ` or `WRITE`)
- capability/policy/approval refs
- max byte limit
- content digest for writes
- expiration
- canonical intent digest

Rules: immutable after `Effect PREPARED`; no polling/background inspection; read payload ephemeral by default. Missing, mismatched or stale request/effect/authority bindings fail closed before clipboard access.

## DesktopActionOutcome

Fields:
- request digest + effect id/effect binding digest + intent digest
- attempted target digest
- control lease id/generation where applicable
- visible-control-channel state digest/generation where applicable
- pixel-hint digest where applicable
- status (`DENIED`, `COMMITTED`, `FAILED_BEFORE_EFFECT`, `UNKNOWN_OUTCOME`, `STALE_TARGET`, `PERMISSION_REVOKED`, `INTERRUPTED`, `NOT_SUPPORTED`)
- post-action observation/evidence ref when available
- reconciliation evidence ref when applicable
- platform error class sanitized
- timing/limit metadata

Rules: post-boundary uncertainty becomes `UNKNOWN_OUTCOME`; conflicting follow-up fails closed until reconciliation. Human takeover or visible-channel loss cannot rewrite an already-crossed outcome; both block new conflicting dispatch while any uncertain outcome is reconciled.

## HumanInterruptEvidence

Fields:
- interrupt id
- authenticated/attributed local source ref
- operation (`PAUSE`, `STOP`, `TAKEOVER`, `RELEASE_HUMAN_EXCLUSIVE`)
- prior lease id/generation + resulting lease id/generation
- accepted timestamp
- input-authority revoked/suspended timestamp
- measured takeover latency
- affected queued/prepared operation refs
- cancellation/reconciliation refs
- canonical evidence digest

Rules: renderer-only state is insufficient to create this evidence. A stale interrupt release cannot re-enable a superseded agent generation.

## SanitizedDesktopDto

Frontend-only projection:
- opaque observation/action/control-state refs
- human-readable labels/state
- sanitized capability, permission, visible-control-channel, stale/unsupported, pause/takeover and terminal-status states
- bounded geometry and action menu
- no raw handles, pointers, access tokens, IPC authentication material, capability tokens, portal file descriptors or privileged session objects
