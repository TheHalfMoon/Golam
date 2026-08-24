# Contract: Strict-Local Egress Control

Strict-local is a mechanized enforcement mode, not a router preference.

## Choke point

All Golam-managed network creation MUST pass a single kernel-authorized egress decision before a socket-capable process receives network access. The authorization key includes principal/process, destination, purpose, locality mode, data labels, credential handle, and capability lease.

## Strict-local rules

- Default network capability is deny.
- Local inference/model sidecars must run without external network unless an explicit local-only loopback permission is required.
- Tools, MCP servers, skill scripts, browser helpers, model runtimes, telemetry, update checks, and optional adapters receive no egress in strict-local mode.
- Loopback access is scoped separately and cannot be used to reach an unauthenticated Golam control surface.
- A component that cannot operate under the required network profile fails clearly; it MUST NOT silently switch to cloud/network mode.
- Every denied or unexpected egress attempt is audited.

## Non-strict modes

Allowed egress is destination/action scoped and may carry taint/secret policy. DNS resolution and redirect/rebinding behavior are part of the policy boundary; a permitted hostname does not automatically permit arbitrary resolved/private targets.

## Verification gate

GolamBench runs strict-local scenarios in an externally observed/sinkholed network environment. Any unexpected egress from any Golam-managed process is a failure.