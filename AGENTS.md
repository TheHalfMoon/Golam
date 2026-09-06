# Golam Agent Instructions

## Current phase

Golam is in **Spec 006 planning: Desktop Computer Control** on branch `spec/006-desktop-computer-control`, PR #23.

Canonical predecessor `main` for this planning unit is `9400d4614318fffb2623ea71522ecd5f0f95f96a`. Live post-merge evidence for Spec 005 records `T005_122=PASS_POST_MERGE_MAIN_CI`, `SPEC_005_IMPLEMENTATION_COMPLETE=YES`, `SPEC_005_CLOSED_CANONICAL=YES`, and successor authorization `T060_CREATE_BOUNDED_SPEC_006`.

Spec 006 product implementation is **not authorized** until the complete planning lifecycle closes canonical: final planning artifacts → exact-head CI → fresh substantive independent review on that unchanged head → finding reconciliation → Ready → guarded expected-head merge → push-triggered CI on the returned merge SHA → canonical planning closeout.

Never hardcode a remembered PR #23 head here as qualification authority. Re-fetch the live PR head/base/checks/reviews before every qualification or lifecycle action.

Open/nonmerged proposals remain noncanonical unless live canonical governance explicitly promotes them. They do not become predecessors or authority merely because related material overlaps the current scope.

## Authority order

1. exact live GitHub/repository truth;
2. `.specify/memory/constitution.md` v1.2.0 or later;
3. frozen Spec 001 program architecture/tasks/contracts/source-permission attestation;
4. canonical Spec 002 closeout package;
5. canonical Spec 003 package and implementation evidence;
6. canonical Spec 004 planning + implementation package and live closeout evidence;
7. canonical Spec 005 planning + implementation closeout evidence on `main@9400d4614318fffb2623ea71522ecd5f0f95f96a`;
8. the current bounded Spec 006 package under `specs/006-desktop-computer-control/` plus exact live PR #23 lifecycle evidence;
9. exact Source Foundry records for every admitted dependency/runtime primitive.

Nonmerged proposals, status-only bot messages, stale comments, stale CI/reviews/hashes, old branch text and prior handoffs cannot override live canonical truth.

## Spec 006 read order

1. `.specify/memory/constitution.md`
2. `specs/001-golam-local-agent-os-foundation/spec.md`
3. `specs/001-golam-local-agent-os-foundation/plan.md`
4. `specs/001-golam-local-agent-os-foundation/tasks.md`, especially T060–T069
5. `specs/001-golam-local-agent-os-foundation/source-permission-attestation.md`
6. canonical Spec 002–005 closeout evidence as predecessor contracts/behavior require
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
18. live PR #23 exact-head CI/review/lifecycle evidence for gates intentionally recorded outside branch content

## Predecessor implementation posture that Spec 006 must preserve

- Consequential execution remains behind canonical ToolRequest + capability/policy/approval + Effect PREPARED + Kernel/Effect Gate + immediate revalidation + durable terminal/reconciliation semantics.
- `UNKNOWN_OUTCOME` blocks conflicting retry until reconciliation; a later feature must not weaken this predecessor invariant.
- Strict-local hard denial dominates tool/protocol/network routing. Local failure never authorizes cloud/remote fallback.
- Generic tools, model output, protocol output and renderer state cannot mint kernel authority.
- Protected Golam kernel resources remain outside generic filesystem/process authority.
- The exact Spec 005 production containment admissions remain bounded to their qualified platform/profile; Spec 006 does not generalize them.
- Shell syntax remains disabled unless separately re-authorized by later canonical scope.
- Managed Markdown/SQLite memory governance, secret/taint boundaries and untrusted Skill/MCP/ACP semantics remain predecessor constraints.

## Spec 006 hard boundaries

