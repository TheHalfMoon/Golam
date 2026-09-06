# Feature Specification: Desktop Computer Control

**Feature Branch**: `spec/006-desktop-computer-control`  
**Status**: Planning  
**Canonical Base**: `9400d4614318fffb2623ea71522ecd5f0f95f96a`  
**Program Authority**: Spec 001 T060–T069

## Problem

Golam needs a local, reviewable computer-control layer that can observe desktop work surfaces, focus a selected surface, capture bounded visual evidence, perform semantic UI actions, and use narrowly governed fallback mechanisms only when stronger constitutional control paths are genuinely unavailable or inapplicable. The control plane must remain local-first, auditable, least-authority, permission-aware, visibly active, immediately human-interruptible and cross-platform without pretending that Windows, macOS and Linux expose equivalent primitives.

## Control-route invariant

Spec 006 participates in, but does not override, the constitutional control-route order:

`domain/application API → native OS automation API → accessibility/semantic tree → browser DOM/protocol → deterministic keyboard/mouse control → vision/pixel fallback`

A weaker path cannot be selected while a stronger applicable route remains available and authorized. If a stronger route has `UNKNOWN_OUTCOME`, conflicting fallback escalation is blocked until reconciliation. Desktop raw-input adapters and pixel-hint producers cannot self-declare fallback eligibility.

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
When all applicable stronger constitutional routes have been evaluated and a semantic/browser/native route is genuinely unavailable, inapplicable, unsupported, denied, or failed safely before any ambiguous side effect, Golam can use raw pointer/keyboard input only when the policy explicitly permits the fallback and the exact action receives required approval/effect authority.

A bounded vision/pixel fallback may supply only an untrusted candidate region/coordinate derived from an explicitly selected capture. It cannot mint authority, fallback eligibility, or bypass exact work-surface/focus/session identity, and it cannot perform OCR/text extraction under Spec 006.

**Acceptance**: no background keylogging, no arbitrary global injection, no secure-desktop interaction, no silent fallback, no escalation past `UNKNOWN_OUTCOME`, and no pixel-derived candidate can execute without separately bound fallback-eligibility and raw-input authority.

### US5 — Explicit clipboard operation (P2)
A user can explicitly authorize bounded clipboard read or write.

**Acceptance**: no silent inspection, no polling, no persistence by default, and clipboard data never becomes authority.

### US6 — Cross-platform graceful degradation (P1)
Golam exposes one typed desktop-control contract while Windows, macOS and Linux adapters report their actual supported features and permission posture.

**Acceptance**: unsupported operations fail closed and are not emulated by a weaker mechanism unless a separately authorized fallback exists and route-order eligibility is established.

### US7 — Immediate visible human pause, stop and takeover (P1)
A local user can see when autonomous computer control is active and can immediately pause agent input, stop computer-control work, or take exclusive control of the interactive session.

**Acceptance**: active autonomous control has a persistent local visible indicator/control surface. Takeover is enforced at the protected lease/input-authority layer, not only in UI state. It revokes or suspends conflicting agent input authority, invalidates stale queued/prepared actions, preserves reconciliation for any already-crossed effect boundary, and has measured takeover latency. Loss of the qualified visible-control channel suspends new autonomous actuation until visibility/control is restored or another qualified visible channel is active.

### US8 — Authenticated desktop client boundary (P1)
The Tauri desktop shell communicates with `golamd` only as an authenticated local client through the existing governed local IPC boundary.

**Acceptance**: renderer/webview compromise cannot obtain client authentication material, mint authority, invoke unrestricted native adapters, or bypass Kernel/Effect Gate checks.

## Functional requirements

