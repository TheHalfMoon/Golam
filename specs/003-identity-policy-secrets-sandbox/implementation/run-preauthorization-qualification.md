# T003-034 RUN_PREAUTHORIZATION Qualification

**Task**: T003-034 — Enforce bounded RUN_PREAUTHORIZATION for unattended irreversible effects and deny generic always-allow behavior.  
**Qualified implementation head**: `0bcaffb231070082be411e2e37959004ce359ad6`  
**CI**: GitHub Actions `ci` run #349 (`33149069868`) — SUCCESS on Windows, macOS, and Ubuntu.  
**Status**: PASS for T003-034 only. This evidence does not transfer to later branch mutations or final Spec 003 closeout.

## Implemented boundary

The protected path is `KernelApi::claim_unattended_irreversible_run_preauthorization` -> `RunPreauthorizationStore::claim_unattended_irreversible`.

The implementation:

- accepts only `RUN_PREAUTHORIZATION` for unattended irreversible execution;
- derives action, resource, risk class, execution semantics, session, and current authorized state from the protected effect record rather than caller-supplied widening fields;
- requires one exact bound session/run; sessionless run authorization is denied;
- revalidates approval freshness, exact action/resource scope, risk, taint digest, and run/session binding before claim;
- rejects non-irreversible effects;
- rejects effects that are not currently `authorized`;
- rejects other approval classes as substitutes;
- enforces a hard maximum of `MAX_UNATTENDED_IRREVERSIBLE_RUN_USES = 256`;
- counts prior reserved/consumed uses fail-closed;
- derives a deterministic `(approval_id, effect_id)` consumption identifier and denies replay of the same protected effect;
- commits approval consumption under an IMMEDIATE SQLite transaction with integrity and authority-security verification before commit;
- does not introduce a global or generic irreversible "always allow" state.

## Direct tests

`crates/golam-ledger/src/run_preauthorization.rs` includes tests proving:

1. bounded claims succeed only until the declared per-run usage limit;
2. other approval classes cannot substitute for `RUN_PREAUTHORIZATION`;
3. sessionless/unbound run scope is denied;
4. replay of one exact effect is denied;
5. a requested allowance above the Spec 003 unattended irreversible ceiling is denied.

Existing approval-use validation additionally rechecks freshness, scope, risk, and taint before this claim path can commit.

## CI repair history

The initial implementation head `69996beb7defb4dacba205fedac6fb0d5866cb04` did not qualify because CI #335 stopped at rustfmt. The next head `de0fb669ba9eb96f4e962cb9ac9a7c2a83f1640e` passed formatting and exposed one Clippy lint on an 8-argument test helper. The final qualification head `0bcaffb231070082be411e2e37959004ce359ad6` applies only the repository-conventional targeted test-helper Clippy allowance and passed the complete CI matrix.

## Exact-head gate result

```text
T003_034=PASS
QUALIFIED_HEAD=0bcaffb231070082be411e2e37959004ce359ad6
CI_RUN=33149069868
CI_RUN_NUMBER=349
WINDOWS=SUCCESS
MACOS=SUCCESS
UBUNTU=SUCCESS
WAIVER=NO
NEXT_TASK=T003-035
```
