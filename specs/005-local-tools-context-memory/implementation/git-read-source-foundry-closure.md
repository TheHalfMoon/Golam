# T005-040 — Git Read Source Foundry Closure Narrowing

**Status**: `FULLY_GOLAM_OWNED_PARSER_DIRECTION_UNDER_QUALIFICATION_NOT_ADMITTED`

**Supersedes candidate preference in**: `git-read-source-foundry.md`

**Exact repository head before this disposition**: `dc36362ac880fb1334da9bd84e1eaa87dd733162`

**Exact repository CI before this disposition**: CI #831 / run `33578698480` — `SUCCESS`

**Task**: T005-040 bounded Git read evidence

## Purpose

This record closes the next Source Foundry narrowing step after exact manifest/API review of the previously preferred `gix-pack` / `gix-object` / `gix-hash` plumbing candidate. It does **not** admit a dependency and does **not** authorize implementation beyond the bounded read-only T005-040 scope.

The prior record remains useful historical evidence for why top-level `gix`, `gix-odb`, `gix-index`, and `gix-ref` were rejected or left unselected. This record supersedes the prior preference for `gix-pack` + `gix-object`; the only current preferred architecture is a Golam-owned parser with separately qualified primitive decompression support and Golam-owned SHA-1.

## Exact source state inspected

```text
SOURCE_REPOSITORY=GitoxideLabs/gitoxide
SOURCE_COMMIT=0c541c7308aee674110dc4dbd2ccda6dceaf41e6
GIX_PACK_VERSION=0.74.0
GIX_OBJECT_VERSION=0.64.0
GIX_HASH_VERSION=0.26.1
GIX_FEATURES_VERSION=0.49.1
GIX_ZLIB_VERSION=0.1.0
LICENSE_FAMILY=MIT_OR_APACHE_2_0
```

## `gix-pack 0.74.0` direct dependency — rejected for T005-040

Disabling `default` removes the `generate` and `streaming-input` feature closures, but it does not produce a decoder-only crate. The exact normal dependency surface still includes:

```text
gix-features 0.49.1      features=[crc32, progress]
gix-zlib 0.1.0
gix-path 0.12.5
gix-hash 0.26.1
gix-chunk 0.8.0
gix-error 0.3.0
gix-object 0.64.0
memmap2 0.9.11
smallvec 1.15.1
thiserror 2.0.18
```

`gix-features/progress` brings the `prodash` progress surface, and `gix-features` has an unconditional Unix `libc` dependency. `gix-pack` also unconditionally brings `memmap2`. Together these enlarge the compiled, license, native-adjacent, unsafe/mmap, and review closure beyond what T005-040 needs.

The package additionally contains pack generation capability behind a feature. Source Foundry prefers eliminating unnecessary capability-bearing code from the compiled boundary rather than relying on wrapper convention.

```text
GIX_PACK_0_74_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_PACK_REJECTION=BROAD_UNCONDITIONAL_CLOSURE_AND_NON_DECODER_ONLY_PACKAGE
GIX_PACK_GENERATE_FEATURE=DENIED
GIX_PACK_STREAMING_INPUT_FEATURE=DENIED
GIX_PACK_PARALLEL_FEATURE=DENIED
```

## `gix-object 0.64.0` direct dependency — rejected for T005-040

The exact package provides immutable **and mutable** Git objects with decoding **and encoding** support. Even with `signature` disabled, its normal closure includes `gix-features/progress`, `gix-hashtable`, `gix-validate`, `gix-actor`, `gix-date`, `gix-utils`, `bstr`, `smallvec`, and formatting/error helpers.

The optional `signature` feature enables `gix-command` and `gix-tempfile`; that feature remains denied. T005-040 requires bounded observation, not a general mutable/encoding object API. A Golam-owned parser for the exact object types required by status/diff/log/tree/blob evidence is a materially smaller authority and review boundary.

```text
GIX_OBJECT_0_64_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_OBJECT_REJECTION=MUTABLE_ENCODING_API_AND_BROAD_NORMAL_CLOSURE
GIX_OBJECT_SIGNATURE_FEATURE=DENIED
GIX_OBJECT_COMMAND_PATH=DENIED
```

