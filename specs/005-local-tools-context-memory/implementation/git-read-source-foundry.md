# T005-040 — In-Process Git Read Source Foundry Candidate

**Status**: `CANDIDATE_UNDER_QUALIFICATION_NOT_ADMITTED`

**Task**: T005-040 bounded Git read evidence

## Why a source qualification is required

T005-040 requires repository identity plus bounded HEAD/ref, status, diff, log, tree and blob observation without mutation authority. Canonical Spec 005 also keeps production native execution at `native:unqualified`, so invoking an external `git` executable is not an eligible Phase D shortcut. Implementing complete Git object/index/ref semantics in-process therefore requires either a Golam-owned implementation or an exact admitted Rust source/dependency surface.

The existing Golam-owned filesystem primitives remain suitable for authorized-root, protected-resource, alias and bounded worktree observation, but they do not themselves decode Git object databases, packfiles or indexes.

## Candidate source

```text
SOURCE=GitoxideLabs/gitoxide
CRATE=gix
VERSION=0.87.0
TAG=gix-v0.87.0
TAG_OBJECT=232c758b33a1d5158a54dc487f41db577fd78596
SOURCE_COMMIT=0c541c7308aee674110dc4dbd2ccda6dceaf41e6
LICENSE=MIT OR Apache-2.0
RUST_VERSION=1.85
CURRENT_ADMISSION=NOT_ADMITTED
```

The exact `gix` manifest at this source commit identifies the crate as `gix 0.87.0`, license `MIT OR Apache-2.0`, Rust 2024 edition and MSRV 1.85.

## Proposed minimal feature posture

No dependency change is authorized by this candidate record. If later qualification closes cleanly, the intended starting posture is:

```text
default-features = false
candidate features = ["index", "revision", "sha1", "sha256"]
```

The purpose of this restricted surface is read-only repository discovery/object/ref/index access needed to implement bounded Golam-owned status/diff/log/tree/blob evidence while avoiding optional process and network facilities.

The following `gix` feature families are explicitly outside the candidate surface unless separately requalified:

```text
command = DENIED
blocking-network-client* = DENIED
async-network-client* = DENIED
credentials = DENIED
worktree-mutation = DENIED
notes = DENIED
merge = DENIED
blocking-http-* = DENIED
```

The upstream `status` convenience feature is **not** admitted by this record because its current feature closure includes attributes-related surfaces that can enable command support. Golam may instead compose read-only status evidence from the admitted minimal index/object/ref surface plus Golam-owned bounded filesystem observation, but that design must compile and pass adversarial qualification before admission.

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

This record is intentionally fail-closed. The following gates are incomplete and therefore the dependency is **not admitted**:

1. capture the exact selected crate feature closure and complete transitive dependency set for `gix 0.87.0` under the proposed features;
2. verify license/notice obligations for the complete selected transitive closure;
3. inspect unsafe/FFI/native-code/build-script surfaces in that exact closure;
4. prove the selected feature closure contains no network client, credential-helper, command/process, hook/filter executable, editor/signing helper or mutation path reachable by the Golam adapter;
5. define repository-opening options that ignore or fail closed on environment/config redirection capable of widening filesystem/process/network authority;
6. implement a read-only Golam adapter that exposes only bounded repository identity, HEAD/ref, status, diff, log, tree and blob evidence;
7. add fixtures for loose and packed objects, SHA-1/SHA-256 repositories as supported, detached/symbolic HEAD, packed refs, index/worktree disagreement, malformed repositories, path escapes, hostile config/environment and bounded-resource failure;
8. run exact-head Windows/macOS/Ubuntu CI on the candidate dependency and adapter;
9. obtain substantive independent semantic/security review of the exact admitted source/feature/adapter head;
10. update this record to `ADMITTED` only after every gate above is evidenced.

Until then:

```text
T005_040=BLOCKED_ON_EXACT_GIT_READ_SOURCE_QUALIFICATION
GIX_0_87_0_ADMITTED=NO
GIT_CHILD_PROCESS_PHASE_D=DENIED_NATIVE_UNQUALIFIED
GIT_NETWORK_PHASE_D=DENIED
GIT_MUTATION_T005_040=DENIED
NEW_DEPENDENCY_ADDED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
WAIVER_TAKEN=NO
```

## Alternatives

A Golam-owned complete Git parser remains eligible if it can satisfy the same object/index/ref/pack/status/diff/log/tree/blob behavior and adversarial gates without external source reuse. A smaller exact set of gitoxide plumbing crates may also replace the top-level `gix` candidate if dependency-closure analysis proves a materially narrower and safer surface. Either alternative requires its own exact Source Foundry disposition before implementation reuse or dependency addition.
