# Contract: Effect Handler / Executor / Reconciler

## Purpose

The effect state machine is only safe when each effect implementation declares executable and recoverable semantics.

## Handler declaration

Every effect handler MUST declare:

- effect family and schema version;
- execution-semantics class: READ_ONLY | IDEMPOTENT_AT_LEAST_ONCE | AT_MOST_ONCE | COMPENSATABLE | IRREVERSIBLE;
- resource/action scope;
- stable idempotency-key derivation when supported;
- precondition validator;
- `execute` behavior;
- `reconcile` behavior;
- timeout/ambiguity policy;
- compensation behavior when applicable;
- verification/evidence requirements.

## Durable ordering

1. The complete effect intent, requester, authorization context, semantics class, and idempotency material MUST be durably committed and fsynced before external execution starts.
2. Execution may begin only with a current kernel-issued authorization/capability token.
3. Outcome/evidence is appended after execution.
4. Crash/network ambiguity enters `UNKNOWN_OUTCOME`; it MUST NOT be translated to ordinary failure.

## Reconciliation rules

- `reconcile` MUST be safe/read-only with respect to the target effect and return a definitive success/failure or remain unknown.
- AT_MOST_ONCE and IRREVERSIBLE effects MUST NOT blind-retry after ambiguity.
- Downstream effects that depend on an `UNKNOWN_OUTCOME` effect MUST remain blocked until reconciliation or explicit manual resolution.
- `MANUAL_REVIEW` is a first-class user-visible state; user resolution is itself auditable.
- Idempotency keys MUST be stable across daemon restart and replay.

## Verification gate

Spec 002 MUST fault-inject crashes at every intent/dispatch/remote-accept/ack/journal boundary and prove no duplicate effect occurs outside the declared semantics.