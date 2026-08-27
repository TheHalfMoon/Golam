# Contract — Identity, Capability Leases & Policy

## Stable request

```text
Authorize(principal, action, resource, context) -> Allow | Deny(reason)
```

Golam owns all request schemas and final decision semantics.

## Evaluation order

1. hard kernel guard;
2. authenticated/principal validity;
3. capability lease validity/scope;
4. active policy evaluation;
5. required approval freshness/scope;
6. typed protected mutation/effect admission.

A denial is monotonic.

## Policy evaluator

Cedar is the preferred candidate, subject to exact dependency qualification.

Requirements:
- only validated policy/schema bundles may activate;
- evaluator error/diagnostic/malformed entity/context => DENY;
- hard Golam denials are evaluated outside/above Cedar;
- Golam normalizes bounded entities/actions/resources/context before evaluation;
- decision evidence records active bundle ID/hash and stable matched rule/reason references;
- no evaluator can mint a Golam lease or approval.

## Lease rules

- only privileged kernel APIs mint leases;
- lease binds principal + action/resource/context scope;
- child scope MUST be a subset/intersection of parent scope;
- expiry/revocation/generation checked at use time;
- untrusted code receives no public authority constructor;
- a principal cannot issue, edit, reactivate or widen its own authority.

## Protected policy mutation

Bundle creation/activation, schema mutation, principal authority mutation and lease issuance/revocation are protected typed effects. They are evaluated under the currently active authority state and require configured step-up approval.

Initial/recovery bootstrap administration is narrowly bounded to the local owner and cannot authorize ordinary product effects.

## Verification

Property/adversarial tests cover hard-deny dominance, policy error fail-closed behavior, bundle corruption, child-widen attempts, expiry, revocation, replay/stale generation, self-grant attempts and protected-mutation bypass.
