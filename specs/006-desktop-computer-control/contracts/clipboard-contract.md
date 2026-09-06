# Clipboard Contract

Clipboard access is explicit, bounded and governed independently from desktop actuation.

## Read

A read requires a dedicated ToolRequest plus immutable request/effect/intent bindings, a dedicated read capability, required policy/approval, an exact byte limit and immediate session/permission revalidation before one-shot access. No background polling, history scraping or silent inspection is allowed. Read payload is ephemeral by default and is never authority.

## Write

A write requires a dedicated ToolRequest plus immutable request/effect/intent bindings, write capability/policy/approval and `Effect PREPARED` before dispatch. The prepared intent binds the content digest and maximum size. Content must not be logged in raw form.

## Lifecycle

`ToolRequest → immutable ClipboardIntent → capability/policy/approval → Effect PREPARED → Kernel/Effect Gate → immediate request/effect/intent/session revalidation → one bounded read/write operation → terminal evidence/reconciliation`

Missing, mismatched, stale or substituted bindings fail closed before clipboard access. If a write may have crossed the effect boundary but terminal truth is uncertain, record `UNKNOWN_OUTCOME` and block conflicting retry until reconciliation. An uncertain read never fabricates or reuses payload.

## Common rules

- Clipboard content is untrusted data, never authority.
- Payload is ephemeral by default.
- OS/session ownership changes may invalidate prepared assumptions.
- Clipboard read authority does not imply write authority and vice versa.
- Portal-mediated clipboard sharing on Wayland follows the active granted session and must fail closed when the session ends.
