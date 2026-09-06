# T005-040 — Primitive Source Foundry Provenance Binding

**Status**: `CANDIDATE_PROVENANCE_BOUND_NOT_ADMITTED`

**Task**: T005-040 bounded Git read evidence

**Repository head before this record**: `1034115755b3ffa48dafa57fd07a9e8521d0a079`

**Exact-head repository CI before this record**: CI #833 / run `33586074062` — `SUCCESS`

## Purpose

This record corrects and narrows the primitive provenance used for the first Golam-owned Git read parser profile. It does not admit a dependency and does not authorize Cargo or implementation changes.

The first parser profile remains SHA-1-only, bounded, read-only, process-free, network-free and fail-closed as frozen in `git-read-format-profile.md`.

## `miniz_oxide 0.9.1` published-source provenance correction

The previously inspected repository master commit `e2214d401a59e91537838cc16eba82454044044f` is **not** the exact VCS source recorded by the published crates.io package.

The published `miniz_oxide 0.9.1` artifact is bound by both its registry checksum and `.cargo_vcs_info.json`:

```text
PUBLISHED_PACKAGE=miniz_oxide 0.9.1
REGISTRY_CHECKSUM=b63fbc4a50860e98e7b2aa7804ded1db5cbc3aff9193adaff57a6931bf7c4b4c
PUBLISHED_VCS_SHA=4e582392df3a739d2b0dfd2c537dc33e8942be38
PUBLISHED_VCS_PATH=miniz_oxide
PUBLISHED_VCS_TREE=52f48224c08d8f9afe004d09c5287747b0242c5f
PUBLISHED_VCS_COMMIT_SIGNATURE=UNSIGNED
```

The exact VCS commit is the 2026-03-13 version-bump commit for `0.9.1`. Its `miniz_oxide/Cargo.toml` changes the package version to `0.9.1`; therefore all source qualification for the published package MUST bind to `4e582392...`, not a later master head.

The unsigned upstream VCS commit is not itself a rejection because the immutable registry package/version/checksum is an independent provenance anchor. Admission MUST bind both anchors and MUST NOT represent a later master commit as equivalent.

```text
MINIZ_OXIDE_0_9_1_PREVIOUS_MASTER_EQUIVALENCE=REJECTED_INEXACT
MINIZ_OXIDE_0_9_1_REGISTRY_CHECKSUM=b63fbc4a50860e98e7b2aa7804ded1db5cbc3aff9193adaff57a6931bf7c4b4c
MINIZ_OXIDE_0_9_1_EXACT_VCS_SOURCE=4e582392df3a739d2b0dfd2c537dc33e8942be38
MINIZ_OXIDE_0_9_1_ADMITTED=NO
```

## Exact selected-feature candidate posture

For T005-040 the candidate posture is deliberately narrower than the crate default:

```text
miniz_oxide = { version = "=0.9.1", default-features = false }
```

Selected intent:

```text
DEFAULT_FEATURES=DISABLED
WITH_ALLOC=DISABLED
STD=DISABLED
SIMD=DISABLED
SERDE=DISABLED
RUSTC_DEP_OF_STD=DISABLED
BLOCK_BOUNDARY=DISABLED
```

The exact manifest still has one required normal dependency under this posture. The candidate records the exact reviewed release rather than the broader upstream semver range:

```text
adler2 = { version = "=2.0.1", default-features = false }
```

The published `miniz_oxide 0.9.1` lock resolves that requirement to:

```text
ADLER2_VERSION=2.0.1
ADLER2_REGISTRY_CHECKSUM=320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa
```

The published lock also lists optional/package-development dependencies because it describes the package workspace resolution. Those dependencies are **not** admitted merely because they appear in that lock. The eventual Golam Cargo closure must be generated from the exact selected feature set and independently inspected before admission.

## `adler2 2.0.1` exact published provenance

The published `adler2 2.0.1` package records:

```text
PUBLISHED_PACKAGE=adler2 2.0.1
PUBLISHED_VCS_SHA=89a031a0f42eeff31c70dc598b398cbf31f1680f
PUBLISHED_VCS_PATH=
REGISTRY_CHECKSUM=320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa
LICENSE=0BSD OR MIT OR Apache-2.0
REPOSITORY=https://github.com/oyvindln/adler2
```

The normalized published manifest has no required external runtime dependency in the ordinary library posture. Its `rustc-std-workspace-core` dependency is optional and exists only for internal Rust standard-library integration. The exact selected source declares `#![forbid(unsafe_code)]`, and the intended T005-040 posture keeps default `std` and rustc-internal feature paths disabled.

```text
ADLER2_2_0_1_EXACT_VCS_SOURCE=89a031a0f42eeff31c70dc598b398cbf31f1680f
ADLER2_2_0_1_SELECTED_EXTERNAL_RUNTIME_DEPENDENCIES=NONE
ADLER2_2_0_1_ADMITTED=NO
```

## Bounded decompression API posture

`miniz_oxide` exposes a synchronous streaming inflate API accepting caller-provided input and output slices and returning exact consumed/written progress. The call itself is non-preemptive: Golam cannot interrupt an in-flight `inflate` call. Therefore the time contract is defined at **bounded synchronous work quanta**, not as an unsupported claim of arbitrary in-call cancellation.

The adapter MUST use the repository-owned `git_read_budget` guard and MUST satisfy all of the following:

