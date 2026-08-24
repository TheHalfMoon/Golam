# Contract: Memory Governance Operations

## Canonical split

Markdown remains canonical durable human knowledge. SQLite remains canonical operational state. Derived indexes are disposable/rebuildable.

## Governed operations

- `ADD`: create a new memory asset with provenance/scope/validity.
- `UPDATE`: revise a memory asset while retaining prior-version provenance.
- `SUPERSEDE`: mark an older assertion as replaced from a defined validity point; history remains inspectable.
- `CONTRADICT`: link incompatible assertions; both remain retrievable and conflict is surfaced.
- `MERGE`: create a new resolved asset referencing all source assets; sources are not silently deleted.
- `EXPIRE`: remove an item from active retrieval after its validity window while retaining governed history as policy permits.
- `FORGET`: remove the requested canonical content from active durable knowledge, write a non-content tombstone, and rebuild/erase all derived indexes/caches that contain it.
- `REDACT`: remove sensitive substrings/content from canonical sources and derived state while preserving non-secret audit metadata.

Already-emitted external artifacts/messages cannot be made unseen; receipts MUST state this limit.

## Write/concurrency model

- The memory service is the single writer for Golam-generated Markdown mutations.
- Generic filesystem-write capabilities MUST NOT bypass memory governance for managed vault paths.
- User hand-edits are allowed. External edits are detected by content hash/version and reconciled on load rather than silently overwritten.
- Conflicting concurrent worker candidates are serialized/promoted by the memory service; last-write-wins is not an acceptable semantic rule.

## Promotion

- Working/run memory auto-expires by policy.
- Promotion to project/user durable scope requires provenance and either explicit user approval or deterministic verification from an authoritative source under a registered rule.
- Live authoritative state outranks memory.
- SECRET_DERIVED content is never promotable.

## Backup and failure

The vault + SQLite state MUST support consistent local backup/restore. Disk-full/torn-write conditions fail closed: no acknowledgement of durable memory or journal mutation until persistence is verified.

## Verification gate

Tests must cover poisoning rejection, surfaced contradictions, external Markdown edits, supersession queries, and post-FORGET/REDACT absence from every derived index/model context.