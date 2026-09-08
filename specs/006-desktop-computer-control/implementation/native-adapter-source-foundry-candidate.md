---
task_scope: T006-016..T006-033 dependency precondition
status: CANDIDATE_QUALIFIED_PENDING_INDEPENDENT_REVIEW
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

Fresh review of exact head `b799b4e0892f4ac526f577d3f74aa8246c19fe2c` returned:

```text
DO_NOT_ADMIT_SPEC006_NATIVE_ADAPTER_DEPENDENCIES
```

It identified two material blockers:

1. license/notice evidence was based on Cargo metadata and did not bind exact published `.crate` archives or equivalent checksum-bound source identity;
2. Linux clipboard enabled `arboard/wayland-data-control`, creating a direct Wayland data-control closure without a separately qualified session boundary.

Forward-only reconciliation then produced:

- `2d8a4984a82b2b77bcbbb4f32406eef3b902284d` — checksum-bound archive auditing and removal of explicit Wayland data-control enablement;
- `55d7dcd964254644475836fb388691347d7434a3` — exact-VCS license evidence and structural replacement of Linux clipboard with `x11-clipboard 0.9.3`;
- `bc7338a8cf43172ea1df3909bc90d7c9c4f7318b` — release-accurate license declaration paths for `objc2` family packages and `cookie-factory`.

The intermediate Source Foundry runs were intentionally allowed to fail closed until each blocker was identified by host-active evidence. No threshold was weakened and no product dependency was admitted during reconciliation.

## Exact-head qualification evidence

The candidate set at exact head `bc7338a8cf43172ea1df3909bc90d7c9c4f7318b` completed both required pre-review gates:

```text
SOURCE_FOUNDRY_RUN=34175192920
SOURCE_FOUNDRY_WINDOWS=SUCCESS
SOURCE_FOUNDRY_MACOS=SUCCESS
SOURCE_FOUNDRY_LINUX=SUCCESS
REGULAR_CI_RUN=34175195531
REGULAR_CI_UBUNTU=SUCCESS
REGULAR_CI_MACOS=SUCCESS
REGULAR_CI_WINDOWS=SUCCESS
PRODUCT_MANIFEST_MUTATION=NO
PRODUCT_NATIVE_ADAPTER_CODE_ADMISSION=NO
SOURCE_FOUNDRY_ADMISSION=NO
```

Host-active Source Foundry evidence at that head:

| Host | Active external packages | Exact-VCS fallbacks | Network-client packages | Blocked archive/license evidence | Result |
|---|---:|---:|---:|---:|---|
| Windows | 60 | 3 | 0 | 0 | PASS |
| macOS | 45 | 8 | 0 | 0 | PASS |
| Linux | 138 | 1 | 0 | 0 | PASS |

Workflow success remains qualification only. This record stays blocked from product use until a fresh substantive independent review on the unchanged exact head explicitly admits the dependency set.

## Checksum archive + exact VCS license evidence

Every host-active registry package is bound to:

- its exact `Cargo.lock` source identity;
- its exact `Cargo.lock` checksum;
- one cached `.crate` archive whose SHA-256 equals that checksum;
- a deterministically selected allowed SPDX path.

When the checksum-bound archive contains a top-level `LICENSE`, `LICENCE`, `COPYING`, `NOTICE`, or `COPYRIGHT` file, the archive is the license-evidence source.

When an exact published archive omits license text, Source Foundry may use exact-VCS fallback only for a hard-coded package allowlist. The fallback must:

1. read the source commit SHA from the checksum-bound archive's own `.cargo_vcs_info.json`;
2. reject missing, malformed, or dirty VCS identity;
3. use a repository and evidence path hard-coded by Golam, not a URL supplied by package metadata;
4. fetch the evidence file at that exact VCS commit;
5. record source commit, `path_in_vcs`, evidence path, byte count, and SHA-256;
6. require the selected SPDX path to match the allowlisted license family;
7. fail closed if any exact source evidence cannot be retrieved or verified.

### Exact Windows fallback evidence

```text
clipboard-win|5.4.1|3b27cf2bfd1adcfa6e0264eb51c1025ddaf0f342||LICENSE|1338|c9bff75738922193e67fa726fa225535870d2aa1059f91452c411736284ad566
xa11y-core|0.13.0|6c3a5878e5e9a36942144854914985dbc7aaaf88|xa11y-core|LICENSE|1070|f2d9c355bfd769a0eabfd81b4fa9b1db8d8f7a4ebaf93f0af63843c2fb9eed04
xa11y-windows|0.13.0|6c3a5878e5e9a36942144854914985dbc7aaaf88|xa11y-windows|LICENSE|1070|f2d9c355bfd769a0eabfd81b4fa9b1db8d8f7a4ebaf93f0af63843c2fb9eed04
```

