# Tasks: Spec 006 — Desktop Computer Control

## Planning lifecycle

- [x] **T006-P001** Re-read canonical Constitution, Spec 001 ordering and computer-control/security/transport contracts from `main@9400d4614318fffb2623ea71522ecd5f0f95f96a`.
- [x] **T006-P002** Reverify T060 successor authority and absence of an existing canonical Spec 006 branch/PR.
- [x] **T006-P003** [P] Mine `TheHalfMoon/Golam-research@a9f633e09d49a85829b8236331b9e21f7e612634` as untrusted reference evidence; qualify exact desktop/computer behavioral evidence while admitting no donor runtime/authority implementation and no donor architecture authority.
- [x] **T006-P004** [P] Qualify official Windows UI Automation and Windows.Graphics.Capture direction.
- [x] **T006-P005** [P] Qualify official macOS AXUIElement/Accessibility and ScreenCaptureKit permission direction.
- [x] **T006-P006** [P] Qualify Linux AT-SPI plus XDG RemoteDesktop/ScreenCast and EIS/libei direction; keep X11 explicit-session only.
- [x] **T006-P007** [P] Qualify Tauri 2 capabilities/permissions as least-privilege frontend boundary; record that implementation dependencies still require exact Source Foundry admission.
- [x] **T006-P008** Complete specification, clarification closeout, research, plan, data model, quickstart, checklist, contracts, tasks and cross-artifact analysis; reconcile constitutional route ordering/fallback eligibility, canonical T062 authenticated Tauri client authority, T064 locked/UAC fail-closed behavior, T067 bounded vision/pixel fallback without OCR, T069 human takeover at lease/input-authority layer, constitutional visible autonomous-control/interruptibility requirements, and dependency Source Foundry preconditions.
- [ ] **T006-P009** Run exact-head CI on the complete repaired planning branch. Pre-repair CI run `34053319742` on `15ca2ba1904fefefb86b8b2f919da365fe2de235` is superseded by the forward-only planning repair and cannot satisfy this gate.
- [ ] **T006-P010** Request fresh independent semantic/security/governance review only after P009 succeeds, bound to the unchanged repaired planning SHA.
- [ ] **T006-P011** Reconcile every material review finding forward-only; if head changes, repeat P009 and P010.
- [ ] **T006-P012** Mark planning PR Ready only after exact-head CI and review pass.
- [ ] **T006-P013** Guarded expected-head merge of the exact qualified planning SHA.
- [ ] **T006-P014** Verify canonical `main` equals the returned merge SHA and require push-triggered CI success on that exact merge SHA.
- [ ] **T006-P015** Record `SPEC_006_PLANNING_CLOSED_CANONICAL=YES`; only then authorize implementation branch creation.

## Implementation Phase A — Pure contracts and fake backend

- [ ] **T006-001** Add versioned desktop-control core types, access modes, limits, capability discovery, canonical `FallbackEligibilityEvidence`, protected control-lease state, bounded `PixelTargetHint`, visible-control-channel state, human interrupt evidence and canonical encodings/digests.
- [ ] **T006-002** Add opaque work-surface and semantic-element identities with stale/substitution validation.
- [ ] **T006-003** Add pure prepared semantic-action, raw-fallback, capture and clipboard intents with distinct immutable request/effect bindings, canonical request/intent digests, exact fallback-eligibility evidence for weaker routes, exact control-lease generation where applicable and distinct authority classes.
- [ ] **T006-004** Add a platform-neutral `DesktopBackend` trait and deterministic fake backend; keep route selection/fallback eligibility, protected lease/input-authority and visible-control-channel safety state above untrusted adapters.
- [ ] **T006-005** Add fake-backend contract tests for constitutional route ordering, stronger-route availability, unreconciled `UNKNOWN_OUTCOME`, observation bounds, permission loss, focus race, stale target, semantic action, capture, pixel-hint staleness/non-authority, raw fallback, clipboard denial, superseded control-lease generation and loss of the qualified visible-control channel; reject missing, mismatched, stale or substituted request/effect/authority/fallback-eligibility bindings before dispatch.

## Phase B — Authority, effect, route, interrupt and evidence lifecycle

