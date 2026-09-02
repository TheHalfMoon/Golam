# T005-040 — Bounded Git Read Format Profile

**Status**: `FORMAT_AND_RESOURCE_PROFILE_FROZEN_PRIMITIVES_NOT_ADMITTED`

**Task**: T005-040 bounded Git read evidence

**Exact repository head before this profile**: `ffe06b7a2129cc10459ac40e7263bb2dd4f272b7`

**Exact repository CI before this profile**: CI #832 / run `33582476418` — `SUCCESS`

## Purpose

This record freezes the first implementation profile for the Golam-owned read-only Git parser before any hash or decompression primitive is admitted and before any Cargo dependency is changed.

It narrows scope deliberately. Unsupported repository formats fail closed with explicit evidence; they do not trigger an external `git` process, network access, ambient Git redirection, or an automatic dependency fallback.

This profile is subordinate to the constitution, canonical Spec 005 contracts/tasks, `git-read-source-foundry.md`, and `git-read-source-foundry-closure.md`.

## Normative first-profile support matrix

```text
REPOSITORY_STORAGE=DOT_GIT_DIRECTORY_ONLY
GITFILE_WORKTREE_INDIRECTION=UNSUPPORTED_FAIL_CLOSED
OBJECT_FORMAT=SHA1_ONLY
SHA256_REPOSITORY=UNSUPPORTED_FAIL_CLOSED
INDEX_V2=SUPPORTED
INDEX_V3=SUPPORTED
INDEX_V4=UNSUPPORTED_FAIL_CLOSED
PACK_VERSION_2=SUPPORTED_CANDIDATE
PACK_VERSION_3=UNSUPPORTED_FAIL_CLOSED
PACK_INDEX_V2=SUPPORTED_CANDIDATE
PACK_INDEX_V1=UNSUPPORTED_FAIL_CLOSED
MULTI_PACK_INDEX=UNSUPPORTED_FAIL_CLOSED
REVERSE_INDEX=NOT_REQUIRED
CRUFT_PACK_METADATA=NOT_REQUIRED
ALTERNATE_OBJECT_DATABASES=DENIED
REPLACE_REFS=DENIED
GRAFTS=DENIED
EXTERNAL_DIFF=DENIED
TEXTCONV_FILTERS=DENIED
SUBMODULE_RECURSION=DENIED_BY_DEFAULT
```

The first profile therefore requires only SHA-1 object identity. SHA-256 support is a later bounded extension of T005-040 or a follow-on implementation refinement and requires its own exact primitive qualification. Encountering a repository whose local repository-format metadata selects SHA-256 must produce `UNSUPPORTED_OBJECT_FORMAT`; it must not silently hash with SHA-1.

## Repository identity and opening

Repository discovery starts from an already authorized Golam root/target identity. The parser may examine only the literal `.git` directory under that authorized repository root in the first profile.

The following do not participate in authority or discovery and must be ignored or rejected rather than honored:

- `GIT_DIR`;
- `GIT_WORK_TREE`;
- `GIT_INDEX_FILE`;
- `GIT_OBJECT_DIRECTORY`;
- `GIT_ALTERNATE_OBJECT_DIRECTORIES`;
- `GIT_CEILING_DIRECTORIES`;
- `GIT_DISCOVERY_ACROSS_FILESYSTEM`;
- `GIT_CONFIG_COUNT`, `GIT_CONFIG_KEY_*`, `GIT_CONFIG_VALUE_*`;
- system/global config redirection;
- repository config includes or `includeIf` expansion outside the bounded local parser allowlist.

Only the minimum local repository-format keys required to identify supported storage may be read from `.git/config`; no configured command/helper/filter/hook/credential/remote/editor/signer is executed.

The repository identity evidence must bind at least:

1. authorized repository-root `ResolvedTargetIdentity`;
2. resolved `.git` directory identity;
3. detected object format;
4. exact observed HEAD representation and resolved ref/object id when available;
5. index identity/digest when status/diff uses the index;
6. object-store directory identity;
7. observation timestamp and configured caps.

## HEAD and refs

Supported:

- detached `HEAD` containing a canonical 40-hex SHA-1 object id;
- symbolic `HEAD` of the form `ref: refs/...` after bounded ref-name validation;
- loose refs under `.git/refs/`;
- `packed-refs` under bounded line/count/byte limits.

Denied or unsupported in the first profile:

- ref paths escaping `.git`;
- symbolic-ref cycles;
- malformed or non-canonical object ids;
- replace refs affecting object identity;
- reflog-derived authority;
- hooks or helpers of any kind.

