# Tasks: Spec 006 — Desktop Computer Control

## Planning lifecycle

- [x] **T006-P001** Re-read canonical Constitution, Spec 001 ordering and computer-control/security/transport contracts from `main@9400d4614318fffb2623ea71522ecd5f0f95f96a`.
- [x] **T006-P002** Reverify T060 successor authority and absence of an existing canonical Spec 006 branch/PR.
- [x] **T006-P003** [P] Mine `TheHalfMoon/Golam-research@a9f633e09d49a85829b8236331b9e21f7e612634` as untrusted reference evidence; qualify exact desktop/computer behavioral evidence while admitting no donor runtime/authority implementation and no donor architecture authority.
- [x] **T006-P004** [P] Qualify official Windows UI Automation and Windows.Graphics.Capture direction.
- [x] **T006-P005** [P] Qualify official macOS AXUIElement/Accessibility and ScreenCaptureKit permission direction.
- [x] **T006-P006** [P] Qualify Linux AT-SPI plus XDG RemoteDesktop/ScreenCast and EIS/libei direction; keep X11 explicit-session only.
- [x] **T006-P007** [P] Qualify Tauri 2 capabilities/permissions as least-privilege frontend boundary.
- [x] **T006-P008** Write specification, plan, research, data model, quickstart, checklist and contracts.
- [ ] **T006-P009** Run exact-head CI on the complete planning branch.
- [ ] **T006-P010** Request fresh independent semantic/security/governance review only after P009 succeeds, bound to the unchanged planning SHA.
- [ ] **T006-P011** Reconcile every material review finding forward-only; if head changes, repeat P009 and P010.
- [ ] **T006-P012** Mark planning PR Ready only after exact-head CI and review pass.
- [ ] **T006-P013** Guarded expected-head merge of the exact qualified planning SHA.
- [ ] **T006-P014** Verify canonical `main` equals the returned merge SHA and require push-triggered CI success on that exact merge SHA.
- [ ] **T006-P015** Record `SPEC_006_PLANNING_CLOSED_CANONICAL=YES`; only then authorize implementation branch creation.

## Implementation Phase A — Pure contracts and fake backend

- [ ] **T006-001** Add versioned desktop-control core types, access modes, limits, capability discovery and canonical encodings/digests.
- [ ] **T006-002** Add opaque work-surface and semantic-element identities with stale/substitution validation.
- [ ] **T006-003** Add pure prepared semantic-action, raw-fallback, capture and clipboard intents with distinct immutable request/effect bindings, canonical request/intent digests and distinct authority classes.
- [ ] **T006-004** Add a platform-neutral `DesktopBackend` trait and deterministic fake backend.
- [ ] **T006-005** Add fake-backend contract tests for observation bounds, permission loss, focus race, stale target, semantic action, capture, fallback and clipboard denial; reject missing, mismatched, stale or substituted request/effect/authority bindings before dispatch.

## Phase B — Authority, effect and evidence lifecycle

- [ ] **T006-006** Add Kernel prepare/revalidate/dispatch/finalize lifecycle for semantic actions with immutable ToolRequest/effect/intent bindings.
- [ ] **T006-007** Add distinct Kernel/Effect lifecycle for raw input fallback with explicit policy/approval requirement and no implicit authority escalation.
- [ ] **T006-008** Add bounded capture lifecycle with ToolRequest creation, capability/policy/approval refs, `Effect PREPARED`, Kernel/Effect Gate dispatch, immediate binding/source/permission revalidation, ephemeral payload default and metadata/digest evidence.
- [ ] **T006-009** Add explicit clipboard read/write lifecycle with immutable request/effect/intent bindings; deny silent/background inspection.
- [ ] **T006-010** Add durable secret-safe desktop-control evidence, request/effect/intent digests, terminal status including `UNKNOWN_OUTCOME`, and reconciliation metadata.
- [ ] **T006-011** Add restart reconciliation for nonterminal desktop effects/sessions without widening authority; conflicting retries remain blocked until terminal truth is established.

## Phase C — Tauri desktop shell and RPC boundary

- [ ] **T006-012** Create/extend Tauri 2 + React + TypeScript desktop client shell without privileged Node/Electron runtime.
- [ ] **T006-013** Define narrow Tauri capabilities/permissions for sanitized desktop observation/action/capture state.
- [ ] **T006-014** Implement typed RPC/command DTOs that expose no raw OS handles, tokens, pointers or unrestricted adapter access.
- [ ] **T006-015** Add frontend permission/unsupported/stale/unknown-outcome UX states with no authority interpretation in the webview.

## Phase D — Semantic observation, focus and identity

