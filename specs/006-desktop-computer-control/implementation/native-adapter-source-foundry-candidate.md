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

This record proposes an exact, platform-scoped dependency set for the native desktop adapters required by Spec 006. It is a Source Foundry candidate only. It does not admit dependencies into Golam product manifests, authorize platform dispatch, or change the constitutional route or authority model.

Qualification occurs in an isolated scratch Cargo project before product manifest mutation. Workflow success is evidence for independent review, not admission by itself.

## Immutable authority constraints

Candidate libraries provide platform mechanics only. They cannot:

- mint `FallbackEligibilityEvidence` or Kernel/Effect Gate authorization;
- own capability, policy, approval, control-lease, or visible-channel authority;
- convert observation, capture bytes, semantic metadata, clipboard content, or pixel hints into actuation authority;
- continue autonomous input after pause, stop, takeover, permission loss, unresolved `UNKNOWN_OUTCOME`, or qualified visible-channel loss;
- bypass Windows locked/UAC/secure-desktop boundaries or a Wayland compositor/XDG portal grant;
- introduce camera, microphone, audio capture, OCR, hidden HTTP/cloud/network fallback, keylogging, or background clipboard polling.

## Windows candidate

### Semantic automation — direct Microsoft projection

```toml
windows = { version = "=0.62.2", default-features = false, features = ["std", "Win32_System_Com", "Win32_UI_Accessibility"] }
```

The two higher-level `uiautomation` candidates are rejected rather than weakened:

- Source Foundry run `34169734322`, Windows job `101887612788`: `uiautomation 0.25.0` control-only failed because `src/core.rs` imported `crate::inputs::MouseButton` while the denied `input` feature correctly compiled the `inputs` module out.
- Source Foundry run `34170134934`, Windows job `101888718026`: exact `uiautomation 0.25.1` reproduced the same unresolved import under the same least-authority feature selection.

Golam therefore does not enable `uiautomation/input` to work around the upstream boundary. The replacement candidate uses Microsoft's `windows 0.62.2` projection directly and restricts the direct semantic dependency to the COM and Win32 accessibility modules required for UI Automation. The workflow verifies the exact resolved feature set for this specific `windows 0.62.2` package and rejects unexpected features.

Any later Golam Windows semantic facade must encapsulate unsafe Windows calls internally, expose only bounded semantic observation/actions through `DesktopBackend`, and keep raw keyboard/mouse simulation in the distinct fallback path governed by trusted fresh fallback eligibility and the Effect Gate.

### Selected window/display capture

```toml
wgc = { version = "=1.0.7", default-features = false }
```

`wgc` remains candidate-only. Its internal Windows API closure is broader than Golam's intended interface, so dependency presence grants no authority. A later admitted facade may expose only selected window/display capture. Optional `tracing` remains disabled.

### Explicit deterministic raw fallback

```toml
enigo = { version = "=0.6.1", default-features = false }
```

Raw input is not semantic automation. Product use remains separately gated by fresh trusted fallback eligibility, explicit policy/approval, the current Kernel/Effect Gate decision, exact target/focus/session state, current lease generation, and qualified visible-control-channel state.

### Explicit text clipboard

```toml
arboard = { version = "=3.6.1", default-features = false }
```

`image-data` remains denied. Product use, if later admitted, is one explicit bounded text read/write operation only; polling/background inspection is forbidden.

## macOS candidate

### Accessibility semantic automation

```toml
axuielement = { version = "=0.9.1", default-features = false }
```

`raw-ffi` and `async` remain disabled. The crate's safe API still contains compatibility keyboard-event helpers; Golam's semantic facade is forbidden from invoking them. Raw input remains a distinct governed fallback path. Accessibility/TCC state is external authority state and permission loss fails closed.

### Selected display/window capture

```toml
screencapturekit = { version = "=10.0.3", default-features = false }
```

All optional macOS-version features remain disabled. Any later adapter must explicitly disable audio capture and register only screen output. Microphone, audio input/output, recording-to-file, content-picker authority, and convenience paths outside the bounded selected-source contract are not admitted by dependency presence.

Source Foundry run `34170134934` proved this macOS candidate compiles, passes its feature denylist, passes active closure license/native/network inventory, and leaves product manifests unchanged. It remains not admitted until fresh independent review accepts the exact candidate.

### Explicit raw fallback and clipboard

```toml
enigo = { version = "=0.6.1", default-features = false }
arboard = { version = "=3.6.1", default-features = false }
```

Linux X11/Wayland/libei and clipboard image features remain unselected.

## Linux candidate

### AT-SPI semantic layer

```toml
atspi = { version = "=0.30.0", default-features = false, features = ["proxies", "connection", "tokio"] }
```

Only semantic connection/proxy/runtime features are requested. Accessibility-service availability and session permission must be observed; unavailable or permission-limited environments return explicit unsupported/denied dispositions.

### Wayland/XDG ScreenCast and RemoteDesktop grant boundary

