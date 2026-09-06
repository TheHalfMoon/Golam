# Feature Specification: Desktop Computer Control

**Feature Branch**: `spec/006-desktop-computer-control`  
**Status**: Planning  
**Canonical Base**: `9400d4614318fffb2623ea71522ecd5f0f95f96a`  
**Program Authority**: Spec 001 T060–T071

## Problem

Golam needs a local, reviewable computer-control layer that can observe desktop work surfaces, focus a selected surface, capture bounded visual evidence, perform semantic UI actions, and use narrowly governed raw input only when a semantic action is unavailable. The control plane must remain local-first, auditable, least-authority, permission-aware and cross-platform without pretending that Windows, macOS and Linux expose equivalent primitives.

## User stories

### US1 — Inspect current work surfaces (P1)
A user can ask Golam to enumerate relevant applications, windows, monitors and semantic UI elements. Golam returns a bounded, sanitized observation with stable-enough identity evidence and explicit unsupported/permission-denied states.

**Acceptance**: no actuation authority is implied by observation; stale surface identity invalidates later prepared actions.

### US2 — Perform a semantic desktop action (P1)
A user can authorize Golam to invoke, select, toggle, set value, focus or otherwise operate a semantic control through the platform accessibility/automation API.

**Acceptance**: action must bind exact target observation, capability, policy, approval/effect state and post-action verification. Raw input is not silently substituted.

### US3 — Capture a bounded selected surface (P1)
A user can explicitly authorize capture of a selected display/window/work surface for immediate reasoning.

**Acceptance**: system/user permission is honored; scope is bounded; capture is ephemeral by default; raw screenshot text extraction/OCR is outside Spec 006; camera/microphone are denied.

### US4 — Governed fallback input (P2)
When semantic action is genuinely unavailable, Golam can use raw pointer/keyboard input only when the policy explicitly permits the fallback and the exact action receives required approval/effect authority.

**Acceptance**: no background keylogging, no arbitrary global injection, no secure-desktop interaction, no silent fallback.

### US5 — Explicit clipboard operation (P2)
A user can explicitly authorize bounded clipboard read or write.

**Acceptance**: no silent inspection, no polling, no persistence by default, and clipboard data never becomes authority.

### US6 — Cross-platform graceful degradation (P1)
Golam exposes one typed desktop-control contract while Windows, macOS and Linux adapters report their actual supported features and permission posture.

**Acceptance**: unsupported operations fail closed and are not emulated by a weaker mechanism unless a separately authorized fallback exists.

## Functional requirements

