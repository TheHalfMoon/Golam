# Windows Opened-Handle Identity Source Foundry — Spec 005

Status: `CANDIDATE_REPAIRED_PENDING_EXACT_HEAD_QUALIFICATION_NOT_ADMITTED`

Scope: bounded repair evidence for the Windows branch of T005-035/T005-036 after the fresh exact-head independent review of PR #21 identified that pathname `FILE_FLAG_OPEN_REPARSE_POINT` plus timestamp/size metadata does not bind the opened Windows handle to the exact file resolved before open.

This document does **not** admit a dependency, modify Cargo state, widen authority, enable mutation/process/network behavior, or change T005-040 Git-read admission state.

## Triggering finding

At exact implementation head `522ac9a50f9b64fb69c5a0aaaea5571b83cb52d6`, official CI #867 / run `33628549223` completed SUCCESS on Windows, macOS and Ubuntu. Fresh CodeRabbit review then confirmed the Unix opened-handle identity gate but kept the Windows parent-replacement finding open because the current Windows `MetadataExt` material contains attributes, size and timestamps but no stable file identity. A replaced parent can therefore redirect the pathname open before the first read, and an outside-root file with matching ordinary metadata cannot be excluded by the current Windows check.

Required repair property:

```text
WINDOWS_OPENED_FILE_HANDLE_IDENTITY == PREOPEN_AUTHORIZED_TARGET_IDENTITY
```

The comparison MUST occur before the first content `Read::read`. Parent replacement, reparse replacement, identity mismatch, unsupported identity observation, or any inability to prove equality MUST fail closed without consuming or returning content bytes.

The already-completed independent review of the Unix branch established the relevant security standard for this finding: opening an unintended handle is not itself sufficient to violate the bounded read contract if stable opened-handle identity is checked before the first content read and mismatch prevents any outside bytes from being consumed or returned. Whether the Windows candidate below provides an equivalent stable identity gate remains subject to fresh independent review; this document does not pre-decide that admission question.

## Standard-library disposition

Rust toolchain authority remains exact stable Rust `1.98.0`.

Rust 1.98 exposes Windows `MetadataExt::volume_serial_number()` and `MetadataExt::file_index()` only behind the unstable `windows_by_handle` feature. Golam's workspace sets `unsafe_code = "forbid"`, and production implementation files additionally use `#![forbid(unsafe_code)]`.

Disposition:

```text
STABLE_STD_HIGH_RES_WINDOWS_FILE_ID=UNAVAILABLE
NIGHTLY_FEATURE=REJECTED
LOCAL_UNSAFE_FFI=REJECTED
```

A qualified safe wrapper is therefore required if Windows content reads remain supported.

## Rejected candidate: `same-file 1.0.6`

Exact crates.io package:

```text
same-file 1.0.6
checksum=93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502
upstream_tag=1.0.6
upstream_tag_object=2bcb146601f1aa991eeb5146f093237363e7ca0b
upstream_commit=5799cd323b8eefd17a089c950dac113f66c89c9e
license=Unlicense OR MIT
```

It is already present transitively in the current Windows dependency graph through `walkdir`, but transitive presence is not admission for a security primitive.

The Windows implementation compares the `BY_HANDLE_FILE_INFORMATION` volume serial plus 64-bit file index while handles remain open. Upstream explicitly documents that ReFS may use 128-bit file identifiers and that the 64-bit index is not universally sufficient as a unique identity.

Disposition:

```text
same-file 1.0.6 = REJECT_FOR_SECURITY_BOUNDARY
```

Reason: it does not provide the high-resolution Windows identity needed to close the review finding across the supported Windows filesystem boundary. Existing transitive presence does not change this decision.

## Candidate: `fence-windows =0.1.0-alpha.2`

Registry identity:

```text
name=fence-windows
version=0.1.0-alpha.2
crates_io_checksum=cc1ddbe5ac1425cc6672799c6c1d9e85a9dd2c8e3647a5309218959c4a808be8
yanked=false
published=2026-08-03T13:00:26Z
rust_version=1.85
license=MIT OR Apache-2.0
```

Upstream provenance:

```text
repository=https://github.com/22elix3r/fence
signed_tag=v0.1.0-alpha.2
tag_object=c36bb7966b0f359b85c3f4c9fda38c215354f612
tag_commit=bfab04dee14a57e166301b4ca8984e6311e9d9b9
tag_signature=VERIFIED_BY_GITHUB
```

Exact reviewed source file containing the needed primitive:

```text
crates/fence-windows/src/filesystem.rs
blob=04844ea7b7034fdf8d0d54e43c26a47b01870902
```

The candidate's public filesystem surface provides:

