# Spec 004 Implementation Baseline

## Authorization

Spec 004 planning is `CLOSED_CANONICAL`.

Implementation base:

`main@8b08ae9f787cb85f1257641d6d332810d7de9fa4`

Canonical planning tree:

`283c818ee7f214e88d6f678fa1d45ff51e8ad0c6`

Planning closeout evidence:

- planning source PR #11 exact head `b11aca2474d7da827d1145ea57c16784be95adbe`;
- exact-head CI #678 / run `33418813154` — Windows/macOS/Ubuntu PASS;
- final substantive external semantic review — BLOCKER=0, MAJOR=0, no remaining material defect;
- lifecycle relay PR #12 — identical commit/tree/content, non-Draft;
- relay CI #679 / run `33424740450` — Windows/macOS/Ubuntu PASS;
- relay consistency review — PASS, zero review threads;
- expected-head guarded merge SHA `8b08ae9f787cb85f1257641d6d332810d7de9fa4`;
- post-merge push CI #680 / run `33425624764` — Windows/macOS/Ubuntu PASS;
- waiver taken: NO.

`SPEC_004_PLANNING_CLOSED_CANONICAL=YES`
`SPEC_004_IMPLEMENTATION_BASE=8b08ae9f787cb85f1257641d6d332810d7de9fa4`

## T004-010 — Canonical re-read

The implementation starts only after re-reading exact canonical `main`, `AGENTS.md`, constitution v1.2.0, frozen Spec 001 authority, canonical Specs 002–003 and the complete canonical Spec 004 planning package.

The implementation must preserve these non-negotiable boundaries:

- model/backend/harness state is not authority state;
- model tool output is candidate data only;
- canonical history survives compaction;
- retry creates a new attempt and cannot blind-replay a protected effect;
- strict-local failure does not authorize cloud/network widening;
- source permission does not equal source admission;
- exact Source Foundry admission precedes any production donor/dependency addition.

`T004-010=PASS`

## T004-011 — Implementation branch

Implementation branch:

`impl/004-harness-local-intelligence`

It was created directly from exact qualified canonical base `8b08ae9f787cb85f1257641d6d332810d7de9fa4` after post-merge CI #680 succeeded.

`T004-011=PASS`

## T004-012 — Seven-crate ownership map

The existing workspace is sufficient for the first bounded implementation slice:

| Crate | Spec 004 ownership |
| --- | --- |
| `golam-core` | Pure IDs, immutable execution/hardware/profile/request/event/candidate/compaction/benchmark types, bounded validation and deterministic state semantics. |
| `golam-ledger` | Canonical request/profile/compaction/calibration/benchmark evidence persistence, append-oriented recovery and projections using the existing SQLite/integrity model. |
| `golam-effects` | Existing protected-effect semantics only; no model-specific semantics. |
| `golam-ipc` | Existing authenticated local transport contracts; no model-specific authority. |
| `golam-kernel` | Existing authority/strict-local decisions only; no model reasoning or backend authority. |
| `golamd` | Unprivileged harness coordinator, scripted/qualified backend lifecycle and routing under kernel decisions. |
| `golam` | Bounded diagnostics/qualification UX only where required by the Spec 004 acceptance slice. |

No independent ownership or testing boundary currently justifies an eighth crate. Creating `golam-harness` or `golam-models` now would be architectural mirroring without evidence and is rejected.

`SPEC004_INITIAL_WORKSPACE_CRATE_COUNT=7`
`SPEC004_NEW_CRATE_REQUIRED=NO`
`T004-012=PASS`

## Initial source-admission state

No production inference source or runtime artifact is admitted at implementation start.

- `mistral.rs` — `PRIMARY_CANDIDATE_NOT_YET_ADMITTED`
- `llama.cpp` — `COMPATIBILITY_CANDIDATE_NOT_YET_ADMITTED`
- Golam-Research — `REFERENCE_ONLY` for current implementation until an exact bounded admission record says otherwise
- grok-build — `REFERENCE_ONLY`
- Goose — `REFERENCE_ONLY`
- DeepSeek Harness — `REFERENCE_ONLY`

`SPEC004_IMPLEMENTATION_PRODUCTION_ADMISSION_COUNT=0`

## Next task

`NEXT_TASK=T004-013`
