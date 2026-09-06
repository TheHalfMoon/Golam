# T005-040 — In-Process Git Read Source Foundry Candidate

> **SUPERSEDED FOR CURRENT CANDIDATE SELECTION**
>
> This record is retained as historical Source Foundry evidence. Its former `gix-pack` / `gix-object` / `gix-hash` preference is superseded by `git-read-source-foundry-closure.md`, `git-read-primitive-source-foundry.md`, and `git-read-primitive-admission-candidate.md`. The only current preferred direction is a Golam-owned bounded Git parser, Golam-owned SHA-1 for legacy Git object identity, and the separately qualified `miniz_oxide 0.9.1` + `adler2 2.0.1` decompression candidate. No dependency is admitted by this historical record.

**Status**: `SUPERSEDED_HISTORICAL_RESEARCH_NOT_ADMITTED`

**Task**: T005-040 bounded Git read evidence

## Why a source qualification is required

T005-040 requires repository identity plus bounded HEAD/ref, status, diff, log, tree and blob observation without mutation authority. Canonical Spec 005 also keeps production native execution at `native:unqualified`, so invoking an external `git` executable is not an eligible Phase D shortcut. Implementing Git object/index/ref semantics in-process therefore requires either a Golam-owned implementation or an exact admitted Rust source/dependency surface.

The existing Golam-owned filesystem primitives remain suitable for authorized-root, protected-resource, alias and bounded worktree observation, but they do not themselves decode Git object databases, packfiles or indexes.

Exact-head repository CI #829 / run `33570078764` completed `SUCCESS` on prior documentation head `01226b541709135cd69f990a60d36c0c5c776847`. Exact-head repository CI #830 / run `33573377833` also completed `SUCCESS` on prior documentation head `24527a712fffd3537c2baf8b9562afe4cd74a89b`. Those runs are historical documentation-only evidence and do not qualify any current dependency or later head.

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

The exact upstream manifest recommends `default-features = false` for library consumers and exposes optional network/process/mutation feature families. Exact manifest inspection nevertheless showed a materially broad **unconditional** dependency surface even when those optional features are disabled, including repository/config/ref/discovery/object/pack/revision/diff/protocol infrastructure:

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

`gix-protocol` is an unconditional top-level dependency even though concrete network transports remain feature-gated. The `revision` feature also implies `index`, while upstream `status` pulls `dirwalk` + `blob-diff`; `blob-diff` pulls `attributes`, and `attributes` enables the top-level `command` feature. Therefore the convenience status/attributes path was rejected for the Phase D read-only boundary.

```text
GIX_0_87_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_STATUS_FEATURE_ADMITTED=NO
GIX_COMMAND_FEATURE_ADMITTED=NO
GIX_NETWORK_FEATURES_ADMITTED=NO
```

## Historical lower-level candidate investigation

The lower-level set initially investigated was:

```text
gix-odb = 0.84.0
gix-index = 0.55.0
gix-object = 0.64.0
gix-hash = 0.26.1
```

All candidates refer to exact source commit `0c541c7308aee674110dc4dbd2ccda6dceaf41e6` unless explicitly superseded.

### `gix-odb 0.84.0` — rejected

Exact manifest and library inspection showed `gix-odb` is not a read-only object decoder. Its public `Store` supports loose object reading and writing and its normal closure includes tempfile, synchronization, mmap, zlib/pack and filesystem infrastructure. That is broader than T005-040.

```text
GIX_ODB_0_84_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_ODB_REJECTION=WRITE_CAPABLE_OBJECT_STORE_AND_BROAD_NORMAL_CLOSURE
```

### `gix-index 0.55.0` — rejected

The crate publicly compiles a write module and its normal dependency closure includes locking, filesystem, mmap and native-adjacent support. T005-040 needs bounded decoding only.

```text
GIX_INDEX_0_55_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_INDEX_REJECTION=UNCONDITIONAL_LOCK_WRITE_SURFACE_FOR_READ_ONLY_TASK
```

### `gix-pack 0.74.0` — historical candidate, later rejected

Earlier research observed that disabling defaults removed `generate` and `streaming-input`, making this surface narrower than `gix-odb`. That observation is retained only as history. Subsequent closure analysis in `git-read-source-foundry-closure.md` rejected `gix-pack` because its remaining unconditional closure still includes progress/libc-adjacent infrastructure, `memmap2`, object infrastructure and more surface than the owned parser requires.

Historical proposed posture was:

```text
gix-pack = { version = "=0.74.0", default-features = false, features = ["sha1", "sha256"] }
GIX_PACK_GENERATE_FEATURE=DENIED
GIX_PACK_STREAMING_INPUT_FEATURE=DENIED
GIX_PACK_PARALLEL_FEATURE=DENIED
GIX_PACK_WRITE_OR_GENERATE_PATH=T005_040_DENIED
```

Current disposition:

```text
GIX_PACK_0_74_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_PACK_CURRENT_CANDIDATE=NO
```

