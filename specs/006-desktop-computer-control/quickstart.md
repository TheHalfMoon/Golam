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

For any action-capable path, require:

`observation → exact target identity → explicit intent → capability/policy/approval → Effect PREPARED → immediate revalidation → adapter dispatch → post-action evidence → terminal reconciliation`

Do not:
- use titles/coordinates/screenshots as sole identity;
- expose native handles to frontend;
- silently escalate semantic failure to raw input;
- capture microphone/camera;
- run OCR on screenshots;
- silently inspect clipboard;
- persist raw capture/clipboard content by default;
- add remote/cloud fallback.

## Suggested implementation verification

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- fake backend adversarial suites on all CI platforms
- native platform tests only where the runner actually supports the relevant OS facility; unsupported/permission-limited paths must be explicit and deterministic

The repository CI workflow remains the qualification authority; local commands are supporting evidence only.
