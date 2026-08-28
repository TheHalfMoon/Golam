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
- Later branch mutations do not transfer these historical qualification runs into final exact-head closeout evidence.

### Phase E — Taint and verifier state

`ACTIVE`

Completed:

- T003-040 qualified at `cb69d638107ca4fe0118c9a61f143ac3ba65a2d3` with CI #359 / run `33150969442` SUCCESS on Windows/macOS/Ubuntu. Evidence: `implementation/taint-baseline-qualification.md`.
- The baseline primitive is data-only in `golam-core`: exact nine-label closed set, frozen codes/names, bounded duplicate-insensitive set representation, and deterministic domain-separated canonical encoding.

Next canonical task: **T003-041** — implement monotonic union propagation for derived artifacts and authority context.

T003-041 bounded design constraints:

- propagation is monotonic set union: all source labels plus transform-introduced labels;
- no API in T003-041 may remove/downgrade labels;
- `ArtifactReceipt` remains content identity only; provenance must be a separate typed wrapper/metadata layer so taint does not alter content hash semantics;
- authorization context must carry relevant taint and bind it into deterministic policy/audit context;
- caller/source order must not change the resulting set or canonical authority input;
- verifier registration, downgrade attestations, sanitizer authority and memory-sink rejection remain T003-042+ and must not be pulled forward.

Remaining Phase E order:

1. T003-041 monotonic union propagation;
2. T003-042 protected verifier/sanitizer registry;
3. T003-043 downgrade attestations as new evidence;
4. T003-044 `SECRET_DERIVED` long-term-memory admission denial boundary;
5. T003-045 deterministic secret-elimination sanitizer evidence;
6. T003-046 multi-hop/self-clear/unregistered-verifier/SECRET_DERIVED adversarial qualification.

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
T003_034=PASS
T003_034_QUALIFIED_HEAD=0bcaffb231070082be411e2e37959004ce359ad6
T003_034_CI_RUN=33149069868
T003_035=PASS
T003_035_QUALIFIED_HEAD=ffc8a66c881b1a34dafe32f79beebe03cceba939
T003_035_CI_RUN=33149715031
PHASE_E_ACTIVE=YES
T003_040=PASS
T003_040_QUALIFIED_HEAD=cb69d638107ca4fe0118c9a61f143ac3ba65a2d3
T003_040_CI_RUN=33150969442
NEXT_TASK=T003-041
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