- **FR-001**: Define a versioned `DesktopController` façade with observation, focus, capture, semantic action, explicit fallback action, clipboard and handle-release operations.
- **FR-002**: Define canonical `WorkSurfaceIdentity`, `SemanticElementIdentity`, `DesktopObservation`, `PreparedDesktopAction`, `CaptureIntent`, `DesktopActionOutcome` and permission/session evidence.
- **FR-003**: Observation must be read-only and bounded by explicit node/depth/window/monitor/byte/time limits.
- **FR-004**: Target identity must bind platform, session, process/application identity where available, surface identity, semantic element identity/path where applicable, and an observation digest/version.
- **FR-005**: A prepared action must revalidate target identity, focus/session state, permission posture and authority immediately before dispatch.
- **FR-006**: Semantic actions are primary; supported action families include invoke, select, toggle, focus, scroll, set-value and equivalent platform-supported semantic operations.
- **FR-007**: Raw pointer/keyboard fallback must be a separate operation class and capability, explicitly selected and governed.
- **FR-008**: Capture must target an explicitly selected display/window/work surface and enforce dimension/frame/byte/time bounds.
- **FR-009**: Raw capture is ephemeral by default. Persistence requires a distinct future/explicit storage authority and is not part of the default capture path.
- **FR-010**: Screenshot pixels and semantic text are untrusted evidence and must not authorize follow-on actions.
- **FR-011**: Clipboard read/write are separate explicit operations; silent/background clipboard inspection is forbidden.
- **FR-012**: The trusted Rust side must expose sanitized DTOs to the Tauri frontend; raw OS handles/pointers/tokens must never cross the frontend boundary.
- **FR-013**: Platform adapters must expose capability discovery and deterministic `NotSupported`, `PermissionDenied`, `StaleTarget`, `AuthorityDenied`, `Interrupted` and `UnknownOutcome` classes.
- **FR-014**: Windows semantic backend uses UI Automation first; capture uses sanctioned Windows capture APIs; raw input uses `SendInput` only as explicit governed fallback.
- **FR-015**: Windows secure desktop is always denied/not supported.
- **FR-016**: macOS semantic backend uses Accessibility/AX APIs and capture uses ScreenCaptureKit under system permission/TCC.
- **FR-017**: Linux semantic backend uses AT-SPI first. X11-specific raw mechanisms are allowed only in an explicitly identified X11 session. Wayland remote desktop/capture uses portal/compositor-mediated sessions and EIS/libei where available.
- **FR-018**: Linux browser-only/strict-local configurations without an admitted local desktop authority must fail closed rather than synthesize control.
- **FR-019**: Native permission grants, portal sessions and capture sessions are external authority state and must be revalidated; they are not converted into blanket Golam authority.
- **FR-020**: All actuation effects must produce secret-safe durable evidence sufficient to reconcile attempted/committed/denied/unknown outcomes.
- **FR-021**: Camera and microphone collection are denied in Spec 006.
- **FR-022**: OCR, computer-vision text extraction and semantic inference from raw screenshot pixels are deferred to Spec 007.
- **FR-023**: No remote/cloud fallback, hidden HTTP dependency or network emission may be introduced by this feature.
- **FR-024**: Fake backend contract tests must exercise every cross-platform contract before native adapter admission.

## Non-functional and security requirements

- **NFR-001**: Strict-local and least-authority defaults.
- **NFR-002**: Deterministic canonical encodings/digests for authority-bearing records.
- **NFR-003**: Secret-safe logs/evidence; no raw clipboard/capture content in ordinary logs.
- **NFR-004**: Bounded work and memory for observation/capture.
- **NFR-005**: Race-aware revalidation before side effects and post-action verification after side effects.
- **NFR-006**: Platform parity is contractual, not implementation-identical; unsupported behavior is explicit.
- **NFR-007**: Tauri capabilities and commands are narrowly scoped; frontend compromise must not yield unrestricted native control.

## Explicit dispositions

`SEMANTIC_FIRST=YES`  
`RAW_COORDINATE_FALLBACK=EXPLICIT_ONLY`  
`TAURI_2_DESKTOP_SHELL=SELECTED`  
`ELECTRON_PRIVILEGED_RUNTIME=NOT_SELECTED`  
`DONOR_ARCHITECTURE_AUTHORITY=NONE`  
`DONOR_DESKTOP_CONTROL_IMPLEMENTATION=NOT_FOUND`  
`WINDOWS_SECURE_DESKTOP=NOT_SUPPORTED`  
`CAMERA_MICROPHONE=DENIED`  
`RAW_SCREENSHOT_TEXT_EXTRACTION=DEFER_TO_SPEC_007`  
`CLIPBOARD=EXPLICIT_CAPABILITY_APPROVAL_ONLY`  
`UNBOUNDED_CAPTURE=DENIED`  
`BACKGROUND_KEYLOGGING=DENIED`  
`SILENT_CLIPBOARD_INSPECTION=DENIED`  
`HIDDEN_NETWORK_FALLBACK=DENIED`

## Success criteria

1. Fake backend proves bounded observation, stale-target invalidation, focus races, permission loss, semantic action success/failure, explicit fallback denial/allow, bounded capture and clipboard gating.
2. Windows/macOS/Linux adapters each pass their supported contract subset and explicit unsupported paths.
3. No camera/mic/OCR/network capability is reachable through Spec 006.
4. Exact-head CI succeeds on Windows, macOS and Ubuntu before final review and merge.
5. Fresh independent semantic/security review reports no unresolved material finding on the exact qualified head.
6. Expected-head merge and post-merge push CI succeed before Spec 006 is closed canonical.
