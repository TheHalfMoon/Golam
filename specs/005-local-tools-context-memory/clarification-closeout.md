# Clarification Closeout — Spec 005

**Status**: CLOSED_FOR_PLANNING

## C1 — Are tool calls authority?

**Decision**: No. A model/tool-call candidate selects a requested operation only. Kernel policy/capability evaluation and the Effect Gate remain authoritative.

`TOOL_CALL != AUTHORITY`

## C2 — Can generic filesystem permission reach Golam protected state?

**Decision**: No. Policy, principal, capability/lease, approvals, secrets, effect journal, integrity state, pairing, egress, skill lock and equivalent protected resources remain excluded from generic filesystem authority even when a containing directory is otherwise visible.

## C3 — What is the path-security model?

**Decision**: Authorization binds an allowed root plus resolved target identity, not a user/model-provided path string. Symlink, reparse-point, junction and alias escapes fail closed. Race-sensitive mutation must retain checked identity through commit or refuse the operation.

## C4 — Is shell/process execution immediately available?

**Decision**: No. Canonical Spec 003 proves only a test-only Linux x86_64 native executor. Production remains `native:unqualified`. Spec 005 must independently qualify a production containment profile before enabling shell/process or executable MCP/skill paths.

## C5 — Does Git bypass tool/effect governance?

**Decision**: No. Read-only Git evidence may feed context. Mutating Git operations are effects. Force/history rewrite remains prohibited under current governance.

## C6 — Does browser tooling include computer control?

**Decision**: No. Spec 005 browser scope is bounded network/document/web-tool behavior. OS window control, accessibility trees, mouse/keyboard injection, screenshots-as-control and DesktopController semantics remain Spec 006.

## C7 — Is context retrieval allowed to infer authority from relevance?

**Decision**: No. Retrieval/ranking/sufficiency are utility signals only. Source authority, taint, observation freshness and permissions remain explicit independent dimensions.

## C8 — Which context tiers are required?

**Decision**: L0 files/search/Git is required. L1 Tree-sitter/LSP is admitted only if measured tasks demonstrate material value. L2 graph/dataflow/runtime is not a baseline requirement.

## C9 — What is canonical memory?

**Decision**: Managed human-readable Markdown is canonical durable knowledge. A dedicated memory-operational SQLite store is canonical operational state and is separate from the protected Effect Gate authority database. FTS/vector/entity/graph caches and summaries are rebuildable derivatives.

## C10 — Who writes managed memory?

**Decision**: One governed Golam memory writer. User hand-edits remain supported and are reconciled rather than silently overwritten. The managed writer binds the exact observed Markdown target identity and digest/version, revalidates them immediately at commit time, and uses conditional compare-and-replace / identity-preserving replacement. Markdown body/front matter is content only; reserved authority-bearing fields enter explicit reconciliation/quarantine rather than changing authority.

## C11 — Can the model promote memory by saying it is true?

**Decision**: No. Promotion requires attributable human approval or deterministic verification against a pre-registered authoritative source/rule. A candidate/model/worker cannot choose or rewrite its own verifier into authority.

`MODEL_ASSERTION != MEMORY_PROMOTION_AUTHORITY`

## C12 — What happens to secret-derived content?

**Decision**: `SECRET_DERIVED` content is rejected from canonical long-term memory. Within Spec 005 that provenance is monotonic: redaction, summarization, transformation, deterministic verification or model claims cannot clear or downgrade it. Only a separately created candidate whose independently sourced provenance never includes `SECRET_DERIVED` may be considered under the normal promotion flow.

`SANITIZATION != DECLASSIFICATION_AUTHORITY`

## C13 — What do FORGET and REDACT mean?

**Decision**: They are managed-memory mutations under the same protected lifecycle as every other Golam-generated memory mutation. An immutable `MemoryMutationIntent` binds the initiating principal, current Kernel authorization, applicable approval/pre-registered verifier evidence, expected current versions, exact expected Markdown target identity and digest/version, exact dedicated memory-operational-SQLite store identity/schema, and a unique Effect identity. Those protected bindings survive PREPARED unchanged. An authorized Effect Gate PREPARED record must be durable in the separate authority database before the first canonical Markdown or memory-SQLite mutation; only the governed memory writer may execute it. Markdown replacement revalidates the exact target/digest/version at commit time and uses conditional identity-preserving replacement. Terminal success requires integrity-chained read-back/reconciliation across the authority journal, exact Markdown state and exact memory-operational store rows. Wrong-store, partial-store, crash/disconnect or otherwise ambiguous completion remains failed/reconciling or `UNKNOWN_OUTCOME` and blocks dependent managed-memory mutations until reconciliation. FORGET/REDACT then remove affected active canonical content, preserve only required non-plaintext audit/governance facts, invalidate/rebuild affected derivatives, and never claim previously emitted external artifacts were retroactively erased.