- Preserve the constitutional control-route order exactly: `domain/application API → native OS automation API → accessibility/semantic tree → browser DOM/protocol → deterministic keyboard/mouse control → vision/pixel fallback`.
- A weaker route requires canonical trusted fallback-eligibility evidence; stronger applicable authorized routes and unreconciled `UNKNOWN_OUTCOME` block weaker escalation.
- Desktop adapters, model output, renderer state and pixel/vision components cannot self-mint fallback eligibility or actuation authority.
- Semantic desktop control is preferred inside Spec 006; deterministic raw input is a distinct explicit fallback.
- Bounded vision/pixel fallback is untrusted candidate geometry only. It cannot mint semantic identity, fallback eligibility, capability, approval or action authority. Raw screenshot OCR/text extraction remains deferred to Spec 007.
- Observation, capture, semantic actuation, raw input, focus and clipboard read/write remain distinct authority/evidence surfaces.
- All side-effect/privacy-sensitive paths preserve exact request/effect/intent/authority bindings, immediate target/session/permission/control-lease/visible-channel revalidation and `UNKNOWN_OUTCOME` reconciliation.
- The Tauri native Rust host must authenticate to `golamd` through the existing authenticated local IPC/client-enrollment boundary. Localhost/same-machine location and renderer state are not authentication.
- Tauri renderer/webview is untrusted and receives no raw native handles, local-client credentials, capability tokens or protected control-lease authority.
- Autonomous computer control must remain visibly indicated to the local user through at least one qualified persistent visible-control channel with immediate pause/stop/takeover. Loss of every qualified visible channel suspends new autonomous actuation fail closed.
- Human pause/stop/takeover is enforced at protected lease/input-authority state; stale renderer/model requests cannot restore a superseded generation.
- Windows locked/non-interactive desktop, UAC/secure desktop and unsupported interactive-session transitions fail closed. Secure desktop/UAC bypass is not supported.
- macOS TCC/Accessibility/Screen Recording and Linux portal/compositor/session state are mutable external prerequisites, never blanket Golam authority.
- No Wayland/compositor bypass, background keylogging, silent clipboard inspection, unbounded capture, camera/microphone collection, screenshot OCR under Spec 006, or hidden remote/cloud/network fallback.
- Golam-research desktop/Electron/VNC/preload behavior is reference evidence only unless a later bounded component is separately Source Foundry admitted. Donor architecture/runtime/trust semantics are not Golam authority.
- Official API research is not dependency admission. Every new crate/package/native binding/library/helper/copied source requires exact Source Foundry admission before manifest/code use.

## Planning execution discipline

Execute `specs/006-desktop-computer-control/tasks.md` in dependency order. Planning-only work may mutate planning/governance artifacts; product Rust/Tauri implementation must not begin until canonical planning closeout.

The planning closeout sequence is fail-closed:

1. T006-P008 complete the full planning package and cross-artifact analysis;
2. T006-P009 run CI on the exact unchanged complete planning head;
3. T006-P010 obtain fresh substantive independent semantic/security/governance review only after that exact-head CI succeeds;
4. T006-P011 reconcile every material finding forward-only; any mutation invalidates affected CI/review and returns to P009;
5. T006-P012 transition PR #23 to Ready only on the unchanged clean qualified head;
6. T006-P013 re-fetch exact base/head and perform only a guarded expected-head merge;
7. T006-P014 require push-triggered canonical-main CI success on the exact returned merge SHA;
8. T006-P015 set planning closure only after P014 succeeds, then and only then create `impl/006-desktop-computer-control` from exact canonical main.

After implementation authority exists, follow the Spec 006 implementation task graph exactly; every dependency introduction is gated by Source Foundry and every final implementation qualification is exact-head.

Never claim tests, CI, review, platform behavior, source admission, security behavior, mergeability or completion without exact evidence. A branch mutation invalidates CI/review evidence bound to the prior head; unchanged canonical predecessor evidence remains valid unless superseded by live truth.

Do not force-push, rebase shared history or destructively rewrite published history.

## Review and merge discipline

Final planning/implementation review must be substantive, independent, exact-head and obtained after exact-head CI. Status-only, billing/rate-limit/unavailable messages, automated summaries without semantic inspection, stale-head output, CI alone or self-review are insufficient.

Codex review remains excluded by founder direction. Use the live repository's available independent review mechanism and require semantic/security/governance findings/reconciliation, not merely a bot presence signal.

Ready/merge authorization is fail-closed. Mark Ready only after the exact head has clean qualifying CI and fresh independent review. Merge only after re-fetching the exact qualified head/base and use expected-head protection. Never use a force/rebase/history rewrite to make a gate pass.

```text
CANONICAL_MAIN=9400d4614318fffb2623ea71522ecd5f0f95f96a
SPEC_005_IMPLEMENTATION_COMPLETE=YES
SPEC_005_CLOSED_CANONICAL=YES
SUCCESSOR_AUTHORITY=T060_CREATE_BOUNDED_SPEC_006
SPEC_006_PLANNING_COMPLETE=NO
SPEC_006_PLANNING_CLOSED_CANONICAL=NO
SPEC_006_PRODUCT_IMPLEMENTATION_AUTHORIZED=NO
PR_READY=NO
MERGE_AUTHORIZED=NO
WAIVER_TAKEN=NO
```
