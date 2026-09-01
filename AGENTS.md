# Golam Agent Instructions

## Current phase

Golam is in **Spec 005 planning: Local Tools, Context & Memory** on branch `spec/005-local-tools-context-memory`.

Canonical `main` is `390ea842837a7d85dca165d9291d5eb54c3f11db` at Spec 004 implementation closeout merge PR #14. Push-triggered canonical-main CI #777 / run `33507343928` completed successfully on Windows, macOS and Ubuntu, including platform-applicable strict-local external observation. Spec 004 is therefore `CLOSED_CANONICAL` for successor ordering even though the historical merge message itself recorded T004-113 as pending at merge time.

Spec 001 T050–T059 authorizes bounded Spec 005 after Spec 004 closes. Spec 005 planning is not canonical until its own exact-head CI, independent semantic review, guarded merge and post-merge main CI complete.

Open PRs #6–#8 are noncanonical planning proposals. They do not become predecessors or authority merely because related material overlaps Spec 005.

## Authority order

1. exact live GitHub/repository truth;
2. `.specify/memory/constitution.md` v1.2.0 or later;
3. frozen Spec 001 program architecture/tasks/contracts/source-permission attestation;
4. canonical Spec 002 closeout package;
5. canonical Spec 003 package and implementation evidence;
6. canonical Spec 004 planning + implementation package and live closeout evidence;
7. exact Spec 005 planning evidence on `spec/005-local-tools-context-memory`;
8. exact Source Foundry records for any source later reaching `ADMITTED`.

Nonmerged proposals, status-only bot messages, stale comments and prior handoffs cannot override live canonical truth.

## Spec 005 planning read order

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

## Hard boundaries

- Preserve the current seven-crate workspace initially. Do not create empty architecture crates.
- Tool/model/protocol output is not authority. Consequential execution stays behind Kernel policy/capability and the Effect Gate.
- Generic filesystem authority never includes protected Golam kernel resources.
- Path strings are not authority. Symlink/reparse/junction/mount aliases and TOCTOU behavior are first-class security constraints.
- Canonical Spec 003 records `PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO`. Shell/process/local executable MCP/skill launch remains unavailable until Spec 005 independently admits an exact production containment profile.
- The inspected Golam-Research `skipApproval: true` shell semantic is explicitly rejected. Never reproduce donor approval bypass.
- L0 files/search/Git context is required. Tree-sitter/LSP is conditional on measured need; graph/dataflow/vector/runtime indexing is not a baseline requirement.
- Managed Markdown is canonical durable memory; SQLite is canonical operational state; derivatives are rebuildable and optional.
- Memory candidates are not durable truth. Promotion requires attributable approval or deterministic pre-registered authoritative verification.
- `SECRET_DERIVED` content cannot enter canonical long-term memory.
- User hand-edited Markdown must be reconciled rather than silently overwritten.
- Agent Skills/MCP/ACP remain untrusted interoperability/configuration boundaries and cannot mint Golam authority.
- Strict-local hard denial dominates tool/protocol/network routing. Local failure never authorizes cloud/remote fallback.
- No planning source is automatically admitted as code/dependency. Exact Source Foundry admission remains required.
- No Desktop/computer control, GolamConnect/channels, workers/scheduler/autonomous learning, broad parity or final release scope in Spec 005.

## Execution discipline

Execute `tasks.md` in dependency order. Do not begin implementation until planning is genuinely `CLOSED_CANONICAL` and the implementation branch is created from the exact verified post-planning main.

Never claim tests/CI/review/containment/platform/security behavior without exact evidence. A branch mutation invalidates CI/review evidence bound to the prior head; unchanged canonical predecessor evidence remains valid unless superseded by live truth.

Do not force-push, rebase shared history or destructively rewrite published history.

## Review discipline

Final planning and implementation review must be substantive, independent, exact-head and obtained after exact-head CI. Status-only, billing/rate-limit/unavailable messages, automated summaries, stale-head output, CI alone or self-review are insufficient.

Codex review remains excluded by founder direction. Use the live repository's available independent review mechanism and require semantic findings/reconciliation, not merely a bot presence signal.

Ready/merge authorization is fail-closed. If the Draft→Ready connector transition is unavailable, a lifecycle relay is permitted only when live canonical precedent applies: identical base/head/tree, zero content delta, its own CI, independent relay-consistency review, then expected-head guarded merge.
