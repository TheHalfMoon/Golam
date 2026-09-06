# Windows Opened-Handle Identity Source Foundry — Spec 005

Status: `RESEARCH_ACTIVE_NO_CANDIDATE_ADMITTED`

Scope: bounded Source Foundry evidence for the Windows branch of T005-035/T005-036 after independent review proved that pathname `FILE_FLAG_OPEN_REPARSE_POINT` plus ordinary metadata does not bind the opened Windows handle to the exact authorized target.

This record does **not** admit a dependency, modify Cargo state, widen authority, enable mutation/process/network behavior, or change T005-040 Git-read admission state.

## Required property

The Windows read boundary must satisfy all of the following before the first content byte can be consumed:

```text
PATH_STRING != TARGET_IDENTITY
ANCESTOR_REPLACEMENT != NAMESPACE_REDIRECTION
FINAL_COMPONENT_REPLACEMENT != ACCEPTED_TARGET
STRICT_LOCAL_OPEN != REMOTE_RECALL
OPENED_FILE_HANDLE_IDENTITY == AUTHORIZED_TARGET_IDENTITY
```

Failure to prove any property fails closed. Unsupported filesystem identity, reparse/junction ambiguity, remote/offline recall risk, identity mismatch, or inability to reconcile the current path after the read is denial, not degraded success.

## Stable Rust disposition

Golam remains pinned to stable Rust `1.98.0` and keeps `unsafe_code = "forbid"`; production implementation files also use `#![forbid(unsafe_code)]`.

Rust 1.98 exposes Windows `MetadataExt::volume_serial_number()` and `file_index()` only behind the unstable `windows_by_handle` feature, and those fields are the legacy 64-bit `BY_HANDLE_FILE_INFORMATION` identity rather than universal `FILE_ID_INFO` 128-bit identity.

```text
STABLE_STD_HIGH_RES_WINDOWS_FILE_ID=UNAVAILABLE
NIGHTLY_FEATURE=REJECTED
LOCAL_UNSAFE_FFI=REJECTED
```

A qualified safe dependency boundary is required for Windows high-resolution opened-handle identity.

## Rejected candidate: `same-file 1.0.6`

Exact package:

```text
same-file 1.0.6
checksum=93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502
license=Unlicense OR MIT
```

Its Windows identity is the volume serial plus legacy 64-bit file index. Upstream documents that this is not universally sufficient on ReFS. Existing transitive presence through `walkdir` is not admission for a security primitive.

```text
same-file 1.0.6=REJECT_FOR_SECURITY_BOUNDARY
```

## Rejected candidate: `fence-windows =0.1.0-alpha.2`

Registry/provenance evidence previously verified:

```text
name=fence-windows
version=0.1.0-alpha.2
crates_io_checksum=cc1ddbe5ac1425cc6672799c6c1d9e85a9dd2c8e3647a5309218959c4a808be8
yanked=false
published=2026-08-03T13:00:26Z
rust_version=1.85
license=MIT OR Apache-2.0
upstream_tag=v0.1.0-alpha.2
tag_object=c36bb7966b0f359b85c3f4c9fda38c215354f612
tag_commit=bfab04dee14a57e166301b4ca8984e6311e9d9b9
filesystem_rs_blob=04844ea7b7034fdf8d0d54e43c26a47b01870902
```

Positive properties remain real:

- `FileIdentity { volume_serial: u64, file_id: [u8; 16] }` comes from Windows `FileIdInfo`;
- directory enumeration supplies 128-bit child IDs;
- an opened child is identity-attested before `NodeHandle` is returned;
- `NodeHandle::try_clone_file` duplicates the already-attested handle;
- the Golam caller surface can remain safe Rust.

However, two properties make this exact release inadmissible for Golam's strict-local read boundary.

### Blocker 1 — pathname child open after enumeration

`DirectoryHandle::open_child` enumerates from the retained directory handle, but then constructs `self.path.join_name(...)` and calls `CreateFileW` by pathname. The retained directory handle is **not** the namespace root of that open. `FILE_FLAG_OPEN_REPARSE_POINT` protects only the final opened component.

