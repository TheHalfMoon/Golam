# Golam Agent Instructions

## Current phase

Golam is in **Spec 006 implementation: Desktop Computer Control** on branch `impl/006-desktop-computer-control`.

Canonical Spec 006 planning closed on `main@c85b4b8f0d6ffccb039645803542d75b3bd47f29` after PR #23 merged exact qualified planning head `2c00a9a33bf3a1d82cbe2cc31891d6ccd3992364` and push-triggered CI run `34059235181` completed SUCCESS on macOS, Ubuntu, and Windows. Live closeout evidence records `SPEC_006_PLANNING_COMPLETE=YES`, `SPEC_006_PLANNING_CLOSED_CANONICAL=YES`, and `SPEC_006_PRODUCT_IMPLEMENTATION_AUTHORIZED=YES`.

Never treat a remembered implementation head, stale CI result, stale review, stale branch, or nonmerged proposal as qualification authority. Re-fetch live repository/PR/check/review state before every lifecycle decision.

## Authority order

1. exact live GitHub/repository truth;
2. `.specify/memory/constitution.md` v1.2.0 or later;
3. frozen Spec 001 program architecture/tasks/contracts/source-permission attestation;
4. canonical Spec 002–005 closeout packages and predecessor implementation evidence;
5. canonical Spec 006 planning package on `main@c85b4b8f0d6ffccb039645803542d75b3bd47f29` plus PR #23 closeout evidence;
6. the current implementation branch and exact live implementation PR lifecycle evidence;
7. exact Source Foundry records for every admitted dependency/runtime primitive.

Nonmerged proposals, status-only bot messages, stale comments, stale CI/reviews/hashes, old branch text, and prior handoffs cannot override live canonical truth.

## Spec 006 implementation read order

1. `.specify/memory/constitution.md`
2. `specs/001-golam-local-agent-os-foundation/spec.md`
3. `specs/001-golam-local-agent-os-foundation/plan.md`
4. `specs/001-golam-local-agent-os-foundation/tasks.md`, especially T060–T069
5. `specs/001-golam-local-agent-os-foundation/source-permission-attestation.md`
6. canonical Spec 002–005 predecessor closeout evidence as needed
7. `specs/006-desktop-computer-control/AGENTS.md`
8. `specs/006-desktop-computer-control/spec.md`
9. `specs/006-desktop-computer-control/clarification-closeout.md`
10. `specs/006-desktop-computer-control/research.md`
11. `specs/006-desktop-computer-control/plan.md`
12. `specs/006-desktop-computer-control/data-model.md`
13. all `specs/006-desktop-computer-control/contracts/`
14. `specs/006-desktop-computer-control/quickstart.md`
15. `specs/006-desktop-computer-control/checklists/requirements.md`
16. `specs/006-desktop-computer-control/tasks.md`
17. `specs/006-desktop-computer-control/analysis.md`
18. live implementation PR exact-head CI/review/lifecycle evidence

## Predecessor posture that Spec 006 must preserve

- Consequential execution remains behind canonical ToolRequest + capability/policy/approval + Effect PREPARED + Kernel/Effect Gate + immediate revalidation + durable terminal/reconciliation semantics.
- `UNKNOWN_OUTCOME` blocks conflicting retry until reconciliation. A later or weaker route never converts uncertainty into authority.
- Strict-local hard denial dominates tool/protocol/network routing. Local failure never authorizes cloud/remote fallback.
- Generic tools, model output, protocol output, renderer state, accessibility text, screenshots, and pixel hints cannot mint kernel authority.
- Protected Golam kernel resources remain outside generic filesystem/process authority.
- Existing production containment admissions remain bounded to their qualified platform/profile.
- Shell syntax remains disabled unless separately re-authorized by later canonical scope.
- Managed Markdown/SQLite memory governance, secret/taint boundaries, and untrusted Skill/MCP/ACP semantics remain predecessor constraints.

## Spec 006 hard boundaries

