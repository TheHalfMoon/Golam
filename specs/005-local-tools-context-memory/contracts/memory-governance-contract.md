# Contract — Memory Governance

**Spec**: 005 Local Tools, Context & Memory

## 1. Canonical ownership

Human-readable managed Markdown is canonical long-lived knowledge. A dedicated memory operational SQLite store is canonical operational memory state. Search indexes, embeddings, graphs, caches and summaries are rebuildable derivatives.

The existing Effect Gate journal remains separate protected authority state in the canonical authority database addressed by `AuthorityLayout::authority_db_path()` and owned by the existing ledger/effect boundary. Spec 005 MUST NOT move Kernel authorization, approval, capability, lease, Effect Gate PREPARED/terminal authority, or audit authority into the memory operational SQLite store. T005-045 freezes the exact memory operational SQLite path/schema, but it remains a separate store from the authority database and every effect-owned memory row MUST bind the exact Effect Gate `effect_id` and mutation-intent digest.

```text
MEMORY_CANDIDATE != DURABLE_MEMORY
MEMORY_INDEX != CANONICAL_MEMORY
RETRIEVAL_SCORE != MEMORY_AUTHORITY
SECRET_DERIVED != CANONICAL_LONG_TERM_MEMORY
MEMORY_OPERATIONAL_SQLITE != AUTHORITY_DATABASE
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

Promotion-authority validation is a prerequisite to enabling the managed writer for mutations that require promotion. The writer MUST NOT initially implement a direct-write path and defer authorization semantics to a later phase.

## 5. Taint and secrets

Taint survives candidate creation, synthesis and memory operations. `SECRET_DERIVED` candidates are rejected from canonical long-term memory.

Within Spec 005, `SECRET_DERIVED` provenance is monotonic and MUST NOT be cleared by redaction, summarization, transformation, deterministic verification, model claims, or any memory operation. Sanitizing text is not declassification authority.

A separately created candidate may be eligible only when its content is independently sourced from evidence whose own provenance never includes `SECRET_DERIVED`; it is a new provenance chain, not a downgraded representation of secret-derived content.

## 6. Single managed writer and Effect Gate lifecycle

Every Golam-generated managed-memory mutation, including `FORGET` and `REDACT`, is a consequential protected mutation. It MUST use an immutable `MemoryMutationIntent` whose protected bindings include, at minimum:

```text
initiating_principal
kernel_authorization_ref
promotion_authority_ref
expected_current_versions
expected_markdown_target_identity_ref
expected_markdown_content_digest
expected_markdown_version
memory_operational_store_ref
effect_id
```

`expected_markdown_target_identity_ref`, `expected_markdown_content_digest`, and `expected_markdown_version` bind the exact observed canonical Markdown state that the writer is permitted to replace. `memory_operational_store_ref` binds the exact dedicated memory-operational-SQLite store identity/schema frozen by T005-045; it is not an authority-database alias. These bindings are part of the immutable mutation-intent digest and MUST survive durable PREPARED state unchanged.

Before the first canonical Markdown or memory operational SQLite mutation, the exact intent MUST be durably persisted and committed as an authorized Effect Gate PREPARED transaction in the existing authority database. The PREPARED commit is the authorization/durability boundary: the authority-database transaction containing the exact `effect_id`, mutation-intent digest and authorization evidence MUST reach the existing durable ledger commit semantics before the writer may mutate canonical Markdown or the memory operational SQLite store. A merely allocated in-memory effect id, uncommitted transaction, or memory-SQLite row is not PREPARED evidence.

The memory operational SQLite store is intentionally separate from the authority database. Spec 005 assumes no cross-database or filesystem/database atomic transaction. The single governed writer therefore MUST bind every operational write to the exact `memory_operational_store_ref`, `effect_id`, expected versions/digests and mutation-intent digest, and terminal success MUST NOT be recorded until the authority journal, canonical Markdown and memory operational SQLite state have been read back and reconciled.

Only the single governed memory writer/handler may execute the prepared mutation. The writer lifecycle is:

```text
MemoryMutationIntent
-> validate scope + taint + provenance
-> validate current Kernel authorization + promotion authority
-> read current canonical version + target identity + expected observed Markdown digest
-> detect user edit/conflict
-> commit durable Effect Gate PREPARED intent in authority database
-> construct next canonical state
-> durable temporary write
-> immediately revalidate expected Markdown digest/version + target identity at commit time
-> conditional compare-and-replace / identity-preserving Markdown replacement
-> commit memory operational SQLite rows to the exact memory_operational_store_ref, bound to effect_id + intent digest
-> invalidate affected derivatives before they may serve
-> post-write/read-back verification across authority journal + Markdown + memory SQLite
-> record integrity-chained terminal Effect Gate outcome + verification evidence
```

The Markdown commit primitive MUST preserve the checked identity through replacement and MUST compare the commit-time observed content against the expected digest/version bound by the prepared intent. If content or identity changed after the earlier lifecycle check, or the platform cannot preserve the checked identity through commit, the writer MUST NOT replace the Markdown. It records attributable `USER_EDIT_DETECTED` or `CONFLICT` reconciliation evidence and resolves the prepared effect as rejected/failed or leaves it nonterminal when outcome evidence is ambiguous. Silent last-writer-wins replacement is forbidden.

No generic filesystem tool, model, plugin, MCP server or skill may bypass this writer for Golam-generated managed memory changes.

A crash, disconnect or other interruption that leaves completion ambiguous MUST produce or remain an `UNKNOWN_OUTCOME` effect state. Dependent managed-memory mutations remain blocked until restart reconciliation determines the exact state and records attributable reconciliation evidence. Absence of a terminal record is never interpreted as success.

Restart reconciliation MUST begin from every PREPARED/nonterminal memory effect in the authority database and compare, by exact `effect_id`, `memory_operational_store_ref`, target identity and expected version/digest bindings: (1) the authority Effect Gate journal, (2) the canonical Markdown target/content digest/version, and (3) the memory operational SQLite rows/version/effect references. No single store is sufficient proof of success. Missing/unreadable stores, disagreement, a file-without-operational-row, an operational-row-without-the expected file, or an unprovable commit boundary remains `UNKNOWN_OUTCOME`/blocked until deterministic reconciliation resolves it. Derivatives are reconciled only after canonical Markdown and memory operational SQLite agree on the effect-owned canonical cut.

Multi-store operations such as `FORGET` and `REDACT` use the same lifecycle. Canonical Markdown removal, memory operational SQLite state, derivative invalidation and any rebuild eligibility must reconcile as one effect-owned mutation outcome; partial completion cannot be silently promoted to success.

## 7. User hand-edits and Markdown authority boundary

User hand-editing of canonical Markdown is allowed. Golam records the last managed digest/version and compares it to current observed content before managed mutation **and again immediately at the conditional replacement boundary**.

If divergence is detected:

- state becomes `USER_EDIT_DETECTED` or `CONFLICT`;
- Golam MUST NOT silently overwrite the user edit;
- reconciliation preserves both the user edit and relevant managed provenance;
- the resulting canonical version is attributable.

Markdown body content and front matter are user-editable content only; they are never an authority source. Scope, taint, provenance authority class, approval, authorization, managed version identity, writer identity, promotion evidence and Effect Gate state are derived only from protected operational/ledger evidence and current Kernel decisions. The parser MUST NOT honor front-matter fields that purport to mint or change those authority-bearing properties.

If user-edited Markdown introduces a reserved authority-bearing field/key or conflicts with managed metadata that is protected outside the Markdown content, the managed parser/reconciler MUST reject that file from automatic managed mutation and place the item into `USER_EDIT_DETECTED`/`CONFLICT` reconciliation (or an equivalent explicit quarantine state) while preserving the user content for review. It MUST NOT import, normalize or silently ignore such a field in a way that changes authority.

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
initiating/creating principal identity
governed committing-writer identity
exact mutation Effect reference
creation time
```

