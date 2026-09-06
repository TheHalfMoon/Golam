# Implementation Plan: Desktop Computer Control

## Summary

Build a Rust-owned semantic-first desktop-control subsystem and a least-privilege Tauri 2 client boundary. The common contract must model observation, target identity, permission/session state, bounded capture, semantic action, explicit raw-input fallback, bounded untrusted pixel hints, clipboard operations, persistent visible autonomous-control state and immediate human pause/stop/takeover. Native adapters are admitted only after fake-backend contract qualification, Source Foundry admission for any introduced dependency/runtime primitive, and platform-specific permission/race testing.

## Technical context

- Trusted core/runtime: Rust workspace (`golam-core`, `golam-kernel`, `golam-ledger`, `golamd`).
- Desktop shell: Tauri 2 + React + TypeScript, with narrow capability files and typed sanitized RPC DTOs. The native Rust host is an authenticated local client of `golamd` through the existing authenticated IPC/client-enrollment boundary; renderer/webview code never owns authentication material or authorization state.
- Visible control: autonomous computer actuation requires at least one qualified persistent local visible-control channel exposing immediate pause/stop/takeover. Trusted Rust tracks channel liveness/visibility; loss of every qualified channel suspends new autonomous actuation rather than continuing invisibly.
- Windows: Microsoft UI Automation for semantic tree/actions; Windows.Graphics.Capture for selected display/window capture; raw input fallback only through explicitly governed `SendInput`; locked desktop, UAC/secure desktop and interactive-session drift fail closed.
- macOS: AXUIElement/Accessibility for semantic observation/action; ScreenCaptureKit under system Screen Recording permission; TCC state treated as external authority state.
- Linux: AT-SPI semantic layer; X11 raw facilities only for identified X11 sessions; Wayland ScreenCast/RemoteDesktop portals plus compositor-provided EIS/libei for granted control.
- Persistence: immutable request/effect/intent/authority/lease-generation/visible-channel-state bindings, terminal status, reconciliation metadata and payload digests only by default; raw captures/clipboard payloads are not ordinary durable evidence.
- Vision/pixel boundary: Spec 006 may consume a bounded pixel-region/coordinate candidate derived from an explicitly selected capture only as untrusted evidence for a separately governed raw fallback. No raw-screenshot OCR/text extraction or pixel-derived authority is admitted.
- Supply chain: official platform documentation selects architecture direction only. Any crate, package, native library, helper or copied donor implementation requires an exact Source Foundry admission record before code/dependency admission.

## Constitution check

- **Spec before implementation**: PASS — this planning PR contains no product implementation.
- **Local-first / strict-local**: PASS — no remote fallback or hidden network dependency.
- **Least authority**: PASS — observation/capture/semantic action/raw input/pixel hint/clipboard are distinct authority or evidence surfaces.
- **Effect governance**: PASS — every side effect uses a distinct request binding, `Effect PREPARED`, Kernel/Effect Gate dispatch and terminal reconciliation.
- **Evidence integrity**: PASS — prepared intents and outcomes bind canonical request, effect, target, authority, lease-generation and visible-channel state.
- **Secrets/privacy**: PASS — raw capture and clipboard content excluded from logs/evidence by default.
- **Visible human control**: PASS — active autonomous computer control requires a qualified visible local control channel with immediate pause/stop/takeover; losing all qualified channels suspends new actuation.
- **Human interruptibility**: PASS — pause/stop/takeover is enforced at protected lease/input authority, not renderer convention.
- **Authenticated client boundary**: PASS — Tauri native host reuses authenticated `golamd` IPC; renderer cannot become a principal by location or UI state.
- **Cross-platform honesty**: PASS — capability discovery + explicit unsupported states replace fake parity.
- **Donor discipline**: PASS — Golam-research is behavioral reference only and cannot define trusted architecture or authority.
- **Source Foundry**: PASS — no dependency/code source is admitted by planning; exact admission is a precondition for any implementation dependency or reused code.

## Architecture

1. **Pure contracts** in `golam-core`: identities, observations, action/capture/clipboard intents, bounded `PixelTargetHint`, `DesktopControlLeaseState`, `VisibleControlChannelState`, immutable request/effect bindings, capability descriptions, errors and canonical digests.
2. **Authority orchestration** in `golam-kernel`: prepare/revalidate/dispatch/finalize lifecycle bound to capability/policy/approval/effect records, current control-lease generation and qualified visible-control-channel state; protected human pause/stop/takeover invalidates conflicting agent input authority.
3. **Durable evidence** in `golam-ledger`: prepared request/effect/intent/lease/visible-channel bindings, terminal outcome, permission/session observation and reconciliation metadata without raw sensitive payloads.
4. **Adapter interface** in `golamd`: fake backend plus Windows/macOS/Linux implementations behind a common trait.
5. **Authenticated Tauri host**: the native Rust application enrolls/authenticates as a local `golamd` client using the existing IPC trust boundary. It forwards only narrow typed requests; client credentials and authority-bearing tokens never enter the webview.
6. **Visible local control surface**: the native host provides or coordinates at least one qualified persistent local indicator/control channel for active autonomous control. Trusted Rust receives/observes its liveness and routes immediate human pause/stop/takeover to protected control-lease authority. Renderer-only liveness is insufficient.
7. **Tauri renderer boundary**: sanitized commands/state only; frontend cannot hold raw platform handles, authenticate itself, mint capabilities or invoke adapters directly.

