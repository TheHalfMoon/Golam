# Spec 003 — Live Implementation Execution Plan

**Status**: IMPLEMENTATION_ACTIVE — PHASE_I_COMPLETE — PHASE_J_CLOSEOUT_ACTIVE  
**Canonical base**: `main@82de7084384009ff3a00522f4e0aef09bf549529`  
**Implementation branch**: `impl/003-identity-policy-secrets-sandbox`  
**Current gate**: follow exact live GitHub PR metadata under the non-self-invalidating closeout evidence rule

## Authority

Live GitHub truth, the constitution, canonical Spec 003 package, `tasks.md`, and exact implementation evidence govern this companion. No force-push, rebase, destructive history rewrite, waiver, CI weakening, unrecorded review substitution, or closeout claim without exact evidence.

Final mutable closeout status is intentionally not mirrored as PASS/PENDING in the qualified branch. `t003-096-live-closeout-evidence-policy.md` makes exact live GitHub PR metadata authoritative for the current head, final exact-head CI, final external review, Ready/merge state, and post-merge canonical-main CI.

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

The full workspace includes the deterministic recognized and deliberately unknown-format secret-canary suite. Therefore T003-090..T003-094 have pre-convergence evidence, but CI #660 is not final closeout evidence after later branch mutations.

### T003-095 — convergence

`PASS` by the convergence mutation that re-read constitution v1.2.0, current `AGENTS.md`, `spec.md`, `plan.md`, all six Spec 003 contracts, implementation-readiness, tasks, implementation evidence and live PR state.

Result:
- constitution: aligned; no amendment required;
- `AGENTS.md`: aligned with Spec 003 implementation;
- all six Spec 003 contracts: semantically aligned; no contract mutation required;
- `spec.md` and `plan.md` planning headers retained as historical planning provenance rather than rewritten as live execution state;
- stale live-state records in tasks, this execution companion, implementation-readiness and PR metadata were reconciled;
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

### T003-096 review cycle — reviewed heads and structural repair

Exact head `5ce152a8a3370b3927eb7b9eeaed838a3e0c7dc6` completed CI #662 / run `33395230450` SUCCESS on Windows/macOS/Ubuntu. A fresh substantive CodeRabbit review was then requested against that unchanged exact head.

That review reported one material closeout-governance finding and no additional material product correctness/security defect. The finding was stale live execution state in this file, `tasks.md`, and `checklists/implementation-readiness.md`: those records still described final CI as pending and retained stale Qodo-era wording despite CI #662 and the founder review-source amendment.

Forward-only repair head `862936b4ea3c62ead65b318ba394b49444722944` reconciled those records and then completed CI #663 / run `33405009138` SUCCESS on Windows/macOS/Ubuntu. A second substantive CodeRabbit review again found no additional material product correctness/security defect, but identified that mutable branch fields such as `FINAL_EXACT_HEAD_CI=PENDING_AFTER_T003_096_REPAIR` necessarily became stale as soon as CI #663 succeeded.

This demonstrates a structural self-invalidation loop: changing a qualified branch merely to record the latest CI or review PASS creates a new exact head and invalidates the evidence being recorded. `t003-096-live-closeout-evidence-policy.md` resolves that issue without weakening any gate: branch artifacts retain stable authority, task rules and historical evidence, while exact live GitHub PR metadata is the authoritative mutable final evidence ledger.

No product code, workflow, dependency, schema, test, authority, secret, taint, egress, or sandbox semantics are changed by this evidence-placement repair.

## Remaining ordered gates

1. After any branch mutation, obtain fresh full Windows/macOS/Ubuntu CI on the resulting exact head. No prior run transfers as final closeout evidence after mutation.
2. **T003-096**: after that CI succeeds, obtain a fresh substantive independent external semantic review on the exact same head from an available repository-integrated reviewer other than Qodo or Codex.
3. If the reviewer finds any material issue, repair it forward-only, then repeat fresh exact-head CI and fresh external review.
4. If the review is clean, record T003-096 PASS in exact live GitHub PR metadata without mutating the qualified branch merely to mirror PASS.
5. **T003-097**: with the exact head unchanged and no unresolved material review threads, move PR Ready when repository lifecycle requirements are satisfied and merge with expected-head protection.
6. **T003-098**: after merge, require canonical `main` post-merge CI SUCCESS on the actual merge commit before `SPEC_003_CLOSED_CANONICAL=YES` or starting Spec 004.

```text
SPEC_003_PLANNING_CLOSED_CANONICAL=YES
SPEC_003_IMPLEMENTATION_AUTHORIZED=YES
PHASE_I_COMPLETE=YES
PHASE_J_CLOSEOUT_ACTIVE=YES
PHASE_J_PRE_CONVERGENCE_HEAD=4430ff95ec81c1f3e0c9683de2043c3b8803fe9e
PHASE_J_PRE_CONVERGENCE_CI_RUN=33357835321
T003_095=PASS
REVIEW_SOURCE_POLICY=INDEPENDENT_EXTERNAL_NON_QODO_NON_CODEX
CLOSEOUT_EVIDENCE_AUTHORITY=LIVE_GITHUB_PR_METADATA
EMBED_LATEST_FINAL_CI_STATUS_IN_QUALIFIED_BRANCH=NO
EMBED_LATEST_FINAL_REVIEW_STATUS_IN_REVIEWED_BRANCH=NO
T003_096_HISTORY_HEAD_1=5ce152a8a3370b3927eb7b9eeaed838a3e0c7dc6
T003_096_HISTORY_CI_1=33395230450
T003_096_HISTORY_HEAD_2=862936b4ea3c62ead65b318ba394b49444722944
T003_096_HISTORY_CI_2=33405009138
T003_096_HISTORY_PRODUCT_SECURITY_FINDINGS=NONE_ADDITIONAL
FINAL_EXACT_HEAD_CI=SEE_LIVE_GITHUB_PR_METADATA
FINAL_EXTERNAL_REVIEW=SEE_LIVE_GITHUB_PR_METADATA
NEXT_TASK=SEE_LIVE_GITHUB_PR_METADATA
SPEC_003_CLOSED_CANONICAL=NO_UNTIL_POST_MERGE_MAIN_CI
WAIVER_TAKEN=NO
```