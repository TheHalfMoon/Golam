# T005-040 — Minimal Git Read Primitive Admission Candidate

**Status**: `READY_FOR_CORRECTED_EXACT_HEAD_CI_AND_INDEPENDENT_REVIEW_NOT_ADMITTED`

**Task**: T005-040 bounded Git read evidence

**Parent repository head**: `809b8db03b54b3c39f2ac0297f1c4fc61ef440fc`

**Parent exact-head CI**: CI #835 / run `33597420816` — `SUCCESS` on Windows, macOS and Ubuntu

## Decision

The first bounded Git-read implementation will use a Golam-owned parser and a Golam-owned SHA-1 implementation. The only external implementation primitives proposed for admission are bounded zlib/DEFLATE inflation and its Adler-32 checksum dependency:

```text
EXTERNAL_PRIMITIVE_1=miniz_oxide 0.9.1
EXTERNAL_PRIMITIVE_2=adler2 2.0.1
SHA1_IMPLEMENTATION=GOLAM_OWNED
SHA1_EXTERNAL_DEPENDENCY=NO
TOP_LEVEL_GIX=REJECTED
GIX_PACK=REJECTED
GIX_OBJECT=REJECTED
GIX_ODB=REJECTED
GIX_INDEX=REJECTED
GIX_REF=REJECTED
GIT_CHILD_PROCESS=DENIED
GIT_NETWORK=DENIED
GIT_MUTATION=DENIED
```

This record is an admission candidate only. No dependency is admitted and no Cargo file is changed by this record. The first independent review of exact head `c8e4303f8986d1de220ca42f3abca0ace498003d` identified a material decompression-time contract gap, so that head did **not** qualify admission. The candidate now requires fresh exact-head CI and fresh substantive independent semantic/security review after the forward-only repair.

## `miniz_oxide 0.9.1`

Exact selected package/source binding:

```text
PACKAGE=miniz_oxide
VERSION=0.9.1
REGISTRY_CHECKSUM=b63fbc4a50860e98e7b2aa7804ded1db5cbc3aff9193adaff57a6931bf7c4b4c
SOURCE_REPOSITORY=https://github.com/Frommi/miniz_oxide
PUBLISHED_VCS_SHA=4e582392df3a739d2b0dfd2c537dc33e8942be38
PUBLISHED_VCS_PATH=miniz_oxide
LICENSE=MIT OR Zlib OR Apache-2.0
LICENSE_FILES=miniz_oxide/LICENSE,miniz_oxide/LICENSE-APACHE.md,miniz_oxide/LICENSE-MIT.md,miniz_oxide/LICENSE-ZLIB.md
```

Selected Cargo posture:

```toml
miniz_oxide = { version = "=0.9.1", default-features = false }
```

Selected feature state:

```text
with-alloc=OFF
std=OFF
block-boundary=OFF
simd=OFF
serde=OFF
rustc-dep-of-std=OFF
```

At the exact VCS source, the package manifest has one required normal dependency under this posture:

```toml
adler2 = { version = "2.0", default-features = false }
```

`miniz_oxide/src/lib.rs` declares `#![forbid(unsafe_code)]`. With `std` disabled it is `no_std`; with `with-alloc` disabled the public `deflate` module is not compiled while the `inflate` module remains available. No Cargo `build.rs` exists in the selected package directory. Optional SIMD, serde, rustc-internal core/alloc and allocation-enabled feature paths are excluded.

The upstream repository contains development/build utility scripts outside the selected Cargo package. They are not Cargo build scripts, are not selected package inputs, and are not executed by Golam.

## `adler2 2.0.1`

Exact selected package/source binding:

```text
PACKAGE=adler2
VERSION=2.0.1
REGISTRY_CHECKSUM=320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa
SOURCE_REPOSITORY=https://github.com/oyvindln/adler2
PUBLISHED_VCS_SHA=89a031a0f42eeff31c70dc598b398cbf31f1680f
LICENSE=0BSD OR MIT OR Apache-2.0
```

Selected Cargo posture:

```toml
adler2 = { version = "=2.0.1", default-features = false }
```

The exact source documents zero ordinary dependencies and zero unsafe code, declares `#![forbid(unsafe_code)]`, and becomes `no_std` when the default `std` feature is disabled. Its only manifest dependency is the optional rustc-standard-library integration package, which is not selected. No Cargo build script, process, network, FFI, native-code, telemetry or filesystem-mutation surface is selected.

