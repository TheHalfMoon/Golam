# T003-005 — Wasmtime/WASI Disposition

**Decision**: `NOT_ADMITTED_NOT_NEEDED`

Spec 003 requires truthful sandbox profiles and a bounded native untrusted-process qualification path. It does not require an executable WASM extension path before Phase H.

Wasmtime/WASI remains a valid candidate for the optional `pure WASM/WASI extension` sandbox profile described by the frozen architecture, but admitting it now would add a large runtime/JIT/unsafe/platform dependency surface without satisfying an immediate predecessor task.

Rules:

- no Wasmtime crate is added during Phase A/B/C/D/E/F/G;
- no plan or API may assume Wasmtime is available;
- `SandboxProfile` remains executor-neutral;
- if T003-075 later needs a concrete WASM executor, T003-005 is reopened and exact version/features/license/unsafe/JIT/hostcall/resource behavior must be qualified before dependency admission;
- Wasmtime must never be described as a universal sandbox for arbitrary native processes.

```text
T003_005=PASS
WASMTIME_ADMITTED=NO
WASMTIME_DISPOSITION=NOT_ADMITTED_NOT_NEEDED
REQUALIFY_IF_T003_075_REQUIRES_EXECUTOR=YES
```
