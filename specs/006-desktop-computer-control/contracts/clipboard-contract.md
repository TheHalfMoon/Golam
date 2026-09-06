# Clipboard Contract

Clipboard access is explicit and bounded.

## Read
Requires a dedicated read capability and any required policy/approval. A read has a maximum payload size and is one-shot. No background polling, history scraping or silent inspection is allowed.

## Write
Requires a dedicated mutation/effect binding. The prepared intent binds the content digest and maximum size. Content must not be logged in raw form.

## Common rules
- Clipboard content is untrusted data, never authority.
- Payload is ephemeral by default.
- OS/session ownership changes may invalidate prepared assumptions.
- Clipboard read authority does not imply write authority and vice versa.
- Portal-mediated clipboard sharing on Wayland follows the active granted session and must fail closed when the session ends.