The later 128-bit identity comparison can stop bytes from being returned from a mismatched node, but it does not prevent the unintended node from being opened in the first place after an ancestor replacement.

### Blocker 2 — strict-local remote recall can occur at open time

Windows hierarchical/cloud storage can mark files with `FILE_ATTRIBUTE_RECALL_ON_OPEN`, `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS`, or `FILE_ATTRIBUTE_OFFLINE`. The Windows native API provides `FILE_OPEN_NO_RECALL` specifically to instruct offline-storage/virtualization filters not to recall file content because of the open.

`fence-windows 0.1.0-alpha.2` exposes recall-related attributes through its metadata model, but regular-file `open_child` requests `GENERIC_READ | FILE_READ_ATTRIBUTES` and does not set `FILE_OPEN_NO_RECALL` before the identity verdict.

A parent race can therefore redirect the pathname open to an offline/cloud-backed object and cause OS/filesystem-filter network recall before Golam can reject the identity. “No socket code in the crate” is not proof of zero egress because the filesystem open itself may trigger retrieval.

Disposition:

```text
fence-windows 0.1.0-alpha.2=REJECT_FOR_CURRENT_STRICT_LOCAL_READ_BOUNDARY
FENCE_IDENTITY_MODEL=USEFUL_REFERENCE_ONLY
```

A future Fence release may be reconsidered only through a fresh exact Source Foundry if it exposes a bounded handle-relative/no-recall read path that preserves stable opened-handle identity.

## New namespace/no-recall candidate: `fs_at =0.2.1`

Exact upstream release evidence under evaluation:

```text
name=fs_at
version=0.2.1
crates_io_checksum=14af6c9694ea25db25baa2a1788703b9e7c6648dcaeeebeb98f7561b5384c036
upstream_commit=e8b58a0682496a0c6ddc9eae80942a2f29a5a7e4
commit_message=Release 0.2.1
published_source_win_rs_blob=6153e4abf44f7c6b795ee0ad8b34a8b6fc2c66bb
license=Apache-2.0
rust_version=1.71.0
default_features=[]
```

The exact manifest declares common `cfg-if` and `cvt`; on Windows it declares `aligned` and `windows-sys 0.52.0` with WDK/Win32 filesystem/security/I/O metadata. Optional `log` and `workaround-procmon` features are not selected by default.

`workaround-procmon` is outside Golam authority and MUST remain disabled because its error-masking behavior is not acceptable for a fail-closed security primitive.

### Material positive property — native handle-relative open

On Windows `fs_at` implements `open_at` using `NtCreateFile` and sets:

```text
OBJECT_ATTRIBUTES.RootDirectory = already_open_parent_handle
OBJECT_ATTRIBUTES.ObjectName = relative_child_name
```

This is the namespace property the rejected Fence release lacked. Renaming or replacing the pathname that originally named the parent cannot redirect the child lookup away from the already-open parent handle.

### Material positive property — caller-controlled no-recall create option

The Windows `OpenOptionsExt::create_options` surface allows the caller to supply the native `NtCreateFile` create options. Golam's permitted profile would require the exact combination needed for the operation, including:

```text
FILE_OPEN_REPARSE_POINT
FILE_OPEN_NO_RECALL
```

`fs_at` itself adds its synchronous-I/O option. No default feature may silently replace this profile.

For metadata-only target observation, `open_path_at` uses only `SYNCHRONIZE` plus explicitly requested access and performs the open relative to the retained parent. For content reads, `open_at` can open relative to the same parent and Golam can read from that **same returned File handle**; no global pathname reopen is required after authorization.

### Remaining blocker — high-resolution identity of the same opened handle

`fs_at 0.2.1` does not expose a safe `FILE_ID_INFO`/128-bit identity query for an arbitrary `std::fs::File` returned by `open_at`/`open_path_at`.

