# T004-075..084 — mistral.rs Exact Source Foundry Qualification

**Decision**: `REJECT`  
**Candidate**: `mistral.rs v0.9.0`  
**Upstream**: `EricLBuehler/mistral.rs`  
**Exact tag**: `v0.9.0`  
**Exact tag commit**: `54957525b8b648108000af2004a4991cf08c0e69`  
**Workspace license**: MIT  
**Workspace MSRV**: Rust 1.94  
**Golam Rust**: Rust 1.98  
**Exact upstream lockfile blob**: `a5ab0e68b6de707c846422576f833c1297e8d91b`

This record qualifies the exact candidate before any Golam dependency, source reuse, runtime artifact, or model execution is admitted. No mistral.rs code or dependency is added by this decision.

## T004-075 — Exact candidate and smallest plausible surface

The exact public Rust SDK candidate is the `mistralrs` crate from tag `v0.9.0` at commit `54957525b8b648108000af2004a4991cf08c0e69`.

The smallest initially plausible Golam surface was:

```text
mistralrs 0.9.0
  -> mistralrs-core 0.9.0
  -> CPU/default-feature inference only
  -> local GGUF file loading only
  -> Golam-owned ModelBackend adapter
```

No CUDA, cuDNN, Metal, Flash Attention, Accelerate, MKL, NCCL, Python, MCP, agent, web-search, shell, Skills, code-execution, server, or model-download capability was requested by Golam.

The candidate does provide an exact local-GGUF example using `GgufModelBuilder` with a local directory, local GGUF filename, and local chat-template file. This establishes that a local-file loading mode exists conceptually. It does not by itself prove that the compiled dependency closure is authority-neutral or incapable of network/tool execution.

`T004_075=PASS`

## T004-076 — License, feature, transitive, build, native, device, and MSRV closure

The exact root workspace declares:

```text
version = 0.9.0
license = MIT
rust-version = 1.94
```

The exact `LICENSE` file is the MIT license with copyright `2024 Eric Buehler` and requires preservation of the copyright and permission notice in copies or substantial portions.

The `mistralrs` crate has no declared default feature set and exposes optional accelerator features including CUDA, cuDNN, Metal, Flash Attention, Accelerate, MKL, NCCL, and `code-execution`.

However, default-feature minimization does not produce a small isolated inference closure. `mistralrs` unconditionally depends on `mistralrs-core`, and the exact `mistralrs-core/Cargo.toml` unconditionally depends on, among other packages:

- `hf-hub`;
- `reqwest`;
- `mistralrs-vision`;
- `mistralrs-quant`;
- `mistralrs-audio`;
- `mistralrs-mcp` with `utoipa`;
- `mistralrs-code-exec`;
- `mistralrs-sandbox`;
- `scraper` / `html2text`;
- `tokio-tungstenite`;
- `interprocess`.

The root workspace pins Candle crates to Git revision `27f20fea993c81ea6d32ce44018f42b68466525e`, not solely to a crates.io release.

Device/native expansion is feature-sensitive:

- CUDA paths introduce `cudaforge`, CUDA/device compilation and optional paged-attention/quantization native closure;
- Metal paths introduce Objective-C/Metal bindings, Candle Metal kernels, and `mistralrs-metal-compile`;
- Flash Attention and NCCL further widen native/device-specific closure;
- `mistralrs-sandbox` uses Linux-specific `seccompiler`, `landlock`, and `nix` when built for Linux;
- the exact upstream lockfile is bound by blob SHA `a5ab0e68b6de707c846422576f833c1297e8d91b`.

Rust 1.94 is compatible with Golam's pinned Rust 1.98. MSRV therefore is not the rejection reason.

`T004_076=PASS`

## T004-077 — Network, model-download, telemetry, and strict-local posture

The exact dependency closure contains both `hf-hub` and HTTP/network clients even when Golam does not request an accelerator feature. The exact SDK documentation demonstrates remote model identifiers as a primary loading path, while the exact `gguf_locally` example demonstrates a local-file path.

Golam strict-local requires a stronger property than "a local path exists": the admitted runtime closure must make network/model-download widening impossible under the selected configuration and must remain externally observable under the existing Spec 003 no-egress gates.

For v0.9.0, network-capable dependencies remain inside the default CPU SDK closure and are not removed by a minimal Golam feature selection. Golam therefore cannot establish a narrow compile-time local-file-only capability boundary from the public crate feature graph.

No claim is made that local GGUF inference necessarily performs egress. The rejection is based on the inability to constrain the admitted in-process closure to the required narrow capability set before runtime.

No telemetry, update, or no-egress runtime PASS is fabricated. Golam did not execute this rejected candidate.

`T004_077=PASS_FAIL_CLOSED`

## T004-078 — Exclusion of backend-owned agent/tool authority surfaces

The exact v0.9.0 SDK source publicly exports or exposes:

- `Agent`, `AgentBuilder`, and agent configuration/events;
- `McpClient` and MCP server/client configuration;
- `CodeExecutionConfig`, code-execution permissions and approvals;
- `ShellConfig`, shell options, and shell Skill mounts;
- sandbox policy and network mode;
- web-search options and search callbacks;
- tool callbacks and agentic tool-call records.

The exact `mistralrs-core` manifest unconditionally depends on `mistralrs-mcp`, `mistralrs-code-exec`, and `mistralrs-sandbox`. The exact `mistralrs-code-exec` package describes Python code execution and depends on MCP/sandbox plus Tokio process support. The exact MCP package includes `reqwest`, WebSocket support, and full Tokio. The sandbox package is explicitly designed for subprocesses spawned by mistral.rs tools/code execution.

