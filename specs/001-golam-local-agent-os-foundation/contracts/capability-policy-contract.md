# Contract: Identity, Capabilities and Policy

## Authorization request

Every protected action is evaluated as:

```text
Authorize(principal, action, resource, context) -> Allow | Deny(reason)
```

The policy engine candidate is Cedar, while Golam owns entity/action schemas and capability semantics.

## Required context

Where applicable:
- requesting user;
- worker/model/skill/channel/device chain;
- session/goal;
- source channel;
- locality mode;
- risk class;
- interactive vs unattended;
- current device lock/permission state;
- network destination;
- data trust labels;
- approval freshness;
- parent capability lease.

## Capability acquisition choke points

Sensitive subsystems expose a single non-bypassable acquisition boundary, including:
- filesystem write;
- shell/process execution;
- network egress;
- sandbox execution;
- credential use;
- browser control;
- desktop input/control;
- git remote write;
- external messaging;
- production/deployment changes.

## Lease rules

- leases are signed/locally authenticated;
- leases are resource- and action-scoped;
- child leases can only narrow;
- expiry/revocation is enforced host-side;
- remote-control messages are checked against the current lease on every protected message.