Although `adler2` is a transitive dependency of the selected `miniz_oxide` posture, Golam will pin it directly to `=2.0.1` so the reviewed closure cannot drift within `miniz_oxide`'s `2.0` semver range. This pin prevents version drift; selected-feature unification is still verified from the eventual exact lock/feature graph before implementation qualification.

## Exact selected external closure

The intended production dependency closure for T005-040 is exactly:

```text
miniz_oxide 0.9.1
└── adler2 2.0.1
```

No selected feature introduces:

- `simd-adler32`;
- `serde`;
- rustc workspace core/alloc packages;
- `libc`;
- `memmap2`;
- FFI/native libraries;
- build scripts;
- process/command helpers;
- network clients/transports;
- credential helpers;
- Git repository/object-store abstractions;
- filesystem mutation APIs owned by the dependency closure.

The eventual Cargo lockfile MUST reproduce this exact closure. Any additional selected package or feature invalidates this admission candidate and requires a new Source Foundry disposition before implementation proceeds.

## Why SHA-1 is Golam-owned

The frozen first Git format profile requires SHA-1 only for legacy Git object identities. The previously inspected `sha1 0.11.0` candidate would add `digest`, `cfg-if`, and target-specific `cpufeatures` closure. That external surface is unnecessary for the bounded algorithm required here.

Golam will therefore implement the FIPS-180-4 SHA-1 compression/padding algorithm directly in Golam-owned safe Rust with:

- no external SHA-1 crate;
- no CPU feature detection dependency;
- no assembly, FFI or unsafe code;
- no heap requirement for the digest core;
- published known-answer vectors plus Git object-identity fixtures;
- explicit domain restriction to Git object identity only.

SHA-1 is cryptographically broken and MUST NOT be used for authorization, capability identity, authority-journal integrity, signature verification, security collision resistance, or any new Golam security primitive.

```text
SHA1_USE=LEGACY_GIT_OBJECT_IDENTITY_ONLY
SHA1_AUTHORITY_SECURITY_PRIMITIVE=NO
SHA1_INTEGRITY_CHAIN_PRIMITIVE=NO
SHA1_SIGNATURE_PRIMITIVE=NO
```

## Bounded decompression adapter contract

Golam may call only the synchronous streaming inflate surface with caller-provided input/output slices. A single in-flight `miniz_oxide::inflate::stream::inflate` call is **non-preemptive**; Golam does not claim it can cancel arbitrary work inside that synchronous call. Instead, every call is enclosed by the repository-owned `git_read_budget::DecompressionDeadline` guard with bounded work quanta and monotonic deadline checks.

Required invariants:

```text
INPUT=CALLER_BOUNDED
OUTPUT=CALLER_BOUNDED_CHUNKS
DECOMPRESSION_INPUT_QUANTUM_BYTES=65536
DECOMPRESSION_OUTPUT_QUANTUM_BYTES=65536
MAX_COMPRESSED_BYTES=FROZEN_PROFILE_CAP
MAX_DECOMPRESSED_BYTES=FROZEN_PROFILE_CAP
OPERATION_DECOMPRESSED_BYTES=FROZEN_PROFILE_CAP
DEADLINE_CLOCK=MONOTONIC_INSTANT
DEADLINE_CHECK=IMMEDIATELY_BEFORE_EVERY_INFLATE_CALL
DEADLINE_CHECK=IMMEDIATELY_AFTER_EVERY_INFLATE_CALL
IN_FLIGHT_SYNCHRONOUS_CALL=NON_PREEMPTIVE
POST_CALL_DEADLINE_OVERRUN=FAIL_CLOSED_DISCARD_STEP_RESULT
EXPIRED_BETWEEN_CHUNKS=REJECT_NEXT_CALL_BEFORE_INFLATE
ALLOCATING_DECOMPRESS_TO_VEC_HELPERS=DENIED
DEFLATE_COMPRESSION_PATH=NOT_COMPILED
STATE_REUSE_FOR_UNTRUSTED_INPUT=ZEROING_RESET_OR_NEW_STATE
PARTIAL_OVERSIZE_OBJECT=NOT_TRUSTED
ZLIB_CHECKSUM=VALIDATED
```

