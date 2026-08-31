# T003-064 — Strict-Local Managed Process-Tree Qualification

**Status**: PASS  
**Qualified implementation head**: `d77cd2b8e78a41153085c279fea698bb794d2d4e`  
**Qualified tree**: `52d7e1cade4586393f3f6902e235db91d314e5c6`  
**Official CI**: #538 / run `33234807292` — SUCCESS on Windows, macOS, and Ubuntu

## Qualified boundary

T003-064 upgrades the repository's independent strict-local external-network observer from daemon-PID-only inspection to complete live process-tree inspection rooted at `golamd`.

On Unix runners the observer:

- snapshots PID/PPID relationships with `ps`;
- computes the transitive descendant closure rooted at the live daemon;
- checks every observed PID for Internet sockets with `lsof`;
- repeats the process-tree snapshot and socket check throughout daemon startup and steady-state observation;
- terminates the observed process tree during qualification cleanup;
- runs a deterministic synthetic PID/PPID traversal self-test that includes child and grandchild relationships before observing `golamd`.

On Windows runners the observer:

- snapshots `Win32_Process` PID/ParentProcessId relationships;
- computes the transitive descendant closure rooted at the live daemon;
- checks `netstat -ano` ownership against every observed process-tree PID;
- repeats the graph and socket check throughout daemon startup and steady-state observation;
- terminates the observed process tree during qualification cleanup;
- runs a deterministic synthetic process-graph self-test that includes child and grandchild relationships before observing `golamd`.

Both observers still require explicit local IPC readiness evidence while requiring zero Internet sockets for every live process owned by the observed Golam tree.

## Strict-local dominance

The existing T003-060 hard guard remains unchanged and continues to deny external `network.egress` before downstream policy, lease, approval or permit evaluation. The complete workspace/adversarial CI suite passed at the T003-064 exact head, so the observer upgrade did not regress hard-guard authority semantics.

A permit cannot convert strict-local external networking into allowed networking. Any future live Golam-managed descendant is included by the same process-tree closure rather than being invisible because it owns a different PID.

## Qualification honesty

T003-064 qualifies the independent descendant-capturing observer and the current daemon process tree. It does **not** claim that a native managed-child executor already exists. T003-074 remains the first task permitted to add the minimum native untrusted-process test executor/profile; its qualification must reuse this process-tree observer so an actual managed descendant's sockets are independently captured.

No external network operation, donor code, Wasmtime dependency, or real secret was added by this task.

```text
T003_064=PASS
MANAGED_PROCESS_TREE_OBSERVER=QUALIFIED
STRICT_LOCAL_DOMINANCE_PRESERVED=YES
NETWORK_CAPABLE_MANAGED_CHILD_LAUNCHED=NO
REAL_SECRETS_USED=NO
DONOR_CODE_ADMITTED=NO
PHASE_G_COMPLETE=YES
NEXT_TASK=T003-070
```
