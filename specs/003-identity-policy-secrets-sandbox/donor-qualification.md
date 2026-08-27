# Donor Qualification — Spec 003

**Status**: PLANNING QUALIFICATION; NO DONOR CODE OR PRODUCT DEPENDENCY ADMITTED

## DQ-001 — Golam-Research / Grok Bot 0.18 reconstruction

- repository: `TheHalfMoon/Golam-research`
- commit: `a9f633e09d49a85829b8236331b9e21f7e612634`
- tree: `b68f24972427952c4934e4364736fec62661044f`
- permission posture: founder-attested eligibility under constitution v1.2.0, subject to exact Source Foundry admission before reuse
- Spec 003 classification: `REFERENCE_ONLY`

Reason:
- repository is primarily reconstructed Electron/TypeScript product behavior;
- targeted searches did not identify a trusted Rust policy/capability/secret/sandbox authority substrate suitable for admission;
- Spec 003 security semantics are already frozen by Golam's constitution and Spec 001 contracts;
- copying reconstructed runtime security code would increase trust/supply-chain risk without improving the Rust privileged boundary.

Allowed planning use:
- historical interaction/UX evidence;
- terminology comparisons where independently validated.

Not admitted:
- source files;
- binaries/assets/installers;
- dependency graph;
- auth/secret/policy/sandbox implementation.

## DQ-002 — Cedar

- source: official Cedar project/documentation
- classification: `PRIMARY_DEPENDENCY_CANDIDATE / NOT_ADMITTED_BY_PLANNING`

Implementation admission requires:
- exact repository/crate/version and source hash;
- license/notice review;
- transitive dependency and unsafe/FFI review;
- default features/network/telemetry behavior;
- parser/evaluator resource bounds;
- schema/policy validation behavior;
- malformed/evaluation-error fail-closed tests;
- deterministic mapping into Golam decision semantics.

## DQ-003 — Wasmtime/WASI

- source: official Wasmtime project/documentation
- classification: `BOUNDED_WASM_SANDBOX_CANDIDATE / NOT_ADMITTED_BY_PLANNING`

Implementation admission is deferred until the WASM sandbox profile task actually needs it and requires exact version/license/dependency/unsafe/JIT/platform qualification. Wasmtime must not become the native-process sandbox or authority owner.

## DQ-004 — OS credential/key protection

Potential platform APIs/libraries for Windows/macOS/Linux remain `UNSELECTED_IMPLEMENTATION_DEPENDENCY`. Planning freezes the abstraction and failure behavior only. Exact backends must be independently qualified before any real secret vault key is entrusted to them.

## Admission decision

```text
GOLAM_RESEARCH_SPEC003=REFERENCE_ONLY
DONOR_CODE_ADMITTED=NO
CEDAR_DEPENDENCY_ADMITTED=NO
WASMTIME_DEPENDENCY_ADMITTED=NO
SECRET_BACKEND_ADMITTED=NO
NEXT_ADMISSION_GATE=BOUNDED_IMPLEMENTATION_TASK_WITH_EXACT_SOURCE_AND_DEPENDENCY_QUALIFICATION
```
