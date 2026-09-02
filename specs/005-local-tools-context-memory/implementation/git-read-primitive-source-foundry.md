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

The published `miniz_oxide 0.9.1` package contains `.cargo_vcs_info.json` with:

```text
PUBLISHED_PACKAGE=miniz_oxide 0.9.1
PUBLISHED_VCS_SHA=4e582392df3a739d2b0dfd2c537dc33e8942be38
PUBLISHED_VCS_PATH=miniz_oxide
PUBLISHED_VCS_TREE=52f48224c08d8f9afe004d09c5287747b0242c5f
PUBLISHED_VCS_COMMIT_SIGNATURE=UNSIGNED
```

The exact VCS commit is the 2026-03-13 version-bump commit for `0.9.1`. Its `miniz_oxide/Cargo.toml` changes the package version to `0.9.1`; therefore all source qualification for the published package MUST bind to `4e582392...`, not a later master head.

The unsigned upstream VCS commit is not itself a rejection because the registry package provides independent immutable package/version/checksum provenance, but admission MUST bind both the registry artifact and its recorded VCS source and MUST not represent the later master commit as equivalent.

```text
MINIZ_OXIDE_0_9_1_PREVIOUS_MASTER_EQUIVALENCE=REJECTED_INEXACT
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

The exact manifest still has one required normal dependency under this posture:

```text
adler2 = { version = "2.0", default-features = false }
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

The normalized published manifest has no required external runtime dependency in the ordinary library posture. Its `rustc-std-workspace-core` dependency is optional and exists only for internal Rust standard-library integration. Published documentation states that the crate has zero dependencies and zero `unsafe` in the normal library surface.

The intended T005-040 posture remains `default-features=false`; the internal rustc-std feature path is denied. This substantially narrows the selected closure, but admission still requires exact source-level verification, license/notices capture and independent Source Foundry verification.

```text
ADLER2_2_0_1_EXACT_VCS_SOURCE=89a031a0f42eeff31c70dc598b398cbf31f1680f
ADLER2_2_0_1_SELECTED_EXTERNAL_RUNTIME_DEPENDENCIES=NONE_EXPECTED_PENDING_VERIFICATION
ADLER2_2_0_1_ADMITTED=NO
```

## Bounded decompression API posture

`miniz_oxide` exposes a streaming inflate API accepting caller-provided input and output slices and returning exact `bytes_consumed`, `bytes_written` and status. That API can support Golam's frozen decompressed-byte and time caps without allocating an unbounded result buffer.

The adapter MUST use caller-bounded output chunks and MUST terminate when any frozen T005-040 cap is reached. It MUST NOT use convenience APIs that allocate output based only on attacker-controlled compressed input.

For untrusted Git object data, reset/reuse MUST use zeroing reset semantics where state reuse occurs. Upstream explicitly warns that the minimum reset policy can retain dictionary bytes and may be unsafe with untrusted input.

Required adapter invariants:

```text
DECOMPRESSION_OUTPUT_BUFFER=CALLER_BOUNDED
TOTAL_DECOMPRESSED_BYTES=FROZEN_CAP_ENFORCED
TOTAL_COMPRESSED_BYTES=FROZEN_CAP_ENFORCED
TIME_BUDGET=FROZEN_CAP_ENFORCED
UNBOUNDED_ALLOCATING_HELPERS=DENIED
UNTRUSTED_STATE_REUSE_WITH_MIN_RESET=DENIED
PROCESS_LAUNCH=NONE
NETWORK=NONE
REPOSITORY_MUTATION=NONE
```

## `sha1 0.11.0` exact published provenance

The published `sha1 0.11.0` package records:

```text
PUBLISHED_PACKAGE=sha1 0.11.0
PUBLISHED_VCS_SHA=2f00175af936de46b3ddefe65c4de93cb4e876e4
PUBLISHED_VCS_PATH=sha1
LICENSE=MIT OR Apache-2.0
REPOSITORY=https://github.com/RustCrypto/hashes
RUST_VERSION=1.85
BUILD_SCRIPT=NONE
```