## `gix-hash 0.26.1` — not selected

`gix-hash` is smaller conceptually but still brings `gix-features/progress`, `faster-hex`, error infrastructure and optional hash backends. The later primitive narrowing selected a Golam-owned SHA-1 implementation for the SHA-1-only first profile, so `gix-hash` is no longer a current candidate.

```text
GIX_HASH_0_26_1_ADMITTED=NO
GIX_HASH_DECISION=NOT_SELECTED_GOLAM_OWNED_SHA1_IS_NARROWER
```

## Compression posture

`gix-zlib 0.1.0` is not selected. Its normal dependency is `zlib-rs 0.6.2`. The Git read parser needs bounded **decompression** of loose objects and packed-object streams, not compression or a general Git pack library.

The narrowed primitive candidate is `miniz_oxide 0.9.1` with `default-features=false`, whose selected external closure is `adler2 2.0.1` only. Both remain **NOT_ADMITTED** until corrected exact-head CI and independent Source Foundry review pass.

```text
GIX_ZLIB_0_1_0_ADMITTED=NO
ZLIB_RS_0_6_2_ADMITTED=NO
MINIZ_OXIDE_0_9_1_CANDIDATE_SELECTED=YES_NOT_ADMITTED
ADLER2_2_0_1_CANDIDATE_SELECTED=YES_NOT_ADMITTED
NEW_COMPRESSION_DEPENDENCY_ADDED=NO
```

## Revised preferred implementation boundary

The only current preferred T005-040 architecture is Golam-owned bounded Git parsing with separately Source-Foundry-qualified primitive decompression and Golam-owned SHA-1:

1. repository-root and `.git` identity discovery only under an already authorized root;
2. bounded `HEAD`, loose-ref, and `packed-refs` parsing;
3. bounded index decoding for explicitly supported index versions/extensions;
4. bounded loose-object header and decompression handling;
5. bounded `.idx` and `.pack` decoding with explicit object-count, delta-depth, decompressed-size, byte, and time caps;
6. Golam-owned commit/tree/blob/tag decoding sufficient for exact observation evidence;
7. Golam-owned status composition from HEAD tree + index + bounded worktree evidence;
8. Golam-owned bounded diff evidence over exact blob/worktree bytes;
9. Golam-owned bounded parent traversal for log evidence;
10. no general repository handle, mutable object model, lock/write API, pack generation API, child process, helper, hook, filter, network transport, or repository mutation surface.

This direction intentionally accepts more Golam code in exchange for a much smaller external trusted/dependency boundary. Parser output remains unprivileged evidence and cannot mint capability, approval, verification authority, or Effect Gate state.

## Bounded synchronous decompression time contract

The selected candidate inflate API is synchronous and non-preemptive while one call is in flight. Golam therefore bounds the synchronous work presented to each call and checks a monotonic deadline at every call boundary.

```text
DECOMPRESSION_INPUT_QUANTUM_BYTES=65536
DECOMPRESSION_OUTPUT_QUANTUM_BYTES=65536
DEADLINE_CLOCK=MONOTONIC_INSTANT
DEADLINE_CHECK=IMMEDIATELY_BEFORE_EVERY_INFLATE_CALL
DEADLINE_CHECK=IMMEDIATELY_AFTER_EVERY_INFLATE_CALL
IN_FLIGHT_SYNCHRONOUS_CALL=NON_PREEMPTIVE
POST_CALL_DEADLINE_OVERRUN=FAIL_CLOSED_DISCARD_STEP_RESULT
EXPIRED_BETWEEN_CHUNKS=REJECT_NEXT_CALL_BEFORE_INFLATE
```

The repository-owned `git_read_budget::DecompressionDeadline` helper implements these caller-side invariants before any external dependency is admitted. Focused tests cover oversized quanta, deadline expiry between chunks with proof that the next callback is not invoked, and an in-flight call that crosses the deadline and is rejected on return. The eventual `miniz_oxide` adapter MUST route every inflate call through this guard; any bypass invalidates qualification.

This is bounded non-preemptive behavior, not a claim that synchronous third-party code can be interrupted mid-call.

