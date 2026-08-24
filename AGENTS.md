# Golam Agent Instructions

## Current phase

Golam is in **Spec 002 planning: Kernel & Durable Session Spine** on branch `spec/002-kernel-durable-session-spine`.

Do not write Rust product implementation until the Spec 002 planning package has completed specification, clarification, research, plan, contracts, checklist, tasks, and cross-artifact analysis, and the planning gate is explicitly closed.

## Authority order

1. exact live GitHub truth;
2. `.specify/memory/constitution.md` (v1.2.0 or later);
3. frozen Spec 001 program architecture;
4. active Spec 002 artifacts;
5. exact admitted donor/source records.

## Spec 002 read order

1. `.specify/memory/constitution.md`
2. `specs/001-golam-local-agent-os-foundation/spec.md`
3. `specs/001-golam-local-agent-os-foundation/plan.md`
4. `specs/001-golam-local-agent-os-foundation/source-permission-attestation.md`
5. `specs/002-kernel-durable-session-spine/spec.md`
6. `specs/002-kernel-durable-session-spine/clarification-closeout.md`
7. `specs/002-kernel-durable-session-spine/research.md`
8. `specs/002-kernel-durable-session-spine/donor-qualification.md`
9. `specs/002-kernel-durable-session-spine/plan.md`
10. `specs/002-kernel-durable-session-spine/data-model.md`
11. all `specs/002-kernel-durable-session-spine/contracts/`
12. `specs/002-kernel-durable-session-spine/quickstart.md`
13. `specs/002-kernel-durable-session-spine/checklists/implementation-readiness.md`
14. `specs/002-kernel-durable-session-spine/tasks.md`
15. `specs/002-kernel-durable-session-spine/analysis.md`

## Spec 002 hard boundaries

- Rust is mandatory for every component implemented by Spec 002.
- Start with no more than 7 real crates/binaries. Do not scaffold empty future crates.
- No model inference, model download, cloud provider, browser, desktop control, GolamConnect, skills, MCP, workers, or real user secrets in Spec 002.
- No real-world consequential effects are required; effect semantics are proven with deterministic simulators/fakes.
- No unauthenticated localhost HTTP server. Local client traffic uses authenticated OS-local IPC only.
- No generic tool or client can write kernel-owned policy/authority/audit/ledger state.
- No blind effect retry after ambiguous outcomes.
- No network egress in strict-local Spec 002 tests except test harnesses explicitly outside Golam-managed process boundaries.
- No donor source is copied merely because permission exists. Record exact source state, permission scope/evidence, selected files, dependency/license obligations, and technical/security qualification first.
- `Golam-Research` is high-value implementation evidence and an authorized-source candidate. Mine it seriously, but preserve the distinction between reconstructed source and original upstream authorship.
- Never claim CI/tests passed unless exact-head evidence exists.
