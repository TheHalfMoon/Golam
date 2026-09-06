# Spec 005 Phase E — Managed Memory Qualification Record

Status: `IMPLEMENTED_PENDING_FINAL_EXACT_HEAD_CI`

This record closes the implementation/qualification accounting for T005-045 through T005-058 without widening authority. It is an implementation evidence map, not a waiver and not final merge qualification.

## Boundaries

- Canonical editable memory remains managed Markdown.
- SQLite remains operational/index/recovery state and does not silently replace canonical Markdown.
- Promotion and control authority remain explicit, persisted, revalidated, and non-self-issued.
- The managed-memory writer remains the only admitted memory mutation path.
- All memory mutations remain bound to the Effect Gate, exact mutation intent, exact store, exact target/content/version expectations, and exact authority evidence.
- `UNKNOWN_OUTCOME` and blocked reconciliation remain fail-closed and block dependent managed-memory mutation.
- Derivative indexes remain discardable/rebuildable projections and cannot become canonical memory.
- Repository/filesystem observations that are both live and authoritative for the same subject/scope outrank conflicting managed-memory claims before score-based context ranking; ambiguous live state fails closed.

## T005-045 through T005-057 implementation evidence

| Task | Qualification surface |
| --- | --- |
| T005-045 | `golam-core::memory_markdown`, `golam-core::memory_storage`, `golam-ledger::memory_operational`, `golam-ledger::memory_restart`, and `golamd::memory_commit` bind editable Markdown, operational state, recovery, and exact target identity. |
| T005-046 | `memory_promotion_authority`, `memory_promotion_gate`, and `memory_promotion_operational` implement human-approved and deterministic-verifier promotion authority with bounded revalidation. |
| T005-047 | Promotion authority/evidence is persisted independently in authority/operational evidence surfaces and integrity-bound before canonical promotion is admitted. |
| T005-048 | `memory_writer_authority`, `managed_memory_writer`, and Effect Store transitions bind mutation intent, initiating principal, exact store, item/version, target identity/content/version, writer identity, and effect identity before execution. |
| T005-049 | `managed_memory_writer`, `memory_writer_readback`, `memory_evidence`, and restart reconciliation re-read durable state before success and preserve ambiguous completion as blocking `UNKNOWN_OUTCOME`. |
| T005-050 | `ManagedMemoryWriter` is the admitted writer boundary; qualified promotion/control records are opaque and are revalidated immediately before PREPARED state is persisted. |
| T005-051 | `memory_restart` plus `memory_restart_adversarial` scans and reconciles cross-store state after restart and fails closed on ambiguous cuts. |
| T005-052 | `golamd::memory_commit` conditionally replaces only the exact observed Markdown file, preserves user edits, and quarantines authority-bearing user content instead of silently overwriting it. |
| T005-053 | Managed memory contracts cover ADD/UPDATE/SUPERSEDE/CONTRADICT/MERGE/EXPIRE with predecessor/conflict/version attribution. |
| T005-054 | FORGET/REDACT use the controlled mutation lifecycle; canonical replacement removes prior plaintext and derivative generation indexes terminal state rather than forgotten plaintext. Partial/ambiguous completion remains blocking. |
| T005-055 | `memory_derivative` generates a deterministic bounded text/metadata projection from the canonical cut. |
| T005-056 | Missing/corrupt/stale derivative state is discarded/rebuilt from canonical memory; rebuild is denied while canonical state is ambiguous. |
| T005-057 | `context_authority::compile_l0_context_with_live_precedence` suppresses conflicting managed memory before ranking when a fresh local filesystem/Git observation exists for the same subject/scope, emits deterministic digest-bound conflict evidence, and fails closed on conflicting live observations or an unproven freshness ordering. |

## T005-058 adversarial qualification matrix

The following cases are covered by repository-owned tests or fail-closed type/runtime boundaries and are exercised by the workspace test suite.

