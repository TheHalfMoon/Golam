# T004-085..092 — llama.cpp Compatibility Qualification

**Decision**: `DEFER`  
**Candidate**: `llama.cpp v0.3.0`  
**Upstream**: `ggml-org/llama.cpp`  
**Qualified observation date**: `2026-09-01`  
**Release-selection source**: upstream GitHub `releases/latest` API response observed on the qualification date  
**Annotated tag object**: `918bca552078be4b3437f93117f542ea39972f5f`  
**Exact release commit**: `c1d0e7a004015f23bc0233470b747b596f29b264`  
**License**: MIT  
**Production artifact added**: NO

This is a compatibility qualification only. No llama.cpp executable, library, model, build artifact, dependency, or runtime path is admitted by this record.

## T004-085 — Exact candidate and configuration

On `2026-09-01`, the upstream GitHub `releases/latest` API identified the non-draft, non-prerelease release `v0.3.0`, published `2026-08-25`, with target commit `c1d0e7a004015f23bc0233470b747b596f29b264`. The qualification is bound to that dated observation and exact release identity; it does not make a time-unbounded claim about what upstream will call latest in the future.

```text
qualified_at = 2026-09-01
release_selection_source = GitHub releases/latest API
release = v0.3.0
release_commit = c1d0e7a004015f23bc0233470b747b596f29b264
```

Golam's only acceptable architecture for this candidate is a supervised out-of-process sidecar. Direct C/C++ FFI inside `golamd` remains rejected by the Spec 004 plan.

A future minimal sidecar build would have to freeze an exact build configuration that disables unrelated standalone artifacts and network-expanding convenience surfaces. At minimum the build review must explicitly set and bind values for:

```text
LLAMA_BUILD_SERVER
LLAMA_BUILD_APP
LLAMA_BUILD_UI
LLAMA_USE_PREBUILT_UI
LLAMA_BUILD_EXAMPLES
LLAMA_BUILD_TESTS
LLAMA_SUBPROCESS
LLAMA_OPENSSL
GGML backend/device options
```

No executable hash exists in this branch because the candidate is deferred before build/runtime admission.

`T004_085=PASS`

## T004-086 — License, build, native, backend, and executable identity

The exact upstream `LICENSE` is MIT with copyright `2023-2026 The ggml authors` and requires preservation of the copyright and permission notice.

The exact root CMake project is C/C++ and release version 0.3.0. Standalone defaults materially exceed Golam's bounded sidecar need:

- shared libraries default ON on ordinary non-MinGW platforms;
- tools, tests, examples, server, and unified app default from standalone mode;
- embedded server UI defaults ON;
- prebuilt UI retrieval support defaults ON when UI is enabled;
- OpenSSL support defaults ON;
- subprocess support defaults ON on ordinary desktop platforms;
- ggml device/backend selection can expand into CUDA, Metal, SYCL, Vulkan, OpenCL, WebGPU and other native/device-specific paths.

The exact `llama-server` target links `server-context`, `llama-common`, multimodal code, UI, and `cpp-httplib`; the server source set also includes tool and MCP modules. Therefore a future build identity must include exact CMake cache/options, compiler/toolchain identity, ggml backend closure, and executable digest in addition to the source commit.

Because no build was authorized, this record does not fabricate an executable digest or resolved platform-native closure. Their absence is part of the defer decision.

`T004_086=PASS_FAIL_CLOSED`

## T004-087 — Authenticated/private local sidecar transport

Generic unauthenticated localhost control is not acceptable for Golam.

The exact v0.3.0 server supports:

- `--host HOST`, including a Unix-domain socket when the address ends in `.sock`;
- `--api-key` / `--api-key-file` authentication;
- ordinary loopback TCP with default host `127.0.0.1`.

A future Golam Unix transport could only be considered with all of the following:

1. a runtime-private Unix socket path under Golam's protected runtime directory;
2. restrictive same-user filesystem ownership/permissions established before use;
3. a fresh per-launch high-entropy API credential delivered through a protected mechanism;
4. no generic public TCP listener;
5. sidecar process identity and launch configuration bound to canonical evidence.

The upstream server documentation does not provide an equivalent Windows named-pipe/private-handle control surface. Loopback TCP plus an API key remains a generic localhost service and does not satisfy the Spec 004 rule by itself.

Golam will not invent an unqualified Windows proxy/wrapper merely to claim cross-platform admission.

