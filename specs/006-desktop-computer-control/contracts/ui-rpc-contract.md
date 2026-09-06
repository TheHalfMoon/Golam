# Desktop UI RPC Contract

The Tauri webview/frontend is an untrusted presentation tier. The native Rust Tauri host is the authenticated local client of `golamd` and must use the existing authenticated IPC/client-enrollment trust boundary.

## Authenticated native-host boundary

- The native Rust host authenticates to `golamd`; localhost, same-machine location, process ancestry, a successful transport connection or a renderer-provided identity is not authentication.
- Client credentials, enrollment secrets, capability tokens and authority-bearing IPC material remain Rust-side and never enter the webview/DOM/JavaScript state.
- Every renderer-originated request is treated as untrusted input. The Rust host and `golamd` independently validate schema, opaque references, authority, control-lease generation, visible-control-channel state and effect state.
- Reconnect requires the existing authenticated-client rules; a disconnected/restarted renderer cannot inherit or recreate authority from cached UI state.

## Qualified visible-control channel

Autonomous computer control requires at least one qualified persistent local indicator/control channel that visibly communicates active control and provides immediate local `PAUSE`, `STOP` and `TAKEOVER` actions.

- Trusted Rust owns or independently observes channel identity/liveness/visibility; renderer DOM state alone is not evidence that a qualified channel exists.
- A Tauri native window, native tray/status surface or platform indicator may qualify only after implementation defines and tests the required visibility/liveness/control guarantees.
- If the only qualified visible-control channel disappears, becomes unobservable or loses its immediate controls, trusted state suspends new autonomous actuation fail closed.
- Renderer reload/crash cannot leave hidden autonomous actuation running solely because stale frontend state claimed the indicator was visible.
- Restoring a qualified channel does not restore stale capability/control-lease/effect authority; ordinary fresh revalidation still applies.

## Allowed frontend data
- sanitized work-surface labels and bounded geometry;
- opaque observation/action/control-state references;
- sanitized capability, permission, visible-control-channel, stale/unsupported, pause/takeover and terminal-status states;
- bounded untrusted pixel-hint visualization metadata where needed for user review;
- explicit user action choices and approval prompts.

## Forbidden frontend data/authority
- raw OS handles/pointers/file descriptors;
- accessibility object references that can be invoked directly;
- capture session handles;
- OS access tokens/secrets;
- `golamd` local-client authentication/enrollment material;
- capability/lease tokens or privileged control-lease mutation authority;
- unrestricted adapter commands;
- generic shell/process/network execution;
- authority inferred from a hidden DOM/frontend flag.

Tauri capabilities/permissions must enumerate the minimum commands each window/webview needs. Rust revalidates every authority-bearing command independently of frontend state. Human pause/stop/takeover must reach the protected Rust control-lease/input-authority path; a renderer-only pause flag is never sufficient.
