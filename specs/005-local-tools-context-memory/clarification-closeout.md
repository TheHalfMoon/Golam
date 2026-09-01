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

**Decision**: Managed human-readable Markdown is canonical durable knowledge. SQLite is canonical operational state. FTS/vector/entity/graph caches and summaries are rebuildable derivatives.

## C10 — Who writes managed memory?

**Decision**: One governed Golam memory writer. User hand-edits remain supported and are reconciled through hash/version identity rather than silently overwritten.

## C11 — Can the model promote memory by saying it is true?

**Decision**: No. Promotion requires attributable human approval or deterministic verification against a pre-registered authoritative source/rule. A candidate/model/worker cannot choose or rewrite its own verifier into authority.

`MODEL_ASSERTION != MEMORY_PROMOTION_AUTHORITY`

## C12 — What happens to secret-derived content?

**Decision**: `SECRET_DERIVED` content is rejected from canonical long-term memory. Redaction or a separately evidenced non-secret-derived representation must occur through governed flow.

## C13 — What do FORGET and REDACT mean?

**Decision**: Remove the affected active canonical content, preserve required audit/governance evidence without retaining forbidden plaintext, invalidate/rebuild every derived index/cache and state honestly that previously emitted external artifacts cannot be retroactively unseen.

## C14 — Are Mem0/Qdrant/vector databases canonical or required?

**Decision**: No. They are research/reference candidates for derivative behavior only. Initial Spec 005 must remain useful without a vector database. `DENSE_VECTOR_INDEX=DEFER_UNTIL_MEASURED_NEED`.

## C15 — Are Agent Skills executable by default?

**Decision**: No. Instruction-only `SKILL.md` packaging may precede executable scripts. Script execution requires the same qualified production containment and capability/effect governance as other untrusted native execution.

## C16 — Does MCP server capability advertisement grant authority?

**Decision**: No. MCP is an interoperability boundary. Server declarations and outputs are untrusted inputs. Golam separately maps allowed operations to current policy/capability/effect authority.

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
