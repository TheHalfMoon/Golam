# Contract: Bootstrap Authorization Interface

Spec 002 defines the stable call shape that Spec 003 will back with Cedar.

```text
Authorize(principal, action, resource, context)
  -> Allow(decision_id)
  | Deny(decision_id, reason_code)
```

## Bootstrap principals

- local owner;
- enrolled local client acting for owner;
- internal kernel service identities;
- deterministic test principals.

No worker/model/channel/device federation yet.

## Default

Deny unless an explicit bootstrap rule allows the exact Spec 002 action/resource.

Allowed classes are narrowly limited to:
- authenticated session read/create/fork/event/goal operations;
- checkpoint/replay diagnostics;
- client enrollment/revocation under explicit local bootstrap flow;
- synthetic effect simulator operations;
- recovery status reads;
- test-only fault injection when test build/config explicitly enables it.

Network egress is denied by default.

## Protected mutations

Client enrollment/revocation, recovery/migration, effect transition and authority-state changes always pass this interface; ordinary storage helpers cannot bypass it.

## Monotonic denial

A Deny cannot be converted to Allow by an effect handler, IPC adapter or daemon composition layer.

## Forward compatibility

Spec 003 may replace policy evaluation internals but cannot weaken call semantics, auditability or deny-by-default behavior without constitutional amendment.