### Exact macOS fallback evidence

```text
dispatch2|0.3.1|8852b424193ca41602281b3d7540d7c8ed51e49a|crates/dispatch2|LICENSE.md|1339|7f976f7e9cb2d87df7230606feb932c3f21ac0e664045a775b600046ff850c54
objc2|0.6.4|8852b424193ca41602281b3d7540d7c8ed51e49a|crates/objc2|LICENSE.md|1339|7f976f7e9cb2d87df7230606feb932c3f21ac0e664045a775b600046ff850c54
objc2-app-kit|0.3.2|7b1abfd750a2cacaea71d6a56ecfb83cb7de560b|framework-crates/objc2-app-kit|LICENSE.md|1339|7f976f7e9cb2d87df7230606feb932c3f21ac0e664045a775b600046ff850c54
objc2-core-foundation|0.3.2|7b1abfd750a2cacaea71d6a56ecfb83cb7de560b|framework-crates/objc2-core-foundation|LICENSE.md|1339|7f976f7e9cb2d87df7230606feb932c3f21ac0e664045a775b600046ff850c54
objc2-core-graphics|0.3.2|7b1abfd750a2cacaea71d6a56ecfb83cb7de560b|framework-crates/objc2-core-graphics|LICENSE.md|1339|7f976f7e9cb2d87df7230606feb932c3f21ac0e664045a775b600046ff850c54
objc2-encode|4.1.0|8d214f5477365ffcbcbb7de058c86ed9a518efb7|crates/objc2-encode|LICENSE.md|1339|7f976f7e9cb2d87df7230606feb932c3f21ac0e664045a775b600046ff850c54
objc2-foundation|0.3.2|7b1abfd750a2cacaea71d6a56ecfb83cb7de560b|framework-crates/objc2-foundation|LICENSE.md|1339|7f976f7e9cb2d87df7230606feb932c3f21ac0e664045a775b600046ff850c54
objc2-io-surface|0.3.2|7b1abfd750a2cacaea71d6a56ecfb83cb7de560b|framework-crates/objc2-io-surface|LICENSE.md|1339|7f976f7e9cb2d87df7230606feb932c3f21ac0e664045a775b600046ff850c54
```

### Exact macOS checksum-bound archive identity

The successful exact-head macOS Source Foundry job `101904425012` in run `34175667738` resolved the following `Cargo.lock` checksum and cached `.crate` archive SHA-256 identities. Each archive SHA-256 exactly equals its lock checksum.

| Package | Version | `Cargo.lock` checksum | `.crate` archive SHA-256 |
|---|---:|---|---|
| `dispatch2` | `0.3.1` | `1e0e367e4e7da84520dedcac1901e4da967309406d1e51017ae1abfb97adbd38` | `1e0e367e4e7da84520dedcac1901e4da967309406d1e51017ae1abfb97adbd38` |
| `objc2` | `0.6.4` | `3a12a8ed07aefc768292f076dc3ac8c48f3781c8f2d5851dd3d98950e8c5a89f` | `3a12a8ed07aefc768292f076dc3ac8c48f3781c8f2d5851dd3d98950e8c5a89f` |
| `objc2-app-kit` | `0.3.2` | `d49e936b501e5c5bf01fda3a9452ff86dc3ea98ad5f283e1455153142d97518c` | `d49e936b501e5c5bf01fda3a9452ff86dc3ea98ad5f283e1455153142d97518c` |
| `objc2-core-foundation` | `0.3.2` | `2a180dd8642fa45cdb7dd721cd4c11b1cadd4929ce112ebd8b9f5803cc79d536` | `2a180dd8642fa45cdb7dd721cd4c11b1cadd4929ce112ebd8b9f5803cc79d536` |
| `objc2-core-graphics` | `0.3.2` | `e022c9d066895efa1345f8e33e584b9f958da2fd4cd116792e15e07e4720a807` | `e022c9d066895efa1345f8e33e584b9f958da2fd4cd116792e15e07e4720a807` |
| `objc2-encode` | `4.1.0` | `ef25abbcd74fb2609453eb695bd2f860d389e457f67dc17cafc8b8cbc89d0c33` | `ef25abbcd74fb2609453eb695bd2f860d389e457f67dc17cafc8b8cbc89d0c33` |
| `objc2-foundation` | `0.3.2` | `e3e0adef53c21f888deb4fa59fc59f7eb17404926ee8a6f59f5df0fd7f9f3272` | `e3e0adef53c21f888deb4fa59fc59f7eb17404926ee8a6f59f5df0fd7f9f3272` |
| `objc2-io-surface` | `0.3.2` | `180788110936d59bab6bd83b6060ffdfffb3b922ba1396b312ae795e1de9d81d` | `180788110936d59bab6bd83b6060ffdfffb3b922ba1396b312ae795e1de9d81d` |

