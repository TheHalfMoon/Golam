# Quickstart: Spec 006 Planning and Implementation Discipline

## Planning stage

1. Work only on `spec/006-desktop-computer-control` from canonical base `9400d4614318fffb2623ea71522ecd5f0f95f96a`.
2. Planning artifacts may change; product Rust/Tauri implementation must not begin until planning closes canonical.
3. Run exact-head CI on the final planning tree.
4. Request fresh independent review only after CI succeeds on that exact SHA.
5. Requalify after every mutation.
6. Mark Ready, merge with expected-head guard, and require push CI on the exact merge SHA.
7. Only then create `impl/006-desktop-computer-control`.

## Implementation invariant

For every side-effect-capable path, including semantic action, raw fallback, capture and clipboard, require:

`observation/source selection → exact target/source identity → ToolRequest + canonical request digest → immutable intent → capability/policy/approval → Effect PREPARED + effect binding digest → Kernel/Effect Gate → immediate request/effect/intent/identity/permission revalidation → bounded adapter dispatch → durable terminal evidence → reconciliation when terminal truth is uncertain`

A timeout, adapter crash, daemon restart, permission loss or other uncertainty after the effect boundary becomes `UNKNOWN_OUTCOME`. Conflicting retry/reuse is blocked until reconciliation establishes terminal truth.

Do not:
- use titles/coordinates/screenshots as sole identity;
- expose native handles to frontend;
- dispatch with missing, mismatched, stale or substituted request/effect/authority bindings;
- silently escalate semantic failure to raw input;
- capture microphone/camera;
- run OCR on screenshots;
- silently inspect clipboard;
- persist raw capture/clipboard content by default;
- retry an uncertain side effect before reconciliation;
- add remote/cloud fallback.

## Suggested implementation verification

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- fake-backend adversarial tests that reject missing/mismatched/stale request/effect/authority bindings
- capture/clipboard permission-loss and uncertain-completion tests
- restart tests proving `UNKNOWN_OUTCOME` survives and blocks conflicting retry until reconciliation
- fake backend adversarial suites on all CI platforms
- native platform tests only where the runner actually supports the relevant OS facility; unsupported/permission-limited paths must be explicit and deterministic

The repository CI workflow remains the qualification authority; local commands are supporting evidence only.
