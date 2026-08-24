# Contract: Event Ledger and Effect Transactions

## Canonical history

`SessionEvent` is append-oriented and versioned. Projection state may be rebuilt from events plus validated checkpoints. Retry/rewind/model-alternative paths create immutable session forks rather than rewriting canonical history. Cross-session causality and security audit ordering follow `ledger-replay-contract.md`.

Required event families include:
- session lifecycle and forks;
- goal/constraint changes;
- context/evidence operations;
- model requests/responses visible to the harness after secret-ingest redaction rules;
- tool/effect proposals;
- authorization/approval decisions;
- capability/lease/protected-resource changes;
- tool/effect execution and reconciliation;
- observations/verifications;
- memory candidate/promotion/governance;
- checkpoint/compaction/reset;
- worker spawn/join/cancel/adopt;
- Connect/device/control events;
- secret-use metadata;
- completion/failure/receipts.

Security-critical event families MUST use mandatory integrity chaining/authentication. Large artifacts are content-addressed rather than embedded unbounded in canonical event rows.

## Effect transaction state machine

```text
PROPOSED
  -> DENIED
  -> AUTHORIZED
       -> APPROVAL_REQUIRED -> AUTHORIZED | DENIED
       -> EXECUTING
            -> SUCCEEDED
            -> FAILED
            -> UNKNOWN_OUTCOME -> RECONCILING -> SUCCEEDED | FAILED | MANUAL_REVIEW
```

`UNKNOWN_OUTCOME` is required for crash/network ambiguity after an external side effect may have occurred.

## Execution semantics

- `READ_ONLY`: replay allowed subject to freshness/privacy rules.
- `IDEMPOTENT_AT_LEAST_ONCE`: retry permitted only with stable idempotency semantics.
- `AT_MOST_ONCE`: never blind-retry after ambiguity; reconcile first.
- `COMPENSATABLE`: execution contract defines compensation and its own authorization/effects.
- `IRREVERSIBLE`: strongest approval/preauthorization and reconciliation requirements.

## Handler contract

Every effect family implements `effect-handler-contract.md`: semantics declaration, idempotency derivation where applicable, durable intent-before-dispatch, `execute`, safe/read-only `reconcile`, timeout/ambiguity policy, compensation and evidence.

The effect intent MUST be fsync-persistent before external execution. Dependent effects MUST remain blocked while a prerequisite is UNKNOWN_OUTCOME. MANUAL_REVIEW is a first-class user-visible/auditable state.

## Security invariants

- authorization decision occurs before effect execution;
- approval freshness is checked at execution;
- denied effect cannot be reclassified downstream as allowed;
- executors accept a current kernel-issued capability/authorization token, not raw model assertion;
- effect authorization context carries relevant taint/provenance;
- protected-resource mutations are effects, not generic file writes;
- receipts reference the exact effect record, integrity chain and verification result;
- secret values are excluded/redacted from events/receipts; secret use is represented by handle metadata only.

## Required verification

Spec 002 fault-injects crashes at every intent/dispatch/remote-accept/ack/journal boundary and proves replay/reconciliation behavior for every execution-semantics class.
