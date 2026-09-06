# Quickstart: Spec 006 Planning and Implementation Discipline

## Planning stage

1. Work only on `spec/006-desktop-computer-control` from canonical base `9400d4614318fffb2623ea71522ecd5f0f95f96a`.
2. Planning artifacts may change; product Rust/Tauri implementation must not begin until planning closes canonical.
3. Complete the Spec Kit lifecycle artifacts, including clarification closeout and cross-artifact analysis, before freezing the planning candidate.
4. Run exact-head CI on the final complete planning tree.
5. Request fresh independent semantic/security/governance review only after CI succeeds on that exact SHA.
6. Requalify after every mutation.
7. Mark Ready, merge with expected-head guard, and require push CI on the exact merge SHA.
8. Only then create `impl/006-desktop-computer-control`.

## Source Foundry invariant

Planning research selects direction, not implementation dependencies. Before introducing any new Rust crate, Tauri/React/TypeScript package, native platform library/binding, helper process or copied donor source, create an exact per-source Source Foundry admission record covering version/revision, permission/license scope, notices, dependency closure, network/build behavior, unsafe/FFI/process boundaries and Golam verification.

Do not mutate a dependency manifest first and attempt to justify it afterward.

## Authenticated desktop-client invariant

The Tauri native Rust host is an authenticated local client of `golamd` through the existing authenticated IPC/client-enrollment boundary. The renderer/webview is an untrusted presentation tier.

Never:
- treat localhost, same-machine location or transport connection as authentication;
- expose local-client credentials, capability material, native handles or privileged session objects to the renderer;
- let renderer state decide authority or bypass Rust-side revalidation.

## Side-effect invariant

For every side-effect-capable path, including semantic action, raw fallback, capture and clipboard, require:

`observation/source selection → exact target/source identity → ToolRequest + canonical request digest → immutable intent → capability/policy/approval → Effect PREPARED + effect binding digest → Kernel/Effect Gate → immediate request/effect/intent/identity/permission/control-lease revalidation → bounded adapter dispatch → durable terminal evidence → reconciliation when terminal truth is uncertain`

A timeout, adapter crash, daemon restart, permission loss, human takeover or other uncertainty after the effect boundary becomes `UNKNOWN_OUTCOME`. Conflicting retry/reuse is blocked until reconciliation establishes terminal truth.

## Vision/pixel fallback invariant

A bounded `PixelTargetHint` may be derived from an explicitly selected authorized capture only as untrusted candidate geometry. It must retain capture/source provenance and expiry. It cannot mint semantic identity or action authority.

Using a pixel hint requires a separate raw-input ToolRequest/effect/capability/policy/approval plus fresh work-surface/focus/session/control-lease revalidation. OCR/text extraction from raw screenshot pixels remains deferred to Spec 007.

## Human interrupt invariant

Local human pause/stop/takeover is enforced at protected control-lease/input-authority state, not only by UI convention. Accepting a protected interrupt advances/suspends/revokes the conflicting agent input generation, blocks new conflicting dispatch, invalidates queued/prepared actions bound to the old generation, and preserves reconciliation for already-crossed effects.

Implementation qualification must measure takeover latency and test stale references, wrong-window hazards, focus theft and reconnect/restart behavior.

## Platform fail-closed invariant

- Windows locked desktop, UAC/secure desktop and interactive-session transitions fail closed; secure desktop is never controlled.
- macOS TCC/Accessibility/Screen Recording permission changes invalidate stale prepared state.
- Linux X11 must be positively identified before any X11-specific raw path; Wayland control/capture remains portal/compositor/user mediated with no bypass.

## Do not

- use titles/coordinates/screenshots as sole identity;
- expose native handles or authentication material to frontend;
- dispatch with missing, mismatched, stale or substituted request/effect/authority/control-lease bindings;
- silently escalate semantic failure to raw input;
- let a pixel hint authorize an action;
- capture microphone/camera;
- run OCR/text extraction on raw screenshots;
- silently inspect clipboard;
- persist raw capture/clipboard content by default;
- retry an uncertain side effect before reconciliation;
- restore agent input from a stale lease generation after human takeover;
- add remote/cloud fallback;
- add an unqualified dependency or donor implementation.

## Suggested implementation verification

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- authenticated Tauri-host/`golamd` local-client tests proving the renderer holds no auth material
- fake-backend adversarial tests that reject missing/mismatched/stale request/effect/authority/control-lease bindings
- pixel-hint tests proving capture/pixel evidence cannot authorize raw input or survive stale source/work-surface generations
- capture/clipboard permission-loss and uncertain-completion tests
- human pause/stop/takeover latency, stale-generation, wrong-window and restart/reconnect tests
- Windows locked/UAC/session-transition fail-closed tests where runner/environment support permits, with deterministic unsupported evidence elsewhere
- restart tests proving `UNKNOWN_OUTCOME` survives and blocks conflicting retry until reconciliation
- fake backend adversarial suites on all CI platforms
- native platform tests only where the runner actually supports the relevant OS facility; unsupported/permission-limited paths must be explicit and deterministic

The repository CI workflow remains the qualification authority; local commands are supporting evidence only.
