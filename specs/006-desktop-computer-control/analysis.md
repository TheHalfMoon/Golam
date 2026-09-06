# Cross-Artifact Analysis: Spec 006 — Desktop Computer Control

## Purpose

This analysis verifies the planning package against the Golam Constitution v1.2.0, canonical Spec 001 Phase 5 authority T060–T069, predecessor security/effect semantics, root repository governance, and the internal Spec 006 artifacts before implementation authorization.

`ANALYSIS_SCOPE=PLANNING_ONLY`
`PRODUCT_IMPLEMENTATION_AUTHORIZED=NO`

## Canonical authority reconciliation

| Canonical requirement | Spec 006 planning disposition | Evidence location |
| --- | --- | --- |
| T060 bounded Spec 006 after Spec 005 close | Present; planning-only branch from canonical base | root `AGENTS.md`, `spec.md`, local `AGENTS.md`, `tasks.md` |
| T061 seriously mine donor desktop behavior without importing Electron trust architecture | Present; pinned behavioral/reference matrix, no runtime/architecture admission | `research.md` |
| T062 Tauri desktop is an authenticated client of `golamd`; renderer untrusted | Present after forward repair | `spec.md` FR-025, `plan.md`, `ui-rpc-contract.md`, `tasks.md` T006-012/T006-014 |
| T063 semantic snapshot/ref/action contract | Present | controller/observation/action/focus contracts |
| T064 Windows UIA-first; locked/UAC/secure desktop fail closed | Present after forward repair | `spec.md` FR-014/FR-015, platform/action contracts, tasks qualification |
| T065 macOS AX/TCC + explicit Accessibility/Screen Recording state | Present | `research.md`, platform/capture contracts, tasks |
| T066 Linux AT-SPI/X11/Wayland honest portal/compositor failures | Present | `research.md`, platform contract, tasks |
| T067 input injection and vision fallbacks only behind semantic failure/need + post-action verification | Present after forward repair: raw input is explicit-only; bounded `PixelTargetHint` is untrusted evidence only; OCR remains deferred; canonical route eligibility is required before weaker fallback | `spec.md`, `plan.md`, `data-model.md`, action/capture contracts, tasks |
| T068 clipboard split; camera/mic deny-by-default | Present | clipboard/capture contracts, `spec.md`, tasks |
| T069 human takeover at lease/input-authority layer + latency/stale-ref/wrong-window tests | Present after forward repair | `spec.md` US7/FR-026/FR-027/NFR-008, `plan.md`, `data-model.md`, action/focus/controller contracts, tasks |

## Constitution reconciliation

### Local ownership / strict-local

PASS. No hidden network/cloud fallback is part of desktop control. New implementation dependencies do not become admitted merely because official platform APIs are selected.

### Rust trusted path / renderer isolation

PASS. Rust owns authority, evidence and adapter dispatch. The Tauri renderer is an untrusted projection. The native Rust host authenticates to `golamd`; renderer state cannot become authentication.

### Constitutional control-route ordering

PASS after forward repair. The exact order is preserved: `domain/application API → native OS automation API → accessibility/semantic tree → browser DOM/protocol → deterministic keyboard/mouse control → vision/pixel fallback`. Trusted orchestration records canonical `FallbackEligibilityEvidence`; a weaker route is denied while a stronger applicable route remains available/authorized, and `UNKNOWN_OUTCOME` blocks conflicting weaker escalation until reconciliation. Raw adapters and pixel-hint producers cannot self-mint route eligibility.

### Explicit non-self-expanding authority

PASS. Observation, capture, semantic action, raw fallback, clipboard and protected control-lease state are separated. Pixel hints are evidence only. No downstream component can turn semantic failure or pixel data into authority or fallback eligibility.

### Consequential effect lifecycle

PASS. Side-effect paths require immutable request/effect/intent bindings, capability/policy/approval, `Effect PREPARED`, Kernel/Effect Gate, immediate route/authority revalidation, terminal evidence and `UNKNOWN_OUTCOME` reconciliation.

### Visible computer control and human interruptibility

PASS after forward repair. Active autonomous computer control requires at least one qualified persistent local visible-control channel with immediate pause/stop/takeover. Trusted Rust tracks channel state rather than trusting DOM visibility. Loss of all qualified visible channels suspends new autonomous actuation fail closed. Human pause/stop/takeover is enforced at protected lease/input authority and invalidates stale generations.

### Source permission / Source Foundry

PASS as a planning constraint. Donor behavioral evidence remains reference-only. Official platform/API research is not dependency admission. Every new crate/package/native binding/helper/copied source requires exact Source Foundry admission before implementation use.

## Cross-artifact invariants

