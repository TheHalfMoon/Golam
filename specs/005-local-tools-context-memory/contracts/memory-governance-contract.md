# Contract — Memory Governance

**Spec**: 005 Local Tools, Context & Memory

## 1. Canonical ownership

Human-readable managed Markdown is canonical long-lived knowledge. SQLite is canonical operational state. Search indexes, embeddings, graphs, caches and summaries are rebuildable derivatives.

```text
MEMORY_CANDIDATE != DURABLE_MEMORY
MEMORY_INDEX != CANONICAL_MEMORY
RETRIEVAL_SCORE != MEMORY_AUTHORITY
SECRET_DERIVED != CANONICAL_LONG_TERM_MEMORY
```

## 2. Scopes

Initial durable scopes are `USER` and `PROJECT`. New scopes require later explicit governance.

## 3. Candidate boundary

Model, worker, tool, MCP, skill, imported document or observation output produces an immutable `MemoryCandidate` with:

- scope;
- proposed content reference;
- exact provenance;
- taint set;
- source authority class;
- creating principal;
- promotion requirement;
- creation time/identity.

Candidate creation is not promotion.

## 4. Promotion authority

Durable promotion requires one of:

1. attributable approval from an authenticated principal currently authorized for the target memory scope/operation; or
2. deterministic verification against an admitted, pre-registered authoritative source/rule with exact source/rule/version evidence.

Free-form text saying `remember`, `approved`, `yes`, or equivalent is content, not authority. A model/candidate/worker cannot register, select, weaken or reinterpret its own verifier into authority.

## 5. Taint and secrets

Taint survives candidate creation, synthesis and memory operations. `SECRET_DERIVED` candidates are rejected from canonical long-term memory.

Within Spec 005, `SECRET_DERIVED` provenance is monotonic and MUST NOT be cleared by redaction, summarization, transformation, deterministic verification, model claims, or any memory operation. Sanitizing text is not declassification authority.

A separately created candidate may be eligible only when its content is independently sourced from evidence whose own provenance never includes `SECRET_DERIVED`; it is a new provenance chain, not a downgraded representation of secret-derived content.

## 6. Single managed writer and Effect Gate lifecycle

Every Golam-generated managed-memory mutation, including `FORGET` and `REDACT`, is a consequential protected mutation. It MUST use an immutable `MemoryMutationIntent` bound to the initiating principal, current Kernel authorization, applicable promotion approval/pre-registered verifier evidence, expected current memory versions, and a unique effect identity.

Before the first canonical Markdown or SQLite mutation, the intent MUST be durably persisted as an authorized Effect Gate PREPARED transaction. Only the single governed memory writer/handler may execute that prepared mutation.

The writer lifecycle is:

```text
MemoryMutationIntent
-> validate scope + taint + provenance
-> validate current Kernel authorization + promotion authority
-> read current canonical version and expected preconditions
-> detect user edit/conflict
-> persist durable Effect Gate PREPARED intent
-> construct next canonical state
-> durable temporary write
-> atomic/identity-preserving Markdown replacement
-> commit/update SQLite operational/version evidence
-> invalidate affected derivatives before they may serve
-> post-write/read-back verification
-> record integrity-chained terminal Effect Gate outcome + verification evidence
```

No generic filesystem tool, model, plugin, MCP server or skill may bypass this writer for Golam-generated managed memory changes.

A crash, disconnect or other interruption that leaves completion ambiguous MUST produce or remain an `UNKNOWN_OUTCOME` effect state. Dependent managed-memory mutations remain blocked until restart reconciliation determines the exact Markdown/SQLite state and records attributable reconciliation evidence. Absence of a terminal record is never interpreted as success.

Multi-store operations such as `FORGET` and `REDACT` use the same lifecycle. Canonical Markdown removal, SQLite operational/version state, derivative invalidation and any rebuild eligibility must reconcile as one effect-owned mutation outcome; partial completion cannot be silently promoted to success.

## 7. User hand-edits

