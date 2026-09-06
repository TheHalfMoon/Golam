# Golam Agent Instructions

## Current phase

Golam is in **Spec 005 final implementation convergence: Local Tools, Context & Memory** on branch `impl/005-local-tools-context-memory`.

Canonical `main` remains `4bd23b218304663349fb2f703cedd40c7a3038af` at Spec 005 planning closeout merge PR #19. Push-triggered canonical-main CI #798 / run `33548321187` completed successfully on Windows, macOS and Ubuntu, including platform-applicable strict-local external observation. Spec 005 planning is therefore `CLOSED_CANONICAL` for implementation ordering.

The implementation branch has completed the bounded implementation/convergence work through T005-116. The next non-waivable gate is T005-117: exact-head Windows/macOS/Ubuntu CI on the final closeout documentation head. T005-118 independent semantic/security review may be requested only after that exact-head CI succeeds. Any branch mutation after CI or review invalidates the affected exact-head evidence.

Open PRs #6–#8 and other nonmerged proposals are noncanonical. They do not become predecessors or authority merely because related material overlaps Spec 005.

## Authority order

1. exact live GitHub/repository truth;
2. `.specify/memory/constitution.md` v1.2.0 or later;
3. frozen Spec 001 program architecture/tasks/contracts/source-permission attestation;
4. canonical Spec 002 closeout package;
5. canonical Spec 003 package and implementation evidence;
6. canonical Spec 004 planning + implementation package and live closeout evidence;
7. canonical Spec 005 planning package plus exact implementation evidence on `impl/005-local-tools-context-memory`;
8. exact Source Foundry records for every admitted dependency/runtime primitive.

Nonmerged proposals, status-only bot messages, stale comments, stale CI/reviews/hashes and prior handoffs cannot override live canonical truth.

## Spec 005 implementation read order

1. `.specify/memory/constitution.md`
2. `specs/001-golam-local-agent-os-foundation/spec.md`
3. `specs/001-golam-local-agent-os-foundation/plan.md`
4. `specs/001-golam-local-agent-os-foundation/tasks.md`
5. `specs/001-golam-local-agent-os-foundation/source-permission-attestation.md`
6. canonical Spec 002 closeout evidence
7. canonical Spec 003 package, especially production sandbox/executor qualification evidence
8. canonical Spec 004 package and implementation closeout evidence
9. `specs/005-local-tools-context-memory/spec.md`
10. `specs/005-local-tools-context-memory/clarification-closeout.md`
11. `specs/005-local-tools-context-memory/research.md`
12. `specs/005-local-tools-context-memory/donor-qualification.md`
13. `specs/005-local-tools-context-memory/plan.md`
14. `specs/005-local-tools-context-memory/data-model.md`
15. all `specs/005-local-tools-context-memory/contracts/`
16. `specs/005-local-tools-context-memory/quickstart.md`
17. `specs/005-local-tools-context-memory/checklists/implementation-readiness.md`
18. `specs/005-local-tools-context-memory/tasks.md`
19. `specs/005-local-tools-context-memory/analysis.md`
20. all exact `specs/005-local-tools-context-memory/implementation/` evidence, especially Phase G/H/I/J closeout records
21. live PR #21 exact-head CI/review/lifecycle evidence for gates intentionally recorded outside branch content

## Current admitted/selected implementation posture

- The exact production containment profile `platform:linux-x86_64-landlock-v4-seccomp-v2` is admitted by the Phase G live T005-077 gate. Do not generalize that admission to macOS, Windows, other Linux architectures, namespaces or other containment mechanisms.
- Governed argv-style process execution is implemented through ToolRequest + Kernel/Effect Gate + the admitted Linux x86_64 executor and is requalified by the ordinary CI governed process-v2 E2E gate.
- Shell syntax remains **not selected and disabled**. Never add shell parsing/launch or reproduce donor `skipApproval` behavior under Spec 005.
- L0 files/in-process-search/Git/context are the selected context baseline.
- `L1=DEFER_NOT_NEEDED`; no Tree-sitter/LSP dependency is admitted by Spec 005.
- `DENSE_VECTOR_INDEX=DEFER_NOT_NEEDED`; no Qdrant/vector service/dependency is admitted by Spec 005.
- `HTTP_TRANSPORT=NOT_REQUIRED`; no hidden HTTP client, cloud fallback or generic document-fetch transport is admitted by Spec 005.
- Remote MCP remains behind explicit network/egress/authenticated-endpoint/credential/secret/redirect/proxy policy and strict-local denial; no missing transport may be invented as implicit authority.
- OS window/input/accessibility/screenshot-as-control behavior is outside Spec 005 and remains a later-spec boundary.

