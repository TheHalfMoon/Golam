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
- OCR/text extraction from raw screenshot pixels is deferred to Spec 007.
- Screenshot pixels and derived digest are untrusted evidence, never authority.

## Lifecycle

`ToolRequest → immutable CaptureIntent → capability/policy/approval → Effect PREPARED → Kernel/Effect Gate → immediate request/effect/intent/source/permission revalidation → bounded native capture → terminal evidence/reconciliation`

Missing, mismatched, stale or substituted request/effect/authority/source bindings fail closed before native capture. Permission revocation, source disappearance or identity drift fails closed. Native capture resources are released deterministically after bounded consumption.

If native capture may have crossed the effect boundary and terminal truth is uncertain, durable status is `UNKNOWN_OUTCOME`. Conflicting retry or reuse of the same authority/target is blocked until reconciliation establishes terminal truth; a timeout or process restart must not convert uncertainty into permission to repeat the side effect.
