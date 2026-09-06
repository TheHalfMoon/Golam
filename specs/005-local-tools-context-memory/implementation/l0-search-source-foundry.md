# T005-038 — In-Process L0 Search Source Foundry Disposition

**Decision**: `GOLAM_OWNED_IN_PROCESS_SEARCH`

**External source admission**: `NOT_APPLICABLE_NO_EXTERNAL_SOURCE`

## Exact implementation surface

The Phase D L0 text-search implementation is constrained to a new Golam-owned Rust module:

- target path: `crates/golamd/src/local_search.rs`;
- trusted-path language: Rust;
- implementation source: Golam canonical Spec 005 contracts and existing Golam-owned filesystem primitives;
- new third-party dependency: none;
- donor code copied or ported: none;
- vendored/generated donor code: none;
- external executable: none;
- process/shell launch: none;
- network/telemetry/secrets behavior: none;
- unsafe/FFI surface: none.

The implementation may use only repository dependencies already admitted before Spec 005 and the Rust standard library unless a later task creates a separate exact Source Foundry admission before changing this disposition.

## Why external admission is not required

The Golam Constitution requires a per-source Source Foundry admission before external source code is copied, ported, vendored, forked or made a direct dependency. T005-039 will do none of those things. This disposition follows the established repository precedent for an implementation slice that reuses no donor code: record the no-reuse decision and keep all external candidates non-admitted rather than inventing an admission record for Golam-owned code.

This decision does not admit or authorize any external search implementation.

## Explicit non-admissions

The planning candidate remains pinned only as research evidence:

```text
RIPGREP_SOURCE_STATE=BurntSushi/ripgrep@3fce3b5bb0236da2df6d99672afb8a719642eca7
RIPGREP_SOURCE_ADMITTED=NO
RIPGREP_CRATE_SURFACE_ADMITTED=NO
RIPGREP_BINARY_ADMITTED=NO
PHASE_D_EXTERNAL_SEARCH_EXECUTABLE=DENIED_NATIVE_UNQUALIFIED
EXTERNAL_SEARCH_RECONSIDERATION_DEPENDS_ON=T005-077
```

No source from Golam-Research, ripgrep, OpenClaw, Hermes Agent or another research candidate may be copied into T005-039 under this disposition.

## Required T005-039 boundaries

The Golam-owned search implementation must:

1. execute in-process only;
2. bind every filesystem observation through the authorized-root/target-resolution contract;
3. use bounded file reads and bounded directory traversal;
4. bind explicit file, match and byte limits plus a finite duration limit;
5. emit attributable match file/line/content provenance;
6. reject special files, alias/reparse boundaries, protected Golam resources and target-resolution failures;
7. perform no shell interpolation and create no child process;
8. perform no network access and introduce no egress capability;
9. preserve failure semantics without returning a misleading successful partial result after a contract violation;
10. remain non-authoritative: search ranking or match output cannot mint capability, approval or verification authority.

## Disposition

```text
T005_038=PASS
L0_SEARCH_IMPLEMENTATION=GOLAM_OWNED_IN_PROCESS
SOURCE_FOUNDRY_EXTERNAL_ADMISSION_REQUIRED=NO
DONOR_CODE_ADMITTED=NO
DONOR_FILES_COPIED=0
NEW_DEPENDENCIES_ADDED=0
EXTERNAL_EXECUTABLES_ADMITTED=0
PROCESS_AUTHORITY_CHANGED=NO
NETWORK_AUTHORITY_CHANGED=NO
WAIVER_TAKEN=NO
NEXT_TASK=T005-039
```