## Required fail-closed repository-opening rules

Before implementation can qualify, the adapter contract must explicitly reject or ignore ambient redirection that could widen the observation root or invoke Git-configured behavior. At minimum:

```text
GIT_DIR=DENIED_AMBIENT_OVERRIDE
GIT_WORK_TREE=DENIED_AMBIENT_OVERRIDE
GIT_INDEX_FILE=DENIED_AMBIENT_OVERRIDE
GIT_OBJECT_DIRECTORY=DENIED_AMBIENT_OVERRIDE
GIT_ALTERNATE_OBJECT_DIRECTORIES=DENIED_AMBIENT_OVERRIDE
GIT_CONFIG_COUNT=DENIED_AMBIENT_OVERRIDE
GIT_CONFIG_KEY_*=DENIED_AMBIENT_OVERRIDE
GIT_CONFIG_VALUE_*=DENIED_AMBIENT_OVERRIDE
GIT_CONFIG_SYSTEM=DENIED_AMBIENT_OVERRIDE
GIT_CONFIG_GLOBAL=DENIED_AMBIENT_OVERRIDE
GIT_CONFIG_NOSYSTEM=NOT_AUTHORITY
GIT_CEILING_DIRECTORIES=NOT_USED_FOR_AUTHORITY
GIT_DISCOVERY_ACROSS_FILESYSTEM=DENIED
HOOKS=NOT_EXECUTED
FILTERS=NOT_EXECUTED
CREDENTIAL_HELPERS=NOT_EXECUTED
EDITORS=NOT_EXECUTED
SIGNERS_VERIFIERS=NOT_EXECUTED
REMOTE_TRANSPORTS=NOT_COMPILED_OR_INVOKED
```

Repository identity must come from Golam's already-authorized root plus resolved target identity, never from ambient environment variables or Git configuration.

## Source Foundry work still required

T005-040 remains blocked. Before any dependency is added or the task is declared implemented, the next bounded units are:

1. keep the frozen exact supported Git index/object/pack formats and resource limits;
2. qualify the exact `miniz_oxide 0.9.1` + `adler2 2.0.1` selected closure, including the bounded-quantum/deadline guard evidence;
3. obtain corrected exact-head Windows/macOS/Ubuntu CI;
4. obtain substantive independent semantic/security review of the unchanged corrected candidate head;
5. only after a clean review record explicit primitive admission;
6. then add the exact pinned Cargo dependencies and verify the actual selected lock/feature delta remains exactly the reviewed closure;
7. implement Golam-owned SHA-1 and Git parsers with every inflate call routed through the qualified guard;
8. add adversarial fixtures for symbolic/detached HEAD, loose/packed refs, index versions/extensions, loose/packed objects, deltas, malformed/truncated/bomb inputs, path escapes, alternates/config/env redirection, SHA-1-only behavior, and worktree/index/HEAD disagreement;
9. run fresh exact-head Windows/macOS/Ubuntu CI on the implementation head;
10. obtain substantive independent implementation review before T005-040 is complete.

Until then:

```text
T005_040=BLOCKED_ON_CORRECTED_PRIMITIVE_REQUALIFICATION_AND_IMPLEMENTATION
TOP_LEVEL_GIX_DIRECT_DEPENDENCY=REJECTED_TOO_BROAD
GIX_ODB_0_84_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_INDEX_0_55_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_REF_0_67_0_ADMITTED=NO
GIX_PACK_0_74_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_OBJECT_0_64_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_HASH_0_26_1_ADMITTED=NO
GIX_ZLIB_0_1_0_ADMITTED=NO
MINIZ_OXIDE_0_9_1_ADMITTED=NO
ADLER2_2_0_1_ADMITTED=NO
DECOMPRESSION_TIME_GUARD_IMPLEMENTED_FOR_QUALIFICATION=YES
NEW_DEPENDENCY_ADDED=NO
GIT_CHILD_PROCESS_PHASE_D=DENIED_NATIVE_UNQUALIFIED
GIT_NETWORK_PHASE_D=DENIED
GIT_MUTATION_T005_040=DENIED
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
WAIVER_TAKEN=NO
```
