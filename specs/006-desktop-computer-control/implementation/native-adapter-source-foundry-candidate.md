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

This record proposes an exact, platform-scoped dependency set for the native desktop adapters required by Spec 006. It is a Source Foundry **candidate only**. It does not admit any dependency into the Golam product manifests, does not authorize platform dispatch, and does not change the constitutional route or authority model.

The candidate exists to qualify dependency closure, Cargo features, licenses/notices, build scripts, native-link requirements and buildability on the matching GitHub-hosted platform before product code or manifest mutation.

## Immutable authority constraints

The candidate may provide platform mechanics only. It cannot:

- mint `FallbackEligibilityEvidence`;
- mint or mutate Kernel/Effect Gate authorization;
- own capability/policy/approval decisions;
- create or restore a desktop control lease;
- claim visible-control-channel qualification from renderer state;
- convert observation, capture bytes or pixel hints into actuation authority;
- continue autonomous input after human pause/stop/takeover or visible-channel loss;
- bypass Windows locked/UAC/secure-desktop boundaries;
- bypass a Wayland compositor or XDG portal grant;
- introduce camera, microphone, OCR, hidden HTTP/cloud/network fallback or background clipboard polling.

## Windows candidate

### Semantic automation

```toml
uiautomation = { version = "=0.25.0", default-features = false, features = ["control"] }
```

Rationale:

- safe high-level Microsoft UI Automation wrapper;
- `control` keeps control-pattern/UI wrappers while default keyboard-input authority is disabled;
- `input`, `clipboard`, `screenshot`, `process`, `dialog`, `event`, `log` and `all` remain denied;
- semantic observation/action cannot inherit raw input, screenshot, clipboard or process authority.

Candidate source:
- https://docs.rs/crate/uiautomation/0.25.0/source/Cargo.toml.orig
- https://docs.rs/crate/uiautomation/0.25.0/features

### Selected window/display capture

```toml
wgc = { version = "=1.0.7", default-features = false }
```

Rationale:

- safe high-level Windows.Graphics.Capture wrapper;
- optional `tracing` remains disabled;
- the dependency's Windows feature closure is broader than Golam's intended API use and therefore requires explicit Source Foundry review rather than being treated as least-authority evidence by itself;
- product code, if later admitted, may expose only selected window/display capture and must not expose media transcoding, storage, cryptography, shell or unrelated Windows APIs merely because they exist in the dependency closure.

Candidate source:
- https://docs.rs/crate/wgc/1.0.7/source/Cargo.toml
- https://docs.rs/crate/wgc/1.0.7/features

### Explicit deterministic raw fallback

```toml
enigo = { version = "=0.6.1", default-features = false }
```

On Windows this selects the platform implementation without Linux X11/Wayland/libei features. Product use remains separately gated by canonical fresh fallback eligibility, explicit authority, the current Kernel/Effect Gate authorization, control-lease generation and visible-control-channel state.

Candidate source:
- https://docs.rs/crate/enigo/0.6.1/source/Cargo.toml.orig

### Explicit text clipboard

```toml
arboard = { version = "=3.6.1", default-features = false }
```

`image-data` is denied. Product use, if later admitted, is one explicit bounded text read/write operation only; polling/background inspection is forbidden.

Candidate source:
- https://docs.rs/crate/arboard/3.6.1/features

## macOS candidate

### Accessibility semantic automation

```toml
axuielement = { version = "=0.9.1", default-features = false }
```

Rationale:

- safe AXUIElement API surface;
- `raw-ffi` is explicitly disabled;
- `async` notification support is not admitted by this candidate;
- Accessibility/TCC state remains external authority state and permission loss fails closed.

Candidate source:
- https://docs.rs/axuielement/0.9.1/axuielement/

### Selected display/window capture

```toml
screencapturekit = { version = "=10.0.3", default-features = false }
```

Special restrictions:

- all optional version features remain disabled, especially `macos_15_0`, which exposes microphone functionality;
- the crate's macOS 13 baseline contains system-audio configuration APIs even with optional features disabled; Golam must explicitly configure audio capture off and register only `SCStreamOutputType::Screen`;
- microphone/audio input/output is not admitted as a Spec 006 capability;
- no recording-to-file, content-picker authority, Metal upload path or screenshot convenience path is implied by dependency presence;
- this is a very recent upstream release and therefore requires exact closure/build-script review plus a fresh independent review before admission.

Candidate source:
- https://docs.rs/crate/screencapturekit/10.0.3

### Explicit raw fallback and clipboard

```toml
enigo = { version = "=0.6.1", default-features = false }
arboard = { version = "=3.6.1", default-features = false }
```

No Linux X11/Wayland/libei or clipboard image features are selected.

## Linux candidate

### AT-SPI semantic layer

```toml
atspi = { version = "=0.30.0", default-features = false, features = ["proxies", "connection", "tokio"] }
```

Rationale:

- pure-Rust/zbus AT-SPI2 protocol implementation at the public safe API layer;
- only semantic query/connection/runtime features are requested;
- runtime availability of accessibility services must be observed and unsupported/permission-limited states remain explicit.