Its internal directory iterator uses `FILE_ID_BOTH_DIR_INFO`, which is not a replacement for `FILE_ID_INFO` 128-bit opened-handle identity. Stable Rust cannot supply the missing high-resolution identity, and popular `file-id` / `win-file-id` APIs are path-based and therefore would reintroduce namespace races if used as the equality primitive.

Accordingly:

```text
fs_at 0.2.1=NAMESPACE_AND_NO_RECALL_CANDIDATE_ONLY
OPENED_HANDLE_FILE_ID_128=UNRESOLVED
DEPENDENCY_ADMITTED=NO
```

No combination is admitted until a safe exact primitive can derive `FILE_ID_INFO { volume serial, 128-bit file id }` from the **same already-open handle**, or independent review proves and canonical contracts explicitly accept an equally strong deterministic identity strategy.

## Required final Windows architecture

Any admitted design must provide this shape without local unsafe code:

```text
AUTHORIZED_ROOT_HANDLE
  -> HANDLE_RELATIVE_COMPONENT_TRAVERSAL
  -> NO_FOLLOW / REPARSE_DENIAL
  -> NO_RECALL OPEN POLICY
  -> SAME OPENED HANDLE RETAINED FOR READ
  -> FILE_ID_INFO_128 FROM THAT HANDLE
  -> DOMAIN_SEPARATED ResolvedTargetIdentity BINDING
  -> READ FROM THAT SAME HANDLE
  -> POST_READ HANDLE ID + CURRENT_PATH RECONCILIATION
```

The target handle itself must be the continuity anchor. A path-based high-resolution ID lookup performed after opening is not equivalent.

## Candidate search rules

A companion identity dependency, replacement crate, or later upstream release is eligible only if all of the following are proven:

1. safe Rust public surface at the Golam call site;
2. exact package/version/checksum/VCS/license/MSRV provenance;
3. obtains `FILE_ID_INFO`-equivalent 128-bit identity from an already-open handle without reopening by pathname;
4. no process, network, credential, telemetry, mutation, environment-dependent error bypass, or unbounded fallback is selected;
5. exact target-only feature/transitive closure is frozen;
6. unsupported filesystems fail closed rather than silently downgrade to ordinary metadata or 64-bit identity;
7. local Golam code remains `#![forbid(unsafe_code)]`;
8. independent review explicitly accepts the exact composition.

## Mandatory implementation qualification after future admission

- exact lockfile and selected-feature/transitive closure inspection;
- Windows parent-directory rename/replacement test proving handle-relative traversal stays pinned;
- final-component replacement test proving identity mismatch is detected;
- reparse/junction test;
- cloud/offline/recall posture test or an equivalent deterministic OS-level proof that `FILE_OPEN_NO_RECALL` is applied before any data-capable open;
- two distinct files with matched ordinary size/timestamp metadata must remain distinguishable by 128-bit opened-handle identity;
- proof that returned bytes come from the same retained target handle used for identity evidence, not a pathname reopen;
- post-read current-path/identity reconciliation;
- full focused tests plus official exact-head Windows/macOS/Ubuntu CI;
- fresh substantive independent semantic/security review on the exact implementation head;
- every material review thread resolved.

## Current decision

```text
WINDOWS_OPENED_HANDLE_IDENTITY_CANDIDATE=NONE_ADMITTED
fence-windows_0_1_0_alpha_2=REJECT_STRICT_LOCAL_RECALL_AND_PATHNAME_OPEN
fs_at_0_2_1=NAMESPACE_NO_RECALL_CANDIDATE_PENDING_IDENTITY_COMPANION
OPENED_HANDLE_FILE_ID_128=BLOCKED_SOURCE_FOUNDRY
SOURCE_FOUNDRY_STATUS=RESEARCH_ACTIVE_NO_CANDIDATE_ADMITTED
DEPENDENCY_ADMITTED=NO
CARGO_MUTATED=NO
WINDOWS_PARENT_RACE_FINDING=OPEN
T005_040=NOT_ADMITTED
WAIVER_TAKEN=NO
```