- [ ] **T006-006** Add Kernel route evaluation plus prepare/revalidate/dispatch/finalize lifecycle for semantic actions with immutable ToolRequest/effect/intent bindings; select the highest applicable authorized constitutional route and bind exact current control-lease generation for interactive effects.
- [ ] **T006-007** Add distinct Kernel/Effect lifecycle for raw input fallback plus protected control-lease/input-authority state, qualified visible-control-channel state, and immediate human `PAUSE`/`STOP`/`TAKEOVER`/release transitions; raw fallback requires canonical fresh fallback-eligibility evidence, explicit policy/approval and cannot inherit semantic/capture/pixel authority. New autonomous input fails closed while a stronger applicable route remains eligible, an `UNKNOWN_OUTCOME` is unresolved, or no qualified visible control channel is active.
- [ ] **T006-008** Add bounded capture lifecycle with ToolRequest creation, capability/policy/approval refs, `Effect PREPARED`, Kernel/Effect Gate dispatch, immediate binding/source/permission revalidation, ephemeral payload default and metadata/digest evidence.
- [ ] **T006-009** Add explicit clipboard read/write lifecycle with immutable request/effect/intent bindings; deny silent/background inspection.
- [ ] **T006-010** Add durable secret-safe route-disposition, desktop-control and human-interrupt evidence, request/effect/intent/fallback-eligibility/control-lease digests, visible-channel transitions, terminal status including `UNKNOWN_OUTCOME`, takeover latency fields and reconciliation metadata.
- [ ] **T006-011** Add restart/reconnect reconciliation for nonterminal desktop effects/sessions, route eligibility, control leases and visible-control-channel state without widening authority; conflicting retries/fallback escalation remain blocked until terminal truth is established, stale lease generations cannot restore agent input after human takeover, and invisible autonomous actuation never resumes from cached UI state.

## Phase C — Authenticated Tauri desktop shell and RPC boundary

- [ ] **T006-012** Before manifest mutation, Source Foundry-qualify the exact selected Tauri 2/React/TypeScript dependency set and bounded closure; then create/extend the Tauri 2 desktop client shell as an authenticated local client of `golamd` through the existing authenticated IPC/client-enrollment boundary, without privileged Node/Electron runtime.
- [ ] **T006-013** Define narrow Tauri capabilities/permissions for sanitized desktop observation/action/capture/control-state surfaces.
- [ ] **T006-014** Implement typed RPC/command DTOs that expose no raw OS handles, tokens, pointers, unrestricted adapter access, `golamd` client authentication material, capability tokens or protected control-lease mutation authority.
- [ ] **T006-015** Add a persistent qualified local visible autonomous-control indicator/control surface with immediate pause/stop/takeover plus frontend authentication-disconnected, permission, unsupported, stale, paused/takeover and unknown-outcome UX states. Renderer-only state cannot create or clear takeover; loss of all qualified visible-control channels suspends new autonomous actuation fail closed.

## Phase D — Semantic observation, focus and identity

- [ ] **T006-016** Implement bounded work-surface/window/monitor enumeration behind platform adapters.
- [ ] **T006-017** Implement semantic element observation with explicit depth/node/time/string bounds.
- [ ] **T006-018** Implement focused work-surface tracking and stale/focus race invalidation, including control-lease generation and human-takeover invalidation for focus-dependent actions.
- [ ] **T006-019** Add cross-platform fake and adapter tests proving observation, semantic text, coordinates, screenshots and pixel hints never imply actuation authority or fallback eligibility.

## Phase E — Bounded capture

- [ ] **T006-020** Implement selected-source capture contract and resource bounds through the governed ToolRequest → Effect PREPARED → Gate → revalidate → dispatch → terminal/reconciliation lifecycle.
- [ ] **T006-021** After exact Source Foundry qualification for any introduced Windows binding/library, implement Windows.Graphics.Capture selected display/window path and permission/support/session-transition tests.
- [ ] **T006-022** After exact Source Foundry qualification for any introduced macOS binding/library, implement ScreenCaptureKit selected-source path and Screen Recording/TCC permission tests.
- [ ] **T006-023** After exact Source Foundry qualification for any introduced Linux binding/library, implement XDG ScreenCast/portal selected-source path where supported; explicit unsupported/session-termination paths elsewhere.
- [ ] **T006-024** Prove capture bytes are ephemeral by default, excluded from ordinary logs/evidence, cannot authorize actions or fallback eligibility, and permission-loss/uncertain-completion paths fail closed with durable `UNKNOWN_OUTCOME` where terminal truth is uncertain.

## Phase F — Semantic actuation

- [ ] **T006-025** After exact Source Foundry qualification for any introduced Windows binding/library, implement Windows UI Automation semantic action adapter and target/locked/UAC/secure-desktop/session-transition revalidation tests.
- [ ] **T006-026** After exact Source Foundry qualification for any introduced macOS binding/library, implement AXUIElement semantic action adapter and permission/stale-element/TCC tests.
- [ ] **T006-027** After exact Source Foundry qualification for any introduced Linux binding/library, implement AT-SPI semantic action adapter and accessibility-service/session tests.
- [ ] **T006-028** Add post-action verification and outcome evidence across supported semantic action families, including route disposition and interrupted/takeover outcomes.

## Phase G — Explicit raw fallback, bounded pixel hint and clipboard

