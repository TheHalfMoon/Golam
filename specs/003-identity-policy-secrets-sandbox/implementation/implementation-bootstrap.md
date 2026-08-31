# Spec 003 Implementation Bootstrap

**Status**: COMPLETE  
**Date**: 2026-08-27  
**Implementation branch**: `impl/003-identity-policy-secrets-sandbox`

## Canonical predecessor

Spec 003 planning was merged from PR #4 after exact-head CI and authorized Qodo qualification.

- planning candidate: `0170dadfb520b499c6acf41c4bc73a051b2ef5f6`
- canonical merge commit: `82de7084384009ff3a00522f4e0aef09bf549529`
- canonical tree: `046966262379ba0e7038e7a1216c6237c2033a94`
- post-merge CI: run #255 / `33036939686`
- post-merge Windows/macOS/Ubuntu result: `SUCCESS`
- waiver: `NO`
- Codex review gate: excluded by founder direction

Therefore:

```text
SPEC_003_PLANNING_CLOSED_CANONICAL=YES
SPEC_003_IMPLEMENTATION_AUTHORIZED=YES
```

## T003-001 — exact-main bootstrap

PASS. Canonical `main` was reread after the planning merge and remained exactly `82de7084384009ff3a00522f4e0aef09bf549529`. The implementation branch was created directly from that commit with no rebase, force update, or intermediate mutation.

## T003-002 — governance reread

PASS. Before product mutation the following authority was reread from exact canonical `main`:

- `.specify/memory/constitution.md` v1.2.0;
- frozen Spec 001 authority/security contracts and program decomposition;
- canonical Spec 002 authorization seam and implementation evidence;
- merged Spec 003 spec, plan, contracts, tasks and Qodo repair reconciliation;
- `AGENTS.md`.

The textual `AGENTS.md` phase banner still described the now-completed planning lifecycle. Live canonical GitHub truth satisfies its explicit transition condition: planning is reviewed, merged, post-merge CI is green, and canonical main has been reread. This implementation branch reconciles that phase banner before product code.

## Preserved implementation spine

No new crate is authorized merely for domain naming. Spec 003 continues inside the canonical seven-package workspace:

```text
crates/golam-core
crates/golam-ledger
crates/golam-effects
crates/golam-ipc
crates/golam-kernel
crates/golamd
crates/golam
```

The workspace remains Rust 2024, `rust-version = 1.98`, exact-version oriented, and `unsafe_code = "forbid"` for Golam workspace code.

## Baseline seam inspected

The Spec 002 kernel already provides:

- `AuthorizationRequest { principal, action, resource, context }`;
- sealed `AuthorityGrant` creation only after an allow;
- monotonic hard safety denial;
- unconditional strict-local denial for `network.egress*`;
- a replaceable `AuthorizationPolicy` trait;
- a deny-by-default/bootstrap evaluator;
- durable authorization audit records;
- authority SQLite schema version 1 with forward-only startup migration and integrity checks.

Spec 003 extends these seams; it does not replace `KernelApi` or widen generic authority.

## Phase A state

Dependency qualification records in this directory are the only source of admission for new Spec 003 dependencies. No dependency is admitted by name alone, no real secret is used, and no donor source is copied.
