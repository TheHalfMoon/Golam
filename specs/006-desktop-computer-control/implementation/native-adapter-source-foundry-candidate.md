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

This record proposes an exact, platform-scoped dependency set for the native desktop adapters required by Spec 006. It is a Source Foundry **candidate only**. It does not admit any dependency into Golam product manifests, authorize platform dispatch, or change the constitutional route or authority model.

The candidate is qualified in an isolated scratch Cargo project before any product manifest mutation. Workflow success is evidence for review, not admission by itself.

## Immutable authority constraints

Candidate libraries may provide platform mechanics only. They cannot:

- mint `FallbackEligibilityEvidence` or Kernel/Effect Gate authorization;
- own capability, policy, approval, control-lease, or visible-channel authority;
- convert observation, capture bytes, semantic metadata, clipboard content, or pixel hints into actuation authority;
- continue autonomous input after pause, stop, takeover, permission loss, unresolved `UNKNOWN_OUTCOME`, or qualified visible-channel loss;
- bypass Windows locked/UAC/secure-desktop boundaries or a Wayland compositor/XDG portal grant;
- introduce camera, microphone, OCR, hidden HTTP/cloud/network fallback, keylogging, or background clipboard polling.

## Windows candidate

### Semantic automation

```toml
uiautomation = { version = "=0.25.1", default-features = false, features = ["control"] }
```

The first candidate used `uiautomation =0.25.0` with the same control-only feature selection. Source Foundry run `34169734322`, Windows job `101887612788`, proved that exact selection unusable: `uiautomation 0.25.0/src/core.rs` imported `crate::inputs::MouseButton` while the `inputs` module was correctly compiled out behind the denied `input` feature. Cargo's registry index simultaneously reported `0.25.1` available. The `0.25.0` result is therefore rejected, not waived.

This repaired candidate tests exact `0.25.1` while keeping `input` denied. Product admission remains blocked unless the exact closure compiles and independent review confirms that semantic observation/actions can remain separated from raw keyboard/mouse authority. If `0.25.1` still requires `input`, Source Foundry must reject or explicitly redesign this boundary rather than silently enabling keyboard/mouse authority.

Regardless of crate API availability, any future Golam semantic adapter is forbidden from calling keyboard/mouse simulation, clipboard, screenshot, process, dialog, event, or log helpers. Raw fallback remains a distinct `enigo` path behind canonical fallback eligibility and the Effect Gate.

### Selected window/display capture

```toml
wgc = { version = "=1.0.7", default-features = false }
```

`wgc` is a safe high-level Windows.Graphics.Capture wrapper, but its internal `windows` feature closure is broader than Golam's intended API. Presence of unrelated Windows APIs in that closure is not product authority. Any later admitted Golam facade may expose only selected window/display capture and must keep media/storage/cryptography/shell/process or unrelated Windows APIs unreachable from the Spec 006 interface. Optional `tracing` remains disabled.

### Explicit deterministic raw fallback

```toml
enigo = { version = "=0.6.1", default-features = false }
```

On Windows no Linux X11/Wayland/libei feature is selected. Product use remains separately gated by trusted fresh fallback eligibility, explicit policy/approval, current Kernel/Effect Gate authorization, exact target/focus/session state, current lease generation, and qualified visible-control-channel state.

### Explicit text clipboard

```toml
arboard = { version = "=3.6.1", default-features = false }
```

`image-data` remains denied. Any future product use is one explicit bounded text read/write operation only; polling/background inspection is forbidden.

## macOS candidate

### Accessibility semantic automation

```toml
axuielement = { version = "=0.9.1", default-features = false }
```

`raw-ffi` and `async` remain disabled. The crate's safe API still contains keyboard-event helpers for compatibility; Golam's semantic facade is forbidden from invoking them. Raw input remains a distinct governed fallback path. Accessibility/TCC state is external authority state and permission loss fails closed.

### Selected display/window capture

```toml
screencapturekit = { version = "=10.0.3", default-features = false }
```