| Required adversarial case | Evidence / expected result |
| --- | --- |
| memory poisoning / stale memory | `context_authority` tests prove even higher-scored conflicting memory is suppressed by live filesystem/Git evidence and conflict evidence is emitted. |
| forged promotion | `memory_promotion_authority_adversarial` corrupts persisted authority security material and requires rejection before reuse. |
| stale/revoked promotion | Promotion authority tests reject stale decisions; adversarial tests revoke a previously validated approval and require revalidation failure. |
| writer-before-validator enablement | Qualified promotion/control records are opaque; writer preparation revalidates the qualified record before PREPARED persistence and cannot construct authority from caller-provided fields. |
| secret-derived / taint downgrade | Memory admission rejects `SECRET_DERIVED`; taint derivation is monotonic and tests prove source/introduced labels cannot be removed. |
| creator / writer / effect attribution | `MemoryVersion` validation and writer/operational persistence bind `created_by_principal`, the fixed managed writer identity, and `mutation_effect_ref`; mismatched effect/store bindings are rejected. |
| stale Markdown digest/version | Pre-commit/finalization validation rejects content-digest mismatch; operational state enforces the exact expected current version and reports stale-current-version failure. |
| target-identity swap | Restart adversarial tests and conditional commit tests reject target/path swaps and never return substituted content as committed memory. |
| edit-after-check-before-replace | `in_place_user_edit_after_check_is_preserved_not_overwritten` and path-swap tests require user bytes to survive and mutation to fail closed. |
| conditional-replace failure | Conditional commit replaces only the exact observed file; mismatched/replaced state is preserved as conflict/ambiguous outcome rather than success. |
| malicious authority-bearing front matter | Managed Markdown validation/parser and memory commit tests reject generated authority-bearing keys and quarantine user-authored authority-bearing Markdown. |
| wrong memory-store binding | Restart adversarial and operational/writer binding checks reject a mismatched memory store. |
| authority-journal / Markdown / memory-SQLite split-store cuts | Restart adversarial split-store digest and stale-scan tests require blocked reconciliation rather than synthesized success. |
| file without row | Restart adversarial test proves the file is never synthesized as canonical memory. |
| row without file | Restart adversarial test requires blocked reconciliation. |
| disk-full / crash | Effect-store disk-full qualification proves pre-dispatch persistence failure rolls back durably; memory restart cut tests require recovery/blocking rather than inferred completion after interrupted writes. |
| PREPARED / terminal / UNKNOWN_OUTCOME | Operational-store tests cover PREPARED binding, immutable terminal status, terminal `UnknownOutcome`, and blocked reconciliation. |
| dependent-mutation blocking | `has_blocking_unknown_outcome()` is tested for both terminal unknown outcome and blocked reconciliation; writer preparation checks this gate before accepting later mutation. |
| stale-memory conflict | T005-057 filesystem/Git precedence tests reject score-based stale-memory override and bind the conflict record to both observations. |
| FORGET/REDACT partial completion / resurrection | Controlled replacement tests prove prior plaintext is absent from Forgotten/Redacted canonical bytes; derivative tests do not index forgotten plaintext; ambiguous operational/restart cuts block rebuild and later mutation rather than resurrecting content. |

## Cross-platform qualification checkpoint

Exact implementation head before this evidence-only record:

`c9a4700c3917ea73937ca60d3e1ee0a2e6b5c984`

GitHub Actions CI #1080 / run `33863595756` completed successfully on Ubuntu, Windows, and macOS for that exact head, including:

- rustfmt;
- Clippy;
- full workspace tests;
- property qualification;
- bounded fuzz smoke;
- platform IPC qualification;
- authenticated daemon IPC qualification;
- adversarial authority qualification;
- strict-local external network observation.

Because this record itself changes the branch head, the Phase E closeout state is intentionally:

`T005_045_THRU_057=IMPLEMENTED`

`T005_058=QUALIFICATION_EVIDENCE_MAPPED`

`FINAL_EXACT_HEAD_CI=PENDING`

`WAIVER_TAKEN=NO`

`NEXT_TASK=FINAL_EXACT_HEAD_PHASE_E_CI_THEN_T005-060`

Phase F must not be treated as qualified until the new exact head passes the required CI matrix.
