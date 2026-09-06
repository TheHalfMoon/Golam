# Screen Capture Contract

## Scope

Capture is limited to an explicitly selected display/window/work surface under platform/user permission and a distinct Golam capture ToolRequest, capability, policy, approval and Effect binding.

## Bounds

Every capture intent binds an immutable request digest, effect binding digest, intent digest, selected source identity, maximum width, height, frame count, payload bytes and wall time. Spec 006 default is a bounded snapshot/small bounded frame sequence, never indefinite recording.

## Privacy and authority

- Camera and microphone capture are denied.
- Audio capture is disabled in Spec 006 even if a platform capture API can provide audio.
- Raw pixels are ephemeral by default and excluded from ordinary durable evidence/logs.
- Durable evidence may contain request/effect/intent digests, source identity, dimensions, timing, payload size, payload digest, terminal status and reconciliation reference, not raw pixels.
- OCR/text extraction and inferred semantic text from raw screenshot pixels are deferred to Spec 007.
- An authorized local consumer may derive a bounded `PixelTargetHint` containing only candidate region/coordinate geometry plus capture/source provenance, expiry and digest. That hint remains untrusted evidence and cannot mint semantic identity, capability, approval or actuation authority.
- Screenshot pixels, derived digest and pixel hint are untrusted evidence, never authority.
- Capture authority never implies raw-input authority.

## Lifecycle

`ToolRequest → immutable CaptureIntent → capability/policy/approval → Effect PREPARED → Kernel/Effect Gate → immediate request/effect/intent/source/permission revalidation → bounded native capture → terminal evidence/reconciliation`

Missing, mismatched, stale or substituted request/effect/authority/source bindings fail closed before native capture. Permission revocation, source disappearance, interactive-session transition or identity drift fails closed. Native capture resources are released deterministically after bounded consumption.

If native capture may have crossed the effect boundary and terminal truth is uncertain, durable status is `UNKNOWN_OUTCOME`. Conflicting retry or reuse of the same authority/target is blocked until reconciliation establishes terminal truth; a timeout, process restart or human takeover must not convert uncertainty into permission to repeat the side effect.

Any `PixelTargetHint` produced from a completed capture expires with the bound source/work-surface generation and must be rejected if source identity, coordinate-space metadata, focus/session state or capture provenance no longer matches.