A ref disagreement or malformed source is an evidence error, never a reason to invoke Git.

## Index profile

Primary format reference: Git `gitformat-index` / `index-format` documentation.

The first profile supports index versions 2 and 3 only. Version 4 path compression is intentionally deferred.

Optional extensions whose signature begins with `A` through `Z` may be skipped only after validating their declared length fits completely inside the bounded index buffer. Any extension whose first signature byte is outside `A` through `Z` is treated as mandatory/semantic and fails closed unless explicitly supported by this profile.

Explicitly unsupported/fail-closed index semantics include:

- sparse-directory (`sdir`) index entries;
- split-index/link semantics;
- unknown mandatory extensions;
- truncated extension payloads;
- path entries containing NUL or invalid traversal semantics;
- checksum mismatch.

```text
MAX_INDEX_BYTES=67108864
MAX_INDEX_ENTRIES=200000
MAX_INDEX_PATH_BYTES=4096
MAX_INDEX_EXTENSIONS=64
```

Index stat metadata is observation evidence only. It does not grant filesystem authority and cannot override live authorized-root resolution.

## Loose-object profile

Loose objects are addressed only by canonical 40-hex SHA-1 ids under the bounded `.git/objects/xx/yyyy...` path derived by Golam code. No alternate object directory is consulted.

The parser must:

1. bound compressed input bytes before decompression;
2. bound decompressed bytes during streaming decode rather than after allocation;
3. validate the canonical `<type> <size>\0` object header;
4. require declared size to equal actual decompressed payload size;
5. recompute the SHA-1 object id over canonical object bytes and require equality with the requested id;
6. accept only `blob`, `tree`, `commit`, and `tag` types required by this task.

```text
MAX_LOOSE_COMPRESSED_BYTES=67108864
MAX_SINGLE_OBJECT_DECOMPRESSED_BYTES=33554432
MAX_OBJECT_HEADER_BYTES=256
```

Objects exceeding a bound return explicit bounded/unavailable evidence and never cause an unbounded allocation or fallback process.

## Pack and pack-index profile

Primary format reference: Git `gitformat-pack` / `pack-format` documentation.

The first profile targets pack format version 2 and pack-index version 2 only. Git documents pack versions 2 and 3 as accepted while modern Git generates version 2; version 3 is deliberately outside this first parser profile.

The decoder must support ordinary objects plus `OBJ_OFS_DELTA` and `OBJ_REF_DELTA` only under explicit delta/resource limits. It must verify pack/index structure and checksums before trusting offsets as evidence.

```text
MAX_PACK_BYTES=1073741824
MAX_PACK_INDEX_BYTES=134217728
MAX_PACK_OBJECTS=500000
MAX_DELTA_DEPTH=64
MAX_SINGLE_OBJECT_DECOMPRESSED_BYTES=33554432
MAX_OPERATION_DECOMPRESSED_BYTES=268435456
MAX_PACKS_PER_OPERATION=64
```

A pack or object that exceeds a bound is not partially trusted. The operation returns bounded failure/missing evidence.

Multi-pack-index is intentionally unsupported in the first profile. If object discovery would require MIDX-only semantics, Golam reports unsupported evidence rather than invoking `git` or scanning without bounds.

## Commit/tree/blob/tag decoding

Golam-owned decoders must consume bounded canonical object bytes and expose only evidence needed by T005-040.

- commit: tree id, parent ids, bounded author/committer metadata, bounded message bytes;
- tree: bounded mode/name/object-id entries with path-component validation;
- blob: bounded exact bytes/digest identity, without textconv/filter execution;
- tag: bounded target/type/name/tagger/message fields where needed to resolve an explicitly observed ref.

Object parser output is non-authoritative. It cannot mint capabilities, approvals, verification authority, or Effect Gate state.

## Status, diff, log and tree observation bounds

Status is composed by Golam from exact HEAD tree + supported index + bounded worktree observations. It does not use Git porcelain and does not execute attributes, filters, textconv, hooks, submodule commands, or external diff drivers.

Diff/log/tree operations must retain the exact repository/HEAD/index/object identities from which evidence was derived.

```text
MAX_STATUS_PATHS=200000
MAX_DIFF_PATHS=10000
MAX_DIFF_TOTAL_BLOB_BYTES=67108864
MAX_LOG_COMMITS=2048
MAX_TREE_ENTRIES_PER_OBJECT=200000
MAX_OBJECT_READS_PER_OPERATION=200000
DEFAULT_GIT_READ_TIME_BUDGET_MS=10000
MAX_GIT_READ_TIME_BUDGET_MS=60000
```

