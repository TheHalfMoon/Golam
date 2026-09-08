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

## Current disposition

```text
PHASE_D_RUNTIME_WIRING_AUDIT=GAP_FOUND
PHASE_D_RECONCILIATION=IN_PROGRESS
T006_016_COMPLETE=NO_PENDING_RUNTIME_WIRING_QUALIFICATION
T006_017_COMPLETE=NO_PENDING_RUNTIME_WIRING_QUALIFICATION
T006_018_COMPLETE=NO_PENDING_RUNTIME_WIRING_QUALIFICATION
T006_019_COMPLETE=NO_PENDING_RUNTIME_WIRING_QUALIFICATION
PHASE_D_COMPLETE=NO
PHASE_E_PRODUCT_IMPLEMENTATION_START=BLOCKED_UNTIL_PHASE_D_RECONCILIATION
FINAL_SPEC006_REVIEW=NOT_PREAPPROVED
SPEC_006_IMPLEMENTATION_COMPLETE=NO
SPEC_006_CLOSED_CANONICAL=NO
WAIVER_TAKEN=NO
```