- **FR-001**: Define a versioned `DesktopController` façade with observation, focus, capture, semantic action, explicit fallback action, clipboard, human-interrupt/takeover and handle-release operations.
- **FR-002**: Define canonical `WorkSurfaceIdentity`, `SemanticElementIdentity`, `DesktopObservation`, `FallbackEligibilityEvidence`, `PreparedDesktopAction`, `CaptureIntent`, `PixelTargetHint`, `DesktopControlLeaseState`, `VisibleControlChannelState`, `DesktopActionOutcome` and permission/session evidence.
- **FR-003**: Observation must be read-only and bounded by explicit node/depth/window/monitor/byte/time limits.
- **FR-004**: Target identity must bind platform, session, process/application identity where available, surface identity, semantic element identity/path where applicable, and an observation digest/version.
- **FR-005**: A prepared action must revalidate target identity, focus/session state, permission posture and authority immediately before dispatch.
- **FR-006**: Semantic actions are primary inside the desktop-control layer; supported action families include invoke, select, toggle, focus, scroll, set-value and equivalent platform-supported semantic operations.
- **FR-007**: Raw pointer/keyboard fallback must be a separate operation class and capability, explicitly selected and governed.
- **FR-008**: Capture must target an explicitly selected display/window/work surface and enforce dimension/frame/byte/time bounds.
- **FR-009**: Raw capture is ephemeral by default. Persistence requires a distinct future/explicit storage authority and is not part of the default capture path.
- **FR-010**: Screenshot pixels, pixel-derived hints and semantic text are untrusted evidence and must not authorize follow-on actions.
- **FR-011**: Clipboard read/write are separate explicit operations; silent/background clipboard inspection is forbidden.
- **FR-012**: The trusted Rust side must expose sanitized DTOs to the Tauri frontend; raw OS handles/pointers/tokens and local-client authentication material must never cross the frontend boundary.
- **FR-013**: Platform adapters must expose capability discovery and deterministic `NotSupported`, `PermissionDenied`, `StaleTarget`, `AuthorityDenied`, `Interrupted` and `UnknownOutcome` classes.
- **FR-014**: Windows semantic backend uses UI Automation first; capture uses sanctioned Windows capture APIs; raw input uses `SendInput` only as explicit governed fallback.
- **FR-015**: Windows lock-screen, UAC/secure-desktop and interactive-session transitions must fail closed. Windows secure desktop is always denied/not supported.
- **FR-016**: macOS semantic backend uses Accessibility/AX APIs and capture uses ScreenCaptureKit under system permission/TCC.
- **FR-017**: Linux semantic backend uses AT-SPI first. X11-specific raw mechanisms are allowed only in an explicitly identified X11 session. Wayland remote desktop/capture uses portal/compositor-mediated sessions and EIS/libei where available.
- **FR-018**: Linux browser-only/strict-local configurations without an admitted local desktop authority must fail closed rather than synthesize control.
- **FR-019**: Native permission grants, portal sessions, capture sessions, focus and interactive-desktop state are external authority state and must be revalidated; they are not converted into blanket Golam authority.
- **FR-020**: All actuation, focus, capture and clipboard effects must produce secret-safe durable evidence sufficient to reconcile attempted/committed/denied/unknown outcomes.
- **FR-021**: Camera and microphone collection are denied in Spec 006.
- **FR-022**: OCR, screenshot text extraction and inferred semantic text from raw pixels are deferred to Spec 007. Spec 006 may carry only a bounded untrusted pixel-region/coordinate hint used as input to a separately governed raw fallback.
- **FR-023**: No remote/cloud fallback, hidden HTTP dependency or network emission may be introduced by this feature.
- **FR-024**: Fake backend contract tests must exercise every cross-platform contract before native adapter admission.
- **FR-025**: The Tauri native Rust host must authenticate to `golamd` through the existing authenticated local IPC/client-enrollment boundary. Renderer state, localhost reachability and transport connection are never authentication or authority.
- **FR-026**: Human pause/stop/takeover must be enforced by protected control-lease/input-authority state. Takeover revokes or suspends conflicting agent input generations before additional dispatch and cannot be undone by a stale model/UI request.
- **FR-027**: Prepared/queued agent input bound to a superseded or revoked lease generation must fail closed. Already-dispatched uncertain effects must remain `UNKNOWN_OUTCOME` until reconciled rather than being silently replayed after takeover.
- **FR-028**: Any third-party crate, JS package, native library, helper or copied donor implementation introduced for Spec 006 requires exact per-source Source Foundry admission before dependency or code admission. Official platform documentation establishes direction only, not dependency admission.
- **FR-029**: Autonomous computer control must maintain a persistent visible local indicator/control channel that exposes immediate pause, stop and takeover. If no qualified visible-control channel is available, new autonomous actuation is suspended fail closed rather than continuing invisibly.
- **FR-030**: Before deterministic keyboard/mouse or vision/pixel fallback is prepared, Golam must produce canonical `FallbackEligibilityEvidence` for the applicable constitutional route sequence. A weaker route is denied while a stronger applicable route remains available/authorized, and `UNKNOWN_OUTCOME` blocks conflicting fallback escalation until reconciliation.

## Non-functional and security requirements