### `gix-object 0.64.0` — historical candidate, later rejected

The exact manifest makes command/tempfile behavior optional behind `signature`, but the crate still exposes mutable/encoding object APIs and a broader normal closure than the owned parser requires. The later closure record therefore rejected it as a direct T005-040 dependency.

```text
GIX_OBJECT_0_64_0_DIRECT_DEPENDENCY_ADMITTED=NO
GIX_OBJECT_CURRENT_CANDIDATE=NO
GIX_OBJECT_SIGNATURE_FEATURE=DENIED
GIX_OBJECT_COMMAND_PATH=DENIED
```

### `gix-hash 0.26.1` — historical candidate, not selected

`gix-hash` was investigated for identity support but remains unadmitted. The current first-profile direction uses Golam-owned SHA-1 restricted to legacy Git object identity.

```text
GIX_HASH_0_26_1_ADMITTED=NO
GIX_HASH_CURRENT_CANDIDATE=NO
GOLAM_OWNED_SHA1_CURRENT_DIRECTION=YES
```

### `gix-ref 0.67.0` — not selected

`gix-ref` remains unselected because its normal dependency surface includes lock/tempfile/mmap behavior. T005-040 uses a Golam-owned bounded parser for `HEAD`, loose refs and `packed-refs`.

## Current architecture that supersedes the historical hybrid

The historical hybrid boundary is **not** the current preference. The current architecture is:

1. Golam-owned repository-root / `.git` identity discovery under the already authorized root;
2. Golam-owned bounded `HEAD`, loose-ref and `packed-refs` parsing;
3. Golam-owned bounded index decoding for the frozen supported index versions/extensions;
4. Golam-owned loose-object/header, pack/index, commit/tree/blob/tag and delta parsing;
5. Golam-owned status, diff and bounded commit-parent traversal;
6. Golam-owned SHA-1 for legacy Git object identity only;
7. candidate external decompression closure limited to `miniz_oxide 0.9.1` + `adler2 2.0.1`, still NOT_ADMITTED;
8. every eventual synchronous inflate call gated by the Golam-owned 64 KiB input/output quantum and before/after monotonic deadline checks;
9. no general Git repository/object-store handle, child process, shell, hook, filter, credential helper, network transport, signer/verifier helper or mutation API.

See the superseding records for the exact primitive provenance, time-bound contract and current gate state.

## Authority and execution boundaries retained from this research

Any eventual Git read implementation MUST preserve:

- repository observation is evidence, never authority;
- repository discovery is bounded to the already authorized workspace root;
- `PATH_STRING != TARGET_IDENTITY`;
- protected Golam resources remain excluded from generic repository/filesystem inspection;
- no child process, shell, hook, credential helper, editor, signer/verifier helper or external executable may be launched;
- no network transport, fetch, push, clone or remote discovery is enabled;
- environment/config inputs that could redirect repository, object database, index, hooks, filters, helpers or executables are ignored or rejected by Golam-owned parsing;
- no Git mutation API is exposed by T005-040;
- force/history rewrite remains unavailable;
- all worktree reads remain subject to bounded byte/count/depth/time and target-identity rules;
- status/diff/log/tree/blob outputs retain repository/ref/object/worktree identity and observation provenance and cannot mint capability, approval or verification authority.

## Historical qualification list — superseded

The earlier list that proposed qualifying `gix-pack` / `gix-object` / `gix-hash` is no longer an active plan. Current qualification is controlled by the superseding closure/provenance/admission-candidate records and requires corrected exact-head CI + independent review before any primitive admission.

Until then:

```text
T005_040=BLOCKED_ON_CURRENT_PRIMITIVE_REQUALIFICATION_AND_IMPLEMENTATION
THIS_RECORD=CURRENT_SELECTION_SUPERSEDED_HISTORICAL_ONLY
TOP_LEVEL_GIX_DIRECT_DEPENDENCY=REJECTED_TOO_BROAD
GIX_ODB_0_84_0_ADMITTED=NO
GIX_INDEX_0_55_0_ADMITTED=NO
GIX_PACK_0_74_0_ADMITTED=NO
GIX_OBJECT_0_64_0_ADMITTED=NO
GIX_HASH_0_26_1_ADMITTED=NO
GIX_REF_0_67_0_ADMITTED=NO
MINIZ_OXIDE_0_9_1_ADMITTED=NO
ADLER2_2_0_1_ADMITTED=NO
GOLAM_OWNED_PARSER_CURRENT_DIRECTION=YES
GOLAM_OWNED_SHA1_CURRENT_DIRECTION=YES
GIT_CHILD_PROCESS_PHASE_D=DENIED_NATIVE_UNQUALIFIED
GIT_NETWORK_PHASE_D=DENIED
GIT_MUTATION_T005_040=DENIED
NEW_DEPENDENCY_ADDED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
WAIVER_TAKEN=NO
```
