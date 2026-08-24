# Contract: Ledger, Forking, Checkpoints, Integrity and Artifact Lifecycle

## Immutable history and forks

A rewind/retry/model-alternative MUST create a new session branch referencing an immutable parent prefix/checkpoint. Canonical prior history is never rewritten. Fork metadata records parent session, parent event sequence/hash, reason, and initiating principal.

## Ordering and causality

- Each session keeps a monotonic sequence.
- Security/audit events additionally receive a kernel-assigned global monotonic audit order (or an equivalently total, stable ordering proven by implementation).
- Worker/session events carry explicit causal parent references where cross-session causality matters.

## Integrity

Integrity chaining is mandatory, not optional, for effect intents/outcomes, authorization/approval decisions, capability/lease changes, Connect pairing/control events, secret-use metadata, memory promotions/governance operations, and receipt records.

Security-critical events include previous-chain hash and content hash; signed checkpoints bind the current chain head where signing material is available.

## Artifacts

Large screenshots, DOM/accessibility snapshots, files, model payload archives, and similar blobs are referenced by content hash rather than embedded unbounded in the ledger.

Artifact metadata includes content hash, type, sensitivity/taint, retention class, encryption state, creating event, and last required replay/checkpoint reference.

Retention/GC MUST preserve anything required by current replay, unresolved effects, active audit requirements, or explicit user retention. Content deletion/redaction updates indexes and leaves non-content tombstone metadata when required for audit.

## Checkpoints

A checkpoint is a projection acceleration artifact, never the canonical source. It records covered event prefix/hash, schema versions, projection state hash, and referenced artifacts. Invalid/corrupt checkpoints are rejected and recovery falls back to the last valid checkpoint or canonical replay.

## Storage failure

Disk-full or failed fsync during canonical append fails closed. Golam MUST NOT report a durable state transition that was not durably persisted.

## Verification gate

Property/fault tests cover forks, concurrent workers, stable global audit ordering, checkpoint corruption, replay equivalence, blob GC safety, and disk exhaustion.