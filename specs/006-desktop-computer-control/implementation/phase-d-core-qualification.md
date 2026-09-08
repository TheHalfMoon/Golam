# Spec 006 Phase D Core Qualification Evidence

## Authority and scope

This record binds only the Phase D core hardening and native-observation qualification evidence accumulated before canonical Phase D closeout. It does not mark T006-016 through T006-019 complete, does not preapprove adapter correctness, and does not satisfy final Spec 006 review or closeout gates.

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

## Materialized core product head

- Materialized product SHA: `cfe227177400c82d542c74762dccee3d9fc40756`
- Commit: `feat(006): harden semantic bounds and focus readback`
- Net diff from the qualified product baseline `773f79cbbe58d9784e4a1b43351fca4b91d275c3` contains only:
  - `crates/golam-core/src/desktop_control.rs`
  - `crates/golam-kernel/src/desktop_dispatch.rs`
- The temporary materialization workflow is absent from the net product tree.

The hardening adds explicit semantic observation depth and wall-time limits to the canonical `DesktopLimits` binding. It also requires a committed focus dispatch to be followed by a new backend observation that preserves the prepared permission/session evidence and confirms the exact intended surface as focused before the kernel can classify the operation as succeeded. Missing, mismatched, stale, invalid, or unavailable readback remains `UNKNOWN_OUTCOME` rather than being promoted to success.

## Core bot-authored head non-execution

- PR CI run on materialized bot-authored head: `34259225942`
- Head SHA: `cfe227177400c82d542c74762dccee3d9fc40756`
- Conclusion: `action_required`
- Jobs returned by GitHub: none (`jobs=[]`).
- Actor and triggering actor: `github-actions[bot]`.

This is recorded as platform non-execution, not as successful product qualification and not as a test failure.

## Native observation API qualification

### Preserved initial probe failure

- Probe head: `fda6e01a690ae2928cb1e5fdb7c20c5c95c319b0`
- Probe run: `34261321491`
- Result: FAILURE.
- Windows and macOS compiled the exact admitted observation APIs and passed semantic-route structural restrictions, but their helper-cleanliness step failed because the probe left untracked Cargo build output before the repository-clean proof.
- Ubuntu failed compilation because `atspi::proxy::accessible::ObjectRefOwned` children were treated as `AccessibleProxy` values instead of being converted through the admitted `ObjectRefExt::as_accessible_proxy` API.
- The run remains failed historical evidence and was not rerun-to-green.

### Forward-only corrected probe

- Corrected probe head: `287aec7d12deedfd3f95705b1348bb3b7cba311a`
- Probe run: `34261891461`
- Result: SUCCESS on Windows, macOS, and Ubuntu.
- The probe compiled the exact admitted platform observation surfaces and proved:
  - Windows semantic observation uses `xa11y_windows::WindowsProvider` without `WindowsInputProvider`, screenshot, raw-input, or Windows.Graphics.Capture dispatch surfaces;
  - macOS observation uses safe AXUIElement and ScreenCaptureKit read-only enumeration without raw FFI or actuation calls;
  - Linux AT-SPI traversal uses the admitted `golam-bounded-linux-async::run_bounded` facade and does not directly reference Tokio;
  - the helper left no product source or manifest mutation.

## Native observation product materialization

- Product adapter commit: `493d27e4d3c5317905ef35d863609dab877fa03c`
- Commit: `feat(006): add bounded native desktop observation adapter`
- Added only `crates/golamd/src/desktop_observation.rs`.
- No product manifest or lockfile changed.
- The adapter is observation-only: it builds canonical `WorkSurfaceIdentity`, `SemanticElementIdentity`, and `DesktopObservation` evidence; it does not mint capability, policy, approval, fallback eligibility, control-lease, visible-channel, or Kernel/Effect Gate authority and imports no raw-input, capture-dispatch, or clipboard actuation API.

### Qualification linkage and preserved format failure

- Integration qualification commit: `bbf0589f830a1d94d0a3fbe65d5927d4cabd49bb`
- Commit: `test(006): compile native desktop observation adapter`
- Added `crates/golamd/tests/desktop_observation_adapter.rs`, which links the product adapter source directly into a Cargo integration-test target so platform-specific code enters normal format/clippy/test qualification.
- Exact-head regular CI run: `34264179085`.
- Result: FAILURE.
- Windows and macOS failed at `cargo +1.98.0 fmt --all -- --check` before clippy/tests. The failure was formatting-only; it remains preserved and was not rerun-to-green.