- `FileIdentity { volume_serial: u64, file_id: [u8; 16] }`, populated from Windows `FileIdInfo`;
- `NodeMetadata`, read from an open handle and carrying that 128-bit identity;
- `RootHandle::open`, which opens the selected root, obtains its final path, reopens that final path with no-follow semantics, and rejects an identity mismatch;
- `DirectoryHandle::entries`, which enumerates names and 128-bit child file IDs from the retained directory handle;
- `DirectoryHandle::open_child` / `open_named_child`, which enumerate from the retained directory handle, construct a pathname using the directory's stored path plus the selected child name, open that pathname with `FILE_FLAG_OPEN_REPARSE_POINT`, then reject `IdentityChanged` unless the opened child's volume/file identity matches the entry enumerated from the retained directory;
- `NodeHandle::try_clone_file`, which clones the already-identity-attested regular-file handle for streaming reads while retaining the original node handle for post-read verification;
- `NodeHandle::refresh_metadata` and `verify_path_identity` for post-observation reconciliation.

### Important upstream opening semantics

`fence-windows 0.1.0-alpha.2` does **not** implement a native Win32 directory-handle-relative child open. `DirectoryHandle::open_child` does not use the retained directory handle as the namespace root of `CreateFileW`; it constructs `self.path.join_name(...)` and performs a pathname open. `FILE_FLAG_OPEN_REPARSE_POINT` protects the final opened component, not replaced ancestors.

The security value of this candidate is therefore narrower and must not be overstated:

```text
PINNED_DIRECTORY_ENUMERATION
  -> EXPECTED_CHILD_FILE_ID_128
  -> PATHNAME_OPEN_NOFOLLOW_FINAL
  -> OPENED_HANDLE_FILE_ID_128
  -> REQUIRE_EQUALITY_BEFORE_ACCEPTING_NODE
  -> ONLY_THEN_CLONE/READ
```

A parent race may cause a pathname open to reach an unintended node. The candidate can be admissible for this finding only if the subsequent 128-bit identity comparison reliably rejects that node before any content read, so no outside-root bytes can be consumed or returned. An accepted intermediate directory becomes the next retained `DirectoryHandle` only after its opened identity matches the identity enumerated from the previously retained directory. This is **identity-attested component traversal**, not a native handle-relative namespace guarantee.

The fresh independent Source Foundry review must decide whether this identity-attested pathname-open model is sufficient for the existing Windows finding. This record does not claim that it is equivalent to `openat`/`openat2`-style directory-relative opening.

## Exact dependency closure

The crates.io record for `fence-windows 0.1.0-alpha.2` declares only these normal dependencies:

```text
thiserror ^2.0.19
windows-sys ^0.61.2, default-features=false, cfg(windows)
```

The exact current Golam lock/CI graph already resolves compatible versions:

```text
thiserror 2.0.20
windows-sys 0.61.2
```

The candidate itself is the only package expected to be newly added by an exact pin. Its Windows feature set requests WDK/Win32 filesystem, security, I/O, job-object, threading, COM and shell API metadata. Those feature modules do not create network/process/mutation authority by themselves, but they increase compiled API surface and therefore MUST be considered by independent review.

The crate has no package build script in its published manifest. Its Windows `tempfile` dependency is dev-only and is not part of Golam runtime closure.

## Candidate risk review

### Positive properties

- safe Rust API for Golam callers;
- high-resolution 128-bit `FileIdInfo` identity instead of the rejected 64-bit legacy index;
- retained-directory enumeration supplies an expected 128-bit child identity before a child is accepted;
- child pathname opens use final-component no-follow and are identity-checked before the returned `NodeHandle` is accepted;
- read handle is cloned from the already-identity-attested `NodeHandle` rather than reopened by pathname after attestation;
- exact signed upstream release and crates.io checksum are available;
- MSRV is below Golam's pinned Rust 1.98;
- no network activity is part of the filesystem primitive.

### Risks / constraints

- release is prerelease (`0.1.0-alpha.2`);
- package is recent and has a smaller maturity window than long-lived filesystem crates;
- child opens remain pathname based, so an unintended handle may be opened during an ancestor race before 128-bit identity attestation rejects it;
- crate contains additional public Windows mutation, ACL, reparse, job-object and system primitives outside Golam's required read-only surface;
- unsafe Win32 implementation exists inside the dependency even though Golam itself remains `unsafe_code=forbid`;
- the dependency requests a broader `windows-sys` feature set than the exact read-only filesystem calls Golam intends to use;
- dependency presence MUST NOT make any Fence mutation/process/system API admissible in Golam.

## Proposed Golam identity-contract integration

No new authority-bearing core field is proposed merely to expose a platform-specific file identifier. `ResolvedTargetIdentity.resolved_target_identity` is already an opaque `BindingDigest` intended to bind the exact resolved target identity.

