# Spec 004 Implementation Convergence Closeout

## Scope

This record covers Phase K convergence tasks T004-105 through T004-107 for the bounded Spec 004 implementation posture.

Implementation base:

`main@8b08ae9f787cb85f1257641d6d332810d7de9fa4`

Pre-closeout qualified implementation head:

`eb79dc7de021601a3dff28a27543eec652bd35c2`

## T004-105 — Cross-artifact and code convergence

The implementation was re-read against:

- constitution v1.2.0;
- frozen Spec 001 architecture, tasks and execution-profile contract;
- canonical Specs 002–003 authority, durability, effect, identity, taint, sandbox and strict-local boundaries;
- the complete canonical Spec 004 planning package;
- exact `mistral.rs v0.9.0` Source Foundry rejection evidence;
- exact `llama.cpp v0.3.0` compatibility deferral evidence;
- the Munder Difflin reference-only record;
- all current Spec 004 implementation code and durable evidence paths.

Convergence confirmed:

- the seven-crate workspace remains preserved;
- `golam-core` owns pure protocol/state/profile/backend semantics;
- `golam-ledger` owns durable harness evidence;
- `golamd` owns unprivileged coordination;
- no model backend, model tool call, benchmark score or routing recommendation becomes authority;
- canonical history survives compaction;
- retry creates a new attempt and does not rewrite prior evidence;
- strict-local denial remains dominant and local failure cannot authorize cloud fallback;
- no production `mistral.rs` or `llama.cpp` dependency/runtime artifact is admitted;
- no later-spec filesystem/shell/git/browser/memory/connect/workers/scheduler scope is introduced.

A convergence defect was found in durable harness evidence persistence: several identity-keyed evidence tables used silent conflict-ignore behavior. That could retain stale evidence when the same identifier was reused with materially different hardware, benchmark, compaction-artifact or calibration content.

The defect was repaired forward-only. Exact duplicate evidence remains idempotent; conflicting identity reuse now fails closed. Regression tests cover the affected evidence classes. The model-event duplicate path already converted ignored insertion to an explicit `SequenceConflict` and required no semantic change.

`T004-105=PASS`
`SILENT_HARNESS_EVIDENCE_IDENTITY_REUSE=REJECTED`
`WAIVER_TAKEN=NO`

## T004-106 — Focused qualification

Focused qualification is covered by the implementation test surface for:

- execution-profile identity and invalidation;
- request-attempt prepare/dispatch/stream/terminal lifecycle;
- accepted partial-output durability and replay;
- cancellation and timeout separation;
- retry/new-attempt semantics and crash classification;
- native/grammar/text tool-call normalization and authority denial;
- projection taint/secret exclusion;
- deterministic compaction source binding, Goal retention, overflow reprojection and failed-activation denial;
- strict-local routing dominance and explicit fallback classes;
- bounded hardware/calibration behavior;
- deterministic scripted harness benchmarking and stale binding rejection;
- durable evidence identity-collision rejection.

`T004-106=PASS`

## T004-107 — Full workspace qualification

CI #752 / run `33492514474` completed SUCCESS on exact head:

`eb79dc7de021601a3dff28a27543eec652bd35c2`

Platform results:

- Windows: SUCCESS
- macOS: SUCCESS
- Ubuntu: SUCCESS

Applicable gates completed successfully:

- format;
- Clippy with warnings denied;
- full tests;
- property qualification;
- bounded fuzz smoke;
- IPC transport qualification;
- authenticated daemon IPC qualification;
- adversarial authority qualification;
- daemon build for external locality observation;
- strict-local external no-egress observation.

This evidence qualifies the implementation and convergence repair before this closeout documentation commit. Because creating/updating closeout documentation mutates the branch head, T004-108 requires a fresh exact-head CI run on the final documentation head before independent review.

`T004-107=PASS`
`T004-108=PENDING_FINAL_EXACT_HEAD_CI`
`INDEPENDENT_REVIEW_STARTED=NO`
`PR_READY=NO`
`MERGE_AUTHORIZED=NO`