`T004_087=PASS_DESIGN_ONLY`

## T004-088 — Strict-local launch policy

If a future predecessor authorizes native sidecar launch, the minimum launch contract must be fail-closed and evidence-bound:

```text
model source = exact local file path only
--model = exact authorized local GGUF path
--offline = required
--model-url = absent
--hf-repo = absent
--hf-file = absent
--docker-repo = absent
--rpc = absent
--agent = disabled
--mcp-servers-json = absent
--webui = disabled
static UI path = absent
subprocess-enabled server tools = disabled by build configuration
inherited environment = cleared except explicit allowlist
network = no external egress
filesystem = model/config read roots only plus bounded runtime IPC path
CPU/memory/time/output/device limits = explicit
executable digest + source/build identity = canonical evidence
```

The exact server exposes `--offline`, local `--model`, remote `--model-url`, Hugging Face repository/file options, Docker repository loading, and RPC options. Golam therefore must bind both the positive local-file flags and the absence of every remote-loading/RPC option.

`T004_088=PASS_DESIGN_ONLY`

## T004-089 — Process launch, cancellation, termination, crash, and restart gate

Canonical Spec 003 blocks this candidate before runtime qualification.

The current production native sandbox executor capability manifest is deliberately:

```text
executor_id = native:unqualified
supported containment controls = none
claims_platform_containment = false
launches_process = false
```

Spec 003 states that unsupported required containment denies before launch. A llama.cpp sidecar necessarily requires native process launch plus filesystem, environment, resource, device/IPC, and strict-local network controls. Those controls are not admitted by the current predecessor.

Therefore Golam cannot truthfully run or qualify llama-server process launch/cancel/termination/crash/restart under the required Spec 003 supervision in Spec 004. Attempting an unsandboxed launch would bypass a canonical evidence-dependent gate.

No runtime process test is claimed.

`T004_089=BLOCKED_BY_CANONICAL_SPEC003_NATIVE_EXECUTOR`

## T004-090 — Compatibility admission decision

Decision: `DEFER`.

The candidate is not rejected permanently because its source surface contains useful compatibility properties:

- exact local model-file loading;
- explicit offline mode;
- Unix-domain socket support;
- API-key authentication;
- an out-of-process executable boundary.

It is not admitted now because:

1. canonical Spec 003 does not admit native process containment/launch;
2. no exact executable/build digest exists;
3. no Windows private authenticated transport equivalent is qualified;
4. no externally observed strict-local sidecar run exists;
5. the default standalone/server build surface is broader than Golam requires and must be explicitly reduced and frozen before future admission.

A later specification may reopen this candidate only after the native executor prerequisite is genuinely admitted and a cross-platform private-control strategy is qualified.

`T004_090=PASS_DEFER`

## T004-091..092 — Not executed after defer

Because T004-090 returned `DEFER`:

- T004-091: **NOT APPLICABLE** — no sidecar adapter or runtime artifact is added;
- T004-092: **NOT APPLICABLE** — no deferred executable path may be represented as externally qualified no-egress/local-control evidence.

The already-qualified generic Golam `ModelBackend` and deterministic scripted backend remain the only implemented Spec 004 backend path.

## Exact evidence index

```text
QUALIFIED_AT=2026-09-01
RELEASE_SELECTION_SOURCE=UPSTREAM_GITHUB_RELEASES_LATEST_API
UPSTREAM_REPOSITORY=ggml-org/llama.cpp
UPSTREAM_RELEASE=v0.3.0
UPSTREAM_TAG_OBJECT=918bca552078be4b3437f93117f542ea39972f5f
UPSTREAM_COMMIT=c1d0e7a004015f23bc0233470b747b596f29b264
UPSTREAM_LICENSE_SHA=e7dca554bcb802f98408383a864404e3aa4eacca
UPSTREAM_ROOT_CMAKE_SHA=730d5561fda5fb5988e46da84292e26a436e3dd6
UPSTREAM_SERVER_CMAKE_SHA=280bd9e19dca30e48073d3071c421695a4cbff4c
DECISION=DEFER
EXECUTABLE_BUILT=NO
EXECUTABLE_DIGEST=NONE
RUNTIME_LAUNCH_CLAIMED=NO
STRICT_LOCAL_SIDECAR_RUNTIME_CLAIMED=NO
WAIVER_TAKEN=NO
```