User hand-editing of canonical Markdown is allowed. Golam records the last managed digest/version and compares it to current observed content before managed mutation.

If divergence is detected:

- state becomes `USER_EDIT_DETECTED` or `CONFLICT`;
- Golam MUST NOT silently overwrite the user edit;
- reconciliation preserves both the user edit and relevant managed provenance;
- the resulting canonical version is attributable.

## 8. Operations

### `ADD`
Create a new logical memory item with scope, provenance and promotion evidence.

### `UPDATE`
Create a new immutable version of an existing item while retaining predecessor relation.

### `SUPERSEDE`
Mark prior knowledge inactive because a new active version replaces it; historical lineage remains auditable.

### `CONTRADICT`
Retain conflicting propositions and surface the conflict. No silent last-write-wins truth collapse.

### `MERGE`
Create a new attributed synthesis while preserving source item/version lineage and conflicts.

### `EXPIRE`
Mark content inactive under an explicit time/policy rule; expiry is not deletion.

### `FORGET`
Remove affected content from active canonical knowledge where policy allows, remove/rewrite affected canonical Markdown, invalidate derivatives and rebuild enabled indexes. Preserve only necessary non-plaintext governance/audit evidence.

### `REDACT`
Remove prohibited sensitive content from active canonical knowledge and derivatives while retaining necessary non-plaintext governance facts. Redaction MUST NOT falsely claim that already-emitted external artifacts have been erased.

## 9. Version identity

Every managed version binds:

```text
item id
immutable version id
scope
canonical Markdown reference
content digest
provenance refs
taint set
active/conflict/expired/redacted state
predecessor version refs
promotion evidence
creation identity/time
```

Material content/provenance change creates a new version; history is not rewritten to make prior versions appear never to have existed, except where canonical REDACT/FORGET semantics require removal of forbidden plaintext while retaining bounded audit facts.

## 10. Derivative indexes

Every derivative generation binds implementation identity and canonical cut digest. Derivatives are discardable and rebuildable.

Startup and canonical memory access MUST remain functional when derivatives are absent or rebuilding. Dense/vector services are not baseline availability dependencies.

A derivative-dependent operation encountering a missing/corrupt generation MUST trigger a governed rebuild from canonical state or fail that derivative-dependent operation closed. It MUST NOT fail canonical Markdown/SQLite memory access merely because the derivative is absent.

`FORGET`/`REDACT` invalidates any derivative generation that may contain affected content before that generation may serve results again.

## 11. Live-state precedence

Remembered claims never outrank fresher authoritative repository/filesystem/device/external state. Retrieval must surface freshness/conflict evidence when such a conflict exists.

## 12. Durability and failure

The writer must fail closed on disk-full, partial write, crash-before-replace, crash-after-replace-before-operational-record, or operational-record-without-readable-canonical-file cases. Restart reconciliation must converge to an attributable state without silently discarding user content or treating an ambiguous effect as successful.

## 13. Export/backup

Human-readable canonical Markdown remains exportable within policy. Protected operational/security state is not swept into generic memory export merely because it is stored locally. Backup/restore preserves version/provenance relationships and requires post-restore derivative rebuild/verification.

## 14. Required adversarial corpus

Qualification includes:

- forged/free-form promotion approval;
- stale/revoked Kernel authorization or approval between intent and prepare;
- candidate-selected verifier;
- `SECRET_DERIVED` promotion attempts;
- attempted taint downgrade via redaction/summarization/transformation;
- user-edit races;
- contradictory updates;
- stale-memory vs live-state conflict;
- crash/restart at every PREPARED/writer/Markdown/SQLite/outcome durability boundary;
- ambiguous completion and dependent-mutation blocking under `UNKNOWN_OUTCOME`;
- disk-full/permission loss;
- FORGET/REDACT partial multi-store completion and derivative resurrection;
- malicious Markdown/front-matter trying to mutate authority;
- derivative index unavailable/corrupt/rebuilt from canonical state.
