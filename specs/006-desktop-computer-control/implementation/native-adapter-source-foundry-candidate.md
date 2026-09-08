---
task_scope: T006-016..T006-033 dependency precondition
status: CANDIDATE_PENDING_QUALIFICATION_AND_REVIEW
recorded_on: 2026-09-08
product_manifest_mutation: false
product_native_adapter_admission: false
waiver_taken: false
---

# Source Foundry candidate — Spec 006 native desktop adapter primitives

## Purpose

This record proposes exact platform-scoped dependencies for the native desktop adapters required by Spec 006. It is a Source Foundry candidate only. It does not admit dependencies into Golam product manifests, authorize platform dispatch, weaken repository lints, or change the constitutional route/authority model.

Qualification occurs in isolated scratch Cargo projects before product manifest mutation. Workflow success is evidence for independent review, not admission by itself.

## Immutable authority constraints

Candidate libraries provide platform mechanics only. They cannot:

- mint `FallbackEligibilityEvidence` or Kernel/Effect Gate authorization;
- own capability, policy, approval, control-lease, or visible-channel authority;
- convert observation, capture bytes, semantic metadata, clipboard content, or pixel hints into actuation authority;
- continue autonomous input after pause, stop, takeover, permission loss, unresolved `UNKNOWN_OUTCOME`, or qualified visible-channel loss;
- bypass Windows locked/UAC/secure-desktop boundaries or a Wayland compositor/XDG portal grant;
- introduce camera, microphone, audio capture, OCR, hidden HTTP/cloud/network fallback, keylogging, or background clipboard polling.

## Independent-review reconciliation history

Fresh review of exact head `b799b4e0892f4ac526f577d3f74aa8246c19fe2c` verified cross-platform CI and the candidate build, but returned:

```text
DO_NOT_ADMIT_SPEC006_NATIVE_ADAPTER_DEPENDENCIES
```

The review identified two material blockers:

1. license/notice evidence was based on Cargo metadata and did not bind exact published `.crate` archives or an equivalent checksum-bound source identity;
2. Linux clipboard enabled `arboard/wayland-data-control`, creating a direct Wayland data-control closure without a separately qualified session boundary.

Commit `2d8a4984a82b2b77bcbbb4f32406eef3b902284d` repaired the archive-checksum audit and removed the explicit `arboard/wayland-data-control` feature. Source Foundry run `34172872076` then exposed two further facts that must be reconciled before admission:

- several exact checksum-bound crates intentionally omit repository license text from their published archive even though they carry SPDX metadata and `.cargo_vcs_info.json`;
- `arboard 3.6.1` still pulled `clipboard_wayland 0.2.2` into the Linux host-active graph with `default-features = false`, so the candidate was not structurally X11-only.

This revision addresses both findings without weakening the gate.

## Checksum archive + exact VCS license evidence

Every host-active registry package remains bound to:

- its exact `Cargo.lock` source identity;
- its exact `Cargo.lock` checksum;
- one cached `.crate` archive whose SHA-256 equals that checksum;
- a deterministically selected allowed SPDX path.

When the checksum-bound archive contains a top-level `LICENSE`, `LICENCE`, `COPYING`, `NOTICE`, or `COPYRIGHT` file, the archive remains the license-evidence source.

When the exact published archive intentionally omits license text, Source Foundry may use exact-VCS fallback only for a hard-coded package allowlist already observed in failed run `34172872076`. The fallback must:

1. read the source commit SHA from the checksum-bound archive's own `.cargo_vcs_info.json`;
2. reject missing, malformed, or dirty VCS identity;
3. use a repository and license path hard-coded by Golam, not a URL supplied by package metadata;
4. fetch the license file at that exact VCS commit;
5. record the source commit, `path_in_vcs`, license path, byte count, and SHA-256;
6. require the selected SPDX path to match the allowlisted license family;
7. fail closed if any exact source license cannot be retrieved or verified.

Current allowlisted fallback families are limited to:

- `clipboard-win 5.4.1` → `DoumanAsh/clipboard-win` exact VCS `LICENSE`, `BSL-1.0`;
- `xa11y-core 0.13.0` and `xa11y-windows 0.13.0` → `xa11y/xa11y` exact VCS `LICENSE`, `MIT`;
- exact `objc2` packages observed in the macOS closure → `madsmtm/objc2` exact VCS `LICENSE-MIT.txt`, `MIT`;
- `libspa-sys 0.9.2` → `pipewire/pipewire-rs` exact VCS `LICENSE`, `MIT`.

This is qualification evidence only. Any product distribution must later preserve the applicable exact license/notice obligations in packaging evidence.

## Windows candidate

### Semantic automation — safe caller-facing UIA backend

```toml
xa11y-core = { version = "=0.13.0", default-features = false }
xa11y-windows = { version = "=0.13.0", default-features = false }
```

Selected reuse strategy: `ADAPTER`.

Source evidence:

- repository: `https://github.com/xa11y/xa11y`;
- exact crate line: `xa11y-windows 0.13.0` + `xa11y-core 0.13.0`;
- declared license: MIT;
- upstream architecture identifies `xa11y-windows` as the Windows UI Automation backend and `WindowsProvider` as its accessibility provider.