The exact release `LICENSE.md` is the repository-level license declaration/mapping evidence used by these published packages. It is not treated as automatic proof that every downstream redistribution concern is resolved. In particular, `objc2` framework bindings are derived from Apple SDK declarations; independent review must determine whether the candidate evidence and planned packaging obligations are sufficient before admission.

### Exact Linux fallback evidence

```text
cookie-factory|0.3.3|21de7966fe0749ef63b504f4950101b7787bcf7e|src|LICENSES/MIT.txt|1086|8ffe9277dbaaf9b842fa9199205239011f96639e3c262d35032d65808cbba3df
```

The checksum-bound `cookie-factory 0.3.3` archive SHA-256 is `9885fa71e26b8ab7855e2ec7cae6e9b380edff76cd052e07c683a0319d51b3a2`, matching its lock checksum before the exact VCS MIT text is accepted as fallback evidence.

Any later product distribution must preserve applicable exact license/notice obligations in packaging evidence.

## Windows candidate

### Semantic automation — safe caller-facing UIA backend

```toml
xa11y-core = { version = "=0.13.0", default-features = false }
xa11y-windows = { version = "=0.13.0", default-features = false }
```

Selected reuse strategy: `ADAPTER`.

The exact qualification observed `WindowsProvider` through a `#![forbid(unsafe_code)]` caller probe. `WindowsInputProvider` and `WindowsScreenshot` remain explicitly excluded from the semantic role. Raw input and capture retain separate Enigo/WGC authority and evidence paths.

Rejected semantic candidates remain `uiautomation 0.25.0` and `0.25.1`, which failed the control-only build when the default raw-input feature was denied. Direct Golam-authored `windows 0.62.2` UIA call sites remain rejected under `unsafe_code = "forbid"`.

### Selected window/display capture

```toml
wgc = { version = "=1.0.7", default-features = false }
```

Optional `tracing` remains disabled. WGC's broad Windows projection closure grants no additional Golam authority; any admitted facade must expose only selected window/display capture.

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

`axuielement/raw-ffi` and `async` remain disabled. Compatibility keyboard-event helpers are not part of Golam semantic dispatch. ScreenCaptureKit optional macOS-version features remain disabled; any admitted adapter must configure screen-only output with audio and microphone absent. Clipboard image features remain denied.

## Linux candidate

```toml
atspi = { version = "=0.30.0", default-features = false, features = ["proxies", "connection", "tokio"] }
ashpd = { version = "=0.13.13", default-features = false, features = ["tokio", "remote_desktop", "screencast"] }
pipewire = { version = "=0.9.2" }
enigo = { version = "=0.6.1", default-features = false, features = ["x11rb"] }
x11-clipboard = { version = "=0.9.3" }
```

AT-SPI is the semantic layer. XDG ScreenCast/RemoteDesktop is user/compositor-granted external authority only; denied `ashpd` features include camera, clipboard, `input_capture`, screenshot, background, and unrelated portal surfaces. PipeWire is frame transport only for a granted ScreenCast session. Enigo is X11-only; Wayland/libei paths are denied.

### Linux clipboard decision

`arboard` is not a Linux candidate. Failed host-active evidence proved that `arboard 3.6.1` introduced `clipboard_wayland 0.2.2` even with `default-features = false`.

Exact-head qualification instead proved:

```text
LINUX_CLIPBOARD_CANDIDATE=X11_CLIPBOARD_0_9_3_ONLY
LINUX_PURE_WAYLAND_CLIPBOARD=NOT_SUPPORTED_PENDING_SEPARATE_SESSION_BOUND_ADMISSION
FEATURE_DENYLIST=PASS
```

The Linux host-active graph contains `x11-clipboard 0.9.3` and excludes `arboard`, `clipboard_wayland`, `wl-clipboard-rs`, and `wayland-data-control`. Pure Wayland clipboard remains `NotSupported` until a separately qualified compositor/session-bound mechanism receives its own Source Foundry admission.

Linux native-link evidence is explicitly surfaced for review:

```text
clang-sys|1.9.1|clang
libspa-sys|0.9.2|libspa-0.2
pipewire-sys|0.9.2|pipewire-0.3
```

