# Focus and Work-Surface Contract

Focus is mutable external state, not authority.

A focus intent binds an immutable ToolRequest/request digest, Effect/effect binding digest, canonical intent digest, exact work-surface identity, capability/policy/approval refs as applicable, the current protected control-lease id/generation, qualified visible-control-channel state for autonomous focus, and prepared session/observation evidence.

## Governed focus lifecycle

`ToolRequest → immutable focus intent → capability/policy/approval → Effect PREPARED → Kernel/Effect Gate → immediate request/effect/intent/work-surface/session/permission/control-lease/visible-channel revalidation → bounded platform focus dispatch → observe actual focus → terminal evidence/reconciliation`

The Kernel/Effect Gate authorization is mandatory for focus dispatch. Missing, stale, mismatched or substituted Gate authorization fails closed before platform focus. Immediately before dispatch, the runtime also revalidates every bound request/effect/intent/authority identity and verifies that the surface incarnation, session, permission state, control-lease generation and required visible-control-channel state still match the prepared focus intent. Missing, mismatched, stale, substituted or superseded state fails closed before platform actuation.

After a focus request, the runtime must observe whether the intended surface actually became focused before allowing any operation whose target depends on focus. If the focus effect may have occurred but terminal truth is uncertain, record durable `UNKNOWN_OUTCOME`; conflicting or focus-dependent action remains blocked until reconciliation establishes terminal truth. Restart, reconnect or a weaker route cannot convert uncertain focus into permission to continue.

A human pause/stop/takeover that advances, suspends or revokes the agent input-authority generation invalidates pending focus-dependent actions bound to the old generation. Human takeover does not assume focus succeeded and cannot silently retarget an already-prepared action to a different window.

Required denial/race cases:
- absent, stale, mismatched or substituted Kernel/Effect Gate authorization;
- target window closed/recreated;
- process/application restarted;
- focus stolen between prepare and execute;
- active session changed;
- permission revoked;
- surface identifier reused with different incarnation;
- user switches workspace/desktop such that the target is no longer eligible;
- human pause/stop/takeover supersedes the bound control-lease generation;
- loss of every required qualified visible-control channel for autonomous focus;
- locked/UAC/secure-desktop transition on Windows;
- stale pixel-hint coordinate-space/source generation for a raw fallback.

A coordinate or pixel hint is never a stable work-surface identity.
