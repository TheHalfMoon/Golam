# Contract: Worker Supervision

This contract is primarily implemented in Spec 008 but is defined now so earlier session/capability design remains compatible.

A worker lifecycle is `CREATED -> RUNNING -> WAITING|COMPLETED|FAILED|CANCELLED`, with crash/restart adoption represented explicitly.

## Required fields

Worker instances record definition/version, parent session/worker, goal slice, workspace/sandbox, capability lease, memory loadout, ExecutionProfile policy, budget, created/started/stopped times, checkpoint, and causal join state.

## Rules

- Worker lease is a narrowing child of its parent/user authority.
- Spawn is authorized and auditable; workers cannot spawn children beyond budget/scope.
- Workspaces/worktrees are isolated when concurrent writes could conflict.
- Cancel propagates through supervised process trees and tool calls.
- Parent completion MUST NOT silently abandon running children; join/cancel/adopt policy is explicit.
- Crashed workers resume only from canonical checkpoints/events with current leases revalidated.
- Worker output is evidence, not authority; parent/kernel independently authorizes effects.