Candidate source:
- https://docs.rs/atspi/0.30.0/atspi/

### Wayland/XDG ScreenCast and RemoteDesktop grant boundary

```toml
ashpd = { version = "=0.13.13", default-features = false, features = ["tokio", "remote_desktop", "screencast"] }
```

Denied features include `frontend`, `backend`, `background`, `camera`, `clipboard`, `input_capture`, `location`, `network_monitor`, `notification`, `open_uri`, `print`, `proxy_resolver`, `screenshot`, `secret`, `usb`, `wallpaper`, `wayland` and `raw_handle`.

`remote_desktop` is admitted only as the user/compositor-granted portal control path. It is not a remote-network feature and cannot be used without a portal session grant. No direct compositor bypass is permitted.

Candidate source:
- https://docs.rs/crate/ashpd/0.13.13/source/Cargo.toml
- https://docs.rs/ashpd/0.13.13/ashpd/desktop/remote_desktop/
- https://docs.rs/ashpd/0.13.13/ashpd/desktop/screencast/

### PipeWire frame transport for a granted ScreenCast session

```toml
pipewire = { version = "=0.9.2" }
```

Why `0.9.2` rather than the newer `0.10.x` line:

- `ashpd 0.13.13` declares its optional PipeWire integration against the `0.9` line;
- using the same established line avoids carrying two incompatible PipeWire crate families in the candidate closure;
- it provides a safe high-level Rust API but necessarily carries `pipewire-sys`/`libspa-sys` FFI, bindgen, build scripts and system-library probing;
- those native/build boundaries are an explicit Source Foundry review target and are not considered admitted by workflow success alone.

Candidate source:
- https://docs.rs/crate/pipewire/0.9.2/source/Cargo.toml.orig
- https://docs.rs/crate/pipewire-sys/0.9.2

### X11-only deterministic raw fallback

```toml
enigo = { version = "=0.6.1", default-features = false, features = ["x11rb"] }
```

`wayland`, `libei_smol`, `libei_tokio` and `xdo` are denied. The Enigo Wayland/libei paths are not used as trusted Wayland authority. Wayland input may later use only the XDG RemoteDesktop/EIS path actually granted by the compositor/user and separately governed by Golam's Effect Gate.

### Explicit text clipboard

```toml
arboard = { version = "=3.6.1", default-features = false, features = ["wayland-data-control"] }
```

`image-data` is denied. Availability of the Wayland data-control protocol is not assumed; unsupported compositor/session states must fail closed. Clipboard polling remains forbidden.

## Candidate qualification workflow

`.github/workflows/spec006-native-adapter-source-foundry.yml` must run the exact candidate independently on Windows, macOS and Ubuntu and must:

1. create an isolated scratch manifest so product manifests remain unchanged;
2. resolve and lock the exact candidate versions using Rust `1.98.0`;
3. record the lock digest, direct dependency tree and enabled feature tree;
4. compile the selected closure on the matching host platform;
5. fail if any denied candidate feature appears;
6. inventory package licenses, custom build scripts and native `links` declarations;
7. fail if an explicit reciprocal/forbidden license or an HTTP client package enters the closure;
8. record FFI/native/build boundaries for independent review rather than silently treating them as safe;
9. prove `Cargo.toml`, `Cargo.lock` and desktop product manifests remain unchanged;
10. emit candidate-only status. Workflow success cannot itself change this record to admitted.

## Required review questions

A fresh independent security/governance review on the exact candidate SHA must answer at least:

- Does the Windows closure expose a materially safer bounded capture path than the rejected/broader alternatives, and are unrelated `wgc` Windows features safely encapsulated?
- Can `uiautomation` control-only use avoid raw input/clipboard/screenshot/process authority?
- Does `axuielement` truly exclude `raw-ffi` from the selected closure?
- Does ScreenCaptureKit build/runtime configuration allow Golam to enforce screen-only, audio-off, microphone-absent behavior deterministically?
- Are Swift/native/FFI build scripts and their source/license obligations acceptable?
- Does Linux ScreenCast frame access require only the user-granted portal PipeWire remote and avoid ambient PipeWire source enumeration?
- Can `ashpd` RemoteDesktop be constrained to a user/compositor-granted session without `input_capture`, camera, clipboard or unrelated portal surfaces?
- Is X11 input explicitly session-scoped while Wayland bypass remains impossible?
- Does text-only clipboard remain distinct from observation/capture/raw authority and avoid background polling?
- Does any candidate introduce hidden HTTP/cloud/network-client behavior or product runtime network authority?

## Explicit non-admissions

```text
WINDOWS_NATIVE_ADAPTERS=NOT_ADMITTED
MACOS_NATIVE_ADAPTERS=NOT_ADMITTED
LINUX_NATIVE_ADAPTERS=NOT_ADMITTED
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
EXACT_VERSION_SET=FROZEN_FOR_QUALIFICATION
WORKFLOW_QUALIFICATION=PENDING
INDEPENDENT_REVIEW=PENDING
SOURCE_FOUNDRY_ADMISSION=NO
PRODUCT_IMPLEMENTATION_USE=BLOCKED
WAIVER_TAKEN=NO
```