The initiating/creating principal, committing writer and mutation Effect are separate identities. Restart/reconciliation MUST preserve all three; a recovered version cannot be relabeled as generically “system-created” merely because the writer committed it after restart.

Material content/provenance change creates a new version; history is not rewritten to make prior versions appear never to have existed, except where canonical REDACT/FORGET semantics require removal of forbidden plaintext while retaining bounded audit facts.

## 10. Derivative indexes

Every derivative generation binds implementation identity and canonical cut digest. Derivatives are discardable and rebuildable.

Startup and canonical memory access MUST remain functional when derivatives are absent or rebuilding. Dense/vector services are not baseline availability dependencies.

A derivative-dependent operation encountering a missing/corrupt generation MUST trigger a governed rebuild from canonical state or fail that derivative-dependent operation closed. It MUST NOT fail canonical Markdown/memory-operational-SQLite access merely because the derivative is absent.

`FORGET`/`REDACT` invalidates any derivative generation that may contain affected content before that generation may serve results again.

## 11. Live-state precedence

Remembered claims never outrank fresher authoritative repository/filesystem/device/external state. Retrieval must surface freshness/conflict evidence when such a conflict exists.

## 12. Durability and failure

The writer must fail closed on disk-full, partial write, crash-before-replace, crash-after-replace-before-memory-SQLite-commit, memory-SQLite-commit-without-readable/expected-canonical-file, authority-PREPARED-without-memory mutation, or any disagreement among authority journal, Markdown and memory operational SQLite. Restart reconciliation must converge to an attributable state without silently discarding user content or treating an ambiguous effect as successful.

No implementation may claim atomic commit across the authority database, filesystem Markdown and separate memory operational SQLite. Instead, the durable PREPARED authority record precedes mutation, every later store binds the same effect identity and expected state, and terminal Effect Gate success follows only after deterministic cross-store read-back/reconciliation.

## 13. Export/backup

Human-readable canonical Markdown remains exportable within policy. Protected operational/security state is not swept into generic memory export merely because it is stored locally. Backup/restore preserves version/provenance relationships and requires post-restore derivative rebuild/verification.

## 14. Required adversarial corpus

Qualification includes:

- forged/free-form promotion approval;
- writer enablement attempted before promotion-authority validation is available;
- stale/revoked Kernel authorization or approval between intent and prepare;
- candidate-selected verifier;
- `SECRET_DERIVED` promotion attempts;
- attempted taint downgrade via redaction/summarization/transformation;
- user-edit races, including an edit after the initial check but before Markdown replacement;
- stale expected Markdown digest/version;
- canonical Markdown target-identity swap between observation and commit;
- inability to preserve expected Markdown identity/digest through conditional replacement;
- wrong or stale `memory_operational_store_ref`;
- contradictory updates;
- stale-memory vs live-state conflict;
- crash/restart at every authority-PREPARED/writer/Markdown/memory-SQLite/outcome durability boundary;
- authority database PREPARED vs Markdown vs memory-SQLite disagreement and restart reconciliation;
- ambiguous completion and dependent-mutation blocking under `UNKNOWN_OUTCOME`;
- creator/writer/effect attribution preservation through restart/reconciliation;
- disk-full/permission loss;
- FORGET/REDACT partial multi-store completion and derivative resurrection;
- malicious Markdown/front-matter trying to mutate scope, taint, provenance, approval, authorization, version or Effect Gate authority;
- derivative index unavailable/corrupt/rebuilt from canonical state.