The normalized manifest shows default features `alloc` + `oid`. T005-040 does not need either, so any candidate admission MUST evaluate:

```text
sha1 = { version = "=0.11.0", default-features = false }
```

Even with default features disabled, direct selected dependencies remain `cfg-if ^1.0`, `digest ^0.11`, and architecture-specific `cpufeatures ^0.3` on aarch64/x86/x86_64. Therefore exact selected transitive closure, including whether `cpufeatures` reaches platform `libc`, must be closed before admission.

Upstream explicitly warns that SHA-1 is cryptographically broken and provides this implementation only for legacy interoperability. Golam may use SHA-1 only to reproduce and validate legacy Git object identities under the frozen SHA-1-only profile. SHA-1 MUST NOT become authorization, capability, integrity-chain or security-collision-resistance evidence.

```text
SHA1_0_11_0_EXACT_VCS_SOURCE=2f00175af936de46b3ddefe65c4de93cb4e876e4
SHA1_0_11_0_SELECTED_POSTURE=DEFAULT_FEATURES_FALSE_CANDIDATE
SHA1_0_11_0_ADMITTED=NO
SHA1_PRIMITIVE_SELECTED=NO
SHA1_USE_IF_ADMITTED=LEGACY_GIT_OBJECT_IDENTITY_ONLY
SHA1_AUTHORIZATION_SECURITY_PRIMITIVE=NO
SHA256_FIRST_PROFILE=DEFERRED_UNSUPPORTED_FAIL_CLOSED
```

## Source Foundry gates still required

Before any primitive may reach `ADMITTED` or appear in Golam `Cargo.toml`/`Cargo.lock`:

1. bind the exact crates.io artifact/version/checksum to exact recorded VCS source where available;
2. capture all license files and redistribution/notices obligations;
3. produce the selected-feature transitive Cargo dependency closure, not the package's all/optional development closure;
4. inspect every selected crate for `unsafe`, FFI, build scripts, native code, process launch, network, telemetry, filesystem mutation and environment-controlled behavior;
5. prove the decompression adapter enforces frozen compressed/decompressed/time caps and cannot allocate attacker-selected unbounded output;
6. complete exact source-level and notice verification for `adler2 2.0.1` at `89a031a0...`;
7. close `sha1 0.11.0` at `2f00175...` plus `digest`/`cfg-if`/target `cpufeatures` selected transitive closure, or reject it in favor of a narrower Golam-owned implementation;
8. obtain independent semantic/security verification of the exact Source Foundry admission record;
9. only then add exact pinned dependencies and implement the parser;
10. any Cargo or parser mutation requires fresh exact-head CI and later T005-040 independent implementation review.

## Current decision

```text
T005_040=BLOCKED_ON_PRIMITIVE_SOURCE_QUALIFICATION_AND_PARSER_IMPLEMENTATION
FORMAT_PROFILE_FROZEN=YES
FIRST_PROFILE_OBJECT_FORMAT=SHA1_ONLY
MINIZ_OXIDE_0_9_1_EXACT_PROVENANCE_BOUND=YES
MINIZ_OXIDE_0_9_1_ADMITTED=NO
ADLER2_2_0_1_EXACT_PROVENANCE_BOUND=YES
ADLER2_2_0_1_ADMITTED=NO
SHA1_0_11_0_EXACT_PROVENANCE_BOUND=YES
SHA1_0_11_0_ADMITTED=NO
SHA1_PRIMITIVE_SELECTED=NO
NEW_DEPENDENCY_ADDED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
GIT_CHILD_PROCESS_PHASE_D=DENIED_NATIVE_UNQUALIFIED
GIT_NETWORK_PHASE_D=DENIED
GIT_MUTATION_T005_040=DENIED
WAIVER_TAKEN=NO
```