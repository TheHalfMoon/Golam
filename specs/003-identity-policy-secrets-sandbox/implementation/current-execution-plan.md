# Spec 003 — Live Implementation Execution Plan

**Status**: IMPLEMENTATION_ACTIVE — PHASE_I_COMPLETE — PHASE_J_CLOSEOUT_ACTIVE  
**Canonical base**: `main@82de7084384009ff3a00522f4e0aef09bf549529`  
**Implementation branch**: `impl/003-identity-policy-secrets-sandbox`  
**Current gate**: fresh exact-head CI after the T003-096 governance-record repair, then fresh substantive independent external review

## Authority

Live GitHub truth, the constitution, canonical Spec 003 package, `tasks.md`, and exact implementation evidence govern this companion. No force-push, rebase, destructive history rewrite, waiver, CI weakening, unrecorded review substitution, or closeout claim without exact evidence.

## Completed implementation phases

- Phase A: T003-001..T003-006 complete.
- Phase B: T003-010..T003-017 complete.
- Phase C: T003-020..T003-024 complete.
- Phase D: T003-030..T003-035 complete.
- Phase E: T003-040..T003-046 complete.
- Phase F: T003-050..T003-057 complete.
- Phase G: T003-060..T003-064 complete.
- Phase H: T003-070..T003-076 complete.
- Phase I: T003-080..T003-084 complete.

### T003-083

`PASS` at `3c21c0a999b946ceccbf3a9418421c357e0ef350` by CI #653 / run `33307346352`, SUCCESS on Windows/macOS/Ubuntu. Evidence: `t003-083-spec002-regression-qualification.md`.

### T003-084

`PASS` at `a3cf1f19c28aead5b9894b579a3fe8a0b9f2a3f0` by CI #656 / run `33357216307`, SUCCESS on Windows/macOS/Ubuntu. Evidence: `t003-084-authority-commit-fault-qualification.md`.

## Phase J closeout

### T003-090..T003-094 — pre-convergence exact-head qualification

CI #660 / run `33357835321` completed SUCCESS at exact head `4430ff95ec81c1f3e0c9683de2043c3b8803fe9e` on Windows/macOS/Ubuntu. It executed:

- pinned formatting and Clippy with warnings denied;
- full workspace tests;
- property qualification;
- bounded fuzz smoke including the new Spec 003 policy/profile/authority corpus;
- platform-applicable IPC transport qualification;
- authenticated daemon IPC qualification;
- adversarial authority qualification;
- daemon build for external locality observation;
- platform-applicable external strict-local process-tree observation.

The full workspace includes the deterministic recognized and deliberately unknown-format secret-canary suite. Therefore T003-090..T003-094 have pre-convergence evidence, but CI #660 is not final closeout evidence after T003-095 mutates the branch.

### T003-095 — convergence

`PASS` by the convergence mutation that re-read constitution v1.2.0, current `AGENTS.md`, `spec.md`, `plan.md`, all six Spec 003 contracts, implementation-readiness, tasks, implementation evidence and live PR state.

Result:
- constitution: aligned; no amendment required;
- `AGENTS.md`: aligned with Spec 003 implementation;
- all six Spec 003 contracts: semantically aligned; no contract mutation required;
- `spec.md` and `plan.md` planning headers retained as historical planning provenance rather than rewritten as live execution state;
- stale live-state records in tasks, this execution companion, implementation-readiness and PR metadata are reconciled;
- no authority semantics, schema, dependency, workflow, product code, test behavior or security boundary changed by convergence.

Evidence: `t003-095-convergence.md`.

### Founder review-source amendment — 2026-08-31

The founder explicitly directed: `skip qodo use others` after Qodo remained externally billing-blocked. Constitution v1.2.0 requires exact reproducible verification but does not bind Spec 003 to a named review vendor. This amendment therefore changes only the task-specific external-review source policy; it is not a waiver and does not remove the external semantic review gate.

