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

A separately evidenced representation may become eligible only if canonical taint rules demonstrate that it is no longer secret-derived; mere redaction language from a model is insufficient.

## 6. Single managed writer

Golam-generated managed-vault mutation flows through one governed writer:

```text
candidate/intent
-> validate scope + taint + provenance
-> validate promotion authority
-> read current canonical version
-> detect user edit/conflict
-> construct next version
-> durable temporary write
-> atomic/identity-preserving replacement
-> record operational/version evidence
-> invalidate derivatives
-> post-write verification
```

No generic filesystem tool, model, plugin, MCP server or skill may bypass this writer for Golam-generated managed memory changes.

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

`FORGET`/`REDACT` invalidates any derivative generation that may contain affected content before that generation may serve results again.

## 11. Live-state precedence

Remembered claims never outrank fresher authoritative repository/filesystem/device/external state. Retrieval must surface freshness/conflict evidence when such a conflict exists.

## 12. Durability and failure

The writer must fail closed on disk-full, partial write, crash-before-replace, crash-after-replace-before-operational-record, or operational-record-without-readable-canonical-file cases. Restart reconciliation must converge to an attributable state without silently discarding user content.

## 13. Export/backup

Human-readable canonical Markdown remains exportable within policy. Protected operational/security state is not swept into generic memory export merely because it is stored locally. Backup/restore preserves version/provenance relationships and requires post-restore derivative rebuild/verification.

## 14. Required adversarial corpus

Qualification includes:

- forged/free-form promotion approval;
- candidate-selected verifier;
- `SECRET_DERIVED` promotion attempts;
- user-edit races;
- contradictory updates;
- stale-memory vs live-state conflict;
- crash/restart at every writer durability boundary;
- disk-full/permission loss;
- FORGET/REDACT derivative resurrection;
- malicious Markdown/front-matter trying to mutate authority;
- derivative index unavailable/corrupt/rebuilt from canonical state.
