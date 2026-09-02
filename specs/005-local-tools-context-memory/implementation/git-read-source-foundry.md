# T005-040 — In-Process Git Read Source Foundry Candidate

**Status**: `NARROWER_PLUMBING_CANDIDATE_UNDER_QUALIFICATION_NOT_ADMITTED`

**Task**: T005-040 bounded Git read evidence

## Why a source qualification is required

T005-040 requires repository identity plus bounded HEAD/ref, status, diff, log, tree and blob observation without mutation authority. Canonical Spec 005 also keeps production native execution at `native:unqualified`, so invoking an external `git` executable is not an eligible Phase D shortcut. Implementing complete Git object/index/ref semantics in-process therefore requires either a Golam-owned implementation or an exact admitted Rust source/dependency surface.

The existing Golam-owned filesystem primitives remain suitable for authorized-root, protected-resource, alias and bounded worktree observation, but they do not themselves decode Git object databases, packfiles or indexes.

Exact-head repository CI #829 / run `33570078764` completed `SUCCESS` on the prior documentation head `01226b541709135cd69f990a60d36c0c5c776847`. Exact-head repository CI #830 / run `33573377833` also completed `SUCCESS` on prior documentation head `24527a712fffd3537c2baf8b9562afe4cd74a89b`. Those runs prove the repository remained green while Source Foundry research was documentation-only; this mutation invalidates them for the new exact head and does not qualify any dependency.

## Top-level `gix` candidate — rejected as the direct dependency surface

The first candidate was:

```text
SOURCE=GitoxideLabs/gitoxide
CRATE=gix
VERSION=0.87.0
TAG=gix-v0.87.0
TAG_OBJECT=232c758b33a1d5158a54dc487f41db577fd78596
SOURCE_COMMIT=0c541c7308aee674110dc4dbd2ccda6dceaf41e6
LICENSE=MIT OR Apache-2.0
RUST_VERSION=1.85
DIRECT_GIX_ADMISSION=REJECTED_TOO_BROAD
```

The exact upstream manifest recommends `default-features = false` for library consumers and exposes optional network/process/mutation feature families. That was initially promising, but exact manifest inspection shows that the top-level crate still has a materially broad **unconditional** dependency surface even when those optional features are disabled. Unconditional dependencies include repository/config/ref/discovery/object/pack/revision/diff/protocol infrastructure such as:

```text
gix-config
gix-discover
gix-ref
gix-refspec
gix-odb
gix-object
gix-pack
gix-revision
gix-revwalk
gix-diff
gix-protocol
gix-url
gix-tempfile
gix-lock
```

`gix-protocol` is an unconditional top-level dependency even though concrete network transports remain feature-gated. This does not prove reachable network execution under the proposed feature set, but it widens the compiled and review surface beyond what T005-040 needs. The `revision` feature also implies `index`, while the upstream `status` feature pulls `dirwalk` + `blob-diff`; `blob-diff` pulls `attributes`, and `attributes` enables the top-level `command` feature. Therefore the convenience `status`/attributes path is ineligible for the Phase D read-only boundary.

Source Foundry chooses the narrower boundary rather than relying only on adapter discipline around an unnecessarily broad top-level crate.

```text
GIX_0_87_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_STATUS_FEATURE_ADMITTED=NO
GIX_COMMAND_FEATURE_ADMITTED=NO
GIX_NETWORK_FEATURES_ADMITTED=NO
```

## Lower-level candidates — exact manifest and API narrowing

The lower-level candidate set was initially:

```text
gix-odb = 0.84.0
gix-index = 0.55.0
gix-object = 0.64.0
gix-hash = 0.26.1
```

All candidates refer to exact source commit `0c541c7308aee674110dc4dbd2ccda6dceaf41e6` unless explicitly superseded.

### `gix-odb 0.84.0` — rejected as the direct T005-040 object-store surface

Exact manifest and library inspection shows `gix-odb` is not a read-only object decoder. Its own public documentation describes the all-round `Store` as supporting **loose object reading and writing** and demonstrates `write_buf`. The compiled crate exposes write-capable object-store behavior and its normal dependency closure includes `tempfile`, `parking_lot`, `arc-swap`, `memmap2`, `gix-zlib`, `gix-pack`, filesystem/path utilities and other object-database infrastructure.

This is materially broader than the Phase D requirement. Golam could theoretically hide the write methods behind a wrapper, but Source Foundry requires minimizing the trusted/review surface before relying on wrapper discipline. `gix-odb` is therefore rejected as a direct dependency for T005-040.