1. **Exact route order** — a weaker control path cannot bypass a stronger applicable authorized route; canonical fallback-eligibility evidence is required and `UNKNOWN_OUTCOME` blocks escalation.
2. **No authority from observation** — consistent across spec, data model and controller/observation contracts.
3. **No authority from pixels** — `PixelTargetHint` carries provenance and bounded geometry only; raw fallback requires independent authority plus fresh fallback-eligibility/target/focus/session/lease/visible-channel validation.
4. **No OCR in Spec 006** — raw screenshot text extraction remains deferred to Spec 007 across spec, research, plan, capture contract and clarification closeout.
5. **No renderer authority** — UI RPC exposes sanitized/opaque refs only; renderer cannot receive native handles or client authentication material.
6. **Autonomous control stays visible** — at least one qualified persistent local visible-control channel must remain active; losing all such channels suspends new actuation and cached renderer state cannot satisfy visibility.
7. **No automatic raw fallback** — semantic failure alone never silently escalates; trusted route eligibility is required.
8. **Human takeover is kernel/lease state** — stale UI/model state cannot re-enable a revoked generation.
9. **Ambiguity never permits replay or weaker escalation** — `UNKNOWN_OUTCOME` blocks conflicting retry/fallback until reconciliation, including across restart/takeover.
10. **Platform permissions remain external mutable state** — they are prerequisites, not Golam grants.
11. **Windows protected desktop boundaries are not bypassed** — locked/UAC/secure-desktop transitions fail closed.
12. **Dependencies remain unadmitted until Source Foundry** — no planning research URL or donor reference is treated as implementation admission.
13. **Root governance follows live successor phase** — root `AGENTS.md` no longer advertises stale Spec 005 convergence; it records closed Spec 005 predecessor state, T060 planning authority and the fail-closed Spec 006 planning lifecycle without hardcoding a transient PR head.

## Requirement-to-implementation-phase coverage

| Requirement family | Planned phase(s) |
| --- | --- |
| Route eligibility, core identities/digests/capabilities/pixel hint/control lease/visible-control channel | A |
| Route/effect/authority/reconciliation/takeover generation/visible-channel suspension | B |
| Authenticated Tauri client + persistent visible control surface + sanitized renderer RPC | C |
| Semantic observation/focus/identity | D |
| Bounded capture | E |
| Native semantic actuation | F |
| Raw input, route eligibility, pixel hint fallback, clipboard, takeover adversarial behavior | G/H |
| Platform/security/session/visible-channel/route-order qualification | H |
| End-to-end route selection/semantic/capture/fallback/takeover/authentication/visibility scenarios | I |
| Cross-artifact audit, exact-head CI/review/merge/post-merge closeout | J |

## Planning gaps found and repaired forward-only

The canonical re-read after initial exact-head CI found five material omissions in the planning package:

1. authenticated Tauri→`golamd` client semantics from T062 were not explicit;
2. T067 vision/pixel fallback had been omitted rather than bounded separately from OCR;
3. T069 human takeover at protected lease/input authority and takeover-latency qualification were missing;
4. locked/UAC/session-transition Windows qualification was underspecified relative to T064;
5. the explicit Spec Kit clarification/analyze artifacts and dependency Source Foundry preconditions were not complete.

A subsequent Constitution/governance-focused audit found three additional material planning/governance omissions before final candidate freeze:

6. active autonomous computer control visibility and the fail-closed behavior when every qualified visible-control channel is lost were not explicit enough;
7. the exact constitutional route order, including domain/app and browser routes outside Spec 006, was not bound to deterministic fallback-eligibility evidence, leaving room for a weaker adapter to interpret ordinary failure as permission to escalate;
8. root `AGENTS.md` still advertised stale Spec 005 final-convergence state and stale pending gates after Spec 005 had already closed canonical and T060 successor planning authority had been established.

All eight omissions were repaired forward-only before final qualification. These are planning/governance defects, not authorization to implement product code. They invalidate qualification use of every pre-repair exact-head CI/review request. The final repaired planning head must receive fresh exact-head CI and fresh independent substantive review.

## Remaining planning gates

- complete cross-artifact repair/reconciliation and freeze the candidate head;
- run exact-head CI on that complete head;
- obtain fresh independent semantic/security/governance review after CI on the unchanged head;
- reconcile every material finding forward-only, repeating CI/review after mutation;
- Ready transition only on an unchanged qualified head;
- guarded expected-head merge;
- push-triggered CI on the exact returned canonical-main merge SHA;
- only then record planning closed canonical and authorize implementation branch creation.

`ANALYSIS_RESULT=NO_KNOWN_UNRESOLVED_MATERIAL_CROSS_ARTIFACT_CONTRADICTION_BEFORE_EXTERNAL_REVIEW`
`EXTERNAL_EXACT_HEAD_REVIEW_STILL_REQUIRED=YES`
`WAIVER_TAKEN=NO`
