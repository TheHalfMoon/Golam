# Spec 003 — Live Implementation Execution Plan

**Status**: IMPLEMENTATION_ACTIVE  
**Canonical base**: `main@82de7084384009ff3a00522f4e0aef09bf549529`  
**Implementation branch**: `impl/003-identity-policy-secrets-sandbox`  
**Purpose**: keep the live implementation sequence, evidence points, and next eligible task inside the repository rather than only in PR metadata or chat handoffs.

## Authority

This file is an implementation-time execution companion to the canonical Spec Kit package:

- `spec.md` defines required behavior and success criteria;
- `plan.md` defines the architecture, sequencing constraints, and exit gates;
- `tasks.md` remains the canonical task ledger;
- contracts and `data-model.md` remain normative for their domains;
- this file records the live execution position and evidence without overriding any of those sources.

If this file diverges from constitution, spec, plan, contracts, data model, tasks, or live canonical GitHub state, those sources win and this file must be repaired.

## Current execution state

### Phase A — Exact-main bootstrap and dependency gates

`COMPLETE`

- T003-001..T003-006 complete.
- Cedar admitted exactly at `4.12.0` under recorded qualification.
- crypto/key-protection dependencies qualified.
- Wasmtime remains `NOT_ADMITTED_NOT_NEEDED`.
- `Golam-Research` remains `REFERENCE_ONLY`; no donor code admitted.

### Phase B — Schema, hard guards and policy lifecycle

`COMPLETE`

- T003-010..T003-017 complete.
- Protected authority schema, authority-security coverage, hard-denial ordering, policy candidate/bundle lifecycle, startup integrity, and authorization evidence are present.

### Phase C — Capability leases

`COMPLETE`

- T003-020..T003-024 complete.
- Sealed lease authority, subset-only derivation, use-time expiry/revocation/generation/principal checks, protected issuance/revocation, and adversarial qualification are present.

### Phase D — Approvals

`COMPLETE_AT_QUALIFIED_HEADS`

- T003-030..T003-033 complete on their recorded exact qualification heads.
- T003-034 qualified at `0bcaffb231070082be411e2e37959004ce359ad6` with CI #349 / run `33149069868` SUCCESS on Windows/macOS/Ubuntu.
- T003-035 qualified at `ffc8a66c881b1a34dafe32f79beebe03cceba939` with CI #354 / run `33149715031` SUCCESS on Windows/macOS/Ubuntu.

### Phase E — Taint and verifier state

`ACTIVE — FINAL QUALIFICATION TASK`

Completed:

- T003-040 qualified at `cb69d638107ca4fe0118c9a61f143ac3ba65a2d3`, CI #359 / run `33150969442`.
- T003-041 qualified at `76e1addf35c92a22d2c5826ca429278cacd598b3`, CI #366 / run `33151556481`.
- T003-042 qualified at `67f74c9b9b75e43b9fa00069050c97c041567184`, CI #373 / run `33152187952`.
- T003-043 qualified at `2f8655b5bdddd17bb9e6eab7bf00f11a210896cb`, CI #388 / run `33154505847`.
- T003-044 qualified at `1a9fcddff4c4dd6a6161547cf89a502750f9bc71`, CI #393 / run `33155122088`.
- T003-045 qualified at `e3b91dcecf0048b183c4c333cd9afda43ee25671`, CI #398 / run `33155929307`. Evidence: `implementation/secret-elimination-sanitizer-qualification.md`.

T003-045 adds the deterministic registered secret-elimination sanitizer mechanism with a distinct protected action, exact rule-kind/source-binding/downgrade-scope verification, immutable source provenance and a separately evidenced result artifact. Only the result can pass the T003-044 memory sink when `SECRET_DERIVED` has actually been eliminated.

Current canonical task: **T003-046** — multi-hop/self-clear/unregistered-verifier/`SECRET_DERIVED` property and adversarial qualification.

Bounded T003-046 implementation intent:

- add test-only qualification rather than new product authority;
- exhaustively prove the canonical-memory sink rejects every baseline taint combination containing `SECRET_DERIVED` and accepts this one sink invariant only when that label is absent;
- prove multi-hop `Provenanced::derive` preserves the union of source and transform labels, including `SECRET_DERIVED` dominance across several derivation hops;
- prove human and normal deterministic-verifier preparation cannot self-clear `SECRET_DERIVED` regardless of asserted evidence;
- prove an unregistered sanitizer/verifier rule cannot commit a downgrade even when a caller supplies an otherwise correctly shaped protected effect and allow decision;
- prove sanitizer output remains a distinct artifact and the original source remains memory-inadmissible after successful sanitization;
- avoid widening production APIs unless a test reveals a real invariant gap;
- after exact-head qualification, mark Phase E complete and begin T003-050, not a future Spec 004/005 task.

### Phase F — Secret vault and broker

`BLOCKED_ON_T003-046`

T003-050..T003-057 remain ordered exactly as `tasks.md`.

### Phase G — Egress permits

`NOT_STARTED`

T003-060..T003-064 remain ordered exactly as `tasks.md`. Strict-local remains an unconditional hard denial and cannot be weakened by policy, leases, approvals, or permits.

### Phase H — Sandbox profiles/admission

`NOT_STARTED`

T003-070..T003-076 remain ordered exactly as `tasks.md`. No native Golam-managed child with network capability may be launched before descendant-capturing external no-egress qualification exists.

### Phase I — Kernel/CLI integration and adversarial qualification

`NOT_STARTED`

T003-080..T003-084 remain ordered exactly as `tasks.md`.

### Phase J — Exact-head closeout

`NOT_STARTED`

T003-090..T003-098 remain mandatory. Historical task greens are evidence only; final closure requires fresh exact-head gates, Spec Kit convergence, authorized Qodo review, merge, and post-merge canonical-main evidence.

## Current invariant set

```text
SPEC_002_CLOSED_CANONICAL=YES
SPEC_003_PLANNING_CLOSED_CANONICAL=YES
SPEC_003_IMPLEMENTATION_AUTHORIZED=YES
PHASE_A_COMPLETE=YES
PHASE_B_COMPLETE=YES
PHASE_C_COMPLETE=YES
PHASE_D_COMPLETE=YES
PHASE_E_ACTIVE=YES
T003_040=PASS
T003_041=PASS
T003_042=PASS
T003_043=PASS
T003_044=PASS
T003_044_QUALIFIED_HEAD=1a9fcddff4c4dd6a6161547cf89a502750f9bc71
T003_044_CI_RUN=33155122088
T003_045=PASS
T003_045_QUALIFIED_HEAD=e3b91dcecf0048b183c4c333cd9afda43ee25671
T003_045_CI_RUN=33155929307
T003_046=ACTIVE
NEXT_TASK=T003-046
SPEC_003_IMPLEMENTATION_COMPLETE=NO
SPEC_003_CLOSED_CANONICAL=NO
PR_READY=NO
```

## Mutation discipline

For every task:

1. re-read relevant normative sources;
2. implement only the bounded eligible task;
3. add focused deterministic/adversarial/property evidence;
4. run fresh CI on the exact head;
5. never mark PASS before exact evidence;
6. record qualification inside the repository;
7. immediately begin the next eligible task unless a real governance blocker exists.

No force-push. No rebase. No destructive history rewrite. No merge or CLOSED_CANONICAL claim without required evidence.