# Desktop Action Contract

## Route ordering

Spec 006 does not authorize bypassing stronger constitutional control routes. Applicable routes are evaluated in this order:

`domain/application API → native OS automation API → accessibility/semantic tree → browser DOM/protocol → deterministic keyboard/mouse control → vision/pixel fallback`

A route may be skipped only when it is genuinely inapplicable, unavailable, unsupported, denied by required authority/permission, or has failed without creating ambiguous side effects. The raw-input adapter and any pixel-hint producer cannot self-assert fallback eligibility.

## Semantic action

Supported abstract families include invoke, select, toggle, focus, scroll, set-value and platform-equivalent semantic actions.

A semantic action intent binds:
- ToolRequest id and canonical request digest;
- Effect id and immutable effect binding digest;
- exact target identity digest;
- action kind and payload digest;
- observation generation;
- capability, policy and approval refs;
- current protected control-lease id/generation where the action can change interactive state;
- canonical intent digest;
- deadline/expiration.

The intent becomes immutable at `Effect PREPARED`. Immediately before dispatch, the runtime revalidates request/effect/intent bindings, target identity, focus/session/permission state, control-lease generation and all authority bindings. Missing, mismatched, stale, substituted or superseded state fails closed before platform actuation.

## Raw fallback

Raw pointer/keyboard input is a distinct operation class with its own ToolRequest, request/effect/intent bindings, capability, explicit fallback policy, required approval, current protected control-lease generation and a canonical `fallback_eligibility_evidence_digest` proving the applicable higher-priority routes were evaluated under the constitutional ordering. Semantic failure cannot manufacture or inherit this authority.

A bounded `PixelTargetHint` may be attached to a raw fallback only as untrusted candidate geometry. The hint binds capture/source provenance, coordinate-space metadata, expiry and digest. It cannot supply semantic identity, capability, policy, approval, Effect authority, fallback eligibility or a valid control lease. Before dispatch the runtime must fresh-observe and bind the exact work surface/focus/session and revalidate the hint against that state. OCR/text extraction from raw screenshot pixels is not part of Spec 006.

Raw fallback is always denied for:
- an applicable higher-priority constitutional route that remains available and authorized;
- missing/stale/substituted fallback-eligibility evidence;
- Windows secure desktop, UAC secure desktop or a locked/non-interactive Windows session;
- background keylogging/listening;
- unspecified global target;
- stale/unknown focused surface;
- stale/expired/substituted pixel hint;
- superseded/revoked control-lease generation;
- unsupported or ungranted Wayland/X11 session posture.

## Human pause, stop and takeover

A protected human interrupt atomically advances, suspends or revokes the conflicting agent input-authority generation before additional conflicting dispatch. Queued/prepared actions bound to an older generation become invalid. Renderer state alone cannot create, clear or override takeover.

For work already dispatched across the effect boundary, takeover requests cancellation only where cancellation is safe and supported. If terminal truth cannot be proven, record `UNKNOWN_OUTCOME` and reconcile; takeover never authorizes blind replay or silently rewrites the prior effect result.

Releasing human-exclusive control requires a protected attributable transition. A stale model/UI request cannot restore a superseded generation.

## Lifecycle

`route evaluation → ToolRequest → immutable action intent + exact control-lease generation → capability/policy/approval → Effect PREPARED → Kernel/Effect Gate → immediate request/effect/intent/target/permission/lease-generation/fallback-eligibility/visible-channel revalidation → bounded platform dispatch → post-action evidence → terminal reconciliation`

## Outcome

Post-action verification records request/effect/intent/control-lease bindings plus a terminal outcome. If a side effect may have occurred but terminal truth cannot be proven, emit `UNKNOWN_OUTCOME` and block conflicting retry until reconciliation. Restart, timeout, permission change or human takeover never widens authority or converts uncertainty into permission to repeat the action.