```text
GIX_ODB_0_84_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_ODB_REJECTION=WRITE_CAPABLE_OBJECT_STORE_AND_BROAD_NORMAL_CLOSURE
```

### `gix-index 0.55.0` — rejected as the direct T005-040 index surface

Exact source inspection shows the crate publicly compiles a `write` module and documents `State` as an in-memory index intended to be altered and eventually written back to disk. Its normal dependency closure unconditionally includes `gix-lock`, `gix-fs`, `memmap2`, `filetime`, and on Unix `rustix`/`libc` filesystem support. The crate itself denies unsafe Rust, but that does not remove the write/lock/native-adjacent dependency surface from the compiled T005-040 boundary.

The T005-040 adapter needs only bounded decoding/observation of an existing index. A Golam-owned bounded parser for the index formats used by supported fixtures is a narrower boundary than compiling the write-capable `gix-index` surface.

```text
GIX_INDEX_0_55_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_INDEX_REJECTION=UNCONDITIONAL_LOCK_WRITE_SURFACE_FOR_READ_ONLY_TASK
```

### `gix-pack 0.74.0` — narrower packed-object candidate, still NOT admitted

Exact manifest inspection shows `gix-pack` has default features `generate` and `streaming-input`, both ineligible for T005-040. With `default-features = false`, however, the pack-generation dependencies (`gix-traverse`, `gix-diff`, `parking_lot`, `gix-hashtable`) and streaming-input tempfile path are optional rather than unconditional. The remaining normal surface includes pack/hash/object/chunk/zlib helpers plus `memmap2`, `smallvec`, and `thiserror`.

This makes `gix-pack` materially narrower than `gix-odb` for **read-only packed-object decoding**, but it remains only a candidate until exact closure/license/unsafe/native/mmap bounds and read-only API reachability are proven.

Proposed candidate feature posture:

```text
gix-pack = { version = "=0.74.0", default-features = false, features = ["sha1", "sha256"] }
GIX_PACK_GENERATE_FEATURE=DENIED
GIX_PACK_STREAMING_INPUT_FEATURE=DENIED
GIX_PACK_PARALLEL_FEATURE=DENIED
GIX_PACK_WRITE_OR_GENERATE_PATH=T005_040_DENIED
```

### `gix-object 0.64.0` — decode candidate, signature helpers denied

The exact manifest makes external command/tempfile behavior optional behind the `signature` feature. T005-040 does not need signing or verification helpers, so `signature` is explicitly denied. The remaining crate still exposes mutable/encoding object APIs in addition to decoding, so Golam must expose only immutable decode views through its adapter and independently verify that no command/tempfile feature is enabled transitively.

```text
GIX_OBJECT_0_64_0_ADMITTED=NO
GIX_OBJECT_SIGNATURE_FEATURE=DENIED
GIX_OBJECT_COMMAND_PATH=DENIED
```

### `gix-hash 0.26.1` — identity candidate, still NOT admitted

`gix-hash` remains under exact closure/license/hash-backend review for SHA-1/SHA-256 object identities. No admission is implied by its small conceptual role.

```text
GIX_HASH_0_26_1_ADMITTED=NO
```

### `gix-ref 0.67.0` — not selected

`gix-ref` remains unselected because its normal dependency surface includes `gix-lock`, `gix-tempfile` and `memmap2`. T005-040 will first attempt a Golam-owned bounded parser for `HEAD`, loose refs and `packed-refs`.

## Revised narrower candidate architecture

The preferred research boundary is now smaller than the previous lower-level set:

1. Golam-owned repository-root / `.git` identity discovery under the already authorized root;
2. Golam-owned bounded `HEAD`, loose-ref and `packed-refs` parsing;
3. Golam-owned bounded index decoding for the exact supported index versions/extensions required by fixtures;
4. Golam-owned bounded loose-object header/zlib decode path if the exact implementation can remain simpler than admitting `gix-odb`;
5. `gix-pack` with `default-features = false` and only exact hash-support features as the candidate for packed-object decoding;
6. `gix-object` + `gix-hash` only if exact closure review proves their selected no-signature/no-command posture and the adapter exposes decode/identity operations only;
7. Golam-owned status composition by comparing index/object identities to existing bounded filesystem observations;
8. Golam-owned diff evidence over bounded blob/worktree byte observations rather than enabling top-level `gix` attributes/command/status features;
9. Golam-owned bounded commit-parent traversal for log evidence, with explicit commit/count/time caps.

