# Contract: Model Backend Boundary

A `ModelBackend` is a replaceable unprivileged inference adapter. It is not Golam's harness, authority root, tool executor, memory system or Task verifier.

## Required conceptual interface

```text
probe(BackendProbeRequest) -> BackendProbeResult
load(ExecutionProfile, HardwareProfile?) -> LoadResult
stream(ModelRequest, CancellationHandle) -> Stream<ModelBackendEvent>
cancel(RequestAttemptId) -> CancelObservation   // when backend requires explicit cancel
unload() -> UnloadObservation
metrics() -> optional bounded BackendMetrics
```

Exact Rust trait signatures belong to implementation after planning merge.

## `probe`

May report only bounded local execution evidence:
- backend/build identity;
- supported local features/devices;
- local runtime availability;
- version/build hash;
- compatible profile constraints.

Probe MUST NOT silently contact a network service in strict-local mode.

## `load`

Load MUST:
- receive an exact immutable `ExecutionProfile`;
- reject incompatible locality/network/device/model requirements;
- report the actual selected backend/build/device/model evidence;
- never silently substitute an explicit-cloud route for local failure;
- preserve exact model/revision/template/profile requirements or report mismatch.

## `stream`

Input is a bounded `ModelRequest`. Output is backend event data only.

Backend events MUST NOT directly call tools, KernelApi mutation endpoints or Effect Gate handlers. Backend-native tool/agent features may be used only as formatting/inference mechanisms when explicitly qualified; execution remains Golam-owned.

## Locality

For `LOCAL` strict-local profiles:
- model artifacts must already be locally authorized/available;
- auto download/update/telemetry/web search/MCP/RPC/cloud fallback is disabled or unreachable;
- unexpected external egress remains denied by the Spec 003 hard gate and causes explicit backend failure;
- backend failure cannot widen locality class.

## In-process Rust backend

An admitted in-process backend MUST have:
- exact crate/source/version and Cargo feature closure;
- complete dependency/build/native boundary record;
- no hidden backend-owned authority/tool execution path;
- independently tested offline behavior;
- bounded panic/crash/resource behavior appropriate to `golamd`.

The current primary candidate is `mistral.rs`, but planning does not admit it.

## Native sidecar backend

A sidecar MUST:
- run as a supervised Golam-managed process under Spec 003 sandbox/egress constraints;
- use a private authenticated OS-local transport or an equivalently protected local channel;
- have bounded environment/inherited handles/stdout/stderr/resources;
- have executable/build identity bound to evidence;
- have explicit termination/cancellation/restart semantics;
- never expose an unauthenticated generic localhost control API;
- not inherit ambient secrets;
- use local-file/offline configuration for strict-local profiles.

The current compatibility candidate is `llama.cpp`, but planning does not admit it.

## `llama.cpp` minimum safety posture if later admitted

- out-of-process by default;
- no direct C/C++ FFI inside `golamd`;
- `--offline` or independently equivalent no-network configuration;
- local model path rather than URL/Hugging Face retrieval on strict-local routes;
- RPC disabled unless separately authorized/qualified in a later scope;
- no unauthenticated loopback server;
- launch configuration and executable hash recorded.

## `mistral.rs` minimum safety posture if later admitted

- select minimal crate/feature closure;
- local-file inference path independently proven offline;
- exclude agent loop, shell, code execution, MCP, web search, Skills and automatic model retrieval from Golam's adapter surface;
- accelerator/native features admitted independently rather than assuming all features are safe/portable;
- backend tool/schema capabilities may produce candidate syntax only.

## Errors

Backend errors normalize to bounded classes such as:
- unavailable/incompatible;
- model not found/local artifact missing;
- invalid model/template/config;
- resource exhaustion;
- transient runtime/transport failure;
- context overflow;
- cancelled;
- timeout;
- protocol violation;
- unexpected egress denied;
- process crash.

An unknown/unclassifiable error defaults to explicit failure, not permissive fallback.

## Security invariants

`BACKEND != AUTHORITY_ROOT`
`BACKEND_TOOL_FEATURE != TOOL_EXECUTION_AUTHORITY`
`LOCAL_PROCESS != AUTHENTICATED_CONTROL_CHANNEL`
`MODEL_DOWNLOAD_FEATURE != STRICT_LOCAL_PERMISSION`
`BACKEND_FAILURE != CLOUD_FALLBACK_PERMISSION`