## Candidate qualification workflow

`.github/workflows/spec006-native-adapter-source-foundry.yml` independently qualifies Windows, macOS, and Ubuntu and must:

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
11. permit exact-VCS license fallback only for hard-coded package families, using `.cargo_vcs_info.json` from the checksum-bound archive and a hard-coded repository/evidence path;
12. inventory active custom build scripts, native `links` declarations, and known HTTP-client packages;
13. surface third-party unsafe/FFI/native/build boundaries for independent review rather than translating dependency-internal unsafe into Golam-authored unsafe authority;
14. prove repository product manifests remain unchanged;
15. emit candidate-only status. Workflow success cannot itself change this record to admitted.

## Required independent review questions

A fresh exact-head security/governance review must answer at least:

- Does every host-active registry package have a unique checksum-bound `.crate` archive?
- For packages whose archive omits license text, is exact-VCS fallback tied to that archive's `.cargo_vcs_info.json` and restricted to hard-coded source/evidence paths?
- Are selected SPDX paths, exact release license declarations, and distribution obligations sufficient for admission, including the `objc2` Apple SDK-derived binding concern?
- Does `xa11y-windows 0.13.0` provide a sufficiently bounded safe caller-facing UIA provider while preserving Golam's `unsafe_code = "forbid"` policy?
- Can Golam's semantic adapter be structurally limited so `WindowsInputProvider` and `WindowsScreenshot` remain unreachable from the semantic route?
- Does the broad WGC closure remain encapsulated behind selected window/display capture only?
- Does `axuielement` exclude `raw-ffi`, and can compatibility keyboard helpers remain unreachable from semantic dispatch?
- Can ScreenCaptureKit be configured deterministically as screen-only, audio-off, microphone-absent?
- Does Linux ScreenCast use only a user-granted portal PipeWire remote rather than ambient source enumeration?
- Can XDG RemoteDesktop remain user/compositor-granted without camera, clipboard, `input_capture`, or unrelated portal surfaces?
- Is X11 input explicitly session-scoped while Wayland input bypass remains impossible?
- Is Linux clipboard structurally X11-only while pure Wayland clipboard fails closed as `NotSupported`?
- Does any active candidate introduce hidden HTTP/cloud/network-client behavior or product runtime network authority?

## Explicit non-admissions

```text
WINDOWS_NATIVE_ADAPTERS=NOT_ADMITTED
MACOS_NATIVE_ADAPTERS=NOT_ADMITTED
LINUX_NATIVE_ADAPTERS=NOT_ADMITTED
UIAUTOMATION_0_25_0_CONTROL_ONLY=REJECTED_COMPILE_FAILURE
UIAUTOMATION_0_25_1_CONTROL_ONLY=REJECTED_COMPILE_FAILURE
WINDOWS_0_62_2_DIRECT_UIA_CALLSITE=REJECTED_UNDER_CURRENT_GOLAM_SAFE_RUST_POLICY
XA11Y_WINDOWS_0_13_0_SEMANTIC_ADAPTER=CANDIDATE_QUALIFIED_PENDING_INDEPENDENT_REVIEW
XA11Y_WINDOWS_INPUT_PROVIDER_FOR_SEMANTIC_ROUTE=DENIED
XA11Y_WINDOWS_SCREENSHOT_FOR_SEMANTIC_ROUTE=DENIED
LINUX_ARBOARD_CANDIDATE=REJECTED_HOST_GRAPH_INCLUDED_CLIPBOARD_WAYLAND
LINUX_X11_CLIPBOARD_0_9_3=CANDIDATE_QUALIFIED_PENDING_INDEPENDENT_REVIEW
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
SPEC006_NATIVE_SOURCE_FOUNDRY_CANDIDATE=QUALIFIED_PENDING_INDEPENDENT_REVIEW
EXACT_QUALIFIED_HEAD=bc7338a8cf43172ea1df3909bc90d7c9c4f7318b
SOURCE_FOUNDRY_RUN=34175192920
REGULAR_CI_RUN=34175195531
WINDOWS_QUALIFICATION=PASS
MACOS_QUALIFICATION=PASS
LINUX_QUALIFICATION=PASS
NETWORK_CLIENT_PACKAGE_COUNT_ALL_HOSTS=0
BLOCKED_ARCHIVE_OR_LICENSE_EVIDENCE_ALL_HOSTS=0
INDEPENDENT_REVIEW=PENDING
SOURCE_FOUNDRY_ADMISSION=NO
PRODUCT_IMPLEMENTATION_USE=BLOCKED
WAIVER_TAKEN=NO
```