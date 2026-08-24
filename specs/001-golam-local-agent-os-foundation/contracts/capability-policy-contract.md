# Contract: Identity, Capabilities and Policy

## Authorization request

Every protected action is evaluated as:

```text
Authorize(principal, action, resource, context) -> Allow | Deny(reason)
```

Spec 002 MUST implement this interface with a deny-by-default bootstrap evaluator before Cedar integration. Cedar is the policy-engine candidate for Spec 003; Golam owns entity/action schemas, capability semantics, protected-resource classes, denial semantics and approval behavior.

## Required context

Where applicable:
- requesting user and authenticated local client/device;
- worker/model/skill/channel/device causal chain;
- session/goal;
- source channel;
- locality mode;
- risk class;
- interactive vs unattended;
- current device lock/permission state;
- network destination;
- data/artifact taint labels;
- approval class/freshness;
- parent capability lease;
- protected-resource classification.

## Capability acquisition choke points

Sensitive subsystems expose a single non-bypassable acquisition boundary, including:
- filesystem write;
- managed-memory write;
- shell/process execution;
- network egress;
- sandbox execution;
- credential use;
- browser control;
- clipboard read/write;
- camera/microphone;
- desktop input/control;
- git remote write;
- external messaging;
- production/deployment changes;
- policy/principal/lease/approval/schedule/skill-lock changes.

## Protected resources

Policy store, principal/lease registry, approvals, secret vault, effect/idempotency journal, audit chain, GolamConnect pairing/revocation state, strict-local egress policy, skill admission/lockfile and schedule authority MUST NOT be writable through generic filesystem/tool capabilities.

Mutating protected resources is itself an elevated effect and requires current policy plus the relevant step-up approval.

## Lease rules

- leases are signed/locally authenticated and kernel-minted;
- leases are resource- and action-scoped;
- child leases can only narrow;
- expiry/revocation is enforced at action execution;
- remote-control messages are checked against current lease generation on every protected message;
- a model/worker/skill/channel cannot create, edit or widen its own lease.

## Approval rules

Approval classes and freshness are defined in `approval-step-up-contract.md`. Denial is monotonic. IRREVERSIBLE unattended work requires explicit bounded RUN_PREAUTHORIZATION.