The 64 KiB input and output caps are the maximum synchronous work quantum passed into one external inflate call. The total operation deadline remains the frozen `DEFAULT_GIT_READ_TIME_BUDGET_MS` / `MAX_GIT_READ_TIME_BUDGET_MS`; a call that returns after the deadline has crossed is rejected immediately and its step result is not trusted.

The Golam-owned guard is implemented **before dependency admission** so the caller-side behavior can be qualified without importing the candidate crate. Its focused tests require:

1. an oversized synchronous work quantum is rejected before the callback is invoked;
2. a deadline that expires between chunks rejects the next chunk before callback invocation;
3. a non-preemptive in-flight callback that crosses the deadline is rejected immediately after it returns.

After external primitive admission, the actual `miniz_oxide` adapter MUST route every inflate call through this same guard. Any direct call that bypasses the guard invalidates qualification.

If the selected API cannot operate within these byte/work/deadline constraints without unbounded allocation, T005-040 fails closed and this primitive candidate is rejected.

## Process, network, filesystem and environment posture

Static library computation is the entire admitted intent. Neither selected crate may be used as authority or as a capability-bearing component.

```text
PROCESS_LAUNCH=NONE
SHELL=NONE
NETWORK=NONE
DNS=NONE
TELEMETRY=NONE
CREDENTIAL_ACCESS=NONE
ENVIRONMENT_AUTHORITY=NONE
GIT_CONFIG_EXECUTION=NONE
HOOK_EXECUTION=NONE
FILTER_EXECUTION=NONE
REMOTE_TRANSPORT=NONE
REPOSITORY_MUTATION=NONE
```

All repository filesystem access remains Golam-owned and passes through the existing authorized-root/target-identity boundary. The decompression/checksum primitives receive byte slices only.

## License and notice posture

Golam may satisfy `miniz_oxide` through the MIT or Apache-2.0 option and must retain the applicable upstream license notices in distribution/source obligations. `adler2` may likewise be consumed under MIT or Apache-2.0 (or 0BSD); Golam will retain its applicable license notice. No copyleft or reciprocal source obligation is introduced by the selected license options.

This record does not redistribute upstream source inside Golam and does not vendor the crates. Any later vendoring would be a separate Source Foundry mutation and requires exact notice/source review.

## Independent review gate

Before either dependency may be written to `Cargo.toml` or `Cargo.lock`, a substantive independent reviewer must inspect the corrected exact candidate head after exact-head CI and verify at minimum:

1. exact package/version/checksum/VCS binding;
2. selected feature and transitive closure accuracy;
3. license/notices posture;
4. no selected unsafe/FFI/native/build-script/process/network/telemetry surface;
5. bounded synchronous decompression quanta, before/after monotonic deadline checks, non-preemptive in-flight semantics, and the focused expiry tests;
6. direct `adler2` pin prevents version drift and eventual selected features remain exact;
7. Golam-owned SHA-1 is the narrower reviewed choice and remains Git-identity-only;
8. no Git mutation, helper, hook, filter, credential, remote or process authority is introduced;
9. no privilege or authority can be minted from primitive output.

Status-only, summary-only, owner/self-review, stale-head review, CI-only output, quota/billing output, or unavailable-provider output is insufficient.

## Current disposition

```text
T005_040=BLOCKED_PENDING_CORRECTED_EXACT_HEAD_CI_AND_INDEPENDENT_PRIMITIVE_REVIEW
PRIMITIVE_ADMISSION_CANDIDATE=READY_FOR_REQUALIFICATION_NOT_ADMITTED
MINIZ_OXIDE_0_9_1_ADMITTED=NO
ADLER2_2_0_1_ADMITTED=NO
EXTERNAL_SHA1_DEPENDENCY=NO
GOLAM_OWNED_SHA1_SELECTED=YES_PENDING_REVIEW
DECOMPRESSION_TIME_GUARD_IMPLEMENTED_FOR_QUALIFICATION=YES
NEW_DEPENDENCY_ADDED=NO
CARGO_LOCK_CHANGED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
GIT_CHILD_PROCESS_PHASE_D=DENIED_NATIVE_UNQUALIFIED
GIT_NETWORK_PHASE_D=DENIED
GIT_MUTATION_T005_040=DENIED
WAIVER_TAKEN=NO
NEXT_GATE=CORRECTED_EXACT_HEAD_CI_THEN_FRESH_INDEPENDENT_SEMANTIC_SECURITY_REVIEW
```