```toml
ashpd = { version = "=0.13.13", default-features = false, features = ["tokio", "remote_desktop", "screencast"] }
```

Denied features include `frontend`, `backend`, `background`, `camera`, `clipboard`, `input_capture`, `location`, `network_monitor`, `notification`, `open_uri`, `print`, `proxy_resolver`, `screenshot`, `secret`, `usb`, `wallpaper`, `wayland`, and `raw_handle`.

`remote_desktop` means only the user/compositor-granted XDG portal path; it is not remote network access and cannot be used without a portal session grant. Direct Wayland compositor bypass remains forbidden.

### PipeWire frame transport for a granted ScreenCast session

```toml
pipewire = { version = "=0.9.2" }
```

The candidate stays on the `0.9` family aligned with `ashpd 0.13.13`. `pipewire-sys`/`libspa-sys`, bindgen, custom build scripts, native links, and system-library probing remain explicit review targets.

Source Foundry run `34170134934`, Linux job `101888717883`, proved the exact candidate builds and passes the feature denylist with `NETWORK_CLIENT_PACKAGES=0`. The job then failed in the first license evaluator because it rejected any expression containing an LGPL alternative, including `r-efi 6.0.0`'s `MIT OR Apache-2.0 OR LGPL-2.1-or-later`. That evaluator behavior is rejected as semantically incorrect: an SPDX `OR` expression has an allowed license path when one branch is acceptable. The repaired workflow evaluates `OR` as any accepted branch and `AND` as all accepted branches; it does not whitelist LGPL and takes no waiver.

The repaired workflow also inventories only package IDs reachable from the host-filtered Cargo resolve graph, avoiding false findings from target-inactive metadata entries.

### X11-only deterministic raw fallback

```toml
enigo = { version = "=0.6.1", default-features = false, features = ["x11rb"] }
```

`wayland`, `libei_smol`, `libei_tokio`, and `xdo` are denied. Enigo's Wayland/libei path is not trusted authority. Wayland input may later use only a user/compositor-granted XDG RemoteDesktop/EIS path governed by Golam's Effect Gate.

### Explicit text clipboard

```toml
arboard = { version = "=3.6.1", default-features = false, features = ["wayland-data-control"] }
```

`image-data` is denied. Availability of Wayland data-control is never assumed; unsupported compositor/session states fail closed. Clipboard polling remains forbidden.

## Candidate qualification workflow

`.github/workflows/spec006-native-adapter-source-foundry.yml` must independently qualify Windows, macOS, and Ubuntu and must:

1. create an isolated scratch manifest so product manifests remain unchanged;
2. resolve exact candidate versions with Rust `1.98.0`;
3. filter metadata and dependency reachability to the actual host target;
4. record host triple, lock digest, direct dependency tree, and enabled feature tree;
5. compile the exact closure on the matching platform;
6. fail if any denied candidate feature appears;
7. on Windows, verify the direct `windows 0.62.2` semantic projection has only the expected hierarchical `std`/COM/accessibility feature set;
8. inventory active licenses, custom build scripts, native `links` declarations, and known HTTP-client packages;
9. require at least one allowed path through every SPDX expression, with `OR`/`AND` semantics preserved rather than substring matching;
10. surface native/FFI/build boundaries for independent review;
11. prove repository product manifests remain unchanged;
12. emit candidate-only status. Workflow success cannot itself change this record to admitted.

## Required independent review questions

A fresh exact-head security/governance review must answer at least:

- Is direct `windows 0.62.2` COM/UI Accessibility projection a sufficiently narrow and maintainable replacement for the rejected `uiautomation` control-only candidates?
- Can the Windows semantic facade keep unsafe projection calls internal and prevent semantic code from reaching raw input helpers?
- Does the broad `wgc` closure remain safely encapsulated behind selected window/display capture only?
- Does `axuielement` exclude `raw-ffi`, and can its compatibility keyboard helper remain unreachable from semantic dispatch?
- Can ScreenCaptureKit be configured deterministically as screen-only, audio-off, microphone-absent?
- Are Swift/native/FFI/build-script and source/license obligations acceptable?
- Does Linux ScreenCast frame access use only the user-granted portal PipeWire remote rather than ambient source enumeration?
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
WINDOWS_0_62_2_DIRECT_UIA_PROJECTION=CANDIDATE_PENDING_QUALIFICATION
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
WAIVER_TAKEN=NO
```

## Candidate result

```text
SPEC006_NATIVE_SOURCE_FOUNDRY_CANDIDATE=OPEN
EXACT_VERSION_SET=REPAIRED_AFTER_UIAUTOMATION_0_25_1_AND_LICENSE_EVALUATOR_FAILURES
WORKFLOW_QUALIFICATION=PENDING
INDEPENDENT_REVIEW=PENDING
SOURCE_FOUNDRY_ADMISSION=NO
PRODUCT_IMPLEMENTATION_USE=BLOCKED
WAIVER_TAKEN=NO
```