- Preserve the constitutional route order exactly: `domain/application API → native OS automation API → accessibility/semantic tree → browser DOM/protocol → deterministic keyboard/mouse control → vision/pixel fallback`.
- A weaker route requires canonical trusted `FallbackEligibilityEvidence`. An applicable stronger authorized route or unreconciled `UNKNOWN_OUTCOME` blocks weaker escalation.
- Desktop adapters, renderer/model output, and pixel/vision components cannot self-mint fallback eligibility or actuation authority.
- Observation, capture, semantic actuation, raw input, focus, clipboard read, and clipboard write remain distinct authority/evidence surfaces.
- Every side-effect/privacy-sensitive path binds exact ToolRequest/request digest, immutable Effect/effect binding, canonical intent digest, capability/policy/approval, target/session/permission state, and control-lease/visible-channel state where applicable.
- Every consequential desktop dispatch traverses `Effect PREPARED → Kernel/Effect Gate → immediate revalidation → bounded adapter dispatch → terminal evidence/reconciliation`.
- Missing, mismatched, stale, substituted, expired, or superseded authority/binding state fails closed before adapter dispatch.
- The Tauri native Rust host authenticates to `golamd` through the existing authenticated local IPC/client-enrollment boundary. Localhost/same-machine location and renderer state are not authentication.
- Renderer/webview code receives no raw privileged handles, local-client credentials, capability tokens, Gate authorization, or protected control-lease mutation authority.
- Autonomous computer actuation requires at least one qualified persistent local visible-control channel with immediate pause/stop/takeover. Loss of every qualified channel suspends new autonomous actuation fail closed.
- Human pause/stop/takeover is enforced at protected lease/input-authority state. Stale generations cannot restore superseded agent authority.
- Windows locked/non-interactive desktop, UAC/secure desktop, and unsupported interactive-session transitions fail closed. Secure-desktop/UAC bypass is not supported.
- macOS TCC/Accessibility/Screen Recording and Linux portal/compositor/session state are mutable external prerequisites, never blanket authority.
- No Wayland/compositor bypass, background keylogging, silent clipboard inspection, unbounded capture, camera/microphone collection, screenshot OCR under Spec 006, or hidden network/cloud fallback.
- Bounded vision/pixel fallback is untrusted geometry evidence only. It cannot mint semantic identity, capability, policy, approval, Gate authorization, or fallback eligibility. Raw screenshot OCR/text extraction remains deferred to Spec 007.
- Golam-research Electron/VNC/preload behavior remains reference evidence only unless a bounded component is separately Source Foundry admitted.
- Official API research is not dependency admission. Every new crate/package/native binding/library/helper/copied source requires exact Source Foundry admission before manifest/code use.

## Implementation execution discipline

Execute `specs/006-desktop-computer-control/tasks.md` in dependency order.

- T006-001 through T006-005 establish pure contracts and deterministic fake-backend qualification before native adapters.
- T006-006 through T006-011 establish Kernel authority/effect/route/interrupt/evidence/reconciliation lifecycle.
- T006-012 through T006-015 require exact Source Foundry admission before Tauri/React/TypeScript manifest mutation.
- Platform-specific dependencies for capture, semantic actuation, raw input, clipboard, portal/EIS, or native bindings require exact Source Foundry admission before use.
- Native adapters cannot select their own route, mint fallback evidence, widen authority, or bypass the Kernel/Effect Gate.
- Fake/adversarial qualification must cover route ordering, stale identity, permission loss, Gate absence/staleness/mismatch/substitution, `UNKNOWN_OUTCOME`, visible-channel loss, and human takeover before native adapter admission.

## Final implementation convergence

The implementation closeout sequence is fail closed:

1. T006-044 complete cross-artifact implementation/evidence audit;
2. T006-045 run full exact-head Windows/macOS/Ubuntu CI and security qualification;
3. T006-046 obtain fresh substantive independent semantic/security/governance review only after T006-045 succeeds on that exact unchanged head;
4. T006-047 reconcile every material finding forward-only; any mutation returns to T006-045/T006-046;
5. T006-048 mark implementation PR Ready only on the unchanged clean qualified head;
6. T006-049 perform only guarded expected-head merge after re-fetching exact base/head;
7. T006-050 require push-triggered canonical-main CI success on the exact returned merge SHA;
8. T006-051 record `SPEC_006_IMPLEMENTATION_COMPLETE=YES` and `SPEC_006_CLOSED_CANONICAL=YES` only after T006-050 succeeds;
9. T006-052 re-read canonical program ordering and determine the next authorized successor. Do not infer successor authority from stale or nonmerged proposals.

Final review must be substantive, independent, exact-head, and obtained after exact-head CI. Status-only, billing/rate-limit/unavailable messages, stale-head output, CI alone, or self-review are insufficient. Codex review remains excluded by founder direction.

Do not force-push, rebase shared history, or destructively rewrite published history. Never claim tests, CI, review, platform behavior, source admission, mergeability, implementation completion, or project completion without exact evidence.

```text
CANONICAL_MAIN=c85b4b8f0d6ffccb039645803542d75b3bd47f29
SPEC_006_PLANNING_COMPLETE=YES
SPEC_006_PLANNING_CLOSED_CANONICAL=YES
SPEC_006_PRODUCT_IMPLEMENTATION_AUTHORIZED=YES
IMPLEMENTATION_BRANCH=impl/006-desktop-computer-control
SPEC_006_IMPLEMENTATION_COMPLETE=NO
SPEC_006_CLOSED_CANONICAL=NO
WAIVER_TAKEN=NO
```
