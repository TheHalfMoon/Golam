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

DNS resolution, redirects and rebinding/private-target changes are part of the authorization boundary and may require revalidation/deny.

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

## Verification

Tests cover strict-local dominance, forbidden destination, DNS/redirect/rebinding cases, cleared environment, forbidden FS/network/device/spawn rights, resource bounds and unsupported-platform denial. External sinkhole/no-egress evidence remains required.