Rationale:

- caller-facing `WindowsProvider` lets Golam remain under the canonical safe-Rust lint policy while dependency-internal FFI/unsafe remains an explicit Source Foundry boundary;
- the semantic adapter may expose only bounded `DesktopBackend` observation/focus/semantic-action behavior;
- `xa11y-windows` also publishes `WindowsInputProvider` and `WindowsScreenshot`; those surfaces are explicitly **not admitted for the semantic role**. Raw input and capture retain separate Enigo/WGC authority and evidence paths;
- dependency presence cannot mint route eligibility, capability, approval, target identity, lease state, or actuation authority.

### Rejected Windows semantic candidates

`uiautomation 0.25.0` and `uiautomation 0.25.1` were rejected after the control-only candidate failed to compile when the default raw-input feature was denied. Golam did not enable the input feature merely to make the semantic candidate build.

Direct Golam-authored `windows 0.62.2` UIA call sites were also rejected under the canonical `unsafe_code = "forbid"` policy. The package may remain transitively present inside admitted dependencies, but that does not authorize Golam-authored unsafe Win32/COM call sites.

### Selected window/display capture

```toml
wgc = { version = "=1.0.7", default-features = false }
```

Optional `tracing` remains disabled. WGC's broader Windows projection closure grants no additional Golam authority. An admitted facade may expose only explicitly selected window/display capture and must not expose audio, storage, transcoding, cryptography, shell, or unrelated Windows APIs merely because they exist transitively.

### Explicit deterministic raw fallback

```toml
enigo = { version = "=0.6.1", default-features = false }
```

Raw input is a separate route and may run only with canonical fresh fallback eligibility, exact capability/policy/approval, current Effect Gate authorization, target/focus/session revalidation, current control-lease generation, and a qualified visible-control channel.

### Explicit text clipboard

```toml
arboard = { version = "=3.6.1", default-features = false }
```

`image-data` remains denied. Product use, if admitted, is one explicit bounded text read/write operation only; polling/background inspection is forbidden.

## macOS candidate

```toml
axuielement = { version = "=0.9.1", default-features = false }
screencapturekit = { version = "=10.0.3", default-features = false }
enigo = { version = "=0.6.1", default-features = false }
arboard = { version = "=3.6.1", default-features = false }
```

`axuielement/raw-ffi` and `async` remain disabled. Compatibility keyboard-event helpers are not part of Golam semantic dispatch. ScreenCaptureKit optional macOS-version features remain disabled; any later adapter must configure screen-only output with audio and microphone absent. Clipboard image features remain denied.

## Linux candidate

```toml
atspi = { version = "=0.30.0", default-features = false, features = ["proxies", "connection", "tokio"] }
ashpd = { version = "=0.13.13", default-features = false, features = ["tokio", "remote_desktop", "screencast"] }
pipewire = { version = "=0.9.2" }
enigo = { version = "=0.6.1", default-features = false, features = ["x11rb"] }
x11-clipboard = { version = "=0.9.3" }
```

AT-SPI is the semantic layer. XDG ScreenCast/RemoteDesktop is user/compositor-granted external authority only; denied `ashpd` features include camera, clipboard, `input_capture`, screenshot, background and unrelated portal surfaces. PipeWire is frame transport only for a granted ScreenCast session. Enigo is X11-only; Wayland/libei paths are denied.

### Linux clipboard decision

`arboard` is no longer a Linux candidate. Run `34172872076` proved that `arboard 3.6.1` introduced `clipboard_wayland 0.2.2` into the Linux host-active graph even with `default-features = false`, contradicting the intended X11-only boundary.

The replacement candidate is exact `x11-clipboard 0.9.3`:

- repository: `https://github.com/quininer/x11-clipboard`;
- declared license: MIT;
- the published crate ships a top-level `LICENSE`;
- the crate is X11-specific and depends on `libc` plus `x11rb` with XFixes support;
- it provides explicit clipboard load/store operations and has no Wayland feature path.

Pure Wayland clipboard remains `NotSupported` under Spec 006 until a separately qualified compositor/session-bound mechanism receives its own Source Foundry admission. No direct Wayland clipboard protocol, `clipboard_wayland`, `wl-clipboard-rs`, `wayland-data-control`, or `arboard` Linux path is admitted by this candidate.

## Candidate qualification workflow

`.github/workflows/spec006-native-adapter-source-foundry.yml` must independently qualify Windows, macOS, and Ubuntu and must:

