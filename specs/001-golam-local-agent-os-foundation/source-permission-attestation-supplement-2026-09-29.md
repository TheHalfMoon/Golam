# Founder Source Permission Attestation Supplement — 2026-09-29

**Recorded:** 2026-09-29  
**Scope:** DBX, Paperclip and Synaplan source families supplied for Golam planning  
**Status:** `FOUNDER_PERMISSION_ATTESTED`

## Attestation

The founder explicitly states that Golam may use, copy, modify, port and selectively reuse source code from:

```text
t8y2/dbx
paperclipai/paperclip
metadist/synaplan
```

The exact public repository states reviewed for this planning pass are:

```text
t8y2/dbx@4269a61e2cf6c19afdcaba41fed1e57d6e5e3512
paperclipai/paperclip@24beb005755465f71a19ec92a85da0958d1b9740
metadist/synaplan@e81eb3431deb3e242c3a114e8cbf08e2fbfd1e88
```

Observed public repository license posture at those exact revisions:

```text
t8y2/dbx                  Apache-2.0
paperclipai/paperclip     MIT
metadist/synaplan         Apache-2.0
```

The detailed technical disposition is recorded in:

- `source-adoption-dbx-paperclip-synaplan-2026-09-29.md`.

## What this permission enables

The permission statement makes exact bounded components eligible to enter the normal Golam Source Foundry lifecycle for one of these reuse modes:

```text
COPY_AS_IS
SELECTIVE_COPY
PORT_TO_RUST
ADAPTER
REIMPLEMENT_BEHAVIOR
BENCHMARK_OR_METHOD_REFERENCE
REJECT
```

The selected mode must minimize trusted-computing-base growth and preserve Golam's canonical Authority / Effect / Evidence / Task boundaries.

## What this permission does not do

This record does not automatically admit:

- an entire repository;
- transitive third-party dependencies;
- generated or vendored code;
- model weights, tokenizers, datasets or training artifacts;
- hosted services, cloud accounts or service terms;
- trademarks, branding or proprietary assets;
- secrets or credentials;
- database drivers that are not selected by the owning implementation spec;
- a donor's authority, task, policy, transaction or tenancy model.

Each copied/ported component must still record:

- exact source path and blob/commit identity;
- selected reuse mode;
- public license and continuing NOTICE/attribution obligations;
- founder permission evidence where separately relevant;
- dependency/runtime closure;
- generated/vendored-code provenance;
- network, telemetry, filesystem, process, secret and unsafe/FFI behavior;
- attack-surface / TCB delta;
- Golam authority/effect mapping;
- independent tests/benchmarks/parity evidence;
- rollback/removal path.

## Source-specific boundaries

### DBX

Apache-2.0 repository source may be considered for bounded reuse. The preferred reuse surface is pure Rust SQL/database capability logic and tests rather than wholesale adoption of all drivers or the complete DBX application.

DBX's two-phase-commit implementation is not granted authority to replace Golam's Effect FSM merely because its source is reusable.

### Paperclip

MIT repository source may be considered for bounded reuse. Paperclip's organization/company control plane, task database and runtime state are donor material only; Golam retains its own Task, identity, authority and Effect truth.

### Synaplan

Apache-2.0 repository source may be considered for bounded reuse. The preferred posture is to port graph validation, portability and readiness semantics into Golam's Rust contracts rather than introduce the Symfony/PHP application runtime as a product dependency.

## Hard rules

```text
FOUNDER_PERMISSION != TECHNICAL_ADMISSION
PUBLIC_LICENSE != WHOLE_REPOSITORY_ADMISSION
SOURCE_PERMISSION != HOSTED_SERVICE_RIGHTS
SOURCE_PERMISSION != TRANSITIVE_THIRD_PARTY_RIGHTS
SOURCE_PERMISSION != MODEL_OR_DATASET_CLEARANCE
COPY_ALLOWED != COPY_PREFERRED
DONOR_TASK_STATE != GOLAM_TASK_TRUTH
DONOR_POLICY != GOLAM_AUTHORITY
DONOR_TRANSACTION_STATE != GOLAM_EFFECT_TRUTH
```

## Admission state machine

```text
REFERENCE
-> VERIFIED_SOURCE_STATE
-> PERMISSION_RECORDED
-> RIGHTS_AND_OBLIGATIONS_RECONCILED
-> BOUNDED_COMPONENT_SELECTED
-> DEPENDENCY_RUNTIME_CLOSURE_RECORDED
-> AUTHORITY_EFFECT_MAPPING_DEFINED
-> TECHNICALLY_QUALIFIED
-> ADMITTED
```

No code from the three source families enters a trusted or product runtime path before the exact selected component reaches `ADMITTED` through an owning implementation spec.

```text
DBX_PERMISSION_ATTESTED_2026_09_29=YES
PAPERCLIP_PERMISSION_ATTESTED_2026_09_29=YES
SYNAPLAN_PERMISSION_ATTESTED_2026_09_29=YES
AUTOMATIC_CODE_ADMISSION=NO
AUTOMATIC_RUNTIME_ADMISSION=NO
AUTOMATIC_MODEL_ADMISSION=NO
WAIVER_TAKEN=NO
```