- **NFR-001**: Strict-local and least-authority defaults.
- **NFR-002**: Deterministic canonical encodings/digests for authority-bearing records.
- **NFR-003**: Secret-safe logs/evidence; no raw clipboard/capture content in ordinary logs.
- **NFR-004**: Bounded work and memory for observation/capture/pixel-hint processing.
- **NFR-005**: Race-aware revalidation before side effects and post-action verification after side effects.
- **NFR-006**: Platform parity is contractual, not implementation-identical; unsupported behavior is explicit.
- **NFR-007**: Tauri capabilities and commands are narrowly scoped; frontend compromise must not yield unrestricted native control.
- **NFR-008**: Human pause/stop/takeover latency must be measured from the protected interrupt signal to conflicting input-authority revocation/suspension, with a deterministic fail-closed bound selected and qualified during implementation.
- **NFR-009**: Desktop-client authentication must reuse the existing authenticated local IPC trust boundary rather than creating a second renderer-owned authentication path.
- **NFR-010**: Visibility of active autonomous computer control is a safety invariant. Loss of the qualified visible-control channel must be detected and must suspend new autonomous input within the implementation's qualified fail-closed bound.
- **NFR-011**: Route-order evaluation and fallback-eligibility records must be deterministic, auditable and non-self-asserted by the weaker adapter/vision component.

## Explicit dispositions

`CONTROL_ROUTE_ORDER=CONSTITUTIONAL_ORDER_REQUIRED`  
`RAW_FALLBACK_REQUIRES_FALLBACK_ELIGIBILITY_EVIDENCE=YES`  
`UNKNOWN_OUTCOME_ALLOWS_WEAKER_FALLBACK=NO`  
`SEMANTIC_FIRST=YES`  
`RAW_COORDINATE_FALLBACK=EXPLICIT_ONLY`  
`VISION_PIXEL_FALLBACK=BOUNDED_UNTRUSTED_HINT_ONLY`  
`VISION_PIXEL_FALLBACK_CAN_MINT_AUTHORITY=NO`  
`TAURI_2_DESKTOP_SHELL=SELECTED`  
`TAURI_GOLAMD_AUTHENTICATED_CLIENT=REQUIRED`  
`RENDERER_AUTHENTICATION_MATERIAL=DENIED`  
`AUTONOMOUS_CONTROL_VISIBLE_INDICATOR=REQUIRED`  
`INVISIBLE_AUTONOMOUS_ACTUATION=FAIL_CLOSED_DENIED`  
`HUMAN_TAKEOVER=LEASE_INPUT_AUTHORITY_LAYER_REQUIRED`  
`ELECTRON_PRIVILEGED_RUNTIME=NOT_SELECTED`  
`DONOR_ARCHITECTURE_AUTHORITY=NONE`  
`DONOR_DESKTOP_CONTROL_IMPLEMENTATION_ADMITTED=NO`  
`WINDOWS_SECURE_DESKTOP=NOT_SUPPORTED`  
`WINDOWS_LOCK_UAC_TRANSITION=FAIL_CLOSED`  
`CAMERA_MICROPHONE=DENIED`  
`RAW_SCREENSHOT_TEXT_EXTRACTION=DEFER_TO_SPEC_007`  
`CLIPBOARD=EXPLICIT_CAPABILITY_APPROVAL_ONLY`  
`UNBOUNDED_CAPTURE=DENIED`  
`BACKGROUND_KEYLOGGING=DENIED`  
`SILENT_CLIPBOARD_INSPECTION=DENIED`  
`HIDDEN_NETWORK_FALLBACK=DENIED`

## Success criteria

1. Fake backend proves bounded observation, stale-target invalidation, focus races, permission loss, semantic action success/failure, exact route ordering/fallback eligibility, explicit fallback denial/allow, bounded pixel-hint handling, bounded capture, clipboard gating, lease-generation invalidation and visible-channel-loss suspension.
2. Windows/macOS/Linux adapters each pass their supported contract subset and explicit unsupported paths, including interactive-session/permission transitions applicable to that platform.
3. No camera/mic/OCR/network capability is reachable through Spec 006, and bounded vision/pixel hints cannot mint authority or fallback eligibility.
4. Tauri desktop authenticates as a local `golamd` client while the renderer receives no authentication material or native authority.
5. Active autonomous computer control remains visibly indicated with immediate local pause/stop/takeover; loss of the qualified visible-control channel suspends new actuation fail closed.
6. Human pause/stop/takeover invalidates conflicting agent input authority at the protected lease/input layer and passes takeover-latency, stale-reference and wrong-window adversarial tests.
7. Weaker fallback cannot run while a stronger applicable constitutional route remains available/authorized, and no fallback escalates past an unreconciled `UNKNOWN_OUTCOME`.
8. Every introduced dependency/runtime primitive has an exact Source Foundry admission record before use.
9. Exact-head CI succeeds on Windows, macOS and Ubuntu before final review and merge.
10. Fresh independent semantic/security/governance review reports no unresolved material finding on the exact qualified head.
11. Expected-head merge and post-merge push CI succeed before Spec 006 is closed canonical.
