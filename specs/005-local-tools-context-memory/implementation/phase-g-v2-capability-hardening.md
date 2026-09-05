---
task_id: T005-072/T005-076
outcome: CAPABILITY_HARDENING_IMPLEMENTED_PENDING_EXACT_HEAD_CI_AND_INDEPENDENT_REVIEW
recorded_on: 2026-09-05
requirement_source: specs/005-local-tools-context-memory/implementation/phase-g-production-containment-requirements.md
implementation_head_before_record: be515cb4cbc658493489df0f3a7b9e7f224e22ad
profile: platform:linux-x86_64-landlock-v4-seccomp-v2
source_foundry_dependency_change: false
production_native_executor_admitted: false
process_launch_enabled: false
waiver_taken: false
next_gate: T005-077 exact-head CI plus independent semantic/security review
---

# Phase G v2 Linux capability hardening

## Requirement

T005-070 froze the following production requirement:

```text
LINUX_CAPABILITIES_AVAILABLE_TO_PAYLOAD=EMPTY_OR_STRICTLY_BOUNDED_BY_EXPLICIT_REVIEWED_NECESSITY
```

The v2 profile has no reviewed need for Linux capabilities. `no_new_privs` is necessary but does not by itself prove that capability sets inherited by the trusted helper are empty before payload execution.

## Fail-closed implementation

Before identity revalidation, resource limits, Landlock restriction, seccomp installation or any later payload transition, v2 now reads `/proc/self/status` and requires each of the following capability sets to be canonical hexadecimal zero:

```text
CapInh=0
CapPrm=0
CapEff=0
CapAmb=0
```

A missing field, malformed hexadecimal field, nonzero field, or unreadable `/proc/self/status` rejects containment establishment. The resulting child receipt records `linux_capability_sets_empty=true`, and the parent v2 supervisor refuses a binding that does not carry the same proof.

`CapBnd` is not claimed to be zero. The profile instead requires zero inheritable/permitted/effective/ambient sets before restriction plus successful `no_new_privs`; no separate reviewed capability necessity is admitted.

Focused tests cover an all-zero synthetic status, a nonzero effective capability set and a missing required field. The hostile Linux x86_64 qualification requires the live child receipt marker:

```text
LINUX_CAPABILITY_SETS_EMPTY=YES
```

## Authority posture

This hardening does not admit the production profile and does not create process-launch authority. It adds no dependency, feature, build script, shell, network surface, external binary, local MCP runtime or executable skill path.

```text
V2_CAPABILITY_HARDENING=IMPLEMENTED_PENDING_EXACT_HEAD_QUALIFICATION
LINUX_CAP_INH_REQUIRED_ZERO=YES
LINUX_CAP_PRM_REQUIRED_ZERO=YES
LINUX_CAP_EFF_REQUIRED_ZERO=YES
LINUX_CAP_AMB_REQUIRED_ZERO=YES
NO_NEW_PRIVS_STILL_REQUIRED=YES
CAPABILITY_NECESSITY_ADMITTED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
PROCESS_LAUNCH_ENABLED=NO
WAIVER_TAKEN=NO
NEXT_GATE=FRESH_V2_EXACT_HEAD_CI_THEN_INDEPENDENT_SEMANTIC_SECURITY_REVIEW
```
