# Contract — Protected Resource Mutation

## Protected classes

Spec 003 treats these as kernel-owned protected authority state:
- policy bundles/schema/active pointer;
- principal authority records;
- capability leases and revocations;
- approvals and consumption records;
- secret vault, versions, redaction/key metadata and handle registry;
- taint verifier/sanitizer registry and downgrade attestations;
- strict-local/egress permit authority;
- sandbox profile definitions/admission records;
- inherited Spec 002 effect/idempotency/client/recovery/audit authority.

## Mutation path

No generic filesystem, shell, plugin, worker, skill, MCP, browser, adapter or client write can mutate protected state.

Every mutation must:
1. enter a typed KernelApi operation;
2. authenticate/identify the principal;
3. pass hard guards, lease, current policy and required approval;
4. create/use a typed elevated effect where consequential;
5. commit the protected row plus mandatory security-integrity evidence atomically where coupled;
6. fail closed on storage/integrity ambiguity.

## Policy self-change rule

A candidate new policy cannot authorize its own activation. Activation is evaluated under the currently active authority state plus required approval.

## Integrity

Every new protected source record receives complete `authority-security` chain coverage or an equivalently strong authenticated integrity design. Missing/tampered coverage is authority corruption, not a projection to rebuild silently.

## Verification

Hostile-adapter and direct-store tests prove unprivileged code cannot mint leases, activate policy, consume/create approvals, read vault plaintext, write verifier rules, grant egress, weaken profiles or forge security evidence.
