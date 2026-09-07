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

Qualification occurs in isolated scratch Cargo projects before product manifest mutation. Workflow success is evidence for independent review, not admission by itself.

## Immutable authority constraints

Candidate libraries provide platform mechanics only. They cannot:

- mint `FallbackEligibilityEvidence` or Kernel/Effect Gate authorization;
- own capability, policy, approval, control-lease, or visible-channel authority;
- convert observation, capture bytes, semantic metadata, clipboard content, or pixel hints into actuation authority;
- continue autonomous input after pause, stop, takeover, permission loss, unresolved `UNKNOWN_OUTCOME`, or qualified visible-channel loss;
- bypass Windows locked/UAC/secure-desktop boundaries or a Wayland compositor/XDG portal grant;
- introduce camera, microphone, audio capture, OCR, hidden HTTP/cloud/network fallback, keylogging, or background clipboard polling.

## Windows candidate

### Semantic automation — direct Microsoft projection candidate

```toml
windows = { version = "=0.62.2", default-features = false, features = ["std", "Win32_System_Com", "Win32_UI_Accessibility"] }
```

The two higher-level `uiautomation` candidates are rejected rather than weakened:

- run `34169734322`, Windows job `101887612788`: `uiautomation 0.25.0` control-only failed because its core imported `inputs::MouseButton` while the denied `input` feature compiled that module out;
- run `34170134934`, Windows job `101888718026`: exact `uiautomation 0.25.1` reproduced the same control-only compile failure.

Golam therefore does not enable `uiautomation/input` to work around the upstream boundary.

Run `34170554684`, Windows job `101889891002`, proved the replacement aggregate candidate compiles successfully. Its first feature guard then failed because `wgc 1.0.7` depends on the same `windows 0.62.2` package and Cargo correctly unified WGC capture/media/graphics features with Golam's semantic projection. That failure is a qualification-methodology failure, not semantic feature admission.

The repaired workflow keeps the aggregate Windows build for real dependency interaction/closure evidence but verifies the semantic projection in a second isolated semantic-only scratch manifest. The semantic probe must resolve exactly the hierarchical feature set implied by `std`, `Win32_System_Com`, and `Win32_UI_Accessibility`; WGC feature unification cannot satisfy or contaminate this proof.

Important implementation constraint: canonical workspace policy currently forbids Golam-authored unsafe code. Source Foundry qualification of the `windows` crate does not authorize weakening that policy, adding `allow(unsafe_code)`, or creating a new unsafe boundary. Fresh independent review must determine whether a Golam-authored semantic adapter can remain within the existing safe-Rust policy. If not, this direct projection is not product-admissible without separate canonical governance; dependency qualification alone cannot create that authority.

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

```toml
axuielement = { version = "=0.9.1", default-features = false }
screencapturekit = { version = "=10.0.3", default-features = false }
enigo = { version = "=0.6.1", default-features = false }
arboard = { version = "=3.6.1", default-features = false }
```

`axuielement/raw-ffi` and `async` remain disabled. Its compatibility keyboard-event helper is not part of Golam semantic dispatch. ScreenCaptureKit optional macOS-version features remain disabled; any later adapter must explicitly configure screen-only output with audio and microphone absent. Clipboard image features remain denied.

Run `34170554684`, macOS job `101889890862`, completed SUCCESS for aggregate build, feature denylist, active closure license/native/network inventory, and no-product-manifest-mutation proof. This is candidate evidence only.

## Linux candidate

```toml
atspi = { version = "=0.30.0", default-features = false, features = ["proxies", "connection", "tokio"] }
ashpd = { version = "=0.13.13", default-features = false, features = ["tokio", "remote_desktop", "screencast"] }
pipewire = { version = "=0.9.2" }
enigo = { version = "=0.6.1", default-features = false, features = ["x11rb"] }
arboard = { version = "=3.6.1", default-features = false, features = ["wayland-data-control"] }
```

AT-SPI is the semantic layer. XDG ScreenCast/RemoteDesktop is user/compositor-granted authority only; denied `ashpd` features include camera, clipboard, `input_capture`, screenshot, background and unrelated portal surfaces. PipeWire is frame transport only for a granted ScreenCast session. Enigo is X11-only; Wayland/libei paths are denied, so Wayland bypass is impossible through this candidate. Clipboard remains explicit text-only and non-polling.

Run `34170134934`, Linux job `101888717883`, proved build + feature denylist + `NETWORK_CLIENT_PACKAGES=0` before the original SPDX checker falsely rejected `r-efi 6.0.0`'s `MIT OR Apache-2.0 OR LGPL-2.1-or-later`. The repaired evaluator preserves SPDX `OR`/`AND` semantics, does not whitelist LGPL, and evaluates only the host-active resolve graph.

Run `34170554684`, Linux job `101889890984`, completed SUCCESS with the repaired evaluator and product-manifest non-mutation proof. This is candidate evidence only.

## Candidate qualification workflow

`.github/workflows/spec006-native-adapter-source-foundry.yml` must independently qualify Windows, macOS, and Ubuntu and must:

1. create isolated scratch manifests so product manifests remain unchanged;
2. resolve exact candidate versions with Rust `1.98.0`;
3. filter metadata and dependency reachability to the actual host target;
4. record host triple, lock digest, dependency tree, and enabled feature tree;
5. compile exact aggregate closures on the matching platform;
6. fail if denied candidate features appear;
7. on Windows, separately compile and inspect an isolated semantic-only `windows 0.62.2` probe so WGC feature unification cannot masquerade as semantic authority;
8. inventory active licenses, custom build scripts, native `links` declarations, and known HTTP-client packages;
9. require at least one allowed path through every SPDX expression, with `OR`/`AND` semantics preserved rather than substring matching;
10. surface native/FFI/build boundaries for independent review;
11. prove repository product manifests remain unchanged;
12. emit candidate-only status. Workflow success cannot itself change this record to admitted.

## Required independent review questions

A fresh exact-head security/governance review must answer at least:

- Does the isolated semantic probe correctly prove the requested direct Windows UIA projection independently of WGC feature unification?
- Given Golam's existing `unsafe_code = "forbid"` policy, is direct `windows 0.62.2` actually product-implementable without weakening canonical safety policy? If not, reject it for product admission rather than creating an implicit unsafe exception.
- Does the broad `wgc` closure remain safely encapsulated behind selected window/display capture only?
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
WINDOWS_0_62_2_DIRECT_UIA_PROJECTION=CANDIDATE_PENDING_QUALIFICATION_AND_SAFE_RUST_REVIEW
WINDOWS_AGGREGATE_FEATURE_UNIFICATION=OBSERVED_NOT_SEMANTIC_AUTHORITY
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
EXACT_VERSION_SET=REPAIRED_AFTER_WINDOWS_FEATURE_UNIFICATION_METHOD_FAILURE
WORKFLOW_QUALIFICATION=PENDING
INDEPENDENT_REVIEW=PENDING
SOURCE_FOUNDRY_ADMISSION=NO
PRODUCT_IMPLEMENTATION_USE=BLOCKED
WAIVER_TAKEN=NO
```
