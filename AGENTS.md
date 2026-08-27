# Golam Agent Instructions

## Current phase

Golam is in **Spec 003 planning: Identity, Policy, Secrets & Sandbox**.

Spec 002 — Kernel & Durable Session Spine — is `CLOSED_CANONICAL` on `main@a04756f242e48faeda802e5b3fd99a0c8d52f53e` after qualified PR #3 merge and post-merge CI #252.

The active Spec 003 planning PR is documentation/governance only. **Do not write Spec 003 Rust product implementation, add product dependencies, or mutate the runtime architecture until the Spec 003 planning package is reviewed and merged to canonical `main`.**

## Authority order

1. exact live GitHub truth;
2. `.specify/memory/constitution.md` (v1.2.0 or later);
3. frozen Spec 001 program architecture and contracts;
4. canonical Spec 002 implementation and closeout evidence;
5. active Spec 003 planning artifacts;
6. exact admitted donor/source records.

## Spec 003 read order

1. `.specify/memory/constitution.md`
2. `specs/001-golam-local-agent-os-foundation/spec.md`
3. `specs/001-golam-local-agent-os-foundation/plan.md`
4. `specs/001-golam-local-agent-os-foundation/source-permission-attestation.md`
5. canonical Spec 002 package and `implementation/` closeout evidence
6. `specs/003-identity-policy-secrets-sandbox/spec.md`
7. `specs/003-identity-policy-secrets-sandbox/clarification-closeout.md`
8. `specs/003-identity-policy-secrets-sandbox/research.md`
9. `specs/003-identity-policy-secrets-sandbox/donor-qualification.md`
10. `specs/003-identity-policy-secrets-sandbox/plan.md`
11. `specs/003-identity-policy-secrets-sandbox/data-model.md`
12. all `specs/003-identity-policy-secrets-sandbox/contracts/`
13. `specs/003-identity-policy-secrets-sandbox/quickstart.md`
14. `specs/003-identity-policy-secrets-sandbox/checklists/implementation-readiness.md`
15. `specs/003-identity-policy-secrets-sandbox/tasks.md`
16. `specs/003-identity-policy-secrets-sandbox/analysis.md`

## Spec 003 hard boundaries

- Planning PR only: no Rust product code, product dependency admission, schema migration, workflow weakening, model/provider integration, or real external effect.
- Preserve the existing seven-package Spec 002 spine. Do not create empty future crates; split only when implementation evidence proves a new ownership/testing boundary is necessary.
- Keep `Authorize(principal, action, resource, context)` semantically stable. Spec 003 replaces the bootstrap evaluator; it does not replace the KernelApi architecture.
- Cedar is a **candidate policy evaluator**, not the owner of Golam authority semantics. Exact crate/version admission requires implementation-time dependency qualification.
- Hard Golam safety denials and strict-local denial are monotonic and dominate Cedar, leases, and approvals.
- Capability/lease authority is kernel-minted, non-self-expanding, expiry/revocation checked at the protected action boundary, and child leases may only narrow.
- Protected policy/principal/lease/approval/secret/egress/sandbox/effect/audit state is not generic filesystem state. Protected mutation is a typed elevated effect.
- Real secrets stay out of model context and untrusted execution where brokerable. Planning and qualification use deterministic canaries, never production credentials.
- `SECRET_DERIVED` content is not eligible for canonical long-term memory. Model/worker/skill/MCP assertions cannot self-clear taint.
- Strict-local egress remains a hard deny before any policy permit. No silent cloud/network fallback.
- Authorization and sandboxing remain separate. Wasmtime/WASI is only a candidate for portable bounded extensions and is not a universal native sandbox.
- `Golam-Research` is `REFERENCE_ONLY` for Spec 003 unless a later Source Foundry record admits exact bounded files. No donor code is admitted by this planning package.
- No model/harness, broad filesystem/shell/browser tool suite, Desktop/computer control, GolamConnect, channels, workers, or scheduler product implementation in Spec 003.
- Never claim CI/tests/review PASS without exact-head evidence. Any branch mutation invalidates prior exact-head qualification evidence.
- Codex review is excluded from the Golam workflow by founder direction. Qodo is the authorized external review source for this sequence; CodeRabbit is not a substitute.
- Spec 003 implementation begins only after this planning package is reviewed, merged, and canonical `main` is reread.
