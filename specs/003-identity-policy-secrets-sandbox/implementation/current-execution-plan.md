# Spec 003 — Live Implementation Execution Plan

**Status**: IMPLEMENTATION_ACTIVE — PHASE_I_COMPLETE — PHASE_J_CLOSEOUT_ACTIVE  
**Canonical base**: `main@82de7084384009ff3a00522f4e0aef09bf549529`  
**Implementation branch**: `impl/003-identity-policy-secrets-sandbox`  
**Current gate**: fresh post-convergence exact-head CI before `T003-096`

## Authority

Live GitHub truth, the constitution, canonical Spec 003 package, `tasks.md`, and exact implementation evidence govern this companion. No force-push, rebase, destructive history rewrite, waiver, CI weakening, review substitution, or closeout claim without exact evidence.

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

## Remaining ordered gates

1. Obtain fresh full Windows/macOS/Ubuntu CI on the exact post-convergence head. No prior run transfers as final closeout evidence after this mutation.
2. **T003-096**: only after that CI succeeds, request fresh authorized Qodo review on the exact same head. Qodo is the required external review source; CodeRabbit/self-review is not a substitute.
3. If Qodo finds any material issue, repair it, then repeat fresh exact-head CI and fresh Qodo review.
4. **T003-097**: prepare exact-head closeout/lifecycle evidence, move PR Ready only when repository lifecycle requirements are satisfied, and merge with exact-head guard.
5. **T003-098**: after merge, require canonical `main` post-merge CI SUCCESS before `SPEC_003_CLOSED_CANONICAL=YES` or starting Spec 004.

```text
SPEC_003_PLANNING_CLOSED_CANONICAL=YES
SPEC_003_IMPLEMENTATION_AUTHORIZED=YES
PHASE_I_COMPLETE=YES
PHASE_J_CLOSEOUT_ACTIVE=YES
PHASE_J_PRE_CONVERGENCE_HEAD=4430ff95ec81c1f3e0c9683de2043c3b8803fe9e
PHASE_J_PRE_CONVERGENCE_CI_RUN=33357835321
T003_095=PASS
FINAL_EXACT_HEAD_CI=PENDING_AFTER_CONVERGENCE
NEXT_TASK=T003-096_AFTER_FRESH_CI
SPEC_003_IMPLEMENTATION_COMPLETE=NO
SPEC_003_CLOSED_CANONICAL=NO
PR_READY=NO
WAIVER_TAKEN=NO
```