Because these packages remain in the core dependency closure without a Golam-selectable feature boundary, Golam cannot prove that the compiled in-process dependency surface excludes backend-owned MCP/code-execution/sandbox capabilities merely by declining to call their APIs.

This fails the Spec 004 requirement that backend-native tool/agent/MCP/shell/code-execution surfaces be excluded from the Golam adapter's admitted dependency surface, not merely left unused by convention.

`T004_078=PASS_REJECTION_EVIDENCE`

## T004-079 — Panic, crash, resource, unsafe/native, and accelerator boundary

An in-process `mistralrs` engine shares the `golamd` process failure domain. A panic, abort, allocator failure, native/device runtime failure, or fatal accelerator error can therefore affect daemon availability unless separately contained. Golam has not produced evidence that this exact v0.9.0 in-process closure converts every relevant failure into a bounded recoverable adapter error.

Optional accelerator paths materially expand native/build boundaries through CUDA/Candle device code, Metal/Objective-C bindings, Flash Attention, NCCL, and platform-specific compilation. Those paths would require separate platform-specific qualification before admission.

A supervised sidecar could provide a stronger crash/resource boundary in principle, but this exact Source Foundry pass does not admit a sidecar merely to rescue the candidate. Doing so would require a separately frozen executable/build identity, private authenticated transport, strict-local launch contract, resource supervision, cancellation/termination semantics, and external no-egress qualification. None of that evidence exists for mistral.rs v0.9.0 in this branch, so it is not treated as PASS.

No panic/crash/resource runtime test is claimed for the rejected candidate.

`T004_079=PASS_FAIL_CLOSED`

## T004-080 — Admission decision

Decision: `REJECT`.

Reasons are cumulative:

1. the smallest public in-process SDK closure still unconditionally carries network/HF, MCP, code-execution, and sandbox dependencies;
2. backend-owned agent/MCP/code-exec/shell/search surfaces cannot be excluded at the dependency-feature boundary required by Golam;
3. in-process admission would share the daemon failure domain without exact crash/resource qualification;
4. optional accelerator support materially widens native/build/device boundaries;
5. a sidecar remains conceptually possible but is not admitted without its own executable identity, authenticated transport, supervision, strict-local launch, and external no-egress evidence.

The decision is not a judgment that mistral.rs is unsafe in general. It is a bounded Golam Spec 004 admission decision for this exact revision and trust/runtime contract.

Any later mistral.rs revision may reopen qualification if it provides a materially narrower inference-only crate/feature closure or a separately qualifiable sidecar surface.

`T004_080=PASS_REJECT`

## T004-081..083 — Not executed after rejection

Because T004-080 returned `REJECT`:

- T004-081: **NOT APPLICABLE** — no dependency, adapter, or lockfile change is authorized;
- T004-082: **NOT APPLICABLE** — no mistral.rs production adapter is implemented;
- T004-083: **NOT APPLICABLE** — no rejected runtime path may be misrepresented as externally qualified no-egress production evidence.

## T004-084 — Preserve the backend contract

Golam retains the already-qualified generic `ModelBackend` contract and deterministic scripted backend. No mistral.rs-specific semantics are added to `golam-core`, `golam-kernel`, or `golam-effects`.

The next authorized path is the Phase I llama.cpp compatibility qualification. That path remains independently gated and does not inherit admission from this record.

`T004_084=PASS`

## Exact evidence index

```text
UPSTREAM_REPOSITORY=EricLBuehler/mistral.rs
UPSTREAM_TAG=v0.9.0
UPSTREAM_COMMIT=54957525b8b648108000af2004a4991cf08c0e69
UPSTREAM_LICENSE_FILE_SHA=4543526959bd11e24b2d66a8fbf29b7f0a44717f
UPSTREAM_ROOT_CARGO_TOML_SHA=6a2fd7ff3879b8f0d48f1a827698092bdc1a7575
UPSTREAM_MISTRALRS_CARGO_TOML_SHA=57c723e04f16c2ba07f2f3832f2e04bd8478f594
UPSTREAM_MISTRALRS_CORE_CARGO_TOML_SHA=991212fb6d1d7c1d0d32a422fc73e1cf9e9527f4
UPSTREAM_MISTRALRS_LIB_RS_SHA=d910475d47521afac177a34acadaaacef687a00d
UPSTREAM_LOCAL_GGUF_EXAMPLE_SHA=a5b1c0c631f90f669a2696249b8bab24931d9c6b
UPSTREAM_MCP_CARGO_TOML_SHA=eeb2a878dee2f34a624cca9a3290a1ea4e148a2e
UPSTREAM_CODE_EXEC_CARGO_TOML_SHA=61c71a8fa2cbe767e85aad78dfb7b31ad25d7115
UPSTREAM_SANDBOX_CARGO_TOML_SHA=bf6c441fe411044c366101b1a7884f5b7cc110be
UPSTREAM_QUANT_CARGO_TOML_SHA=48ff8c89d40fbd985d8668b831e7a5eb6339aeae
UPSTREAM_PAGED_ATTN_CARGO_TOML_SHA=f4198098f136755a26ed133d1529e16128367176
UPSTREAM_CARGO_LOCK_SHA=a5ab0e68b6de707c846422576f833c1297e8d91b
DECISION=REJECT
PRODUCTION_DEPENDENCY_ADDED=NO
RUNTIME_QUALIFICATION_CLAIMED=NO
WAIVER_TAKEN=NO
```
