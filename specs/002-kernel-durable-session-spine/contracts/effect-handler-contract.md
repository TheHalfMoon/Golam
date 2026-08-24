# Contract: Effect Handler / Reconciler

## Handler declaration

Every handler exposes immutable metadata:

- handler ID/version;
- supported action/resource types;
- execution semantics: READ_ONLY | IDEMPOTENT_AT_LEAST_ONCE | AT_MOST_ONCE | COMPENSATABLE | IRREVERSIBLE;
- idempotency support;
- reconciliation support/class;
- execution timeout;
- reconciliation timeout;
- whether manual review may be required.

## Lifecycle

1. request proposed;
2. kernel authorizes/denies;
3. durable effect intent + authorized transition commits;
4. handler attempt record commits;
5. execute is dispatched;
6. definitive response -> durable success/failure transition;
7. ambiguous crash/network result -> `UNKNOWN_OUTCOME`;
8. reconcile is read-only with respect to the external target and returns definitive success/failure/unknown;
9. unknown may remain manual-review; dependent effects stay blocked.

## Idempotency

`IDEMPOTENT_AT_LEAST_ONCE` uses a stable key derived before execution and persisted with intent.

`AT_MOST_ONCE` and `IRREVERSIBLE` never redispatch after an ambiguous attempt unless reconciliation proves the prior attempt did not happen and policy explicitly permits another attempt.

## Simulators in Spec 002

Required deterministic fake handlers:
- pure read;
- idempotent remote-like write with key lookup;
- at-most-once write with queryable status;
- compensatable write with compensation record;
- irreversible write with intentionally ambiguous ack path.

Fault injector can crash before/after every durable transition and simulated remote boundary.

## Dependency blocking

An effect with dependencies cannot enter EXECUTING if any dependency is not definitively SUCCEEDED or an explicitly permitted alternate terminal state.
