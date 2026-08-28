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

The original `plan.md` metadata records the planning branch/base at the time the plan was frozen. That historical header does not mean implementation is still a planning PR; the live execution position is recorded here and in `tasks.md` while the architecture and exit gates in `plan.md` remain canonical.

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
- T003-035 includes production typed approval revocation as a monotonic protected mutation plus expiry/revocation/replay/double-use/scope-overreach/taint-mismatch coverage.

### Phase E — Taint and verifier state

`ACTIVE`

Completed:

- T003-040 qualified at `cb69d638107ca4fe0118c9a61f143ac3ba65a2d3` with CI #359 / run `33150969442` SUCCESS on Windows/macOS/Ubuntu. Evidence: `implementation/taint-baseline-qualification.md`.
- T003-041 qualified at `76e1addf35c92a22d2c5826ca429278cacd598b3` with CI #366 / run `33151556481` SUCCESS on Windows/macOS/Ubuntu. Evidence: `implementation/taint-propagation-qualification.md`.
- T003-042 qualified at `67f74c9b9b75e43b9fa00069050c97c041567184` with CI #373 / run `33152187952` SUCCESS on Windows/macOS/Ubuntu. Evidence: `implementation/verifier-registry-qualification.md`.
- T003-043 qualified at `2f8655b5bdddd17bb9e6eab7bf00f11a210896cb` with CI #388 / run `33154505847` SUCCESS on Windows/macOS/Ubuntu. Evidence: `implementation/taint-downgrade-attestation-qualification.md`.
- T003-044 qualified at `1a9fcddff4c4dd6a6161547cf89a502750f9bc71` with CI #393 / run `33155122088` SUCCESS on Windows/macOS/Ubuntu. Evidence: `implementation/secret-derived-memory-admission-qualification.md`.

T003-044 freezes a small side-effect-free trusted-path guard for future canonical long-term-memory admission. Any `TaintSet` containing `SECRET_DERIVED` denies, including after multi-source monotonic derivation or alongside trusted labels. The task does not implement the Spec 005 memory product.

Current canonical task: **T003-045** — deterministic secret-elimination sanitizer evidence path producing a separately evidenced non-secret-derived artifact.

Bounded T003-045 implementation intent:

- reuse the protected verifier/sanitizer registry already qualified by T003-042 and the `taint_attestations` schema already qualified by T003-043; no schema or dependency expansion is expected;
- add the data-model-reserved `secret_elimination_sanitizer` attestation mechanism and a typed sanitizer evidence boundary;
- require source provenance to contain `SECRET_DERIVED` and require the result to be a distinct derived artifact whose result labels no longer contain `SECRET_DERIVED`;
- preserve every source artifact ID and source label as immutable evidence; the sanitizer never edits source provenance in place;
- bind execution to a current exact protected authorization decision and exact authorized at-most-once taint-authority effect;
- require an active registered `secret_elimination_sanitizer` rule, exact authority-source binding and a canonical allowed-downgrade set covering every removed label;
- a normal deterministic verifier cannot act as a secret-elimination sanitizer and a sanitizer cannot silently remove unrelated labels beyond its registered downgrade scope;
- the separately evidenced sanitizer result should pass the T003-044 canonical-memory sink guard only because its own result provenance no longer contains `SECRET_DERIVED`, never because the source was mutated or the sink guard was bypassed;
- keep T003-046 adversarial multi-hop/self-clear qualification separate.

Remaining Phase E order:

1. T003-045 deterministic secret-elimination sanitizer evidence;
2. T003-046 multi-hop/self-clear/unregistered-verifier/SECRET_DERIVED adversarial qualification.

Do not begin Phase F until Phase E predecessor tasks are complete and exact-head qualified.

### Phase F — Secret vault and broker

`NOT_STARTED`

T003-050..T003-057 remain ordered exactly as `tasks.md`.

### Phase G — Egress permits

`NOT_STARTED`

T003-060..T003-064 remain ordered exactly as `tasks.md`. Strict-local remains an unconditional hard denial and cannot be weakened by policy, leases, approvals, or permits.

### Phase H — Sandbox profiles/admission

`NOT_STARTED`

T003-070..T003-076 remain ordered exactly as `tasks.md`. No native Golam-managed child with network capability may be launched before descendant-capturing external no-egress qualification exists.

### Phase I — Kernel/CLI integration and adversarial qualification

`NOT_STARTED`

T003-080..T003-084 remain ordered exactly as `tasks.md`. Normal KernelApi integration work is owned here; earlier bounded ledger components must not silently widen this phase.

### Phase J — Exact-head closeout

`NOT_STARTED`

T003-090..T003-098 remain mandatory. Historical per-task green runs are regression/qualification evidence only. Final closure requires fresh exact-head CI, full adversarial/property/canary/process-tree qualification, Spec Kit convergence, authorized post-CI Qodo review with no unresolved material findings, merge, and post-merge canonical-main evidence.

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
T003_034=PASS
T003_034_QUALIFIED_HEAD=0bcaffb231070082be411e2e37959004ce359ad6
T003_034_CI_RUN=33149069868
T003_035=PASS
T003_035_QUALIFIED_HEAD=ffc8a66c881b1a34dafe32f79beebe03cceba939
T003_035_CI_RUN=33149715031
T003_040=PASS
T003_040_QUALIFIED_HEAD=cb69d638107ca4fe0118c9a61f143ac3ba65a2d3
T003_040_CI_RUN=33150969442
T003_041=PASS
T003_041_QUALIFIED_HEAD=76e1addf35c92a22d2c5826ca429278cacd598b3
T003_041_CI_RUN=33151556481
T003_042=PASS
T003_042_QUALIFIED_HEAD=67f74c9b9b75e43b9fa00069050c97c041567184
T003_042_CI_RUN=33152187952
T003_043=PASS
T003_043_QUALIFIED_HEAD=2f8655b5bdddd17bb9e6eab7bf00f11a210896cb
T003_043_CI_RUN=33154505847
T003_044=PASS
T003_044_QUALIFIED_HEAD=1a9fcddff4c4dd6a6161547cf89a502750f9bc71
T003_044_CI_RUN=33155122088
T003_045=ACTIVE
NEXT_TASK=T003-045
SPEC_003_IMPLEMENTATION_COMPLETE=NO
SPEC_003_CLOSED_CANONICAL=NO
PR_READY=NO
```

## Mutation discipline

For every task:

1. re-read the relevant spec/plan/contracts/data-model/task entry;
2. implement only the bounded eligible task;
3. add focused deterministic/adversarial/property evidence appropriate to the invariant;
4. run fresh CI on the new exact head;
5. never mark PASS before exact evidence exists;
6. record the qualification head/run in repository evidence and `tasks.md`;
7. immediately begin the next genuinely eligible task unless a real repository/governance blocker exists.

No force-push. No rebase. No destructive history rewrite. No merge or CLOSED_CANONICAL claim without the required exact-head/post-merge evidence.