This architecture deliberately rejects a general Git repository or object-store handle. T005-040 should receive only bounded observation functions and typed evidence records.

## Authority and execution boundaries

Any eventual Git read implementation MUST preserve:

- repository observation is evidence, never authority;
- repository discovery is bounded to the already authorized workspace root;
- `PATH_STRING != TARGET_IDENTITY`;
- protected Golam resources remain excluded from generic repository/filesystem inspection;
- no child process, shell, hook, credential helper, editor, signer/verifier helper or external executable may be launched;
- no network transport, fetch, push, clone or remote discovery is enabled;
- environment/config inputs that could redirect repository, object database, index, hooks, filters, helpers or executables must be disabled, ignored or explicitly bounded by Golam policy before use;
- no Git mutation API is exposed by the T005-040 read surface;
- force/history rewrite remains unavailable;
- all worktree reads remain subject to existing byte/count/depth/time and target-identity bounds;
- status/diff/log/tree/blob outputs retain repository/ref/object/worktree identity and observation provenance and cannot mint capability, approval or verification authority.

## Qualification still required before `ADMITTED`

This record remains fail-closed. The following gates are incomplete:

1. capture the exact proposed `gix-pack` / `gix-object` / `gix-hash` selected-feature transitive dependency set;
2. verify license/notice obligations for that complete selected closure;
3. inspect unsafe/FFI/native-code/build-script surfaces in that exact closure, especially `memmap2`, hash/compression implementations and generated/native artifacts;
4. prove no selected crate feature or Golam adapter path can launch network clients, credential helpers, commands, hooks, filters, editors, signing helpers or other executables;
5. prove no selected adapter path performs ref/index/object/worktree mutation, lock acquisition for writes, tempfile-backed replacement, pack generation or object writes;
6. define bounded repository-opening rules that ignore or fail closed on environment/config redirection (`GIT_DIR`, worktree/index/object alternates/config/include/helper/filter/hook redirection and equivalents);
7. implement and adversarially test Golam-owned bounded HEAD/ref/packed-refs and index parsing before reconsidering `gix-ref` or `gix-index`;
8. determine whether Golam-owned loose-object decoding plus the candidate `gix-pack` packed path fully replaces `gix-odb`; if not, do not silently reintroduce `gix-odb`—create a new explicit Source Foundry disposition;
9. implement a narrow read-only Golam adapter exposing only bounded repository identity, HEAD/ref, status, diff, log, tree and blob evidence;
10. add fixtures for loose and packed objects, supported SHA-1/SHA-256 repositories, detached/symbolic HEAD, loose/packed refs, index/worktree disagreement, malformed repositories, object alternates/config/env redirection, path escapes and bounded-resource failure;
11. run exact-head Windows/macOS/Ubuntu CI on the exact dependency/adapter head;
12. obtain substantive independent semantic/security review of the exact admitted source/features/adapter head;
13. update this record to `ADMITTED` only after every gate above is evidenced.

Until then:

```text
T005_040=BLOCKED_ON_EXACT_GIT_READ_SOURCE_QUALIFICATION
TOP_LEVEL_GIX_DIRECT_DEPENDENCY=REJECTED_TOO_BROAD
GIX_ODB_0_84_0_ADMITTED=NO
GIX_INDEX_0_55_0_ADMITTED=NO
GIX_PACK_0_74_0_ADMITTED=NO
GIX_OBJECT_0_64_0_ADMITTED=NO
GIX_HASH_0_26_1_ADMITTED=NO
GIX_REF_0_67_0_ADMITTED=NO
GIT_CHILD_PROCESS_PHASE_D=DENIED_NATIVE_UNQUALIFIED
GIT_NETWORK_PHASE_D=DENIED
GIT_MUTATION_T005_040=DENIED
NEW_DEPENDENCY_ADDED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
WAIVER_TAKEN=NO
```

## Alternatives

A fully Golam-owned Git parser remains eligible if it can satisfy the same object/index/ref/pack/status/diff/log/tree/blob behavior and adversarial gates without external source reuse. The current preferred research direction is a hybrid boundary: Golam-owned repository/ref/index/loose-object parsing plus the smallest exact no-generate/no-streaming `gix-pack` packed-object decoder surface that can be fully qualified, with `gix-object`/`gix-hash` admitted only if their exact selected closure remains bounded. No crate may enter `Cargo.toml` until the corresponding exact Source Foundry admission closes.