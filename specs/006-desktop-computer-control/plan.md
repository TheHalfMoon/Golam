# Implementation Plan: Desktop Computer Control

## Summary

Build a Rust-owned semantic-first desktop-control subsystem and a least-privilege Tauri 2 client boundary. The common contract must model observation, target identity, permission/session state, bounded capture, semantic action, explicit raw-input fallback and clipboard operations. Native adapters are admitted only after fake-backend contract qualification and platform-specific permission/race testing.

## Technical context

- Trusted core/runtime: Rust workspace (`golam-core`, `golam-kernel`, `golam-ledger`, `golamd`).
- Desktop shell: Tauri 2 + React + TypeScript, with narrow capability files and typed sanitized RPC DTOs.
- Windows: Microsoft UI Automation for semantic tree/actions; Windows.Graphics.Capture for selected display/window capture; raw input fallback only through explicitly governed `SendInput`; secure desktop denied.
- macOS: AXUIElement/Accessibility for semantic observation/action; ScreenCaptureKit under system Screen Recording permission; TCC state treated as external authority state.
- Linux: AT-SPI semantic layer; X11 raw facilities only for identified X11 sessions; Wayland ScreenCast/RemoteDesktop portals plus compositor-provided EIS/libei for granted control.
- Persistence: authority/effect metadata and digests only by default; raw captures/clipboard payloads are not ordinary durable evidence.

## Constitution check

- **Spec before implementation**: PASS — this planning PR contains no product implementation.
- **Local-first / strict-local**: PASS — no remote fallback or hidden network dependency.
- **Least authority**: PASS — observation/capture/semantic action/raw input/clipboard are distinct authority surfaces.
- **Effect governance**: PASS — all side effects traverse Kernel/Effect Gate.
- **Evidence integrity**: PASS — prepared action and outcomes bind canonical target/authority state.
- **Secrets/privacy**: PASS — raw capture and clipboard content excluded from logs/evidence by default.
- **Cross-platform honesty**: PASS — capability discovery + explicit unsupported states replace fake parity.
- **Donor discipline**: PASS — Golam-research is reference-only and cannot define trusted architecture.

## Architecture

1. **Pure contracts** in `golam-core`: identities, observations, action/capture intents, capability descriptions, errors and canonical digests.
2. **Authority orchestration** in `golam-kernel`: prepare/revalidate/dispatch/finalize lifecycle bound to capability/policy/approval/effect records.
3. **Durable evidence** in `golam-ledger`: prepared intent, action outcome, permission/session observation and reconciliation metadata without raw sensitive payloads.
4. **Adapter interface** in `golamd`: fake backend plus Windows/macOS/Linux implementations behind a common trait.
5. **Tauri boundary**: sanitized commands only; frontend cannot hold raw platform handles or invoke adapters directly.

## Action lifecycle

`observe → select target → prepare intent → capability/policy/approval → Effect PREPARED → immediate identity/permission revalidation → platform dispatch → post-action observation → Effect terminal/reconciliation`

If execution may have crossed the side-effect boundary and terminal truth is uncertain, the result is `UNKNOWN_OUTCOME` and blocks conflicting follow-up until reconciliation.

## Capture lifecycle

`select explicit source → verify capture capability + system permission → create bounded capture intent → capture bounded frame/snapshot → compute digest/metadata → hand ephemeral bytes only to authorized local consumer → release native resources`

No OCR or screenshot-derived semantic authority exists in Spec 006.

## Platform admission sequence

1. Fake backend contract and adversarial suite.
2. Windows semantic observation/action + selected capture + permission/identity tests.
3. macOS accessibility + ScreenCaptureKit permission/session tests.
4. Linux AT-SPI + explicit X11 capability + Wayland portal/EIS capability/session tests.
5. Cross-platform unsupported-state and stale/focus race tests.

## Final convergence

Exact-head format/Clippy/tests/property/fuzz/security/platform qualification → fresh independent semantic/security review on unchanged SHA → finding reconciliation → Ready → expected-head guarded merge → push-triggered CI on merge SHA → canonical closeout.
