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

## Windows semantic-candidate history

### Rejected: `uiautomation 0.25.0`

Source Foundry run `34169734322`, Windows job `101887612788`, proved the exact control-only candidate fails to compile when its default `input` authority is correctly denied: upstream core imports `inputs::MouseButton` while the `inputs` module is compiled out.

Disposition: `REJECT`.

### Rejected: `uiautomation 0.25.1`

Source Foundry run `34170134934`, Windows job `101888718026`, reproduced the same control-only feature-gating failure on exact `0.25.1`.

Golam does not enable `uiautomation/input` as a workaround because semantic automation must not inherit raw keyboard/mouse authority.

Disposition: `REJECT`.

### Rejected for Golam-authored semantic call sites: direct `windows 0.62.2`

The direct Microsoft projection candidate compiled successfully in Source Foundry run `34170554684`. That run also exposed ordinary Cargo feature unification between the direct projection and WGC's use of the same `windows 0.62.2` package, which was handled by an isolated semantic feature probe in the next candidate revision.

However, canonical Golam workspace policy forbids Golam-authored unsafe code. Direct Win32/COM activation and many projected API call sites require `unsafe` in caller code. Source Foundry dependency buildability cannot authorize `allow(unsafe_code)`, a lint exception, or an implicit new unsafe boundary.

Disposition: `REJECT` for the Spec 006 Golam-authored semantic adapter under current canonical safety policy. The package may remain transitively present through admitted third-party wrappers/capture libraries; that is not direct semantic-call-site authority.

## Windows candidate

### Semantic automation — safe caller-facing UIA backend

```toml
xa11y-core = { version = "=0.13.0", default-features = false }
xa11y-windows = { version = "=0.13.0", default-features = false }
```

Selected reuse strategy: `ADAPTER`.

Source/release evidence:

- repository: `https://github.com/xa11y/xa11y`;
- exact crate line: `xa11y-windows 0.13.0` + `xa11y-core 0.13.0`;
- upstream license declaration: MIT;
- upstream architecture identifies `xa11y-windows` as the Windows UI Automation backend and `WindowsProvider` as its accessibility provider.

Rationale:

- caller-facing `WindowsProvider` allows Golam code to remain under existing safe-Rust lint policy while dependency-internal FFI/unsafe remains an explicit Source Foundry review boundary;
- the semantic adapter will expose only bounded `DesktopBackend` observation/focus/semantic-action behavior;
- `xa11y-windows` also publishes `WindowsInputProvider` and `WindowsScreenshot`; those surfaces are explicitly **not admitted for the semantic role** and must remain unreachable from the Golam semantic adapter. Raw input and capture retain their separate Enigo/WGC authority/evidence paths;
- dependency presence cannot mint route eligibility, capability, approval, target identity, lease state, or actuation authority.

The workflow compiles a safe-Rust caller probe that references `WindowsProvider`, inventories the exact active closure, and records the broad upstream surface for independent review. A later product adapter must be code-reviewed to ensure it never imports or invokes `WindowsInputProvider` or `WindowsScreenshot`.

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

Source Foundry run `34170554684`, macOS job `101889890862`, completed SUCCESS for aggregate build, feature denylist, active closure license/native/network inventory, and product-manifest non-mutation proof. This is historical candidate evidence only; the final exact-head candidate still requires a fresh complete run and review.

## Linux candidate

```toml
atspi = { version = "=0.30.0", default-features = false, features = ["proxies", "connection", "tokio"] }
ashpd = { version = "=0.13.13", default-features = false, features = ["tokio", "remote_desktop", "screencast"] }
pipewire = { version = "=0.9.2" }
enigo = { version = "=0.6.1", default-features = false, features = ["x11rb"] }
arboard = { version = "=3.6.1", default-features = false, features = ["wayland-data-control"] }
```