## C14 — Are Mem0/Qdrant/vector databases canonical or required?

**Decision**: No. They are research/reference candidates for derivative behavior only. Initial Spec 005 must remain useful without a vector database. `DENSE_VECTOR_INDEX=DEFER_UNTIL_MEASURED_NEED`.

## C15 — Are Agent Skills executable by default?

**Decision**: No. Instruction-only `SKILL.md` packaging may precede executable scripts. Script execution requires the same qualified production containment and capability/effect governance as other untrusted native execution. Every instruction activation or executable dispatch binds and immediately revalidates the exact reviewed package/version/content digest and reviewed local capability mapping; deprecation, revocation, replacement, unknown state or version/digest/mapping mismatch invalidates queued, prepared-but-not-dispatched, cached capability/approval and dispatch-decision state rather than replaying prior authority.

## C16 — Does MCP server capability advertisement grant authority?

**Decision**: No. MCP is an interoperability boundary. Server declarations and outputs are untrusted inputs. Golam separately maps allowed operations to current policy/capability/effect authority. Every local or remote dispatch binds and immediately revalidates the exact reviewed MCP binding identity/digest, version lock and Golam-local mapping identity/digest; deprecated, revoked, replaced, unreviewed, unknown or mismatched bindings invalidate queued/prepared calls, stale mapped descriptors, cached capability/approval material and dispatch decisions.

## C17 — Is the official MCP Rust SDK automatically admitted?

**Decision**: No. Official provenance is useful evidence but direct dependency use still requires exact Source Foundry admission for the selected release/features/dependency/network/process surface.

## C18 — What is ACP's role?

**Decision**: IDE/client interoperability only. ACP transport does not create authority and must preserve authenticated local-client enrollment and scoped capability semantics.

## C19 — Do noncanonical PRs #6–#8 become predecessors now?

**Decision**: No. They remain planning proposals. Relevant memory/product patterns may be considered as research input, but only canonical Spec 001 T050–T059 and canonical predecessors authorize Spec 005 scope.

## C20 — Are learning, workers, scheduling or computer-control features pulled into Spec 005 because research sources contain them?

**Decision**: No. Those remain later-spec scope.

## C21 — Can planning add donor code or dependencies?

**Decision**: No. Planning records exact source state, candidate mechanisms, rejections and implementation admission gates. Code/dependency admission occurs only after an exact per-source Source Foundry record reaches `ADMITTED`.

## C22 — How is strict-local preserved?

**Decision**: Strict-local is a kernel-level hard denial for all managed components. A missing local tool/index/model/MCP capability fails clearly; it never authorizes cloud, remote provider, telemetry, download or network fallback.

## C23 — How are credential-bearing network hops and redirects handled?

**Decision**: General egress permission is insufficient to disclose a credential. Before a brokered secret or sensitive authorization material is attached, the hop must use authenticated encrypted endpoint transport and the credential must be scoped to that authorized origin/endpoint and operation. Redirects, proxy transitions, origin changes or protocol changes strip sensitive headers/cookies/bodies, revalidate egress and endpoint identity, and re-broker credentials only under fresh explicit scope. Credential-bearing downgrade or unprovable endpoint identity fails closed.

`EGRESS_ALLOWED != CREDENTIAL_DISCLOSURE_AUTHORIZED`

## C24 — Can a missing derivative index block canonical memory?

**Decision**: No. Canonical Markdown/SQLite startup and ordinary canonical memory reads remain available while derivatives are absent or rebuilding. A derivative-dependent operation must trigger governed rebuild from canonical state; if the required derivative cannot be rebuilt and qualified, only that derivative-dependent operation fails closed.

`DERIVATIVE_UNAVAILABLE != CANONICAL_MEMORY_UNAVAILABLE`

## C25 — Can a protected ToolRequest change after durable prepare?

**Decision**: No. Authority-relevant request bindings are immutable after durable prepare. Retry, target/operation/precondition changes, authority-context changes, or a materially different resolution result require a new request/effect identity.

## C26 — Can the memory operational SQLite store become an authority database alias?

**Decision**: No. The dedicated memory-operational store is canonical operational memory state only. Kernel authorization, approval/capability/lease authority and Effect Gate PREPARED/terminal authority remain in the protected authority database. Every effect-owned memory row binds the exact memory-store identity/schema, Effect identity and mutation-intent digest, and terminal success requires deterministic cross-store read-back rather than assuming atomicity or authority equivalence.
