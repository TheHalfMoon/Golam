# Contract — Egress Authorization & Sandbox Admission

Authorization and containment are distinct. A process needs authority to request work and an enforceable sandbox/launch profile to bound execution.

## Strict-local egress

The existing kernel strict-local external-network denial executes before policy, lease, approval or egress permit evaluation. In strict-local mode no external permit can make external networking available.

Loopback is separately classified/scoped and cannot be used as an unauthenticated Golam control path.

## Non-strict egress permit

A permit binds:
- principal/process;
- action/purpose;
- destination/protocol/port scope;
- time/usage bounds;
- taint/provenance;
- optional secret handle;
- parent lease/decision.

Hostname authorization never authorizes an arbitrary resolved address or later redirect target. DNS resolution is part of the protected execution boundary: before socket creation, every effective resolved endpoint must be inside the already-authorized destination scope or receive a fresh authorization decision. Redirects, rebinding, resolution changes, protocol/port changes, and transitions to private/link-local/loopback targets require mandatory revalidation before following/connecting; if the new effective destination is not explicitly authorized, execution denies. A prior hostname permit cannot be reused as authority for a changed effective destination.

## Sandbox profile

Profile declares:
- filesystem read/write roots;
- network class/permit requirement;
- environment allowlist;
- process spawning/child policy;
- CPU/memory/time/output bounds;
- device access;
- IPC endpoints;
- inherited capability/secret handles;
- required platform executor/controls.

Environment begins cleared. Profile rights cannot exceed the active lease/policy/egress decision.

## Profile classes

Reserve at least:
- pure WASM/WASI extension;
- native untrusted subprocess;
- MCP server;
- skill helper;
- browser/protocol helper;
- local model sidecar.

Later product integrations may instantiate these profiles in later specs; Spec 003 establishes the protected profile/admission machinery.

## Executor honesty

A profile is not containment proof. Admission must resolve all required controls to a supported executor before launch. Unsupported required enforcement denies. Wasmtime/WASI may be used for portable bounded extensions after dependency qualification; it is not a universal native sandbox.

Before Spec 003 launches any Golam-managed child process with network capability, the external strict-local qualification observer must be upgraded from daemon-PID-only observation to cover the complete Golam-managed process tree, or use an equivalent sinkholed/network boundary that captures descendants independently of PID ownership. A descendant socket is a Golam-managed egress attempt and must fail qualification exactly like a daemon socket.

## Verification

Tests cover strict-local dominance, forbidden destination, mandatory DNS/redirect/rebinding/private-target reauthorization, changed-endpoint denial, cleared environment, forbidden FS/network/device/spawn rights, resource bounds and unsupported-platform denial. External sinkhole/no-egress evidence must cover every Golam-managed process, including descendants.