- Qodo is excluded from this Spec 003 closeout sequence by the latest founder direction.
- Codex remains excluded.
- T003-096 requires a fresh substantive independent external semantic review on the exact CI-qualified head from an available repository-integrated reviewer such as CodeRabbit, Cubic, Greptile, or an equivalent independent service.
- A summary-only, status-only, rate-limited, billing-blocked, unavailable, stale-head or self-authored result does not satisfy T003-096.
- Any material finding requires repair, fresh exact-head three-platform CI, and a fresh substantive independent external review.

Evidence: `t003-096-review-source-amendment.md`.

### T003-096 review cycle — reviewed head and repair

Exact head `5ce152a8a3370b3927eb7b9eeaed838a3e0c7dc6` completed CI #662 / run `33395230450` SUCCESS on Windows/macOS/Ubuntu. A fresh substantive CodeRabbit review was then requested against that unchanged exact head.

The substantive review reported one material closeout-governance finding and no additional material product correctness/security defect. The finding was stale live execution state in this file, `tasks.md`, and `checklists/implementation-readiness.md`: those records still described final CI as pending and retained stale Qodo-era wording despite CI #662 and the founder review-source amendment.

This repair updates only repository-owned governance/execution records. It changes no product code, workflow, dependency, schema, test, authority, secret, taint, egress, or sandbox semantics. Because this repair mutates the branch, CI #662 and that CodeRabbit result remain valid evidence for the reviewed head but do not transfer as final qualification for the repaired head.

## Remaining ordered gates

1. Obtain fresh full Windows/macOS/Ubuntu CI on the exact repaired head. No prior run transfers as final closeout evidence after this mutation.
2. **T003-096**: after that CI succeeds, obtain a fresh substantive independent external semantic review on the exact same repaired head from an available repository-integrated reviewer other than Qodo or Codex.
3. If the reviewer finds any material issue, repair it, then repeat fresh exact-head CI and fresh external review.
4. **T003-097**: only after a clean substantive exact-head review, prepare exact-head closeout/lifecycle evidence, move PR Ready when repository lifecycle requirements are satisfied, and merge with expected-head protection.
5. **T003-098**: after merge, require canonical `main` post-merge CI SUCCESS before `SPEC_003_CLOSED_CANONICAL=YES` or starting Spec 004.

```text
SPEC_003_PLANNING_CLOSED_CANONICAL=YES
SPEC_003_IMPLEMENTATION_AUTHORIZED=YES
PHASE_I_COMPLETE=YES
PHASE_J_CLOSEOUT_ACTIVE=YES
PHASE_J_PRE_CONVERGENCE_HEAD=4430ff95ec81c1f3e0c9683de2043c3b8803fe9e
PHASE_J_PRE_CONVERGENCE_CI_RUN=33357835321
T003_095=PASS
REVIEW_SOURCE_POLICY=INDEPENDENT_EXTERNAL_NON_QODO_NON_CODEX
T003_096_REVIEWED_HEAD=5ce152a8a3370b3927eb7b9eeaed838a3e0c7dc6
T003_096_REVIEWED_HEAD_CI_RUN=33395230450
T003_096_REVIEWED_HEAD_CI=PASS
T003_096_REVIEW_RESULT=MATERIAL_GOVERNANCE_FINDING_REPAIR_REQUIRED
T003_096_PRODUCT_SECURITY_FINDINGS=NONE_ADDITIONAL
T003_096_REPAIR=APPLIED_FORWARD_ONLY
FINAL_EXACT_HEAD_CI=PENDING_AFTER_T003_096_REPAIR
NEXT_TASK=FRESH_EXACT_HEAD_CI_THEN_FRESH_EXTERNAL_REVIEW
SPEC_003_IMPLEMENTATION_COMPLETE=NO
SPEC_003_CLOSED_CANONICAL=NO
PR_READY=NO
WAIVER_TAKEN=NO
```
