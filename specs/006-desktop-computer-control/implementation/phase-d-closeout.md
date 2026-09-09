# Spec 006 Phase D Closeout

## Audit correction

Phase D closeout was initially recorded against exact head `3908dbb94f9407959e64e518badf67309cdd115c` after regular CI run `34269326487` succeeded on Windows, macOS, and Ubuntu. A subsequent runtime-wiring audit found that `crates/golamd/src/desktop_observation.rs` was compiled through the integration-test `#[path]` linkage but was not exported from the `golamd` library module graph.

That distinction is material for product qualification. The historical exact-head CI remains valid evidence for the code it tested, but it is insufficient to claim the native observation adapter is part of the product runtime surface.

No history is rewritten and no prior failure/success evidence is discarded. Phase D is reopened until the adapter is wired into the product library and the corrected product head receives fresh qualification.

## Preserved historical qualification

```text
HISTORICAL_PHASE_D_HEAD=3908dbb94f9407959e64e518badf67309cdd115c
HISTORICAL_EXACT_HEAD_CI_RUN=34269326487
HISTORICAL_CI_MACOS_JOB=102206616201
HISTORICAL_CI_UBUNTU_JOB=102206616407
HISTORICAL_CI_WINDOWS_JOB=102206616627
HISTORICAL_CI_MACOS=SUCCESS
HISTORICAL_CI_UBUNTU=SUCCESS
HISTORICAL_CI_WINDOWS=SUCCESS
RUNTIME_MODULE_WIRING_AT_HISTORICAL_HEAD=NO
WAIVER_TAKEN=NO
```

The historical matrix completed format, Clippy, tests, property qualification, bounded fuzz smoke, authenticated daemon IPC qualification, adversarial authority qualification, daemon build, and applicable strict-local network observation. Linux additionally completed applicable native-containment and governed-process qualification.

## Required reconciliation

The forward-only Phase D reconciliation must:

1. export `desktop_observation` through the `golamd` product library module graph;
2. make the integration qualification import the product module instead of compiling a private duplicate through `#[path]`;
3. qualify the exact deterministic patch on Windows, macOS, and Ubuntu;
4. preserve the observation-only structural restrictions and all authority boundaries;
5. obtain fresh regular CI on a human-authored descendant containing the materialized runtime wiring;
6. only then mark T006-016..T006-019 and Phase D complete.

## Preserved authority boundaries

- Native observation cannot mint capability, policy, approval, fallback eligibility, Kernel/Effect Gate authorization, control-lease authority or visible-channel state.
- Observation/capture/focus/semantic action/raw input/clipboard remain distinct authority domains.
- No raw-input or clipboard authority is admitted by Phase D.
- No camera, microphone, OCR, hidden HTTP/cloud fallback, Windows secure-desktop/UAC bypass, Wayland bypass or donor authority is introduced.
- Exact Source Foundry admission remains mandatory for every dependency/runtime primitive used by later phases.

## Runtime-wiring reconciliation evidence

The runtime-wiring audit is reconciled forward-only. Historical CI and failed qualification attempts remain preserved.

### Product runtime wiring

- Initial runtime-wiring materializer run `34285130893` failed before product commit because `cargo fmt --check` required the integration-test import to be reordered; no product commit was created.
- Corrected materializer run `34285237017` succeeded after exact helper-chain, bounded-diff, format, Clippy and adapter-test qualification.
- Product commit `c2cdf62d6bf19b2a70897774844e80f37d961282` (`fix(006): wire native observation into golamd runtime`) exported `desktop_observation` through the `golamd` library module graph and changed the integration test to import the product module.
- Human cleanup commit `4701d21eb39e288c258ddf00526f2784844a92de` removed the temporary runtime-wiring helper.

### Preserved macOS failure and forward-only repair

- Regular CI run `34285589422` on clean head `4701d21eb39e288c258ddf00526f2784844a92de` succeeded on Ubuntu and Windows but failed on macOS Clippy because `empty_platform_snapshot` was unused in non-test macOS builds.
- First macOS cfg-repair helper `674a05989960590d25827e78a19001d2cbd87527`, run `34287393002`, failed before product commit because the helper had not built the Tauri `frontendDist` required by workspace Clippy; no product commit was created.
- Forward-only helper repair `636651dd72ea2054468f9c9c00b24f9a80167203` aligned qualification with regular CI.
- Corrected helper run `34287724816`, job `102267065785`, succeeded through frontend qualification, Rust 1.98.0 format, workspace all-target Clippy, adapter test, exact product-diff proof and guarded push.
- Product repair commit `3125190fb9859dbcb9c4887d0662dc6bb296aa4a` (`fix(006): scope empty observation snapshot on macOS`) scopes the empty snapshot helper to production targets that use it plus tests.
- Human cleanup commit `02db0ce8cb1e0d6ba13aec6da57c1f76e743cc10` removed the temporary macOS repair helper.

### Exact clean-head qualification

Regular PR CI run `34288135133` completed with conclusion `success` on exact clean head `02db0ce8cb1e0d6ba13aec6da57c1f76e743cc10`.

- Ubuntu job `102268344321`: SUCCESS.
- Windows job `102268344391`: SUCCESS.
- macOS job `102268344457`: SUCCESS.

The matrix completed the applicable frontend closure, format, workspace Clippy, workspace tests, property qualification, bounded fuzz smoke, IPC qualification, authenticated daemon IPC, adversarial authority qualification, daemon build and strict-local external-network observation. Ubuntu additionally completed native-containment and governed-process qualification.

The final adapter remains observation-only. Observation, semantic text, coordinates and pixel evidence cannot mint capability, policy, approval, fallback eligibility, Kernel/Effect Gate authorization, control-lease authority or visible-channel state. Governed focus still requires exact current Gate/request/effect/target/session/permission/lease/channel revalidation and successful focus readback before terminal success; uncertain focus remains `UNKNOWN_OUTCOME`.

## Current disposition

```text
PHASE_D_RUNTIME_WIRING_AUDIT=RECONCILED
PHASE_D_RECONCILIATION=COMPLETE
PHASE_D_EXACT_CLEAN_HEAD=02db0ce8cb1e0d6ba13aec6da57c1f76e743cc10
PHASE_D_EXACT_HEAD_CI_RUN=34288135133
PHASE_D_CI_UBUNTU=SUCCESS
PHASE_D_CI_WINDOWS=SUCCESS
PHASE_D_CI_MACOS=SUCCESS
T006_016_COMPLETE=YES
T006_017_COMPLETE=YES
T006_018_COMPLETE=YES
T006_019_COMPLETE=YES
PHASE_D_COMPLETE=YES
PHASE_E_PRODUCT_IMPLEMENTATION_START=AUTHORIZED_BY_DEPENDENCY_ORDER
FINAL_SPEC006_REVIEW=NOT_PREAPPROVED
SPEC_006_IMPLEMENTATION_COMPLETE=NO
SPEC_006_CLOSED_CANONICAL=NO
WAIVER_TAKEN=NO
```
