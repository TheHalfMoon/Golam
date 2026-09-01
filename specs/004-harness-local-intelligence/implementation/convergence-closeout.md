# Spec 004 Implementation Convergence Closeout

## Scope

This record covers Phase K convergence tasks T004-105 through T004-107 and the forward-only reconciliation required before final exact-head qualification.

Implementation base:

`main@8b08ae9f787cb85f1257641d6d332810d7de9fa4`

Initial convergence repair head qualified before closeout documentation:

`eb79dc7de021601a3dff28a27543eec652bd35c2`

Latest pre-documentation reconciliation head:

`3690616152f1c044688c460ffd17d578bf3d9ad8`

## T004-105 — Cross-artifact and code convergence

The implementation was re-read against:

- constitution v1.2.0;
- frozen Spec 001 architecture, tasks and execution-profile contract;
- canonical Specs 002–003 authority, durability, effect, identity, taint, sandbox and strict-local boundaries;
- the complete canonical Spec 004 planning package;
- exact `mistral.rs v0.9.0` Source Foundry rejection evidence;
- dated exact `llama.cpp v0.3.0` compatibility deferral evidence;
- the exact-commit Munder Difflin reference-only record;
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

The first convergence pass found a durable-evidence identity defect: several identity-keyed evidence tables used silent conflict-ignore behavior. That could retain stale evidence when the same identifier was reused with materially different hardware, benchmark, compaction-artifact or calibration content. The defect was repaired forward-only. Exact duplicate evidence remains idempotent; conflicting identity reuse fails closed.

A substantive independent CodeRabbit review was later completed for the older exact range `8b08ae9f787cb85f1257641d6d332810d7de9fa4..b9a6516630839995e34d899ab20f95bacf28ad3a`. That review does not satisfy T004-109 for the current branch head, but its material findings were treated as reconciliation input and repaired rather than waived.

Reconciled findings include:

- canonical harness identifier parsing now accepts only the fixed-width lowercase hexadecimal representation;
- durable request-attempt state transitions reject terminal rewrites and invalid regressions;
- accepted model-event evidence requires exact next sequence and rejects post-terminal append;
- compaction artifacts, benchmark records and calibration runs reject missing durable parents;
- cancellation and timeout terminal evidence is persisted before propagating backend cancellation errors;
- deterministic calibration enforces workload runtime bounds and rejects timestamp arithmetic overflow;
- duplicate compaction references fail closed;
- benchmark binding and workload identities use canonical SHA-256 rather than a bespoke non-cryptographic digest;
- `llama.cpp` compatibility evidence is bound to a dated upstream release observation and exact commit;
- Munder Difflin reference evidence is bound to an exact observed upstream commit rather than mutable `main`.

No finding was waived. Any fresh material finding on the final exact head remains blocking.

`T004-105=PASS`
`SILENT_HARNESS_EVIDENCE_IDENTITY_REUSE=REJECTED`
`OLD_REVIEW_RANGE=b9a6516630839995e34d899ab20f95bacf28ad3a`
`OLD_REVIEW_COUNTS_AS_FINAL_REVIEW=NO`
`WAIVER_TAKEN=NO`

## T004-106 — Focused qualification

Focused qualification is covered by the implementation test surface for:

- execution-profile and canonical harness identifier identity/validation;
- request-attempt prepare/dispatch/stream/terminal lifecycle;
- durable state-transition and parent-integrity enforcement;
- accepted partial-output durability, exact-sequence replay and post-terminal denial;
- cancellation and timeout separation, including backend cancellation failure;
- retry/new-attempt semantics and crash classification;
- native/grammar/text tool-call normalization and authority denial;
- projection taint/secret exclusion and duplicate compaction-ref rejection;
- deterministic compaction source binding, Goal retention, overflow reprojection and failed-activation denial;
- strict-local routing dominance and explicit fallback classes;
- bounded hardware/calibration behavior and runtime arithmetic bounds;
- deterministic scripted harness benchmarking, SHA-256 binding and stale binding rejection;
- durable evidence identity-collision and missing-parent rejection.

`T004-106=PASS`

## T004-107 — Full workspace qualification before final documentation head

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

Subsequent review-driven reconciliation intentionally mutated the branch and therefore invalidated all older exact-head CI/review evidence for T004-108/T004-109. Intermediate CI runs are diagnostic only. The final branch head created by this documentation update must receive a fresh three-platform exact-head CI pass before any final independent semantic review is requested.

`T004-107=PASS`
`T004-108=PENDING_FINAL_EXACT_HEAD_CI`
`T004-109=PENDING_FRESH_EXACT_HEAD_REVIEW`
`PR_READY=NO`
`MERGE_AUTHORIZED=NO`
`SPEC_004_IMPLEMENTATION_COMPLETE=NO`
`SPEC_004_CLOSED_CANONICAL=NO`
