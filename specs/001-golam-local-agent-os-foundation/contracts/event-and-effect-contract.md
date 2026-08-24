# Contract: Event Ledger and Effect Transactions

## Canonical history

`SessionEvent` is append-oriented and versioned. Projection state may be rebuilt from events plus validated checkpoints.

Required event families include:
- session lifecycle;
- goal/constraint changes;
- context/evidence operations;
- model requests/responses visible to the harness;
- tool/effect proposals;
- authorization/approval decisions;
- tool/effect execution;
- observations/verifications;
- memory candidate/promotion;
- checkpoint/compaction/reset;
- worker spawn/join/cancel;
- Connect/device events;
- completion/failure.

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
- `IDEMPOTENT_AT_LEAST_ONCE`: retry permitted with stable idempotency key.
- `AT_MOST_ONCE`: do not blindly retry; reconcile first.
- `COMPENSATABLE`: retry/rollback policy must define compensation.
- `IRREVERSIBLE`: explicit approval/risk controls and strongest reconciliation evidence required.

## Security invariants

- authorization decision occurs before effect execution;
- denied effect cannot be reclassified downstream as allowed;
- executors accept an authorized capability token/lease, not raw model assertion;
- receipts reference the exact effect record and verification result;
- secret values are excluded from events/receipts.