## Hard boundaries

- Preserve the current seven-crate workspace unless canonical later governance explicitly authorizes a justified split.
- Tool/model/protocol output is not authority. Consequential execution stays behind Kernel policy/capability and the Effect Gate.
- Generic filesystem authority never includes protected Golam kernel resources.
- Path strings are not authority. Symlink/reparse/junction/mount aliases and TOCTOU behavior are first-class security constraints.
- Production process claims are limited to the exact admitted Linux x86_64 profile. Unsupported platforms fail closed; no cross-platform containment equivalence is inferred.
- The inspected Golam-Research `skipApproval: true` shell semantic is explicitly rejected. Never reproduce donor approval bypass.
- Managed Markdown is canonical durable memory; SQLite is canonical operational state; derivatives are rebuildable and optional.
- Memory candidates are not durable truth. Promotion requires attributable approval or deterministic pre-registered authoritative verification.
- `SECRET_DERIVED` content cannot enter canonical long-term memory.
- User hand-edited Markdown must be reconciled rather than silently overwritten.
- Agent Skills/MCP/ACP remain untrusted interoperability/configuration boundaries and cannot mint Golam authority.
- Skill/MCP lifecycle/version/digest/mapping changes invalidate stale queued/prepared/cached capability, approval and dispatch material.
- Strict-local hard denial dominates tool/protocol/network routing. Local failure never authorizes cloud/remote fallback.
- No planning source is automatically admitted as code/dependency. Exact Source Foundry admission remains required.
- No Desktop/computer control, GolamConnect/channels, workers/scheduler/autonomous learning, broad parity or final release scope is admitted by Spec 005.

## Execution discipline

Execute `tasks.md` in dependency order. The implementation/convergence work through T005-116 is recorded in exact phase evidence and `implementation/phase-j-convergence-closeout.md`.

The current required sequence is fail-closed:

1. T005-117 exact-head Windows/macOS/Ubuntu CI on the unchanged final implementation head;
2. T005-118 fresh substantive independent semantic/security review on that same head after CI;
3. T005-119 reconcile every material finding forward-only, repeating CI and review after any mutation;
4. T005-120 transition PR #21 to Ready only on the unchanged clean qualified head;
5. T005-121 re-fetch live base/head and perform only a guarded expected-head merge;
6. T005-122 require push-triggered canonical-main CI success on the returned merge SHA;
7. T005-123 set implementation closure only after canonical-main CI succeeds;
8. T005-124 re-read canonical main/program ordering before entering any successor scope.

Never claim tests/CI/review/containment/platform/security behavior without exact evidence. A branch mutation invalidates CI/review evidence bound to the prior head; unchanged canonical predecessor evidence remains valid unless superseded by live truth.

Do not force-push, rebase shared history or destructively rewrite published history.

## Review discipline

Final implementation review must be substantive, independent, exact-head and obtained after T005-117 exact-head CI. Status-only, billing/rate-limit/unavailable messages, automated summaries without semantic inspection, stale-head output, CI alone or self-review are insufficient.

Codex review remains excluded by founder direction. Use the live repository's available independent review mechanism and require semantic/security findings/reconciliation, not merely a bot presence signal.

Ready/merge authorization is fail-closed. The connector currently supports the Draft→Ready lifecycle transition directly; do not use a relay unless the live transition actually fails and canonical same-SHA relay precedent applies. Merge only after re-fetching the exact qualified head/base and preserving expected-head protection.

```text
SPEC_005_PLANNING_CLOSED_CANONICAL=YES
T005_110=PASS
T005_111=PASS
T005_112=PASS
T005_113=PASS
T005_114=PASS
T005_115=PASS_PRE_CLOSEOUT_HEAD
T005_116=PASS_PRE_CLOSEOUT_HEAD
T005_117=PENDING_FINAL_EXACT_HEAD_CI
T005_118=PENDING_FRESH_EXACT_HEAD_REVIEW
PR_READY=NO
MERGE_AUTHORIZED=NO
SPEC_005_IMPLEMENTATION_COMPLETE=NO
SPEC_005_CLOSED_CANONICAL=NO
WAIVER_TAKEN=NO
```
