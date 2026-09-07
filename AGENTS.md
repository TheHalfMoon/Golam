# Golam Agent Instructions

## Live-state rule

Never treat this file, a prior chat, a handoff, a cached SHA, a bot summary or an open proposal as live qualification authority.

Read `specs/CURRENT.md` for **durable canonical lifecycle state**, then re-fetch exact GitHub state for every mutable fact required by the action you are about to take.

Before any CI qualification, independent review request, Ready transition, merge, post-merge closeout or successor-authority decision, re-fetch at minimum:

```text
CURRENT_MAIN
CURRENT_PR_HEAD_AND_BASE
CURRENT_DIFF
CURRENT_CHECKS
CURRENT_REVIEWS_AND_THREADS
EXPECTED_HEAD
```

A branch mutation invalidates CI/review evidence bound to the prior head.

## Current canonical program posture

At the canonical base represented by this proposal:

```text
CANONICAL_MAIN=c85b4b8f0d6ffccb039645803542d75b3bd47f29
SPEC_005_IMPLEMENTATION_COMPLETE=YES
SPEC_005_CLOSED_CANONICAL=YES
SPEC_006_PLANNING_CLOSED_CANONICAL=YES
SPEC_006_PRODUCT_IMPLEMENTATION_AUTHORIZED=YES
ACTIVE_CANONICAL_IMPLEMENTATION_UNIT=SPEC_006_DESKTOP_COMPUTER_CONTROL
ACTIVE_IMPLEMENTATION_PR=24
ACTIVE_IMPLEMENTATION_PR_IS_CANONICAL=NO_UNTIL_MERGED_AND_POST_MERGE_QUALIFIED
```

PR #24 must be re-fetched live before using its head/check/review/task state. Do not hardcode a remembered head here.

The program-superiority material under `specs/001-golam-local-agent-os-foundation/program-superiority-2026.md` and `program-superiority-tasks.md` is planning/governance proposal material only until merged. It does **not** widen PR #24 or authorize future product units.

## Authority order

1. exact live GitHub/repository truth for mutable state;
2. `.specify/memory/constitution.md` v1.2.0 or later;
3. canonical Spec 001 architecture/contracts/tasks/source-permission governance;
4. `specs/CURRENT.md` for durable canonical lifecycle state;
5. canonical closed predecessor specs and closeout evidence;
6. the currently authorized bounded spec package;
7. exact Source Foundry records for every admitted source/dependency/runtime primitive;
8. exact-head CI/review/evidence for the lifecycle action being attempted.

Open PRs, issues, experimental branches, source candidates, benchmark results and bot-generated summaries are noncanonical unless canonical governance explicitly promotes them.

## Spec 006 implementation read order

1. `.specify/memory/constitution.md`
2. `specs/CURRENT.md`
3. `specs/001-golam-local-agent-os-foundation/spec.md`
4. `specs/001-golam-local-agent-os-foundation/plan.md`
5. `specs/001-golam-local-agent-os-foundation/tasks.md`, especially T060–T069
6. canonical Spec 002–005 closeout evidence as required by the current task
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
18. live exact-head PR #24 CI/review/lifecycle evidence

## Program-superiority proposal read order

When reviewing the roadmap proposal rather than implementing Spec 006:

1. Constitution;
2. canonical Spec 001 spec/plan/tasks/contracts;
3. `specs/001-golam-local-agent-os-foundation/program-superiority-2026.md`;
4. `specs/001-golam-local-agent-os-foundation/program-superiority-tasks.md`;
5. `specs/001-golam-local-agent-os-foundation/competitive-source-register-2026.md`;
6. `specs/CURRENT.md`;
7. exact live PR for the proposal and exact review/CI state.

Do not implement future superiority tasks directly from the overlay. Each product task still requires its own bounded Spec Kit lifecycle and predecessor/successor authorization.

## Invariants every feature must preserve

- Consequential execution remains behind canonical ToolRequest + capability/policy/approval + Effect PREPARED + Kernel/Effect Gate + immediate revalidation + durable terminal/reconciliation semantics.
- `UNKNOWN_OUTCOME` blocks conflicting retry and dependent work until reconciliation.
- Strict-local hard denial dominates model/tool/protocol/network routing. Local failure never creates cloud fallback authority.
- Generic tools, model output, protocol output, workers, skills, MCP servers, plugins, renderers, vision output and benchmark fixtures cannot mint kernel authority.
- Protected kernel state remains outside generic filesystem/process/browser/computer-control capability.
- Memory remains governed user-owned evidence; live authoritative source state outranks memory.
- Skills/routines are versioned instruction/code artifacts, never authority; learned candidates do not activate themselves.
- Every new dependency, package, native helper, binary, copied source or donor implementation requires exact Source Foundry admission before product use.
- Do not force-push, rebase shared history or destructively rewrite published history.

## Computer-control hard boundaries

Preserve the constitutional route order exactly:

```text
domain/application API
-> native OS automation API
-> accessibility/semantic tree
-> browser DOM/protocol
-> deterministic keyboard/mouse control
-> vision/pixel fallback
```

A weaker route requires trusted fallback-eligibility evidence. Stronger applicable routes and unreconciled `UNKNOWN_OUTCOME` block weaker escalation.

Desktop adapters, model output, renderer state and pixel/vision components cannot self-mint fallback eligibility or actuation authority.

Observation, focus, capture, semantic action, raw input and clipboard read/write are distinct authority/evidence surfaces.

The Tauri native Rust host authenticates through existing local `golamd` IPC/client enrollment. Localhost/same-machine location and renderer state are not authentication. The renderer receives no credential, raw native handle, capability token, approval material or protected lease authority.

Autonomous computer actuation requires a qualified visible local control channel with immediate pause/stop/takeover. Loss of every qualified visible channel suspends new actuation fail closed. Human takeover is enforced at protected lease/input authority so stale renderer/model requests cannot restore a superseded generation.

Windows secure desktop/UAC bypass, Wayland/compositor bypass, background keylogging, silent clipboard inspection, unbounded capture, camera/microphone collection and hidden remote/cloud fallback remain denied.

Raw screenshot OCR/text extraction is **not** owned by Spec 007 GolamConnect. It remains outside active Spec 006 and belongs to a future bounded Visual Semantics unit only after canonical successor authorization.

## Cross-spec route ownership

The program-superiority proposal identifies a browser/application route-ownership gap. Until a canonical successor unit defines the global route-provider contract, active Spec 006 must not infer that an unknown browser/application route is unavailable merely because its provider lives outside the current crate/spec.

No future route provider may mint its own authority. Applicability/availability evidence and action authority remain separate.

## Review and merge discipline

Planning and implementation review must be substantive, independent, exact-head and obtained after exact-head CI when the owning lifecycle requires it. Status-only, rate-limit/billing/unavailable output, automated summary without semantic inspection, stale-head review, CI alone or self-review are insufficient.

Ready/merge authorization is fail closed. Mark Ready only on an unchanged clean qualified head. Re-fetch exact base/head immediately before merge and use expected-head protection. Post-merge canonical-main CI belongs to the exact returned merge SHA.

Never claim tests, review, runtime/platform behavior, source admission, benchmark status, security behavior, mergeability, readiness or completion without exact evidence.