- [ ] **T006-016** Implement bounded work-surface/window/monitor enumeration behind platform adapters.
- [ ] **T006-017** Implement semantic element observation with explicit depth/node/time/string bounds.
- [ ] **T006-018** Implement focused work-surface tracking and stale/focus race invalidation.
- [ ] **T006-019** Add cross-platform fake and adapter tests proving observation never implies actuation authority.

## Phase E — Bounded capture

- [ ] **T006-020** Implement selected-source capture contract and resource bounds through the governed ToolRequest → Effect PREPARED → Gate → revalidate → dispatch → terminal/reconciliation lifecycle.
- [ ] **T006-021** Windows adapter: Windows.Graphics.Capture selected display/window path and permission/support tests.
- [ ] **T006-022** macOS adapter: ScreenCaptureKit selected source path and Screen Recording permission tests.
- [ ] **T006-023** Linux adapter: XDG ScreenCast/portal selected-source path where supported; explicit unsupported paths elsewhere.
- [ ] **T006-024** Prove capture bytes are ephemeral by default, excluded from ordinary logs/evidence, cannot authorize actions, and permission-loss/uncertain-completion paths fail closed with durable `UNKNOWN_OUTCOME` where terminal truth is uncertain.

## Phase F — Semantic actuation

- [ ] **T006-025** Windows UI Automation semantic action adapter and target revalidation tests.
- [ ] **T006-026** macOS AXUIElement semantic action adapter and permission/stale-element tests.
- [ ] **T006-027** Linux AT-SPI semantic action adapter and accessibility-service/session tests.
- [ ] **T006-028** Add post-action verification and outcome evidence across supported semantic action families.

## Phase G — Explicit raw fallback and clipboard

- [ ] **T006-029** Windows explicit `SendInput` fallback behind separate capability/policy/approval/effect; permanently deny secure desktop.
- [ ] **T006-030** macOS explicit raw input fallback only where OS permission and policy permit; no background capture/keylogging.
- [ ] **T006-031** Linux raw input: explicit X11 session only or compositor/user-granted Wayland RemoteDesktop/EIS path; no bypass.
- [ ] **T006-032** Implement explicit clipboard read/write adapters with immutable request/effect/intent bindings, permission/capability gating and no polling.
- [ ] **T006-033** Add adversarial tests proving semantic failure cannot silently escalate to raw input or clipboard authority.

## Phase H — Security and platform qualification

- [ ] **T006-034** Prove camera and microphone APIs are absent/denied from the Spec 006 control surface.
- [ ] **T006-035** Prove no OCR/text extraction from raw screenshots and no screenshot-derived authority before Spec 007.
- [ ] **T006-036** Prove no hidden network/remote/cloud fallback or HTTP dependency in desktop-control execution.
- [ ] **T006-037** Add malicious titles/accessibility text/clipboard/screenshot metadata tests as untrusted data.
- [ ] **T006-038** Add stale window reuse, process restart, focus theft, permission revocation, capture permission loss after prepare, uncertain completion, portal-session termination and target substitution adversarial tests; prove `UNKNOWN_OUTCOME` blocks conflicting retry until reconciliation.
- [ ] **T006-039** Run supported Windows/macOS/Linux adapter qualification with deterministic explicit unsupported dispositions.

## Phase I — Core Alpha desktop E2E

- [ ] **T006-040** E2E: observe surface → choose semantic target → authorize → semantic action → deterministic post-action evidence.
- [ ] **T006-041** E2E: explicitly select window/display → create bound capture request/effect → bounded capture → ephemeral local consumption → terminal evidence → release resources.
- [ ] **T006-042** E2E: semantic action unavailable → raw fallback denied by default → explicitly authorized fallback succeeds only under matching authority.
- [ ] **T006-043** E2E: permission/focus/identity/binding drift between prepare and execute fails closed without side effect; uncertain completion survives restart and blocks conflicting retry until reconciliation.

## Phase J — Final convergence

- [ ] **T006-044** Cross-artifact audit: every FR/NFR maps to implementation, tests and evidence; remove temporary qualification scaffolding.
- [ ] **T006-045** Full exact-head Windows/macOS/Ubuntu CI and security qualification.
- [ ] **T006-046** Fresh independent semantic/security/governance review on exact unchanged final implementation SHA.
- [ ] **T006-047** Reconcile findings; any mutation invalidates prior final CI/review and requires fresh qualification.
- [ ] **T006-048** Mark implementation PR Ready only after exact-head gates pass.
- [ ] **T006-049** Guarded expected-head merge; no force/rebase/history rewrite.
- [ ] **T006-050** Post-merge push CI on exact canonical merge SHA.
- [ ] **T006-051** Record `SPEC_006_IMPLEMENTATION_COMPLETE=YES` and `SPEC_006_CLOSED_CANONICAL=YES` only after T006-050 passes.
- [ ] **T006-052** Re-read canonical program ordering and determine the next authorized successor unit; do not infer authority from stale planning PRs.
