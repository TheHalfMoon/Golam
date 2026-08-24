# Contract: Storage, Checkpoint and Recovery

## Authority DB

SQLite is canonical operational/authority storage for Spec 002.

Required startup checks:
- data directory ownership/permissions;
- schema version/migration state;
- SQLite integrity/quick check;
- event/audit hash-chain verification;
- incomplete effect state scan;
- checkpoint artifact hash verification.

## Fail-closed rule

If authority DB integrity or canonical hash chain cannot be trusted, `golamd` MUST NOT enter privileged serving mode. It may expose a minimal local recovery/diagnostic command path that cannot mutate ordinary authority except through explicit recovery procedures.

Do not silently reset or best-effort salvage authority rows.

## Non-authority artifact writes

Use content-addressed immutable files. Write to temporary file, fsync/close as required by failure model, verify hash/size, then atomically rename/install. Orphan temp files may be removed at startup.

## Checkpoint failure

A corrupt/missing checkpoint is non-fatal if canonical history remains healthy. Mark invalid and replay from previous valid checkpoint/genesis.

## SQLite pragmas

Exact WAL/synchronous/busy-timeout settings are implementation decisions validated by crash/reboot tests. Security/effect-intent commits cannot rely on a weaker durability mode that acknowledges data before the supported failure model considers it durable.

## Disk-full behavior

- failure before durable intent commit -> no dispatch;
- failure recording post-dispatch outcome -> effect becomes/reconstructs as ambiguous and must reconcile;
- implementation evaluates a preallocated recovery reserve to regain minimal journal capacity under disk-full; tests must prove it before treating it as a guarantee.

## Migration

Forward-only embedded migrations with version checks. Before any migration capable of destructive rewrite, create/verify a local protected backup or equivalent rollback evidence. Unknown future schema -> fail closed, never downgrade automatically.
