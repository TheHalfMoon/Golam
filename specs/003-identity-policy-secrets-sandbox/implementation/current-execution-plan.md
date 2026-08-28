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

T003-041 established monotonic `TaintSet::union`, a provenance carrier that does not alter the wrapped value's identity, derived-artifact propagation tests, and typed authority-context provenance tests. There is still no downgrade/removal API.

T003-042 established a protected verifier/sanitizer registry over the existing `verifier_rules` schema. Registration is bounded/canonical, rejects untrusted/generated/secret-derived registration provenance before mutation, and requires exact current authorization, exact at-most-once elevated effect, exact ONCE approval with registration taint digest, atomic approval consumption, and fresh `authority-security-v2` coverage. The registry stores authority; it does not itself perform a downgrade.

Next canonical task: **T003-043** — implement human/deterministic-verifier downgrade attestations as new evidence rather than in-place source mutation.

T003-043 bounded design constraints:

- reuse the already-created protected `taint_attestations` schema unless a proven invariant requires a migration;
- a downgrade must create a new attestation/derived-artifact evidence record; source provenance is immutable;
- human downgrade and deterministic-verifier downgrade are distinct mechanisms;
- deterministic-verifier downgrade must reference an active registered rule whose allowed downgrade set covers the requested label removal;
- the verifier/rule evidence must be independent of the tainted source and cannot be supplied as self-authenticating model/channel/MCP/plugin content;
- human downgrade must be exact protected authority work under current authorization and approval scope; a free-form content assertion is not approval;
- `SECRET_DERIVED` is not cleared by the normal human/deterministic-verifier path; its separately authorized deterministic secret-elimination path remains T003-045;
- result labels are a separately evidenced set and source labels/rows remain unchanged;
- the attestation row and fresh `authority-security` evidence commit atomically;
- this task does not yet implement the canonical long-term-memory sink (T003-044) or secret-elimination sanitizer execution (T003-045).

Remaining Phase E order:

1. T003-043 downgrade attestations as new evidence;
2. T003-044 `SECRET_DERIVED` long-term-memory admission denial boundary;
3. T003-045 deterministic secret-elimination sanitizer evidence;
4. T003-046 multi-hop/self-clear/unregistered-verifier/SECRET_DERIVED adversarial qualification.

Do not begin Phase F until the Phase E predecessor tasks required by `tasks.md` are complete and qualified.

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
NEXT_TASK=T003-043
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