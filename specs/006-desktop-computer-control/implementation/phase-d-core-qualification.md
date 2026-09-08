# Spec 006 Phase D Core Qualification Evidence

## Authority and scope

This record binds only the Phase D core hardening that precedes native semantic observation adapters. It does not mark T006-016 through T006-019 complete, does not preapprove adapter correctness, and does not satisfy final Spec 006 review or closeout gates.

## Qualified baseline

- Product baseline SHA: `773f79cbbe58d9784e4a1b43351fca4b91d275c3`
- Exact-head regular CI run: `34256610049`
- Result: SUCCESS on Windows, macOS, and Ubuntu.
- The run completed the repository CI matrix, including format, clippy, tests, property qualification, bounded fuzz smoke, platform IPC qualification, authenticated daemon IPC qualification, adversarial authority qualification, daemon build, and strict-local network observation. Linux also completed the applicable native-containment and governed-process qualification steps.

## Phase D core materialization history

### Preserved failed qualification attempt

- Helper-only trigger SHA: `b120793ab29a25cde4e94a5e0b4b96abfd3e8306`
- Materialization run: `34258522270`
- Job: `102170327233`
- Result: FAILURE.
- Exact failure: `cargo clippy --locked -p golam-core -p golam-kernel --all-targets -- -D warnings` rejected four test-only `field_reassign_with_default` instances in the new semantic-observation bound tests.
- No product commit was created by this failed run.
- The failure remains historical evidence and was not rerun-to-green.

### Forward-only corrected qualification attempt

- Helper-only repair SHA: `de5ded30a0f4be63f871cdfe86bf2f75d432e4c8`
- Materialization run: `34258878710`
- Job: `102171529904`
- Result: SUCCESS.
- The exact product base remained `773f79cbbe58d9784e4a1b43351fca4b91d275c3`; every commit between that product base and the corrected helper head changed only `.github/workflows/spec006-phase-d-core-materialize.yml`.
- The corrected run completed:
  - exact product-base/helper-only-chain guard;
  - bounded patch application;
  - Rust 1.98.0 installation;
  - `cargo +1.98.0 fmt --all`;
  - `cargo +1.98.0 clippy --locked -p golam-core -p golam-kernel --all-targets -- -D warnings`;
  - `cargo +1.98.0 test --locked -p golam-core -p golam-kernel --all-targets`;
  - bounded-diff proof;
  - self-removal of the temporary helper;
  - product commit creation.

## Materialized product head

- Materialized product SHA: `cfe227177400c82d542c74762dccee3d9fc40756`
- Commit: `feat(006): harden semantic bounds and focus readback`
- Net diff from the qualified product baseline `773f79cbbe58d9784e4a1b43351fca4b91d275c3` contains only:
  - `crates/golam-core/src/desktop_control.rs`
  - `crates/golam-kernel/src/desktop_dispatch.rs`
- The temporary materialization workflow is absent from the net product tree.

The hardening adds explicit semantic observation depth and wall-time limits to the canonical `DesktopLimits` binding. It also requires a committed focus dispatch to be followed by a new backend observation that preserves the prepared permission/session evidence and confirms the exact intended surface as focused before the kernel can classify the operation as succeeded. Missing, mismatched, stale, invalid, or unavailable readback remains `UNKNOWN_OUTCOME` rather than being promoted to success.

## Bot-authored head non-execution

- PR CI run on materialized bot-authored head: `34259225942`
- Head SHA: `cfe227177400c82d542c74762dccee3d9fc40756`
- Conclusion: `action_required`
- Jobs returned by GitHub: none (`jobs=[]`).
- Actor and triggering actor: `github-actions[bot]`.

This is recorded as platform non-execution, not as successful product qualification and not as a test failure. A new human-authored head must receive fresh exact-head regular CI before this hardening is treated as qualified for continued Phase D implementation.

## Current disposition

`PHASE_D_CORE_HARDENING_MATERIALIZED=YES`
`PHASE_D_CORE_EXACT_HEAD_REGULAR_CI=PENDING`
`T006_016_T006_019_COMPLETE=NO`
`FINAL_SPEC006_REVIEW=NOT_PREAPPROVED`
`SPEC_006_IMPLEMENTATION_COMPLETE=NO`
`SPEC_006_CLOSED_CANONICAL=NO`
`WAIVER_TAKEN=NO`
