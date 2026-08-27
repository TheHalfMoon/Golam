# Contract — Approval & Step-Up Authorization

Approvals narrow and temporarily activate authority already permitted by hard guards, principal identity, lease scope and policy. They do not create unconstrained authority.

## Classes

- `ONCE`: one exact protected operation/effect.
- `SESSION_SCOPED`: bounded actions/resources for one session until expiry/revocation.
- `TIME_BOXED`: bounded scope until a short expiry.
- `OPERATION_PATTERN`: narrow operation/resource pattern with quantitative limits.
- `RUN_PREAUTHORIZATION`: explicit bounded scope for unattended work.

## Binding

Every approval binds:
- approver principal/device;
- class;
- action/resource/effect or operation-pattern scope;
- risk class;
- material taint/provenance digest;
- parent authorization decision/context;
- issue/expiry time;
- usage/quantity limits.

## Freshness

Approval validity is rechecked immediately before the protected action/effect executes. Expired, revoked, context-mismatched or scope-mismatched approvals deny.

## ONCE consumption

ONCE approvals use durable atomic reservation/consumption. Concurrent callers cannot reserve the same one-shot authority for two successful operations. Crash recovery either proves the consumption belongs to the same operation/effect or fails closed; it does not create a second allowance.

## Unattended irreversible work

Requires explicit bounded RUN_PREAUTHORIZATION. There is no global "always allow irreversible" state.

## Monotonic denial

Approval cannot override strict-local denial, protected-resource hard denial, invalid/revoked lease, policy forbid/no-permit or other upstream safety denial.

## Verification

Tests cover expiry, revocation, replay, double-submit, process loss between reserve/consume, batch/pattern overreach, context/taint mismatch and unattended irreversible requests.
