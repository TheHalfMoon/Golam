# T003-056 — Canary Qualification Boundary

T003-056 qualification is intentionally split into two independent guarantees.

1. Explicit user-designated secret entry sends the complete submitted value through the same protected vault path regardless of whether its format is recognized. Deterministic recognized-format and deliberately unknown-format canaries must both remain absent from raw durable authority files, canonical/model-visible event payloads, rendered error surfaces and output from an environment-cleared unauthorized subprocess that can inspect only durable authority bytes.
2. Free-text recognition is bounded defense in depth only. The detector reports only a recognized kind, never matched secret material or offsets, has an explicit input-size bound, and is not consulted by the explicit-entry path.

The current Spec 003 slice has no model runtime or prompt compiler; `session_events` is the durable model-visible canonical surface. Later model/prompt integration remains governed by Spec 004 and must preserve this boundary rather than retroactively weakening T003-056.

No real credentials are used by this qualification.