- [ ] **T006-029** After exact Source Foundry qualification for any introduced Windows binding/library, implement explicit `SendInput` fallback behind canonical fallback-eligibility evidence plus separate capability/policy/approval/effect/control-lease generation; permanently deny locked/UAC/secure desktop control.
- [ ] **T006-030** After exact Source Foundry qualification for any introduced macOS binding/library, implement explicit raw input fallback only after route eligibility is established and OS permission/policy permit it; no background capture/keylogging.
- [ ] **T006-031** After exact Source Foundry qualification for any introduced Linux binding/library, implement raw input only after route eligibility is established, for explicit X11 sessions or compositor/user-granted Wayland RemoteDesktop/EIS path; no bypass.
- [ ] **T006-032** After exact Source Foundry qualification for any introduced clipboard binding/library, implement explicit clipboard read/write adapters with immutable request/effect/intent bindings, permission/capability gating and no polling.
- [ ] **T006-033** Implement bounded `PixelTargetHint` plumbing from explicitly authorized capture to raw fallback as untrusted geometry only; no OCR/text extraction, no authority or fallback-eligibility minting, no stale-source reuse. Add adversarial tests proving semantic failure, capture success or pixel hints cannot silently escalate to raw input/clipboard authority and human takeover blocks stale input generations.

## Phase H — Security and platform qualification

- [ ] **T006-034** Prove camera and microphone APIs are absent/denied from the Spec 006 control surface.
- [ ] **T006-035** Prove no OCR/text extraction from raw screenshots and no screenshot/pixel-hint-derived authority or fallback eligibility before Spec 007.
- [ ] **T006-036** Prove no hidden network/remote/cloud fallback or HTTP dependency in desktop-control execution.
- [ ] **T006-037** Add malicious titles/accessibility text/clipboard/screenshot metadata/pixel-hint provenance tests as untrusted data; none can influence route eligibility without trusted evaluation evidence.
- [ ] **T006-038** Add stronger-route-still-available, stale/missing fallback eligibility, unreconciled `UNKNOWN_OUTCOME`, stale window reuse, process restart, focus theft, wrong-window substitution, permission revocation, Windows locked/UAC/secure-desktop/session transitions, capture permission loss after prepare, uncertain completion, portal-session termination, target substitution, visible-control-channel loss, human pause/stop/takeover, stale lease generation and takeover-latency adversarial tests; prove weaker escalation/invisible autonomous actuation is suspended and `UNKNOWN_OUTCOME` blocks conflicting retry until reconciliation.
- [ ] **T006-039** Run supported Windows/macOS/Linux adapter qualification with deterministic explicit unsupported/permission-limited dispositions and exact Source Foundry evidence for every introduced dependency/runtime primitive.

## Phase I — Core Alpha desktop E2E

- [ ] **T006-040** E2E: authenticated Tauri native host enrolls/authenticates to `golamd` without exposing auth material to renderer → persistent visible autonomous-control indicator is active → route evaluation selects highest applicable authorized route → observe surface → choose semantic target → authorize → semantic action → deterministic post-action evidence.
- [ ] **T006-041** E2E: explicitly select window/display → create bound capture request/effect → bounded capture → ephemeral local consumption → optional bounded pixel hint remains non-authoritative/non-eligibility-bearing → terminal evidence → release resources.
- [ ] **T006-042** E2E: stronger routes unavailable/inapplicable/denied/safely failed → canonical fallback eligibility permits deterministic raw path → raw fallback remains denied by default → optional bounded pixel hint cannot authorize action → explicitly authorized matching fallback succeeds only under fresh target/focus/session/control-lease/visible-channel authority.
- [ ] **T006-043** E2E: stronger route becomes available, `UNKNOWN_OUTCOME`, permission/focus/identity/binding/control-lease drift between prepare and execute all fail closed without weaker escalation; loss of the qualified visible-control channel suspends new actuation; human takeover invalidates queued/prepared stale input within measured bound; uncertain completion survives restart/reconnect and blocks conflicting retry until reconciliation.

## Phase J — Final convergence

- [ ] **T006-044** Cross-artifact audit: every FR/NFR and canonical T060–T069/Constitution requirement maps to implementation, tests and evidence; clarification/analysis remain consistent; remove temporary qualification scaffolding.
- [ ] **T006-045** Full exact-head Windows/macOS/Ubuntu CI and security qualification.
- [ ] **T006-046** Fresh independent semantic/security/governance review on exact unchanged final implementation SHA.
- [ ] **T006-047** Reconcile findings; any mutation invalidates prior final CI/review and requires fresh qualification.
- [ ] **T006-048** Mark implementation PR Ready only after exact-head gates pass.
- [ ] **T006-049** Guarded expected-head merge; no force/rebase/history rewrite.
- [ ] **T006-050** Post-merge push CI on exact canonical merge SHA.
- [ ] **T006-051** Record `SPEC_006_IMPLEMENTATION_COMPLETE=YES` and `SPEC_006_CLOSED_CANONICAL=YES` only after T006-050 passes.
- [ ] **T006-052** Re-read canonical program ordering and determine the next authorized successor unit; do not infer authority from stale planning PRs.