### Canonical formatter materialization

- Human helper trigger: `b2fa080cdf75b0b83a1c1807d0fa4528ef6a5d57`
- Helper run: `34264359830`
- Result: SUCCESS.
- The helper bound the exact trigger head, installed Rust 1.98.0 with rustfmt, ran `cargo +1.98.0 fmt --all`, reran format check, proved the pre-self-removal diff contained only:
  - `crates/golamd/src/desktop_observation.rs`
  - `crates/golamd/tests/desktop_observation_adapter.rs`
- The helper then removed `.github/workflows/spec006-phase-d-format-materialize.yml`, proved the final bounded diff contained only the helper deletion plus those two formatted files, and created forward-only commit `33466023e8c696f1fd430a8e70dc41a547a2bb54` (`style(006): format native observation adapter`).
- The temporary formatter workflow is absent from the resulting product tree.

### Preserved exact-head tri-OS Clippy failure

- Human evidence anchor: `8c8e7bce31807cf2ec53d08b637e11634f13379e`.
- Exact-head regular CI run: `34264531350`.
- Result: FAILURE on Windows, macOS, and Ubuntu after all three platforms passed format.
- The common Clippy failures were two missing slice borrows in opaque observation identity construction.
- Windows additionally reported an unused `xa11y_core::Element` import.
- Ubuntu additionally reported an unused outer `ObjectRefExt` import and two redundant `unwrap_or` calls on `path_as_str()` values that are already `&str`.
- This failed run is preserved and was not rerun-to-green.

### Preserved first Clippy-repair materializer failure

- Helper trigger: `a61e50d48942625ac5c2c96ee09ab818cd658eaa`.
- Materializer run: `34265116095`.
- Result: FAILURE before product commit.
- Exact helper-chain binding and all five evidence-driven compile corrections succeeded.
- Linux qualification then reached two test-only `field_reassign_with_default` Clippy failures in the sanitizer and semantic-node-budget tests.
- Because qualification failed, the bounded-diff/self-removal/commit step was skipped and no product repair was pushed by this attempt.
- The failed run remains preserved and was not rerun-to-green.

### Forward-only corrected Clippy repair

- Updated helper trigger: `975d1a90e6c141fbc140c9549b89b98a0f318c8e`.
- Materializer run: `34265458551`.
- Result: SUCCESS.
- The helper proved the complete chain from `8c8e7bce31807cf2ec53d08b637e11634f13379e` through the helper trigger changed only `.github/workflows/spec006-phase-d-clippy-repair-materialize.yml` before applying product corrections.
- It applied exactly the five previously evidenced compile corrections plus two test-only struct-initializer corrections, each guarded by exact replacement counts.
- Linux qualification completed:
  - `cargo +1.98.0 fmt --all -- --check`;
  - `cargo +1.98.0 clippy --locked -p golamd --tests -- -D warnings`;
  - `cargo +1.98.0 test --locked -p golamd --test desktop_observation_adapter`.
- The helper proved the product diff contained only `crates/golamd/src/desktop_observation.rs`, removed itself, proved the final diff contained only the helper deletion plus that source file, and created forward-only bot commit `a7f3caba6a993b3e15332d6d7454edb5b9cd979b` (`fix(006): correct native observation adapter compile errors`).
- The temporary Clippy-repair helper is absent from the resulting product tree.

## Current disposition

`PHASE_D_CORE_HARDENING_MATERIALIZED=YES`
`PHASE_D_CORE_EXACT_HEAD_REGULAR_CI=SUCCESS_ON_F5760C06_PRE_ADAPTER_HEAD`
`PHASE_D_NATIVE_API_PROBE=SUCCESS_TRI_OS`
`PHASE_D_NATIVE_OBSERVATION_ADAPTER_MATERIALIZED=YES`
`PHASE_D_NATIVE_OBSERVATION_ADAPTER_LINUX_CLIPPY_TEST=SUCCESS_ON_A7F3CABA_PRODUCT_LOGIC`
`PHASE_D_NATIVE_OBSERVATION_ADAPTER_EXACT_HEAD_REGULAR_CI=PENDING`
`T006_016_T006_019_COMPLETE=NO`
`FINAL_SPEC006_REVIEW=NOT_PREAPPROVED`
`SPEC_006_IMPLEMENTATION_COMPLETE=NO`
`SPEC_006_CLOSED_CANONICAL=NO`
`WAIVER_TAKEN=NO`
