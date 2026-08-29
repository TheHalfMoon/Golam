# T003-072 Sandbox Enforcement Qualification

## Scope

T003-072 enforces the bounded rights represented by a qualified `SandboxLaunchPlan` without claiming operating-system containment or launching a process.

## Exact qualification

- Qualified implementation head: `241f36a6fbbaad93d46aed1970353a9270b05431`
- Qualified tree: `37d65e8dc72f0c9ed4935ac997d2de06e653b101`
- Official CI: #565 / run `33237725527`
- Windows: SUCCESS
- macOS: SUCCESS
- Ubuntu: SUCCESS

The official run completed `fmt`, `clippy`, workspace tests, property qualification, bounded fuzz smoke, authenticated IPC qualification, adversarial authority qualification, and the platform-appropriate external strict-local observation on the exact qualified head.

## Security properties

- Launch rights are resolved only as a subset of the already qualified launch plan.
- The environment starts cleared and only explicitly allowlisted variable names may be requested.
- Filesystem read/write roots cannot widen beyond profile bounds.
- External network rights require the pre-existing non-strict permit-bound plan; strict-local remains dominant.
- Spawn rights can only narrow the profile rule.
- CPU, memory, time, and output limits resolve to the stricter bound.
- Device, IPC, and inherited-handle requests must be explicitly declared.
- The descriptor carries no ambient authority inheritance.
- The descriptor does not claim OS/platform containment.
- No Golam-managed native child process was launched by this task.

```text
T003_072=PASS
ENVIRONMENT_CLEARED_BY_CONTRACT=YES
AMBIENT_ENV_INHERITANCE=NO
FS_RIGHTS_WIDENING_POSSIBLE=NO
NETWORK_RIGHTS_WIDENING_POSSIBLE=NO
SPAWN_RIGHTS_WIDENING_POSSIBLE=NO
DEVICE_IPC_HANDLE_WIDENING_POSSIBLE=NO
PLATFORM_CONTAINMENT_CLAIMED=NO
NETWORK_CAPABLE_MANAGED_CHILD_LAUNCHED=NO
NEXT_TASK=T003-073
```
