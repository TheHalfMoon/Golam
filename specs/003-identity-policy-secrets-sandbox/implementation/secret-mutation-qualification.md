# T003-052 — Protected Secret Mutation Qualification

**Task**: `T003-052`  
**Decision**: `PASS`  
**Qualified implementation head**: `b70689c1e4836f1540541f45d66cdd5a3f514dec`  
**CI**: run number `434`, run ID `33163396509`  
**CI conclusion**: `SUCCESS` on Windows, macOS, and Ubuntu

## Qualified boundary

T003-052 implements Golam-owned protected secret create, rotate/version, and revoke transitions on top of the T003-050 opaque secret interfaces and T003-051 encrypted vault/key-protection core.

Qualified properties:

- secret mutation preparation remains crate-internal and plaintext-bearing request types are not exposed as a generic public read/write surface;
- create, rotate, and revoke are typed elevated mutations bound to an exact durable `allow` authorization decision, exact authorized at-most-once effect, and exact unconsumed ONCE approval;
- approval binding additionally requires the current parent authorization decision;
- every mutation starts with authority integrity and `authority-security` verification and fails closed on ambiguity or stale authority;
- create encrypts secret bytes before inserting any `secret_versions` row;
- durable `secret_versions` store ciphertext, algorithm/nonce metadata, and associated-data hash, never plaintext;
- create commits `SecretRecord`, initial immutable `SecretVersion`, authenticated security snapshots, and ONCE approval consumption atomically in one `BEGIN IMMEDIATE` transaction;
- rotation uses compare-and-set current-version evidence, retires the prior version without rewriting its historical ciphertext, creates a new immutable encrypted version, advances `current_version`, records security snapshots, and consumes approval atomically;
- revocation is monotonic: it changes an active record to `revoked` with a durable timestamp without deleting or rewriting historical versions;
- stale current-version evidence, duplicate/replayed creation, already-retired versions, already-revoked secrets, stale authority, effect mismatch, approval mismatch/reuse, integrity failure, key-protection/vault failure, or transaction failure all fail closed;
- nonce reuse remains a hard failure because every mutation reconstructs the vault nonce registry from persisted algorithm metadata before sealing;
- the secret request intent uses a per-request random keyed commitment rather than persisting a plain unkeyed hash of the secret value;
- deterministic canary values are used in qualification tests; no real credentials are used;
- the test-only fake `KeyProtector` is not production-selectable.

## Atomicity and adversarial evidence

Focused tests prove:

1. create -> rotate -> revoke forms authenticated protected transitions, keeps ciphertext free of the deterministic canary bytes, retires the prior version, advances the current version, and preserves authority integrity;
2. unavailable key protection aborts creation without inserting secret rows or consuming the approval;
3. stale rotation version evidence fails before approval consumption and leaves the current version unchanged.

These tests execute inside the ordinary workspace test gate and passed on Windows, macOS, and Ubuntu at the qualified head.

## Exact-head CI evidence

CI #434 / run `33163396509` executed against exact head `b70689c1e4836f1540541f45d66cdd5a3f514dec` and completed with overall conclusion `success`.

Platform jobs:

- `rust-windows-latest`: SUCCESS
- `rust-macos-latest`: SUCCESS
- `rust-ubuntu-latest`: SUCCESS

The run includes pinned formatting, Clippy, workspace tests, property qualification, bounded fuzz smoke, IPC qualification, authenticated daemon IPC, adversarial authority qualification, daemon build, and strict-local external observation as applicable per platform.

## Scope discipline

T003-052 does not implement or claim:

- `BrokerSecretUse` authorization or credential application;
- plaintext return to callers;
- argv/environment/process injection fallback;
- explicit user-designated secret-entry UX;
- generic network permission derived from a secret binding;
- real-secret qualification.

`SECRET_BROKER_BINDING != GENERAL_NETWORK_PERMISSION` remains invariant.

## Next task

`T003-053` is the next canonical eligible task: implement `BrokerSecretUse` authorization around opaque handle, purpose, destination/process, lease/policy/approval, and locality state while keeping plaintext inside the trusted broker boundary whenever possible.
