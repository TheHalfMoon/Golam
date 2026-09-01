# T005-040 — In-Process Git Read Source Foundry Candidate

**Status**: `NARROWER_PLUMBING_CANDIDATE_UNDER_QUALIFICATION_NOT_ADMITTED`

**Task**: T005-040 bounded Git read evidence

## Why a source qualification is required

T005-040 requires repository identity plus bounded HEAD/ref, status, diff, log, tree and blob observation without mutation authority. Canonical Spec 005 also keeps production native execution at `native:unqualified`, so invoking an external `git` executable is not an eligible Phase D shortcut. Implementing complete Git object/index/ref semantics in-process therefore requires either a Golam-owned implementation or an exact admitted Rust source/dependency surface.

The existing Golam-owned filesystem primitives remain suitable for authorized-root, protected-resource, alias and bounded worktree observation, but they do not themselves decode Git object databases, packfiles or indexes.

Exact-head repository CI #829 / run `33570078764` completed `SUCCESS` on the prior documentation head `01226b541709135cd69f990a60d36c0c5c776847`. This proves the repository remained green while Source Foundry research was documentation-only; this mutation invalidates that run for the new exact head and does not qualify any dependency.

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

## Narrower candidate architecture

The next candidate is a lower-level gitoxide plumbing surface plus Golam-owned parsing/composition. No crate is admitted yet.

Candidate crates at exact source commit `0c541c7308aee674110dc4dbd2ccda6dceaf41e6`:

```text
gix-odb = 0.84.0          # loose/packed object database reads
gix-index = 0.55.0        # index decoding
gix-object = 0.64.0       # commit/tree/blob/tag object parsing
gix-hash = 0.26.1         # SHA-1/SHA-256 object identities
```

`gix-ref 0.67.0` was inspected but is **not yet selected**. Its exact manifest includes `gix-lock`, `gix-tempfile` and `memmap2` in the normal dependency surface. T005-040 may instead use a Golam-owned bounded parser for `HEAD`, loose refs and `packed-refs`, keeping reference observation read-only and avoiding mutation-oriented ref plumbing if the parser can satisfy all adversarial fixtures.

Exact manifest observations for the selected-under-study crates:

- `gix-odb 0.84.0` has explicit `sha1`/`sha256` features and depends on object/pack/hash/fs parsing infrastructure. It also depends on `tempfile`, `parking_lot`, `arc-swap` and `memmap2`; these require closure, unsafe/native/build-script and write-reachability review before admission.
- `gix-index 0.55.0` has explicit `sha1`/`sha256` features and depends on `gix-lock`, filesystem utilities, `memmap2`, `filetime`, and Unix `rustix`/`libc`. Its crate surface supports index construction/mutation as well as reading, so Golam must prove the adapter exposes only bounded decoding/observation and that no lock/write API is reachable from T005-040.
- `gix-object 0.64.0` and `gix-hash 0.26.1` are required transitively for typed object decoding and object identities; their exact feature/dependency closure remains part of this candidate audit.

The narrower design target is:

1. Golam-owned repository-root / `.git` identity discovery under the already authorized root;
2. Golam-owned bounded `HEAD`, loose-ref and `packed-refs` parsing if exact fixtures prove this is sufficient;
3. `gix-odb` for bounded loose/pack object lookup;
4. `gix-object` + `gix-hash` for commit/tree/blob/tag decoding and identities;
5. `gix-index` for bounded index decoding only;
6. Golam-owned status composition by comparing index/object identities to existing bounded filesystem observations;
7. Golam-owned diff evidence over bounded blob/worktree byte observations rather than enabling top-level `gix` attributes/command/status features;
8. Golam-owned bounded commit-parent traversal for log evidence, with explicit commit/count/time caps.

This composition is intentionally read-only and must not expose a general Git repository handle with mutation/network/helper surfaces.

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

1. capture the exact selected lower-level crate feature closure and complete transitive dependency set;
2. verify license/notice obligations for that complete selected closure;
3. inspect unsafe/FFI/native-code/build-script surfaces in that exact closure, including `memmap2`, `rustix`/`libc`, compression/hash implementation and any generated/native artifacts;
4. prove no selected crate feature or Golam adapter path can launch network clients, credential helpers, commands, hooks, filters, editors, signing helpers or other executables;
5. prove no selected adapter path performs ref/index/object/worktree mutation, lock acquisition for writes, tempfile-backed replacement or object writes;
6. define bounded repository-opening rules that ignore or fail closed on environment/config redirection (`GIT_DIR`, worktree/index/object alternates/config/include/helper/filter/hook redirection and equivalents);
7. decide whether Golam-owned HEAD/ref/packed-refs parsing fully replaces `gix-ref`; if not, qualify the exact `gix-ref` read surface separately;
8. implement a narrow read-only Golam adapter exposing only bounded repository identity, HEAD/ref, status, diff, log, tree and blob evidence;
9. add fixtures for loose and packed objects, supported SHA-1/SHA-256 repositories, detached/symbolic HEAD, loose/packed refs, index/worktree disagreement, malformed repositories, object alternates/config/env redirection, path escapes and bounded-resource failure;
10. run exact-head Windows/macOS/Ubuntu CI on the exact dependency/adapter head;
11. obtain substantive independent semantic/security review of the exact admitted source/features/adapter head;
12. update this record to `ADMITTED` only after every gate above is evidenced.

Until then:

```text
T005_040=BLOCKED_ON_EXACT_GIT_READ_SOURCE_QUALIFICATION
TOP_LEVEL_GIX_DIRECT_DEPENDENCY=REJECTED_TOO_BROAD
GIX_ODB_0_84_0_ADMITTED=NO
GIX_INDEX_0_55_0_ADMITTED=NO
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

A fully Golam-owned Git parser remains eligible if it can satisfy the same object/index/ref/pack/status/diff/log/tree/blob behavior and adversarial gates without external source reuse. The current preferred research direction is the narrower lower-level gitoxide plumbing candidate above because it removes the top-level `gix` protocol/config/revision convenience surface while retaining mature pack/index/object decoding. It is still only a candidate and cannot enter `Cargo.toml` until exact Source Foundry admission closes.