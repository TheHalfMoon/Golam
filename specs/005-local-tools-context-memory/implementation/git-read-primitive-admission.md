# T005-040 — Minimal Git Read Primitive Admission

**Status**: `ADMITTED_FOR_BOUNDED_T005_040_IMPLEMENTATION`

**Task**: T005-040 bounded Git read evidence

**Qualified candidate head**: `e10914d250398a428cc33dd4021d843784b5d5bb`

**Exact-head CI**: CI #873 / run `33644192574` — `SUCCESS` on Windows, macOS and Ubuntu

**Independent Source Foundry review**: CodeRabbit substantive semantic/security review on the unchanged qualified head returned `ADMIT_MINIMAL_PRIMITIVES` with no material blocker.

## Admission

The following exact packages are admitted solely as bounded decompression/checksum primitives for the first Golam-owned T005-040 Git-read parser profile:

```toml
miniz_oxide = { version = "=0.9.1", default-features = false }
adler2 = { version = "=2.0.1", default-features = false }
```

Exact immutable registry/source bindings:

```text
MINIZ_OXIDE_VERSION=0.9.1
MINIZ_OXIDE_REGISTRY_CHECKSUM=b63fbc4a50860e98e7b2aa7804ded1db5cbc3aff9193adaff57a6931bf7c4b4c
MINIZ_OXIDE_PUBLISHED_VCS_SHA=4e582392df3a739d2b0dfd2c537dc33e8942be38
MINIZ_OXIDE_PUBLISHED_VCS_PATH=miniz_oxide
MINIZ_OXIDE_LICENSE=MIT OR Zlib OR Apache-2.0

ADLER2_VERSION=2.0.1
ADLER2_REGISTRY_CHECKSUM=320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa
ADLER2_PUBLISHED_VCS_SHA=89a031a0f42eeff31c70dc598b398cbf31f1680f
ADLER2_LICENSE=0BSD OR MIT OR Apache-2.0
```

The direct exact `adler2` pin is mandatory even though it is the selected `miniz_oxide` dependency. This binds the reviewed version instead of allowing drift within the upstream `2.0` requirement. Exact selected features and the generated lockfile remain subject to post-mutation inspection.

## Exact admitted boundary

The admitted dependency intent is computation over caller-provided byte slices only. It does not admit any Git repository abstraction or any additional authority.

Selected `miniz_oxide` feature posture:

```text
DEFAULT_FEATURES=OFF
WITH_ALLOC=OFF
STD=OFF
BLOCK_BOUNDARY=OFF
SIMD=OFF
SERDE=OFF
RUSTC_DEP_OF_STD=OFF
DEFLATE_COMPRESSION_PATH=NOT_COMPILED
```

Selected external closure is required to remain exactly:

```text
miniz_oxide 0.9.1
└── adler2 2.0.1
```

Any additional selected runtime package or feature invalidates this admission and requires a new Source Foundry disposition before parser implementation may rely on it.

The reviewed selected packages contain no admitted Cargo build script, native source, FFI, process launch, network client, telemetry, credential helper, Git helper, or filesystem-mutation authority. Both reviewed crate roots forbid unsafe Rust. Optional/development/upstream-workspace surfaces are not admitted merely because they exist outside the selected package closure.

## Mandatory decompression call gate

Every `miniz_oxide` inflate call used by Golam MUST route through the already-qualified repository-owned `git_read_budget::DecompressionDeadline` boundary.

```text
DECOMPRESSION_INPUT_QUANTUM_BYTES=65536
DECOMPRESSION_OUTPUT_QUANTUM_BYTES=65536
DEADLINE_CLOCK=MONOTONIC_INSTANT
DEADLINE_CHECK=IMMEDIATELY_BEFORE_EVERY_INFLATE_CALL
DEADLINE_CHECK=IMMEDIATELY_AFTER_EVERY_INFLATE_CALL
IN_FLIGHT_SYNCHRONOUS_CALL=NON_PREEMPTIVE
POST_CALL_DEADLINE_OVERRUN=FAIL_CLOSED_DISCARD_STEP_RESULT
EXPIRED_BETWEEN_CHUNKS=REJECT_NEXT_CALL_BEFORE_INFLATE
TOTAL_COMPRESSED_BYTES=FROZEN_PROFILE_CAP
TOTAL_DECOMPRESSED_BYTES=FROZEN_PROFILE_CAP
ALLOCATING_DECOMPRESS_TO_VEC_HELPERS=DENIED
```

