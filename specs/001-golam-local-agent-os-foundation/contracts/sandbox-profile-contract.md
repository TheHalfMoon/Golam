# Contract: Sandbox Profiles

Authorization and sandboxing are separate: the kernel authorizes; a sandbox limits the damage if an authorized or parsed component misbehaves.

## Profile classes

Golam MUST define explicit sandbox profiles for at least:

- pure WASM/WASI extension;
- native untrusted subprocess;
- MCP server process;
- skill script/helper process;
- browser/protocol helper;
- local model sidecar where applicable.

Each profile declares filesystem roots, writable paths, network destinations, environment variables, process spawning, CPU/memory/time/output limits, device access, IPC endpoints, and inherited capability handles.

## Required defaults

- MCP servers and skill scripts are bounded subprocesses; they do not inherit daemon secrets or ambient filesystem/network access.
- Results from MCP/plugin/skill-script processes are tainted as untrusted/plugin-derived regardless of transport success.
- Network is deny-by-default in strict-local mode and otherwise scoped to explicit egress leases.
- Environment starts cleared and receives only approved variables/handles.
- Process trees are supervised and cancellable.

Wasmtime/WASI is appropriate for portable bounded extensions but MUST NOT be treated as a universal sandbox for arbitrary native tools.

## Verification gate

Escape, resource-exhaustion, inherited-secret, forbidden-network, forbidden-filesystem, and child-process tests are required per implemented platform/profile.