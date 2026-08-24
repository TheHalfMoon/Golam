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
append_session_event(event_request) -> SessionEvent
append_goal_version(goal_request) -> GoalVersion
create_checkpoint(session, projection) -> Checkpoint
verify_checkpoint(checkpoint) -> Verified | Invalid
propose_effect(intent) -> EffectRecord
transition_effect(effect, expected_state, next_state, evidence) -> EffectRecord
authorize(principal, action, resource, context) -> Allow | Deny
network_egress_authorize(request) -> Allow | Deny
read_recovery_status() -> RecoveryStatus
```

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

## Compromise test

A test-only hostile adapter receives all public non-kernel APIs and filesystem access to an ordinary workspace. It must fail to:
- mint an authorization result/token;
- mutate authority DB through a generic path;
- append/forge a canonical event without KernelApi;
- modify audit chain head;
- revoke/enroll a client.
