# Desktop Controller Contract

## Purpose

Expose one versioned local desktop-control façade without erasing platform permission and capability differences. The façade is trusted Rust-side authority; renderer/webview state is never an authorization source.

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
6. Semantic failure never triggers raw fallback automatically.
7. Capture, raw input and clipboard read/write are separate capability/effect classes.
8. Request/effect/intent/target/session bindings are revalidated immediately before adapter dispatch; missing, mismatched, stale or substituted state fails closed.
9. Input/focus operations bind the current protected control-lease generation. A human pause/stop/takeover that supersedes that generation prevents dispatch of stale queued/prepared operations.
10. A `PixelTargetHint` is bounded untrusted evidence only. It cannot mint a target identity, capability or raw-input authorization and must be revalidated against fresh source/work-surface/focus/session state before a separately authorized raw fallback.
11. Post-boundary uncertainty becomes durable `UNKNOWN_OUTCOME` where a side effect may have occurred and blocks conflicting retry until reconciliation.
12. Human interrupt/takeover is enforced in protected Rust authority state and cannot depend on renderer flags.
13. No remote fallback is part of this interface.
