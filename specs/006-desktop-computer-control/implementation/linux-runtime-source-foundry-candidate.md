---
task_scope: T006-016..T006-033 Linux async runtime direct-use precondition
status: CANDIDATE_PENDING_ISOLATED_QUALIFICATION_AND_INDEPENDENT_REVIEW
recorded_on: 2026-09-08
product_manifest_mutation: false
product_runtime_admission: false
waiver_taken: false
---

# Source Foundry candidate — Spec 006 Linux bounded async driver

## Purpose

The already admitted Linux native dependency set uses asynchronous AT-SPI and XDG portal APIs. The product manifest currently enables `atspi 0.30.0` and `ashpd 0.13.13` with their `tokio` integration features, while `tokio` is only a transitive package in the admitted Cargo lock. Transitive presence is not direct dependency authority.

This candidate qualifies one exact direct runtime use before any product manifest mutation:

```toml
tokio = { version = "=1.53.1", default-features = false, features = ["rt", "time"] }
```

The direct role is limited to a bounded current-thread Linux async driver for already admitted AT-SPI/portal futures. It does not authorize a general application runtime, background worker system, network client, hidden service, authority-bearing scheduler, or any future-spec behavior.

## Why direct runtime use is required

`atspi::AccessibilityConnection::new()` and the AT-SPI proxy traversal surface are asynchronous. Spec 006 requires bounded Linux semantic observation and later portal operations while repository policy forbids treating a transitive crate as direct implementation authority. The safe direct path therefore requires a separately reviewed runtime dependency rather than importing transitive `tokio` implicitly or adding an unqualified helper executor.

## Isolation gate

`.github/workflows/spec006-linux-runtime-source-foundry.yml` creates two isolated Linux scratch crates:

1. the exact already admitted Linux native dependency set;
2. the same dependency set plus the exact direct Tokio declaration above.

The gate requires:

- Rust `1.98.0`;
- exact `tokio 1.53.1` checksum `202caea871b69668250d242070849eb495be178ed697a3e98aebce5bc81a0bed` from the generated lock and downloaded crate archive;
- identical resolved external package closure between baseline and candidate;
- identical external Cargo lock package records, including versions, sources, checksums, and dependency edges;
- no local scratch package field change except one added direct Tokio dependency edge;
- no Tokio feature delta beyond explicitly requested `rt` and `time` features;
- successful `#![forbid(unsafe_code)]` caller compilation;
- a successful bounded current-thread runtime/timeout probe;
- successful construction and dropping of the admitted AT-SPI connection future without transitive-import tricks;
- checksum-bound `.crate` archive identity and embedded license/notice evidence;
- no product-manifest mutation.

The full scratch lock bytes are expected to differ because Cargo records the candidate's new direct dependency edge on the local scratch package. That byte difference is not admitted as an unconstrained lock change: the workflow must prove that every external lock package record is unchanged and that the local package delta is exactly one direct Tokio edge and nothing else.

Workflow success is qualification evidence only. It is not admission.

## Evidence interpretation

A successful isolation run must report the exact baseline and candidate lock sizes and SHA-256 hashes, plus all of:

```text
DIRECT_RUNTIME_PACKAGE_CLOSURE_CHANGED=NO
DIRECT_RUNTIME_EXTERNAL_LOCK_PACKAGE_CLOSURE_CHANGED=NO
DIRECT_RUNTIME_LOCAL_ROOT_OTHER_FIELDS_CHANGED=NO
DIRECT_RUNTIME_LOCAL_ROOT_DEPENDENCY_ADDED=tokio...
DIRECT_RUNTIME_FULL_LOCK_BYTES_CHANGED=YES_EXPECTED_LOCAL_ROOT_DIRECT_EDGE_ONLY
```

This replaces the earlier invalid full-lock-byte-identity assumption exposed by the first-attempt evidence. No failed attempt is reclassified or erased.

## Immutable authority constraints

Tokio remains an execution primitive only. It cannot:

- mint capability, policy, approval, Kernel/Effect Gate authorization, fallback eligibility, control-lease authority, or visible-channel qualification;
- turn semantic text, coordinates, screenshots, clipboard data, pixel hints, runtime completion, or task scheduling into action authority;
- create hidden network/cloud fallback or independent egress authority;
- continue autonomous actuation after pause, stop, takeover, permission/session drift, qualified visible-channel loss, or unresolved `UNKNOWN_OUTCOME`;
- authorize background polling, keylogging, clipboard monitoring, camera/microphone access, OCR, secure-desktop bypass, or Wayland bypass;
- widen Spec 006 into workers, automations, browser/application semantic control, GolamConnect, or another future program unit.

Any future Tokio role outside the bounded Linux native adapter driver requires separate canonical authority and qualification.

## Required independent review disposition

After the isolation workflow and regular exact-head CI both succeed on an unchanged candidate head, request a fresh substantive independent Source Foundry semantic/security/supply-chain review.

Admission requires an explicit disposition equivalent to:

```text
ADMIT_SPEC006_LINUX_BOUNDED_ASYNC_DRIVER=YES
TOKIO_DIRECT_VERSION=1.53.1
TOKIO_DIRECT_ROLE=BOUNDED_LINUX_ASYNC_DRIVER_ONLY
DIRECT_RUNTIME_PACKAGE_CLOSURE_CHANGED=NO
DIRECT_RUNTIME_EXTERNAL_LOCK_PACKAGE_CLOSURE_CHANGED=NO
DIRECT_RUNTIME_FULL_LOCK_BYTES_CHANGED=YES_EXPECTED_LOCAL_ROOT_DIRECT_EDGE_ONLY
DIRECT_RUNTIME_LOCAL_ROOT_DEPENDENCY_DELTA=TOKIO_DIRECT_EDGE_ONLY
PRODUCT_MANIFEST_MUTATION=AUTHORIZED_ONLY_FOR_EXACT_REVIEWED_TOKIO_DECLARATION
T006_016_T006_033_IMPLEMENTATION_CORRECTNESS=NOT_PREAPPROVED
FINAL_SPEC006_REVIEW=NOT_PREAPPROVED
WAIVER_TAKEN=NO
```

Until that review exists on the exact unchanged qualified head:

```text
SOURCE_FOUNDRY_ADMISSION=NO
PRODUCT_RUNTIME_ADMISSION=NO
PRODUCT_MANIFEST_MUTATION=BLOCKED
```
