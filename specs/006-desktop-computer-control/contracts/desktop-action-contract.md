# Desktop Action Contract

## Semantic action

Supported abstract families include invoke, select, toggle, focus, scroll, set-value and platform-equivalent semantic actions.

A semantic action intent binds:
- ToolRequest id and canonical request digest;
- Effect id and immutable effect binding digest;
- exact target identity digest;
- action kind and payload digest;
- observation generation;
- capability, policy and approval refs;
- canonical intent digest;
- deadline/expiration.

The intent becomes immutable at `Effect PREPARED`. Immediately before dispatch, the runtime revalidates request/effect/intent bindings, target identity, focus/session/permission state and all authority bindings. Missing, mismatched, stale or substituted state fails closed before platform actuation.

## Raw fallback

Raw pointer/keyboard input is a distinct operation class with its own ToolRequest, request/effect/intent bindings, capability, explicit fallback policy and required approval. Semantic failure cannot manufacture or inherit this authority.

Raw fallback is always denied for:
- Windows secure desktop;
- background keylogging/listening;
- unspecified global target;
- stale/unknown focused surface;
- unsupported or ungranted Wayland/X11 session posture.

## Lifecycle

`ToolRequest → immutable action intent → capability/policy/approval → Effect PREPARED → Kernel/Effect Gate → immediate request/effect/intent/target/permission revalidation → bounded platform dispatch → post-action evidence → terminal reconciliation`

## Outcome

Post-action verification records request/effect/intent bindings plus a terminal outcome. If a side effect may have occurred but terminal truth cannot be proven, emit `UNKNOWN_OUTCOME` and block conflicting retry until reconciliation. Restart or timeout never widens authority or converts uncertainty into permission to repeat the action.