```text
DECOMPRESSION_INPUT_QUANTUM_BYTES=65536
DECOMPRESSION_OUTPUT_QUANTUM_BYTES=65536
DEADLINE_CLOCK=MONOTONIC_INSTANT
DEADLINE_CHECK=IMMEDIATELY_BEFORE_EVERY_INFLATE_CALL
DEADLINE_CHECK=IMMEDIATELY_AFTER_EVERY_INFLATE_CALL
IN_FLIGHT_SYNCHRONOUS_CALL=NON_PREEMPTIVE
POST_CALL_DEADLINE_OVERRUN=FAIL_CLOSED_DISCARD_STEP_RESULT
EXPIRED_BETWEEN_CHUNKS=REJECT_NEXT_CALL_BEFORE_INFLATE
TOTAL_DECOMPRESSED_BYTES=FROZEN_CAP_ENFORCED
TOTAL_COMPRESSED_BYTES=FROZEN_CAP_ENFORCED
UNBOUNDED_ALLOCATING_HELPERS=DENIED
UNTRUSTED_STATE_REUSE_WITH_MIN_RESET=DENIED
PROCESS_LAUNCH=NONE
NETWORK=NONE
REPOSITORY_MUTATION=NONE
```

The 64 KiB input and output slices bound the maximum work presented to one non-preemptive synchronous call. The operation-level deadline still uses the frozen `DEFAULT_GIT_READ_TIME_BUDGET_MS` / `MAX_GIT_READ_TIME_BUDGET_MS`; expiry is checked before and after every call. If a call crosses the deadline, its result is not trusted and no later chunk may run.

A Golam-owned qualification helper exists without any external dependency and must remain the single call gate when the actual primitive adapter is wired. Its tests prove:

1. input/output larger than one synchronous quantum are rejected before the callback runs;
2. deadline expiry between chunks rejects the next callback before invocation;
3. an in-flight non-preemptive call that crosses the deadline is rejected immediately after return.

This proof does **not** admit `miniz_oxide`; it proves the caller-side policy required before external primitive admission. After Cargo admission, the actual `miniz_oxide` adapter must call through this exact guard and receive fresh exact-head CI and independent review.

For untrusted Git object data, reset/reuse MUST use zeroing reset semantics where state reuse occurs. Convenience APIs that allocate output from attacker-controlled compressed input remain denied.

## SHA-1 disposition

The earlier external `sha1 0.11.0` investigation is retained as historical evidence only. Its selected posture would add `digest`, `cfg-if`, and target-specific `cpufeatures`, which is a larger external trust surface than this task needs.

The current and only preferred SHA-1 direction is a Golam-owned safe-Rust implementation restricted to legacy Git object identity. It MUST NOT be used as authorization, capability, audit-chain, signature, or new collision-resistance security evidence.

```text
SHA1_0_11_0_ADMITTED=NO
EXTERNAL_SHA1_DEPENDENCY_SELECTED=NO
GOLAM_OWNED_SHA1_SELECTED=YES_PENDING_IMPLEMENTATION_QUALIFICATION
SHA1_USE=LEGACY_GIT_OBJECT_IDENTITY_ONLY
SHA1_AUTHORIZATION_SECURITY_PRIMITIVE=NO
SHA256_FIRST_PROFILE=DEFERRED_UNSUPPORTED_FAIL_CLOSED
```

## Source Foundry gates still required

Before either external primitive may reach `ADMITTED` or appear in Golam `Cargo.toml`/`Cargo.lock`:

1. bind the exact crates.io artifact/version/checksum to exact recorded VCS source where available;
2. capture all license files and redistribution/notices obligations;
3. produce the selected-feature transitive Cargo dependency closure, not the package's all/optional development closure;
4. inspect every selected crate for `unsafe`, FFI, build scripts, native code, process launch, network, telemetry, filesystem mutation and environment-controlled behavior;
5. verify the repository-owned bounded-quantum/deadline guard and its deadline-expiry tests on exact-head CI;
6. independently review the exact Source Foundry candidate plus guard on the unchanged exact head;
7. only then record explicit admission;
8. after admission, add exact pinned dependencies and wire the real adapter through the already-qualified guard;
9. any Cargo/parser mutation receives fresh exact-head CI and independent implementation review.

## Current decision

```text
T005_040=BLOCKED_PENDING_CORRECTED_EXACT_HEAD_CI_AND_INDEPENDENT_PRIMITIVE_REVIEW
FORMAT_PROFILE_FROZEN=YES
FIRST_PROFILE_OBJECT_FORMAT=SHA1_ONLY
MINIZ_OXIDE_0_9_1_EXACT_PROVENANCE_BOUND=YES
MINIZ_OXIDE_0_9_1_ADMITTED=NO
ADLER2_2_0_1_EXACT_PROVENANCE_BOUND=YES
ADLER2_2_0_1_ADMITTED=NO
EXTERNAL_SHA1_DEPENDENCY_SELECTED=NO
GOLAM_OWNED_SHA1_SELECTED=YES_PENDING_IMPLEMENTATION_QUALIFICATION
DECOMPRESSION_TIME_GUARD_IMPLEMENTED_FOR_QUALIFICATION=YES
NEW_DEPENDENCY_ADDED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
GIT_CHILD_PROCESS_PHASE_D=DENIED_NATIVE_UNQUALIFIED
GIT_NETWORK_PHASE_D=DENIED
GIT_MUTATION_T005_040=DENIED
WAIVER_TAKEN=NO
```
