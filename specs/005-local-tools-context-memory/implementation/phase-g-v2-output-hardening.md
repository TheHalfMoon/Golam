---
task_id: T005-072/T005-076
outcome: COMBINED_OUTPUT_HARDENING_IMPLEMENTED_PENDING_EXACT_HEAD_CI_AND_INDEPENDENT_REVIEW
recorded_on: 2026-09-05
profile: platform:linux-x86_64-landlock-v4-seccomp-v2
implementation_head_before_record: 54fe15bc18df97a480c04133f8f181acff2c3bef
source_foundry_dependency_change: false
production_native_executor_admitted: false
process_launch_enabled: false
waiver_taken: false
next_gate: T005-077 exact-head CI plus independent semantic/security review
---

# Phase G v2 combined stdout/stderr output hardening

## Finding

The v2 parent supervisor contract is a combined stdout/stderr byte budget. The first hostile v2 harness captured stdout but inherited stderr, so its original output-flood test could prove stdout enforcement only. That was insufficient evidence for the combined-channel claim.

## Repair

The qualification-only parent now captures both child stdout and child stderr. The hostile output payload contributes bytes to both channels under one `RootProcessSupervisor` instance and one `max_stdout_stderr_bytes` budget.

The test requires all of the following before success:

```text
STDOUT_CAPTURED=YES
STDERR_CAPTURED=YES
STDOUT_ACCOUNTED_IN_SHARED_BUDGET=YES
STDERR_ACCOUNTED_IN_SHARED_BUDGET=YES
STDOUT_ALONE_BELOW_LIMIT=YES
COMBINED_STDOUT_STDERR_EXCEEDS_LIMIT=YES
TERMINATION_REQUEST_DISPATCHED=YES
ACCEPTED_OUTPUT_NEVER_EXCEEDS_LIMIT=YES
ROOT_TERMINAL_RECONCILED=YES
```

The probe emits `SUPERVISOR_OUTPUT_COMBINED_STDOUT_STDERR=YES` only after the cross-channel accumulation causes the same supervisor budget to exceed its bound and termination is requested. Cancellation/termination remains non-terminal until the exact root terminal observation succeeds.

## Authority posture

This repair changes qualification evidence only. It does not admit a production profile, create a process-launch path, widen filesystem/network/device/IPC/handle authority, enable shell syntax, or admit an external runtime.

```text
V2_COMBINED_OUTPUT_HARDENING=IMPLEMENTED_PENDING_EXACT_HEAD_QUALIFICATION
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
PROCESS_LAUNCH_ENABLED=NO
WAIVER_TAKEN=NO
NEXT_GATE=FRESH_V2_EXACT_HEAD_CI_THEN_INDEPENDENT_SEMANTIC_SECURITY_REVIEW
```