All optional macOS-version features remain disabled, especially feature levels that expose microphone support. Baseline APIs can still configure system audio, so any future Golam implementation must explicitly disable audio capture and register only screen output. Microphone, audio input/output, recording-to-file, content-picker authority, and convenience paths outside the bounded selected-source contract are not admitted by dependency presence.

Because `10.0.3` is a newly released upstream version, its exact Swift/native/build closure and license obligations require independent review before admission.

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

`ashpd 0.13.13` aligns its optional PipeWire integration with the `0.9` family, so the candidate uses that line rather than introducing a second incompatible PipeWire family. `pipewire` provides a safe high-level API but carries `pipewire-sys`/`libspa-sys`, bindgen, custom build scripts, and system-library probing. Those native/FFI/build boundaries are explicit review targets and are not admitted merely because the matrix compiles.

### X11-only deterministic raw fallback

```toml
enigo = { version = "=0.6.1", default-features = false, features = ["x11rb"] }
```

`wayland`, `libei_smol`, `libei_tokio`, and `xdo` are denied. Enigo's Wayland/libei path is not used as trusted authority. Wayland input may later use only a user/compositor-granted XDG RemoteDesktop/EIS path governed by Golam's Effect Gate.

### Explicit text clipboard

```toml
arboard = { version = "=3.6.1", default-features = false, features = ["wayland-data-control"] }
```

`image-data` is denied. Availability of Wayland data-control is never assumed; unsupported compositor/session states fail closed. Clipboard polling remains forbidden.

## Candidate qualification workflow

`.github/workflows/spec006-native-adapter-source-foundry.yml` must independently qualify Windows, macOS, and Ubuntu and must:

1. create an isolated scratch manifest; product manifests remain unchanged;
2. resolve exact versions with Rust `1.98.0`;
3. record lock digest, dependency tree, and enabled feature tree;
4. compile the exact closure on the matching platform;
5. fail if any denied candidate feature appears;
6. inventory licenses, custom build scripts, and native `links` declarations;
7. fail if an explicit forbidden/reciprocal license or HTTP client package enters the closure;
8. surface FFI/native/build boundaries for independent review rather than treating them as automatically safe;
9. prove repository product manifests remain unchanged;
10. emit candidate-only status. Workflow success cannot itself change this record to admitted.

## Required independent review questions

A fresh exact-head security/governance review must answer at least:

- Is `uiautomation 0.25.1` control-only genuinely buildable with `input` absent, and can the Golam facade prevent semantic code from reaching raw input helpers?
- Does the broad `wgc` Windows API closure remain safely encapsulated behind selected window/display capture only?
- Does `axuielement` exclude `raw-ffi`, and can its keyboard helper remain unreachable from semantic dispatch?
- Can ScreenCaptureKit be configured deterministically as screen-only, audio-off, microphone-absent?
- Are Swift/native/FFI/build-script and source/license obligations acceptable?
- Does Linux ScreenCast frame access use only the user-granted portal PipeWire remote rather than ambient source enumeration?
- Can XDG RemoteDesktop remain user/compositor-granted without camera, clipboard, `input_capture`, or unrelated portal surfaces?
- Is X11 input explicitly session-scoped while Wayland bypass remains impossible?
- Does text-only clipboard remain separate from observation/capture/raw authority and avoid polling?
- Does any candidate introduce hidden HTTP/cloud/network-client behavior or product runtime network authority?

## Explicit non-admissions

```text
WINDOWS_NATIVE_ADAPTERS=NOT_ADMITTED
MACOS_NATIVE_ADAPTERS=NOT_ADMITTED
LINUX_NATIVE_ADAPTERS=NOT_ADMITTED
UIAUTOMATION_0_25_0_CONTROL_ONLY=REJECTED_COMPILE_FAILURE
UIAUTOMATION_0_25_1_CONTROL_ONLY=CANDIDATE_PENDING_QUALIFICATION
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
EXACT_VERSION_SET=REPAIRED_AFTER_WINDOWS_0_25_0_FAILURE
WORKFLOW_QUALIFICATION=PENDING
INDEPENDENT_REVIEW=PENDING
SOURCE_FOUNDRY_ADMISSION=NO
PRODUCT_IMPLEMENTATION_USE=BLOCKED
WAIVER_TAKEN=NO
```
