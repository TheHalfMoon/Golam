# Golam Agent Instructions

## Current phase

Golam is in **Spec 003 implementation: Identity, Policy, Secrets & Sandbox** on branch `impl/003-identity-policy-secrets-sandbox`.

Spec 002 — Kernel & Durable Session Spine — is `CLOSED_CANONICAL`.

Spec 003 planning is `CLOSED_CANONICAL` at `main@82de7084384009ff3a00522f4e0aef09bf549529`, tree `046966262379ba0e7038e7a1216c6237c2033a94`, after qualified PR #4 merge and post-merge CI #255.

Product implementation is authorized only to the extent defined by the merged Spec 003 package and its task/dependency ordering.

## Authority order

1. exact live GitHub truth;
2. `.specify/memory/constitution.md` (v1.2.0 or later);
3. frozen Spec 001 program architecture and contracts;
4. canonical Spec 002 implementation and closeout evidence;
5. canonical merged Spec 003 artifacts and implementation evidence;
6. exact admitted dependency/source records.

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
17. all `specs/003-identity-policy-secrets-sandbox/implementation/` evidence.

## Spec 003 hard boundaries

- Preserve the existing seven-package Spec 002 spine. Do not create empty domain crates; split only if implementation evidence proves an independent ownership/testing boundary is necessary.
- Keep `Authorize(principal, action, resource, context)` semantically stable and process-splittable. Spec 003 replaces the normal bootstrap policy evaluator; it does not replace KernelApi.
- Authorization ordering is hard guards -> authenticated principal -> capability lease -> policy -> required approval -> typed protected mutation/effect. Earlier denial is monotonic.
- Cedar is an evaluator dependency only. Golam owns authority schemas, normalization, hard denials, capabilities, protected resources, approval semantics and final allow/deny behavior.
- Cedar evaluation diagnostics/errors are Golam `DENY`, even though Cedar itself uses skip-on-error semantics.
- Do not enable Cedar experimental/tolerant/WASM/protobuf features without a new qualification record.
- Capability authority is kernel-minted, principal-bound, non-self-expanding, use-time expiry/revocation checked, and child leases may only narrow.
- Protected policy/principal/lease/approval/secret/taint-verifier/egress/sandbox/effect/audit state is not generic filesystem state. Protected mutation is typed elevated work under current authority.
- Real secrets stay out of model context and untrusted execution wherever brokerable. Tests use deterministic canaries only.
- Vault ciphertext is authenticated encryption; master-key protection uses explicitly selected OS stores and fails closed when unavailable. No plaintext/env/argv fallback.
- `SECRET_DERIVED` content is not eligible for canonical long-term memory. Model/worker/skill/MCP assertions cannot self-clear taint.
- Strict-local egress remains a hard deny before Cedar/lease/approval/permit evaluation. No policy or permit can override it.
- Effective destination changes across DNS resolution, redirect, rebinding, protocol/port, private/link-local/loopback transitions require reauthorization before connect/follow.
- Before any network-capable Golam-managed native child is launched, external strict-local qualification must observe the complete managed process tree or an equivalent descendant-capturing sinkholed boundary.
- Authorization and sandboxing are separate. A sandbox profile is not containment proof. Unsupported required controls deny pre-launch.
- Wasmtime/WASI is currently `NOT_ADMITTED_NOT_NEEDED`; reopen qualification only if T003-075 genuinely requires it. It is never a universal native sandbox.
- `Golam-Research` remains `REFERENCE_ONLY` for Spec 003 unless a later Source Foundry record admits exact bounded files. No donor code is admitted by Phase A.
- No model/harness, broad tool suite, Desktop/computer control, GolamConnect, channels, workers, or scheduler implementation in Spec 003.
- Never claim CI/tests/review PASS without exact-head evidence. Any implementation branch mutation invalidates earlier exact-head closeout evidence.
- Codex review is excluded from the Golam workflow by founder direction. Qodo is the authorized external review source for this sequence; CodeRabbit is not a substitute.
- Do not mark the implementation PR Ready, merge it, claim `SPEC_003_CLOSED_CANONICAL`, or start Spec 004 until the ordered Spec 003 tasks and final exact-head/post-merge gates are genuinely satisfied.
