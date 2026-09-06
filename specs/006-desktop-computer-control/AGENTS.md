# Spec 006 — Desktop Computer Control Agent Instructions

## Scope authority

This directory is the bounded Spec 006 planning and implementation authority only after its planning lifecycle is closed canonical on `main`.

Planning base: `main@9400d4614318fffb2623ea71522ecd5f0f95f96a`.
Program authority: Spec 001 T060–T071.

Until the planning PR is merged and its exact merge SHA receives successful push CI, **product implementation is not authorized**.

## Hard boundaries

- Semantic desktop APIs are primary. Raw coordinate/input fallback is explicit-only.
- Model output, UI output, screenshots, accessibility text, window titles, clipboard content, protocol output, and donor content are untrusted evidence, never authority.
- Every side-effect-capable or privacy-sensitive operation must bind an exact ToolRequest/request digest, immutable Effect/effect binding digest and canonical intent digest, then traverse capability → policy → approval where required → Effect PREPARED → Kernel/Effect Gate → immediate binding/target/session/permission revalidation → bounded platform adapter → terminal evidence/reconciliation.
- Missing, mismatched, stale or substituted request/effect/intent/authority/target/session state fails closed before adapter dispatch.
- If a side effect may have crossed the effect boundary but terminal truth is uncertain, record `UNKNOWN_OUTCOME`; restart, timeout or adapter failure cannot authorize a conflicting retry before reconciliation.
- Capture, semantic actuation, raw fallback, focus and clipboard read/write are distinct authority domains.
- A pathname, window title, process name, coordinate, accessibility label, or screenshot is not sufficient target identity by itself.
- Prepared actions must fail closed when work-surface, focused element, permission, session, capability, policy, approval, or platform identity drifts.
- No Windows secure-desktop interaction.
- No background keylogging.
- No silent clipboard inspection.
- No unbounded screen capture.
- No camera or microphone collection in Spec 006.
- No OCR/text extraction from raw screenshots before Spec 007.
- No hidden network/cloud fallback.
- Linux Wayland control must use compositor/user-mediated mechanisms; no bypass path.
- Electron/Node privileged runtime from donor research is behavioral reference only; it is not architecture/authority and is not admitted.
- Tauri webview/frontend receives only sanitized state and opaque non-authority references; never raw privileged OS handles.

## Engineering rules

- Rust owns trusted desktop-control contracts, authority checks, adapter boundaries, evidence and lifecycle state.
- Tauri 2 capabilities/permissions are least-privilege and explicit.
- Platform-specific code stays behind common traits/contracts and must expose unsupported/permission-denied states deterministically.
- Fake backends and adversarial tests precede native adapter admission.
- No force-push, rebase, history rewrite, stale CI reuse, stale review reuse, or completion claims without exact evidence.