Reaching a cap produces explicit insufficient/bounded evidence so the Context Compiler can report missing requirements or replan within its own bounded policy. It never authorizes recursive unbounded retrieval.

## Shallow repositories and partial/promisor state

A present `.git/shallow` file is permitted only as an explicit history boundary. Log evidence must identify that history is shallow and must not claim completeness beyond the boundary.

Promisor/partial-clone missing objects are never fetched in T005-040. A missing object produces local missing-evidence state. Network fetch is denied.

## Primitive qualification frontier

The Git parser itself remains Golam-owned. External code, if admitted, is limited to primitive decompression/hash support with no repository/object-store abstraction.

Current candidate observations:

### Decompression

`Frommi/miniz_oxide` is the preferred primitive to qualify next because upstream describes it as a pure-Rust miniz/DEFLATE/zlib replacement using no unsafe code. The exact inspected upstream master state is:

```text
SOURCE_REPOSITORY=Frommi/miniz_oxide
SOURCE_COMMIT=e2214d401a59e91537838cc16eba82454044044f
SOURCE_TREE=60b235aa935a227fbc14780ec20ace3bfe3a1df5
CRATE=miniz_oxide
CRATE_VERSION_AT_SOURCE=0.9.1
LICENSE=MIT OR Zlib OR Apache-2.0
NORMAL_REQUIRED_DEPENDENCY=adler2 2.0 (default-features=false)
DEFAULT_FEATURE=with-alloc
OPTIONAL_NOT_SELECTED=simd,serde,rustc-dep-of-std
```

This is a **candidate only**, not an admission. Exact published-crate/source equivalence, `adler2` source/license/unsafe/build-script closure, selected API behavior under bounded streaming decode, and independent verification are still required.

### Hashing

The first profile needs SHA-1 only. Do not add SHA-256 merely for future compatibility. A minimal exact SHA-1 primitive must be Source-Foundry-qualified separately, including source/package equivalence, dependencies, optional CPU/assembly/native features, unsafe/build-script behavior, license/notices, and cross-platform CI.

SHA-256 remains `DEFERRED_UNSUPPORTED_OBJECT_FORMAT` for this first profile.

```text
MINIZ_OXIDE_0_9_1_ADMITTED=NO
ADLER2_2_X_ADMITTED=NO
SHA1_PRIMITIVE_SELECTED=NO
SHA256_PRIMITIVE_SELECTED=NO
NEW_DEPENDENCY_ADDED=NO
```

## Adversarial requirements created by this profile

T005-040 qualification must include fixtures for at least:

- detached and symbolic HEAD;
- loose and packed refs;
- ref cycles/escape attempts;
- index v2 and v3;
- index v4 rejection;
- unknown mandatory index extension rejection;
- sparse/split index rejection;
- checksum mismatch/truncation/declared-length overflow;
- loose object valid/corrupt/truncated/decompression-bomb/size-mismatch/hash-mismatch cases;
- pack v2 + idx v2 ordinary objects;
- OFS_DELTA and REF_DELTA within and beyond depth/size limits;
- pack/index checksum or offset corruption;
- pack v3 rejection;
- MIDX-only discovery rejection;
- SHA-256 repository rejection without fallback;
- shallow history boundary evidence;
- missing promisor object with proof of no network fetch;
- ambient Git environment/config redirection denial;
- hook/filter/textconv/credential/remote/helper non-execution;
- worktree/index/HEAD disagreement;
- path aliases escaping the authorized root.

## Current disposition

```text
T005_040=BLOCKED_ON_PRIMITIVE_SOURCE_QUALIFICATION_AND_PARSER_IMPLEMENTATION
FORMAT_PROFILE_FROZEN=YES
FIRST_PROFILE_OBJECT_FORMAT=SHA1_ONLY
SHA256_REPOSITORY_SUPPORT=NO_FAIL_CLOSED
GIT_CHILD_PROCESS_PHASE_D=DENIED_NATIVE_UNQUALIFIED
GIT_NETWORK_PHASE_D=DENIED
GIT_MUTATION_T005_040=DENIED
MINIZ_OXIDE_0_9_1_ADMITTED=NO
SHA1_PRIMITIVE_SELECTED=NO
NEW_DEPENDENCY_ADDED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
WAIVER_TAKEN=NO
NEXT_UNIT=QUALIFY_EXACT_DECOMPRESSION_AND_SHA1_PRIMITIVES
```