If the candidate is independently admitted, the Windows implementation SHOULD preserve the public platform-neutral shape and change the Windows identity derivation so that the domain-separated `BindingDigest` includes the exact `FileIdentity { volume_serial, file_id[16] }` obtained from the accepted `NodeHandle`, plus only the deterministic identity material required by the existing contract. The same derivation must be used for pre-read resolution and opened-handle comparison.

The current Windows digest based only on pathname plus attributes/size/timestamps is explicitly insufficient and MUST NOT be reused as the security equality check for the repaired read boundary.

`observed_metadata_digest` remains distinct observation evidence. It may bind deterministic `NodeMetadata` fields from the same accepted handle, but it MUST NOT substitute for the stable 128-bit target-identity binding.

If independent review determines that an explicit core representation is required instead of an opaque digest, that is a Source Foundry/contract repair prerequisite and no dependency admission may occur until it is resolved.

## Proposed bounded admission

If and only if an independent exact-head Source Foundry review accepts this candidate, the permitted manifest change is limited to the Windows target of `golamd`:

```toml
[target.'cfg(windows)'.dependencies]
fence-windows = "=0.1.0-alpha.2"
```

No other Fence crate is admitted.

Permitted application API surface is limited to:

```text
RootHandle
DirectoryHandle
DirectoryEntry
NodeHandle
NodeKind
NodeMetadata
FileIdentity
WindowsError
```

Only read-only root opening/identity attestation, retained-directory enumeration, child open + identity attestation, identity observation, handle cloning for reads, and identity refresh/reconciliation are permitted. Mutation, restore, ACL modification, process/job execution, shell/system lookup, stream mutation, or any other Fence API is outside authority and MUST NOT be called.

## Required implementation shape after admission

On Windows only:

1. Open and identity-attest the authorized root with `RootHandle`; bind its 128-bit identity into the resolver's existing domain-separated root identity contract.
2. Normalize the requested target lexically as today; reject parent/root/prefix components.
3. For each child component, enumerate the selected name from the current retained `DirectoryHandle`, obtaining the expected 128-bit child file ID; call the candidate's `open_child`/`open_named_child`, understanding that upstream performs a pathname open and then requires the opened 128-bit identity to equal the enumerated identity.
4. Accept an intermediate node as the next retained directory only after the identity check succeeds and the node is an ordinary directory; convert that accepted node into the next `DirectoryHandle`.
5. Require the final accepted `NodeHandle` to be an ordinary file for content reads.
6. Derive the same domain-separated Windows `resolved_target_identity` binding from the final accepted node's `FileIdentity` that was produced during pre-read resolution, and require exact equality before any content read.
7. Clone the file from the accepted node using `try_clone_file`; do not perform another content-file pathname reopen after identity attestation.
8. Perform all stable identity and type checks before the first `Read::read`.
9. After bounded read completion, refresh metadata on the retained node handle and re-resolve/reconcile current path state. Any identity mismatch, unreadable state, or inability to re-establish the required contract fails closed.

Unix behavior remains unchanged because the fresh independent review accepted the current Unix dev+ino opened-handle binding and deterministic parent-race test.

## Mandatory qualification before the finding can close

- official exact-head Windows/macOS/Ubuntu CI after every branch mutation;
- `cargo fmt --check` and `clippy -D warnings`;
- full tests and existing qualification suites;
- deterministic Windows parent-directory replacement adversarial test proving no outside-root byte can be consumed or returned;
- Windows final-component replacement/reparse test;
- Windows 128-bit identity mismatch test using two distinct opened file identities even when ordinary size/timestamp metadata is made equal where the filesystem permits;
- explicit evidence that read bytes originate from the clone of the identity-attested `NodeHandle`, never from a pathname reopen after attestation;
- exact lockfile and selected-feature closure inspection after any Cargo mutation;
- fresh substantive independent semantic/security review on the exact implementation head;
- all material review threads resolved.

## Current decision

```text
WINDOWS_OPENED_HANDLE_IDENTITY_CANDIDATE=fence-windows 0.1.0-alpha.2
CANDIDATE_CHECKSUM=cc1ddbe5ac1425cc6672799c6c1d9e85a9dd2c8e3647a5309218959c4a808be8
SOURCE_PROVENANCE=VERIFIED
UPSTREAM_CHILD_OPEN=NATIVE_DIRECTORY_HANDLE_RELATIVE_NO
UPSTREAM_CHILD_OPEN=PATHNAME_OPEN_THEN_FILE_ID_128_ATTESTATION
SOURCE_FOUNDRY_STATUS=REPAIRED_PENDING_EXACT_HEAD_CI_AND_INDEPENDENT_REVIEW
DEPENDENCY_ADMITTED=NO
CARGO_MUTATED=NO
WINDOWS_PARENT_RACE_FINDING=OPEN_UNTIL_IMPLEMENTATION_QUALIFIES
T005_040=NOT_ADMITTED
```
