# Desktop Controller Contract

## Purpose

Expose one versioned local desktop-control façade without erasing platform permission and capability differences.

## Required interface

Conceptual operations:
- `capabilities()`
- `observe(request)`
- `focus_work_surface(intent)`
- `perform_semantic_action(intent)`
- `perform_raw_fallback(intent)`
- `capture(intent)`
- `clipboard_read(intent)`
- `clipboard_write(intent)`
- `release_handles(scope)`

## Contract rules

1. Observation is read-only and cannot mint actuation authority.
2. Every side-effect-capable or privacy-sensitive operation requires an already-authorized immutable ToolRequest/request digest, Effect/effect binding digest, canonical intent digest and matching capability/policy/approval bindings as applicable.
3. Adapter capability discovery is descriptive only.
4. Native handles remain adapter-private.
5. Platform adapters return deterministic typed unsupported/permission/stale/unknown outcomes.
6. Semantic failure never triggers raw fallback automatically.
7. Capture, raw input and clipboard read/write are separate capability/effect classes.
8. Request/effect/intent/target/session bindings are revalidated immediately before adapter dispatch; missing, mismatched, stale or substituted state fails closed.
9. Post-boundary uncertainty becomes durable `UNKNOWN_OUTCOME` where a side effect may have occurred and blocks conflicting retry until reconciliation.
10. No remote fallback is part of this interface.
