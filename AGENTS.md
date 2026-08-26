# Golam Agent Instructions

## Current phase

Golam is in **Spec 002 implementation qualification and closeout: Kernel & Durable Session Spine** on branch `impl/002-kernel-durable-session-spine`, PR #3.

The Spec 002 planning package was merged to canonical `main` at `cfcc90f452e7115bfb104f886e09c309a5d57a1c`. Rust implementation is therefore authorized only to the extent defined by the merged Spec 002 package and its task order.

PR #3 remains Draft until separately authorized to become Ready. Do not merge PR #3 without separate explicit founder authorization. Do not start Spec 003 until Spec 002 is merged and closed canonical.

## Authority order

1. exact live GitHub truth;
2. `.specify/memory/constitution.md` (v1.2.0 or later);
3. frozen Spec 001 program architecture;
4. active Spec 002 artifacts, including implementation evidence and task state;
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
16. all `specs/002-kernel-durable-session-spine/implementation/` evidence.

## Spec 002 hard boundaries

- Rust is mandatory for every product component implemented by Spec 002.
- Keep exactly the bounded seven-package implementation spine unless a reviewed Spec 002 requirement proves another package necessary. Do not scaffold empty future crates.
- No model inference, model download, cloud provider, browser, desktop control, GolamConnect, skills, MCP, workers, or real user secrets in Spec 002.
- No real-world consequential effects are required; effect semantics are proven with deterministic simulators/fakes.
- No unauthenticated localhost HTTP/TCP control surface. Local client traffic uses authenticated OS-local IPC only.
- No generic tool or client can write kernel-owned policy/authority/audit/ledger state.
- Do not expose a generic caller-selected canonical `EventKind` append surface that could forge reserved system event families; invariant-coupled canonical events are emitted through their owning typed KernelApi operations.
- No blind effect retry after ambiguous outcomes.
- No network egress in strict-local Spec 002 product code. Test harnesses may observe the Golam-managed process from outside its boundary.
- Security-critical client enrollment/revocation, authorization, effect, recovery, and canonical event evidence must retain mandatory tamper-evident integrity coverage.
- No donor source is copied merely because permission exists. Record exact source state, permission scope/evidence, selected files, dependency/license obligations, and technical/security qualification first.
- `Golam-Research` is high-value implementation evidence and an authorized-source candidate. Mine it seriously, but preserve the distinction between reconstructed source and original upstream authorship.
- Never claim CI/tests passed unless exact-head evidence exists.
- A documentation/task closeout commit does not inherit PASS merely because an earlier code head was green; run the required exact-head gate again.
