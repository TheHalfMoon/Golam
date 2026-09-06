# Desktop UI RPC Contract

The Tauri webview/frontend is an untrusted presentation tier.

## Allowed frontend data
- sanitized work-surface labels and bounded geometry;
- opaque observation/action references;
- sanitized capability, permission, stale/unsupported and terminal-status states;
- explicit user action choices and approval prompts.

## Forbidden frontend data/authority
- raw OS handles/pointers/file descriptors;
- accessibility object references that can be invoked directly;
- capture session handles;
- OS access tokens/secrets;
- unrestricted adapter commands;
- generic shell/process/network execution;
- authority inferred from a hidden DOM/frontend flag.

Tauri capabilities/permissions must enumerate the minimum commands each window/webview needs. Rust revalidates every authority-bearing command independently of frontend state.
