---
task_scope: T006-016..T006-033 Linux async runtime direct-use precondition
status: CANDIDATE_REQUALIFICATION_PENDING_INDEPENDENT_REVIEW
recorded_on: 2026-09-08
product_manifest_mutation: false
product_runtime_admission: false
waiver_taken: false
---

# Source Foundry candidate — Spec 006 Linux bounded async driver

## Purpose

The already admitted Linux native dependency set uses asynchronous AT-SPI and XDG portal APIs. The admitted closure already resolves `tokio 1.53.1` transitively with a broad feature union. Transitive presence is not direct implementation authority, and a direct `tokio` dependency on `golamd` would expose a materially broader API surface than the bounded Spec 006 driver requires.

The first direct-root candidate was therefore **not admitted** by independent review on exact head `77f25ae618714bd5f2fb6d8412fc87db613bae1a` (PR #24 comment `5587556390`). The review verified the closure/checksum/runtime probe but correctly found:

```text
RESOLVED_TOKIO_FEATURES_INCLUDE_FS_IO_NET_PROCESS_RT_MULTI_THREAD_AND_SIGNAL_SUPPORT=YES
DIRECT_PRODUCT_TOKIO_API_SURFACE_STRUCTURALLY_RESTRICTED_TO_BOUNDED_DRIVER=NO
ADMIT_SPEC006_LINUX_BOUNDED_ASYNC_DRIVER=NO
PRODUCT_MANIFEST_LOCK_MUTATION=NOT_AUTHORIZED
```

No denied admission is reclassified or erased. Product manifests remain unchanged by this record.

## Revised structural design

The candidate now requires an internal bounded facade crate, planned as `golam-bounded-linux-async`, to be the **only Golam product crate with a direct Tokio dependency**:

```toml
tokio = { version = "=1.53.1", default-features = false, features = ["rt", "time"] }
```

`golamd` must depend only on the internal facade crate. It must **not** declare Tokio directly. Cargo therefore does not pass Tokio as a named direct dependency to the `golamd` compilation unit, so `golamd` source cannot resolve `tokio::*` merely because Tokio exists transitively in the lock graph.

The facade public API is intentionally synchronous and bounded:

- accept one caller-provided future plus an explicit positive timeout;
- cap the timeout at 30 seconds;
- create one current-thread runtime for the bounded call;
- enable timer support only through the facade implementation;
- execute the future to completion or return a typed timeout/runtime-construction error;
- expose no Tokio `Runtime`, `Handle`, task/spawn API, networking API, process API, filesystem API, signal API, synchronization primitive, or Tokio type/re-export.

The facade is an execution primitive only. It owns no desktop authority, target selection, permission state, route eligibility, control lease, visible-channel state, reconciliation decision, or platform result interpretation.

## Exact external package identity

Tokio remains exactly:

```text
TOKIO_DIRECT_VERSION=1.53.1
TOKIO_LOCK_CHECKSUM=202caea871b69668250d242070849eb495be178ed697a3e98aebce5bc81a0bed
TOKIO_LICENSE_EVIDENCE_PATH=tokio-1.53.1/LICENSE
TOKIO_LICENSE_EVIDENCE_BYTES=1070
TOKIO_LICENSE_EVIDENCE_SHA256=253cd04c6714889df2d32f3f64d669179a1c95c76ac43c40882c52eb06bc3552
```

The successful closure-control run on exact head `77f25ae618714bd5f2fb6d8412fc87db613bae1a` proved:

```text
BASELINE_EXTERNAL_PACKAGE_COUNT=177
CANDIDATE_EXTERNAL_PACKAGE_COUNT=177
BASELINE_EXTERNAL_LOCK_RECORD_COUNT=177
CANDIDATE_EXTERNAL_LOCK_RECORD_COUNT=177
DIRECT_RUNTIME_PACKAGE_CLOSURE_CHANGED=NO
DIRECT_RUNTIME_EXTERNAL_LOCK_PACKAGE_CLOSURE_CHANGED=NO
TOKIO_DIRECT_FEATURE_DELTA=[]
```

`TOKIO_DIRECT_FEATURE_DELTA=[]` means the proposed direct edge does not expand the resolved Tokio feature union. It does **not** mean the resolved Tokio build lacks broader features; the already admitted transitive graph enables additional Tokio features. The structural facade is therefore required even though the external closure is unchanged.

## Qualification gates

Two complementary workflows are evidence, not authority.

### Closure and archive control

`.github/workflows/spec006-linux-runtime-source-foundry.yml` compares the admitted Linux dependency baseline with the same external graph plus the exact Tokio declaration. It proves external package/lock identity, checksum/license evidence, the current-thread runtime/timeout primitive, and no product-manifest mutation. Its scratch direct-root topology is a conservative closure-delta control only; it is **not** the final product topology after the independent review rejected direct Tokio ownership by `golamd`.

### Structural API-surface control

`.github/workflows/spec006-linux-runtime-facade-source-foundry.yml` models the revised final dependency direction with two isolated crates:

1. a facade crate whose sole direct external runtime dependency is exact Tokio `1.53.1` with requested `rt,time` features;
2. a consumer crate that depends on the facade and admitted AT-SPI but does not depend directly on Tokio.

The gate must prove all of:

```text
FACADE_DIRECT_TOKIO_OWNER=YES
FACADE_PUBLIC_TOKIO_REEXPORT=NO
FACADE_ALLOWED_TOKIO_REFERENCES=NEW_CURRENT_THREAD_AND_TIMEOUT_ONLY
CONSUMER_DIRECT_TOKIO_DEPENDENCY=NO
CONSUMER_TOKIO_NAME_RESOLUTION=COMPILE_FAIL
DIRECT_PRODUCT_TOKIO_API_SURFACE_STRUCTURALLY_RESTRICTED_TO_BOUNDED_DRIVER=YES
PRODUCT_MANIFEST_MUTATION=NO
```

The compile-fail control intentionally attempts to use `tokio::runtime::Builder::new_multi_thread()` from the consumer. Qualification succeeds only when Cargo/Rust rejects that reference because the consumer has no direct Tokio dependency.

## Proposed product mutation after admission

If and only if exact-head Source Foundry workflows, exact-head regular CI, and a fresh independent review all succeed on one unchanged candidate head, the authorized minimum product mutation is:

1. add one internal workspace crate `golam-bounded-linux-async` implementing the reviewed facade shape;
2. give that crate the exact direct Tokio declaration above on the Linux Spec 006 path;
3. make `golamd` depend on the internal facade crate rather than on Tokio;
4. update the Cargo lock only for the Cargo-resolved internal package/dependency-edge topology;
5. add repository qualification that fails if `golamd` or another Spec 006 consumer declares Tokio directly, resolves `tokio::*` in consumer source, or the facade gains a denied Tokio API/re-export;
6. use the facade only for bounded already-admitted Linux AT-SPI/portal futures.

This admission would not pre-approve any native observation, focus, action, capture, raw-input, clipboard, portal, or reconciliation implementation.

## Immutable authority constraints

The facade/Tokio combination cannot:

- mint capability, policy, approval, Kernel/Effect Gate authorization, fallback eligibility, control-lease authority, or visible-channel qualification;
- turn semantic text, coordinates, screenshots, clipboard data, pixel hints, runtime completion, or task scheduling into action authority;
- create hidden network/cloud fallback or independent egress authority;
- expose Tokio networking/process/filesystem/signal/task APIs to `golamd` or adapter consumers;
- create an unbounded/multi-thread background runtime surface for Spec 006;
- continue autonomous actuation after pause, stop, takeover, permission/session drift, qualified visible-channel loss, or unresolved `UNKNOWN_OUTCOME`;
- authorize background polling, keylogging, clipboard monitoring, camera/microphone access, OCR, secure-desktop bypass, or Wayland bypass;
- widen Spec 006 into workers, automations, browser/application semantic control, GolamConnect, or another future program unit.

Any future Tokio role outside the bounded internal Linux async facade requires separate canonical authority and qualification.

## Required independent review disposition

After **both** Source Foundry workflows and regular exact-head CI succeed on the same unchanged candidate head, request a fresh substantive independent Source Foundry semantic/security/supply-chain review.

Admission requires an explicit disposition equivalent to:

```text
ADMIT_SPEC006_LINUX_BOUNDED_ASYNC_DRIVER=YES
TOKIO_DIRECT_VERSION=1.53.1
TOKIO_DIRECT_OWNER=GOLAM_BOUNDED_LINUX_ASYNC_FACADE_ONLY
GOLAMD_DIRECT_TOKIO_DEPENDENCY=NO
FACADE_PUBLIC_TOKIO_REEXPORT=NO
FACADE_ALLOWED_TOKIO_REFERENCES=NEW_CURRENT_THREAD_AND_TIMEOUT_ONLY
CONSUMER_TOKIO_NAME_RESOLUTION=COMPILE_FAIL
DIRECT_PRODUCT_TOKIO_API_SURFACE_STRUCTURALLY_RESTRICTED_TO_BOUNDED_DRIVER=YES
DIRECT_RUNTIME_PACKAGE_CLOSURE_CHANGED=NO
DIRECT_RUNTIME_EXTERNAL_LOCK_PACKAGE_CLOSURE_CHANGED=NO
PRODUCT_MANIFEST_LOCK_MUTATION=AUTHORIZED_MINIMUM_FACADE_TOPOLOGY_ONLY
T006_016_T006_033_IMPLEMENTATION_CORRECTNESS=NOT_PREAPPROVED
FINAL_SPEC006_REVIEW=NOT_PREAPPROVED
WAIVER_TAKEN=NO
```

Until that exact-head review exists:

```text
SOURCE_FOUNDRY_ADMISSION=NO
PRODUCT_RUNTIME_ADMISSION=NO
PRODUCT_MANIFEST_MUTATION=BLOCKED
```