1. create isolated scratch manifests so product manifests remain unchanged;
2. resolve exact candidate versions with Rust `1.98.0`;
3. filter metadata and dependency reachability to the actual host target;
4. record host triple, lock digest, dependency tree, and enabled feature tree;
5. compile exact aggregate closures on the matching platform;
6. on Windows, compile a `#![forbid(unsafe_code)]` caller probe that references `xa11y_windows::WindowsProvider`;
7. fail if denied features enter WGC/Enigo/arboard/macOS/Linux candidate paths;
8. on Linux, require `x11-clipboard 0.9.3` and fail if `arboard`, `clipboard_wayland`, `wl-clipboard-rs`, or `wayland-data-control` enters the active graph;
9. bind every host-active registry package to the exact `Cargo.lock` checksum and matching cached `.crate` archive SHA-256;
10. inspect checksum-bound archives for top-level license/notice files and deterministic SPDX path;
11. permit exact-VCS license fallback only for the hard-coded observed package families, using `.cargo_vcs_info.json` from the checksum-bound archive and a hard-coded repository/license path;
12. inventory active custom build scripts, native `links` declarations, and known HTTP-client packages;
13. surface third-party unsafe/FFI/native/build boundaries for independent review rather than translating dependency-internal unsafe into Golam-authored unsafe authority;
14. prove repository product manifests remain unchanged;
15. emit candidate-only status. Workflow success cannot itself change this record to admitted.

## Required independent review questions

A fresh exact-head security/governance review must answer at least:

- Does every host-active registry package have a unique checksum-bound `.crate` archive?
- For packages whose archive omits license text, is the exact-VCS fallback cryptographically tied to that archive's `.cargo_vcs_info.json` and restricted to the hard-coded source/license path?
- Are the selected SPDX paths and distribution obligations unambiguous?
- Does `xa11y-windows 0.13.0` provide a sufficiently bounded safe caller-facing UIA provider while preserving Golam's `unsafe_code = "forbid"` policy?
- Can Golam's semantic adapter be structurally limited so `WindowsInputProvider` and `WindowsScreenshot` remain unreachable from the semantic route?
- Does the broad WGC closure remain encapsulated behind selected window/display capture only?
- Does `axuielement` exclude `raw-ffi`, and can compatibility keyboard helpers remain unreachable from semantic dispatch?
- Can ScreenCaptureKit be configured deterministically as screen-only, audio-off, microphone-absent?
- Does Linux ScreenCast use only the user-granted portal PipeWire remote rather than ambient source enumeration?
- Can XDG RemoteDesktop remain user/compositor-granted without camera, clipboard, `input_capture`, or unrelated portal surfaces?
- Is X11 input explicitly session-scoped while Wayland input bypass remains impossible?
- Is Linux clipboard structurally X11-only with `x11-clipboard 0.9.3`, while pure Wayland clipboard fails closed as `NotSupported`?
- Does any active candidate introduce hidden HTTP/cloud/network-client behavior or product runtime network authority?

## Explicit non-admissions

```text
WINDOWS_NATIVE_ADAPTERS=NOT_ADMITTED
MACOS_NATIVE_ADAPTERS=NOT_ADMITTED
LINUX_NATIVE_ADAPTERS=NOT_ADMITTED
UIAUTOMATION_0_25_0_CONTROL_ONLY=REJECTED_COMPILE_FAILURE
UIAUTOMATION_0_25_1_CONTROL_ONLY=REJECTED_COMPILE_FAILURE
WINDOWS_0_62_2_DIRECT_UIA_CALLSITE=REJECTED_UNDER_CURRENT_GOLAM_SAFE_RUST_POLICY
XA11Y_WINDOWS_0_13_0_SEMANTIC_ADAPTER=CANDIDATE_PENDING_QUALIFICATION_AND_REVIEW
XA11Y_WINDOWS_INPUT_PROVIDER_FOR_SEMANTIC_ROUTE=DENIED
XA11Y_WINDOWS_SCREENSHOT_FOR_SEMANTIC_ROUTE=DENIED
LINUX_ARBOARD_CANDIDATE=REJECTED_HOST_GRAPH_INCLUDED_CLIPBOARD_WAYLAND
LINUX_X11_CLIPBOARD_0_9_3=CANDIDATE_PENDING_QUALIFICATION_AND_REVIEW
LINUX_WAYLAND_CLIPBOARD=NOT_ADMITTED
LINUX_DIRECT_WAYLAND_CLIPBOARD_PROTOCOL=DENIED
PRODUCT_MANIFEST_MUTATION=BLOCKED
PLATFORM_DISPATCH=BLOCKED
CAMERA=DENIED
MICROPHONE=DENIED
AUDIO_CAPTURE=DENIED
OCR=DENIED
HIDDEN_NETWORK_OR_CLOUD_FALLBACK=DENIED
WAYLAND_BYPASS=DENIED
RENDERER_AUTHORITY=DENIED
FALLBACK_ELIGIBILITY_MINTING_BY_ADAPTER=DENIED
GOLAM_UNSAFE_POLICY_WEAKENING=NOT_AUTHORIZED
WAIVER_TAKEN=NO
```

## Candidate result

```text
SPEC006_NATIVE_SOURCE_FOUNDRY_CANDIDATE=OPEN
EXACT_VERSION_SET=REPAIRED_FOR_EXACT_VCS_LICENSE_AND_X11_ONLY_CLIPBOARD
WORKFLOW_QUALIFICATION=PENDING
INDEPENDENT_REVIEW=PENDING
SOURCE_FOUNDRY_ADMISSION=NO
PRODUCT_IMPLEMENTATION_USE=BLOCKED
WAIVER_TAKEN=NO
```