## Action lifecycle

`observe → select target → create ToolRequest → prepare immutable intent bound to current control-lease generation + qualified visible-control-channel state → capability/policy/approval → Effect PREPARED → Kernel/Effect Gate → immediate identity/permission/binding/lease-generation/visible-channel revalidation → platform dispatch → post-action observation → Effect terminal/reconciliation`

If execution may have crossed the side-effect boundary and terminal truth is uncertain, the result is `UNKNOWN_OUTCOME` and blocks conflicting follow-up until reconciliation.

If no qualified visible-control channel is active at final revalidation, new autonomous interactive actuation fails closed before platform dispatch.

## Capture lifecycle

`select explicit source → create ToolRequest → prepare immutable capture intent with capability/policy/approval + exact source/limits → Effect PREPARED → Kernel/Effect Gate → immediate request/effect/intent/source/permission revalidation → bounded native capture → compute digest/metadata → finalize Effect terminal status → hand ephemeral bytes only to authorized local consumer → release native resources`

If capture may have crossed the effect boundary but terminal truth is uncertain, persist `UNKNOWN_OUTCOME`; block conflicting retry or reuse until reconciliation determines terminal truth. No OCR or screenshot-derived semantic authority exists in Spec 006.

## Vision/pixel fallback lifecycle

The constitutional last-resort vision/pixel path is intentionally narrower than a semantic vision subsystem:

`bounded explicitly authorized capture → local consumer proposes bounded PixelTargetHint(region/coordinate + capture/source provenance) → treat hint as untrusted evidence only → fresh work-surface/focus/session observation → create separate RAW_INPUT_FALLBACK ToolRequest → explicit fallback capability/policy/approval → Effect PREPARED → immediate hint/source/target/lease-generation/visible-channel/permission revalidation → bounded raw dispatch → post-action evidence/reconciliation`

Rules:
- semantic/native/accessibility paths remain preferred;
- the hint cannot contain or manufacture an OS handle, capability, approval or semantic identity;
- OCR/text extraction from raw pixels remains deferred to Spec 007;
- the hint alone is never sufficient target identity and never authorizes raw input;
- capture authority never implies raw-input authority;
- loss of the qualified visible-control channel blocks raw dispatch like any other autonomous actuation.

## Clipboard lifecycle

`create explicit read/write ToolRequest → bind immutable request/effect/intent + capability/policy/approval → Effect PREPARED → immediate binding/permission revalidation → one bounded clipboard operation → terminal evidence/reconciliation → discard read payload unless separately authorized`

Clipboard polling/background inspection is never an implementation path.

## Human pause/stop/takeover and visibility lifecycle

Human interrupt is protected control-plane authority, not a renderer flag:

`qualified visible local control channel active → local human interrupt → authenticate/attribute interrupt source → atomically advance/revoke/suspend agent input lease generation → block new conflicting prepare/dispatch → invalidate queued/prepared actions bound to prior generation → cancel adapter work where cancellation is safe → preserve terminal/UNKNOWN_OUTCOME reconciliation for already-crossed effects → expose human-exclusive or paused state`

Visibility failure is also protected safety state:

`trusted visible-channel liveness/visibility loss → no other qualified visible channel active → mark autonomous actuation suspended → reject new interactive actuation before adapter dispatch → preserve observations/evidence/reconciliation allowed by separate authority → restore actuation only after a qualified visible channel is re-established and ordinary authority is freshly revalidated`

Requirements:
- stale model/UI requests cannot restore a revoked generation or fabricate visible-channel liveness;
- takeover remains effective across daemon/client reconnect until explicitly released through protected policy;
- takeover latency is measured from accepted protected interrupt to conflicting input-authority revocation/suspension;
- wrong-window, focus theft, stale refs and session transitions remain fail closed during takeover transitions;
- renderer crash/reload cannot cause silent continued actuation if it removes the only qualified visible-control channel.

## Dependency and platform admission sequence

1. Pure/fake-backend contracts use existing admitted primitives where possible; any new dependency requires exact Source Foundry qualification before manifest mutation.
2. Before Tauri/React/TypeScript dependency introduction, qualify exact selected versions, dependency closure, permissions, network/build behavior and notices through Source Foundry.
3. Windows semantic/capture/raw adapters may introduce only exact qualified platform crates/libraries after Source Foundry admission; qualify locked/UAC/secure-desktop and session-transition fail-closed behavior.
4. macOS accessibility/capture/raw adapters may introduce only exact qualified bindings/libraries after Source Foundry admission; qualify TCC permission changes and stale element/session behavior.
5. Linux AT-SPI/X11/Wayland portal/EIS adapters may introduce only exact qualified bindings/libraries after Source Foundry admission; qualify compositor/portal/session termination behavior.
6. Cross-platform fake/native qualification covers unsupported-state, permission-loss, uncertain-completion, stale/focus race, pixel-hint non-authority, visible-channel loss, human takeover and reconciliation.
7. Donor behavioral evidence remains reference-only unless a later bounded source is separately admitted through Source Foundry; no donor runtime or architecture is implicitly admitted.

## Final convergence

Cross-artifact analysis with no unresolved material contradiction → exact-head format/Clippy/tests/property/fuzz/security/platform qualification → fresh independent semantic/security/governance review on unchanged SHA → finding reconciliation → Ready → expected-head guarded merge → push-triggered CI on merge SHA → canonical closeout → successor-authority re-read.