AT-SPI is the semantic layer. XDG ScreenCast/RemoteDesktop is user/compositor-granted authority only; denied `ashpd` features include camera, clipboard, `input_capture`, screenshot, background and unrelated portal surfaces. PipeWire is frame transport only for a granted ScreenCast session. Enigo is X11-only; Wayland/libei paths are denied. Clipboard remains explicit text-only and non-polling.

Run `34170134934`, Linux job `101888717883`, proved build + feature denylist + `NETWORK_CLIENT_PACKAGES=0` before the original SPDX checker falsely rejected `r-efi 6.0.0`'s `MIT OR Apache-2.0 OR LGPL-2.1-or-later`. The repaired evaluator preserves SPDX `OR`/`AND` semantics, does not whitelist LGPL, and evaluates only the host-active resolve graph.

Run `34170554684`, Linux job `101889890984`, completed SUCCESS with the repaired evaluator and product-manifest non-mutation proof. This remains historical candidate evidence until the final exact-head run succeeds.

## Candidate qualification workflow

`.github/workflows/spec006-native-adapter-source-foundry.yml` must independently qualify Windows, macOS, and Ubuntu and must:

1. create isolated scratch manifests so product manifests remain unchanged;
2. resolve exact candidate versions with Rust `1.98.0`;
3. filter metadata and dependency reachability to the actual host target;
4. record host triple, lock digest, dependency tree, and enabled feature tree;
5. compile exact aggregate closures on the matching platform;
6. on Windows, compile a `#![forbid(unsafe_code)]` caller probe that references `xa11y_windows::WindowsProvider`;
7. fail if denied features enter WGC/Enigo/arboard/macOS/Linux candidate paths;
8. inventory active licenses, custom build scripts, native `links` declarations, and known HTTP-client packages;
9. require at least one allowed path through every SPDX expression, preserving `OR`/`AND` semantics rather than substring matching;
10. surface third-party unsafe/FFI/native/build boundaries for independent review rather than translating dependency-internal unsafe into Golam-authored unsafe authority;
11. prove repository product manifests remain unchanged;
12. emit candidate-only status. Workflow success cannot itself change this record to admitted.

## Required independent review questions

A fresh exact-head security/governance review must answer at least:

- Does `xa11y-windows 0.13.0` provide a sufficiently bounded safe caller-facing UIA provider for Spec 006 while preserving Golam's `unsafe_code = "forbid"` policy?
- Can Golam's semantic adapter be kept structurally limited to `WindowsProvider`/semantic provider behavior so `WindowsInputProvider` and `WindowsScreenshot` remain unreachable from the semantic route?
- Are xa11y's dependency-internal unsafe/FFI boundaries, recent-release maturity, license/notice obligations and platform verification posture acceptable?
- Does the broad WGC closure remain safely encapsulated behind selected window/display capture only?
- Does `axuielement` exclude `raw-ffi`, and can compatibility keyboard helpers remain unreachable from semantic dispatch?
- Can ScreenCaptureKit be configured deterministically as screen-only, audio-off, microphone-absent?
- Are native/FFI/build-script and source/license obligations acceptable?
- Does Linux ScreenCast use only the user-granted portal PipeWire remote rather than ambient source enumeration?
- Can XDG RemoteDesktop remain user/compositor-granted without camera, clipboard, `input_capture`, or unrelated portal surfaces?
- Is X11 input explicitly session-scoped while Wayland bypass remains impossible?
- Does text-only clipboard remain separate from observation/capture/raw authority and avoid polling?
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
EXACT_VERSION_SET=REPAIRED_TO_SAFE_CALLER_FACING_WINDOWS_UIA_CANDIDATE
WORKFLOW_QUALIFICATION=PENDING
INDEPENDENT_REVIEW=PENDING
SOURCE_FOUNDRY_ADMISSION=NO
PRODUCT_IMPLEMENTATION_USE=BLOCKED
WAIVER_TAKEN=NO
```
