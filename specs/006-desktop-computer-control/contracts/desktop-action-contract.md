# Desktop Action Contract

## Semantic action

Supported abstract families include invoke, select, toggle, focus, scroll, set-value and platform-equivalent semantic actions.

A semantic action intent binds:
- exact target identity digest;
- action kind and payload digest;
- observation generation;
- capability, policy, approval and Effect refs;
- deadline/expiration.

Immediately before dispatch, the runtime revalidates target identity, focus/session/permission state and all authority bindings. Drift fails closed.

## Raw fallback

Raw pointer/keyboard input is a distinct operation class. It requires explicit fallback policy and required approval/effect authority. Semantic failure cannot manufacture this authority.

Raw fallback is always denied for:
- Windows secure desktop;
- background keylogging/listening;
- unspecified global target;
- stale/unknown focused surface;
- unsupported or ungranted Wayland/X11 session posture.

## Outcome

Post-action verification records a terminal outcome. If a side effect may have occurred but terminal truth cannot be proven, emit `UNKNOWN_OUTCOME` and block conflicting retry until reconciliation.