The qualified head contains focused tests proving oversized work quanta are rejected before invocation, expiry between chunks rejects the next invocation, and an in-flight non-preemptive overrun is rejected immediately after return. The external adapter must preserve this exact call gate; any direct inflate path bypassing it is outside admission.

## SHA-1 boundary

No external SHA-1 package is admitted. T005-040 uses a Golam-owned safe-Rust SHA-1 implementation only for legacy Git object identity.

```text
EXTERNAL_SHA1_DEPENDENCY=NO
SHA1_USE=LEGACY_GIT_OBJECT_IDENTITY_ONLY
SHA1_AUTHORIZATION_SECURITY_PRIMITIVE=NO
SHA1_CAPABILITY_PRIMITIVE=NO
SHA1_AUTHORITY_JOURNAL_INTEGRITY_PRIMITIVE=NO
SHA1_SIGNATURE_PRIMITIVE=NO
SHA1_NEW_SECURITY_COLLISION_RESISTANCE=NO
```

## Platform and authority posture

This admission does not change filesystem-platform qualification. Windows content read and in-process text search remain explicitly unsupported/fail-closed until a separate Windows opened-handle identity/no-recall primitive is independently admitted. The primitive admission does not re-enable that denied surface.

```text
PROCESS_LAUNCH=NONE
SHELL=NONE
NETWORK=NONE
DNS=NONE
TELEMETRY=NONE
CREDENTIAL_ACCESS=NONE
GIT_CHILD_PROCESS=DENIED
GIT_REMOTE=DENIED
GIT_HOOK=DENIED
GIT_FILTER=DENIED
GIT_TEXTCONV=DENIED
GIT_MUTATION=DENIED
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
WINDOWS_CONTENT_READ=UNSUPPORTED_FAIL_CLOSED
WINDOWS_IN_PROCESS_TEXT_SEARCH=UNSUPPORTED_FAIL_CLOSED
```

Primitive output is non-authoritative evidence. It cannot mint capability material, approval, authorization, verification authority, taint clearing, Effect Gate state, or protected-state mutation authority.

## Post-admission qualification obligations

Admission authorizes only the exact Cargo mutation and bounded implementation required by T005-040. It does not pre-qualify the resulting dependency graph or parser code.

After Cargo mutation and again after parser mutation, the implementation MUST obtain:

1. exact generated lockfile inspection proving the selected runtime closure is only `miniz_oxide 0.9.1 -> adler2 2.0.1`;
2. selected-feature inspection proving no excluded feature is unified back in;
3. `cargo fmt --check`, Clippy with warnings denied, focused tests and full repository tests;
4. official exact-head Windows/macOS/Ubuntu CI;
5. fresh substantive independent semantic/security review on the unchanged implementation head;
6. reconciliation of every material finding before any T005-040 completion claim.

## Decision

```text
T005_040_PRIMITIVE_SOURCE_FOUNDRY=QUALIFIED
PRIMITIVE_ADMISSION=ADMIT_MINIMAL_PRIMITIVES
ADMISSION_EVIDENCE_HEAD=e10914d250398a428cc33dd4021d843784b5d5bb
ADMISSION_EVIDENCE_CI_RUN=33644192574
MINIZ_OXIDE_0_9_1_ADMITTED=YES_BOUNDED_T005_040_ONLY
ADLER2_2_0_1_ADMITTED=YES_BOUNDED_T005_040_ONLY
CARGO_MUTATION_AUTHORIZED=YES_EXACT_PINS_ONLY
PARSER_IMPLEMENTATION_AUTHORIZED=AFTER_EXACT_LOCK_FEATURE_CLOSURE_QUALIFIES
NEW_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
NEXT_GATE=EXACT_CARGO_MUTATION_LOCK_FEATURE_CLOSURE_CI_AND_REVIEW
```
