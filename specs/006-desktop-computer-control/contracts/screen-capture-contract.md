# Screen Capture Contract

## Scope

Capture is limited to an explicitly selected display/window/work surface under platform/user permission and Golam capture authority.

## Bounds

Every capture intent binds maximum width, height, frame count, payload bytes and wall time. Spec 006 default is a bounded snapshot/small bounded frame sequence, never indefinite recording.

## Privacy and authority

- Camera and microphone capture are denied.
- Audio capture is disabled in Spec 006 even if a platform capture API can provide audio.
- Raw pixels are ephemeral by default and excluded from ordinary durable evidence/logs.
- Durable evidence may contain source identity, dimensions, timing, payload size and payload digest, not raw pixels.
- OCR/text extraction from raw screenshot pixels is deferred to Spec 007.
- Screenshot pixels and derived digest are untrusted evidence, never authority.

## Lifecycle

Revalidate source identity and system permission immediately before capture. Permission revocation, source disappearance or identity drift fails closed. Native capture resources are released deterministically after bounded consumption.
