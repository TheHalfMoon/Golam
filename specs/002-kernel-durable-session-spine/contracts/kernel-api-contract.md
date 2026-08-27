# Contract: Privileged Kernel API

## Boundary

Only `golam-kernel` may own/mutate protected authority state or mint authority-bearing internal tokens. All callers cross explicit typed methods. Constructors for sealed authority material are not public outside the crate.

The API must remain implementable over in-process calls or future IPC without semantic changes.

## Required operations

Conceptual interface:

```text
open_authenticated_client(context) -> ClientSession | Deny
revoke_client(client_id) -> Result
create_session(request) -> Session
fork_session(parent_session, through_seq) -> Session
append_goal_version(goal_request) -> GoalVersion
create_checkpoint(session, projection) -> Checkpoint
verify_checkpoint(checkpoint) -> Verified | Invalid
propose_effect(intent) -> EffectRecord
transition_effect(effect, expected_state, next_state, evidence) -> EffectRecord
authorize(principal, action, resource, context) -> Allow | Deny
network_egress_authorize(request) -> Allow | Deny
read_recovery_status() -> RecoveryStatus
```

### Typed canonical-event construction

`SessionEvent` append is an internal kernel/ledger primitive, not a public caller-selected `(EventKind, bytes)` mutation surface.

Spec 002 creates canonical session events through typed domain operations such as session creation, fork, goal versioning, and checkpoint creation. A domain operation that requires both a canonical event and a companion protected record MUST commit those invariant-coupled records through the owning typed path.

A client or adapter MUST NOT be able to choose reserved system event families directly. In particular, a generic public append API must not permit a caller to forge event families such as checkpoint/effect/goal lifecycle evidence without the corresponding domain record and authorization path.

A later product event family may add a dedicated typed request and event kind under its owning spec. That extension must preserve authorization, auditability, schema/version checks, canonical ordering, and integrity chaining.

## Protected resources

At minimum:
- client identity/enrollment/revocation state;
- session/event/goal canonical tables;
- effect intent/transitions/attempts;
- audit/hash-chain heads;
- DB migrations;
- runtime server epoch/authentication material;
- recovery/quarantine metadata;
- future policy/secrets/pairing roots reserved by Spec 001.

No generic file/path API can address these as ordinary resources.

## Sealed authority rules

- callers submit requests, not preconstructed `AuthorizedEffect` objects;
- internal authorization grants contain nonce/epoch/expiry or are non-cloneable scoped values where practical;
- deserializing bytes cannot directly construct privileged token types;
- stale server epoch invalidates transient authority;
- unsafe code is forbidden in Golam kernel code.

## Integrity rule

Security-critical client enrollment/revocation, authorization decisions, effect intent/transitions/attempts, recovery incidents, and security-critical canonical events MUST be protected by mandatory tamper-evident integrity chaining or an equivalently strong authenticated integrity mechanism. Missing integrity coverage is a fail-closed authority-store integrity failure.

## Compromise test

A test-only hostile adapter receives all public non-kernel APIs and filesystem access to an ordinary workspace. It must fail to:
- mint an authorization result/token;
- mutate authority DB through a generic path;
- append/forge a canonical event without the owning KernelApi domain operation;
- modify audit chain head;
- revoke/enroll a client.
