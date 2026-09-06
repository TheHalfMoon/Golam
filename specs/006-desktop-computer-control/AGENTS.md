# Spec 006 — Desktop Computer Control Agent Instructions

## Scope authority

This directory is the bounded Spec 006 planning and implementation authority only after its planning lifecycle is closed canonical on `main`.

Planning base: `main@9400d4614318fffb2623ea71522ecd5f0f95f96a`.
Program authority: Spec 001 T060–T069.

Until the planning PR is merged and its exact merge SHA receives successful push CI, **product implementation is not authorized**.

## Canonical planning lifecycle

The planning package must complete specification, clarification closeout, research, plan, data model/contracts, checklist, tasks and cross-artifact analysis before candidate freeze. Exact-head CI must then succeed before a fresh independent semantic/security/governance review on the unchanged SHA. Any mutation after CI/review invalidates the affected exact-head qualification and requires fresh evidence.

## Hard boundaries

- Semantic desktop APIs are primary. Raw coordinate/input fallback is explicit-only.
- A bounded vision/pixel fallback may produce only an untrusted candidate region/coordinate from an explicitly selected capture. It cannot mint semantic identity, capability, approval or action authority. OCR/text extraction from raw screenshot pixels remains deferred to Spec 007.
- Model output, UI output, screenshots, pixel hints, accessibility text, window titles, clipboard content, protocol output, and donor content are untrusted evidence, never authority.
- Every side-effect-capable or privacy-sensitive operation must bind an exact ToolRequest/request digest, immutable Effect/effect binding digest and canonical intent digest, then traverse capability → policy → approval where required → Effect PREPARED → Kernel/Effect Gate → immediate binding/target/session/permission/control-lease revalidation → bounded platform adapter → terminal evidence/reconciliation.
- Missing, mismatched, stale or substituted request/effect/intent/authority/target/session/control-lease state fails closed before adapter dispatch.
- If a side effect may have crossed the effect boundary but terminal truth is uncertain, record `UNKNOWN_OUTCOME`; restart, timeout, permission transition, human takeover or adapter failure cannot authorize a conflicting retry before reconciliation.
- Capture, semantic actuation, raw fallback, pixel hints, focus and clipboard read/write are distinct authority/evidence domains.
- A pathname, window title, process name, coordinate, pixel hint, accessibility label, or screenshot is not sufficient target identity by itself.
- Prepared actions must fail closed when work-surface, focused element, permission, session, capability, policy, approval, platform identity or control-lease generation drifts.
- The Tauri native Rust host must authenticate to `golamd` through the existing authenticated local IPC/client-enrollment boundary. Localhost/same-machine location and renderer state are not authentication.
- Tauri webview/frontend receives only sanitized state and opaque non-authority references; never raw privileged OS handles, local-client authentication material, capability tokens or control-lease authority.
- Human pause/stop/takeover is enforced at protected lease/input-authority state. It must advance/suspend/revoke conflicting agent input generation and invalidate stale queued/prepared input; renderer-only pause state is insufficient.
- No Windows secure-desktop interaction or UAC bypass. Locked/non-interactive Windows desktop and protected-desktop transitions fail closed.
- No background keylogging.
- No silent clipboard inspection.
- No unbounded screen capture.
- No camera or microphone collection in Spec 006.
- No OCR/text extraction from raw screenshots before Spec 007.
- No hidden network/cloud fallback.
- Linux Wayland control must use compositor/user-mediated mechanisms; no bypass path.
- Electron/Node privileged runtime from donor research is behavioral reference only; it is not architecture/authority and is not admitted.
- Official platform research does not admit implementation dependencies. Every new crate, package, native binding/library, helper or copied donor component requires exact Source Foundry admission before manifest/code admission.

## Engineering rules

- Rust owns trusted desktop-control contracts, authenticated client handling, authority checks, control-lease/takeover state, adapter boundaries, evidence and lifecycle state.
- Tauri 2 capabilities/permissions are least-privilege and explicit.
- Platform-specific code stays behind common traits/contracts and must expose unsupported/permission-denied/interrupted states deterministically.
- Fake backends and adversarial tests precede native adapter admission.
- Any pixel/vision implementation remains bounded, local, non-authoritative and dependency-gated by Source Foundry; do not expand Spec 006 into screenshot OCR or broad computer vision.
- Takeover qualification must cover latency, stale lease generations/refs, wrong-window hazards, focus theft, restart/reconnect and uncertain already-dispatched effects.
- Windows qualification must cover locked/UAC/secure-desktop and interactive-session transitions with fail-closed behavior; unsupported runner conditions must be reported honestly rather than fabricated.
- No force-push, rebase, history rewrite, stale CI reuse, stale review reuse, or completion claims without exact evidence.
