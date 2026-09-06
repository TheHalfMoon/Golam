# Clarification Closeout: Spec 006 — Desktop Computer Control

## Status

`CLARIFICATION_STATUS=CLOSED_FOR_PLANNING`

This record closes the Spec Kit clarification stage for the planning package. It does not authorize product implementation. Implementation remains blocked until the full planning lifecycle closes canonical on `main` after exact-head CI, fresh independent review, guarded merge and post-merge CI.

## Clarification 1 — What does semantic-first mean and how is fallback selected?

Decision: Golam preserves the constitutional ordering exactly:

`domain/application API → native OS automation API → accessibility/semantic tree → browser DOM/protocol → deterministic keyboard/mouse control → vision/pixel fallback`

Spec 006 owns desktop-control routes but cannot use its weaker fallbacks to bypass a stronger applicable route elsewhere in Golam. Before deterministic raw input or vision/pixel fallback is prepared, trusted orchestration creates canonical `FallbackEligibilityEvidence` showing the disposition of each applicable stronger route. A weaker route is denied while a stronger route remains available/authorized. An unreconciled `UNKNOWN_OUTCOME` blocks conflicting fallback escalation. Raw adapters and pixel-hint producers cannot self-mint fallback eligibility.

`CONTROL_ROUTE_ORDER=CONSTITUTIONAL_ORDER_REQUIRED`
`RAW_INPUT_FALLBACK=EXPLICIT_ONLY`
`FALLBACK_ELIGIBILITY_EVIDENCE=REQUIRED`
`UNKNOWN_OUTCOME_ALLOWS_WEAKER_FALLBACK=NO`
`VISION_PIXEL_FALLBACK=BOUNDED_UNTRUSTED_HINT_ONLY`

## Clarification 2 — Does Spec 006 perform screenshot OCR or infer semantic text from pixels?

No. Raw screenshot OCR/text extraction remains deferred to Spec 007. Spec 006 may carry a bounded `PixelTargetHint` containing a region/coordinate plus capture/source provenance as untrusted evidence for a separately governed raw-input action. The hint cannot mint capability, approval, semantic identity, target authority or fallback eligibility.

`RAW_SCREENSHOT_TEXT_EXTRACTION=DEFER_TO_SPEC_007`
`PIXEL_HINT_CAN_MINT_AUTHORITY=NO`
`PIXEL_HINT_CAN_MINT_FALLBACK_ELIGIBILITY=NO`

## Clarification 3 — How does the Tauri desktop shell trust `golamd`?

The native Rust Tauri host is an authenticated local client of `golamd` through the existing authenticated local IPC/client-enrollment trust boundary. Loopback/local-machine location, successful transport connection, renderer state and window identity are not authentication. Authentication material and authority-bearing tokens never enter the renderer/webview.

`TAURI_GOLAMD_AUTHENTICATED_CLIENT=REQUIRED`
`RENDERER_AUTHENTICATION_MATERIAL=DENIED`

## Clarification 4 — Where is pause/stop/takeover enforced?

At protected lease/input-authority state in the trusted path. Human pause/stop/takeover advances, suspends or revokes the conflicting agent input generation before further dispatch. UI state is only a projection. Stale requests cannot restore an old generation. Effects that may already have crossed the side-effect boundary retain terminal/`UNKNOWN_OUTCOME` reconciliation requirements.

`HUMAN_TAKEOVER=LEASE_INPUT_AUTHORITY_LAYER_REQUIRED`
`STALE_AGENT_GENERATION_REENABLE=DENIED`

## Clarification 5 — What is the Windows locked/UAC posture?

Locked interactive desktop, UAC/secure-desktop transitions and unsupported interactive-session changes fail closed. Golam does not automate Windows secure desktop and does not bypass UAC or secure-desktop boundaries.

`WINDOWS_SECURE_DESKTOP=NOT_SUPPORTED`
`WINDOWS_LOCK_UAC_TRANSITION=FAIL_CLOSED`

## Clarification 6 — Does platform permission grant Golam authority?

No. Accessibility/TCC/Screen Recording, Windows capture/interactive session state, XDG portal sessions, EIS/libei grants and similar OS state are external mutable prerequisites. They are revalidated near dispatch but never converted into blanket Golam capability or policy approval.

`OS_PERMISSION_IMPLIES_GOLAM_AUTHORITY=NO`

## Clarification 7 — Can donor desktop behavior define Golam architecture?

No. `TheHalfMoon/Golam-research@a9f633e09d49a85829b8236331b9e21f7e612634` remains qualified behavioral/reference evidence only. Electron/Node/preload/VNC/privileged IPC does not become Golam architecture, authority or dependency by reference.

`DONOR_DESKTOP_BEHAVIORAL_REFERENCE=QUALIFIED`
`DONOR_ARCHITECTURE_AUTHORITY=NONE`
`DONOR_RUNTIME_STACK_ADMITTED=NO`

## Clarification 8 — Does planning select dependencies?

Planning selects platform/API direction but does not admit implementation dependencies. Before any new Rust crate, JS package, native library, helper, vendored/copied code or donor component is introduced, its exact state and dependency closure must pass Source Foundry admission under the Constitution.

`PLANNING_SOURCE_DIRECTION_IS_DEPENDENCY_ADMISSION=NO`
`SOURCE_FOUNDRY_BEFORE_DEPENDENCY_ADMISSION=YES`

## Clarification 9 — What happens after ambiguous completion or human takeover?

Any operation that may have crossed the protected effect boundary but lacks proven terminal truth records `UNKNOWN_OUTCOME`. A timeout, restart, permission transition or takeover does not authorize retry or fallback escalation. Conflicting follow-up stays blocked until reconciliation establishes terminal truth.

`UNKNOWN_OUTCOME_CONFLICTING_RETRY=BLOCKED_UNTIL_RECONCILIATION`

## Clarification 10 — Must autonomous computer control remain visibly indicated?

Yes. Active autonomous computer control requires at least one qualified persistent local visible-control channel exposing immediate pause, stop and takeover. Trusted Rust must track or independently observe channel liveness/visibility; a DOM flag or cached renderer state is insufficient. If every qualified visible-control channel is lost, new autonomous actuation is suspended fail closed. Restoring visibility does not restore stale action authority; ordinary fresh validation still applies.

`AUTONOMOUS_CONTROL_VISIBLE_INDICATOR=REQUIRED`
`INVISIBLE_AUTONOMOUS_ACTUATION=FAIL_CLOSED_DENIED`

## Clarification 11 — When is implementation authorized?

Only after the planning PR is exact-head qualified, freshly reviewed, reconciled, marked Ready, merged with expected-head protection, and the exact returned merge SHA passes push-triggered canonical-main CI. Until then, no Spec 006 product implementation branch/code is authorized.

`SPEC_006_PRODUCT_IMPLEMENTATION_AUTHORIZED=NO_UNTIL_PLANNING_CLOSED_CANONICAL`
`WAIVER_TAKEN=NO`
