# Desktop Controller Contract

## Purpose

Expose one versioned local desktop-control façade without erasing platform permission and capability differences. The façade is trusted Rust-side authority; renderer/webview state is never an authorization source. Route selection/fallback eligibility is trusted orchestration and cannot be delegated to a weaker adapter.

## Required interface

Conceptual operations:
- `capabilities()`
- `observe(request)`
- `focus_work_surface(intent)`
- `perform_semantic_action(intent)`
- `perform_raw_fallback(intent)`
- `capture(intent)`
- `register_pixel_target_hint(hint)`
- `clipboard_read(intent)`
- `clipboard_write(intent)`
- `pause_control(interrupt)`
- `stop_control(interrupt)`
- `takeover_control(interrupt)`
- `release_human_exclusive(interrupt)`
- `release_handles(scope)`

## Contract rules

1. Observation is read-only and cannot mint actuation authority.
2. Every side-effect-capable or privacy-sensitive operation requires an already-authorized immutable ToolRequest/request digest, Effect/effect binding digest, canonical intent digest and matching capability/policy/approval bindings as applicable.
3. Adapter capability discovery is descriptive only.
4. Native handles remain adapter-private.
5. Platform adapters return deterministic typed unsupported/permission/stale/interrupted/unknown outcomes.
6. Preserve constitutional route order: domain/app → native OS automation → accessibility/semantic tree → browser DOM/protocol → deterministic keyboard/mouse → vision/pixel. A weaker fallback requires fresh canonical `FallbackEligibilityEvidence`; the adapter cannot self-mint it.
7. Semantic failure alone never triggers raw fallback automatically. An unreconciled `UNKNOWN_OUTCOME` blocks conflicting fallback escalation.
8. Capture, raw input and clipboard read/write are separate capability/effect classes.
9. Request/effect/intent/target/session/fallback-eligibility bindings are revalidated immediately before adapter dispatch; missing, mismatched, stale or substituted state fails closed.
10. Input/focus operations bind the current protected control-lease generation. A human pause/stop/takeover that supersedes that generation prevents dispatch of stale queued/prepared operations.
11. A `PixelTargetHint` is bounded untrusted evidence only. It cannot mint a target identity, capability, fallback eligibility or raw-input authorization and must be revalidated against fresh source/work-surface/focus/session state before a separately authorized raw fallback.
12. Autonomous interactive actuation requires at least one qualified visible-control channel at final revalidation; losing all qualified channels suspends new actuation fail closed.
13. Post-boundary uncertainty becomes durable `UNKNOWN_OUTCOME` where a side effect may have occurred and blocks conflicting retry/fallback until reconciliation.
14. Human interrupt/takeover is enforced in protected Rust authority state and cannot depend on renderer flags.
15. No remote fallback is part of this interface.
