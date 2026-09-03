# Spec 005 Phase D Qualification

Status: `QUALIFIED_PENDING_THIS_EVIDENCE_HEAD_CI`

## Qualified implementation head

The Phase D implementation was qualified on exact branch head:

`2a86311ee0b888a7fdd51769066f65db224edd7c`

GitHub Actions workflow `ci` run `#960` (`33775548685`) completed `SUCCESS` for all three platform jobs:

- `rust-ubuntu-latest`: format, Clippy, full tests, property qualification, bounded fuzz smoke, Unix IPC, authenticated daemon IPC, adversarial authority qualification, daemon build, and strict-local external network observation all succeeded; Windows-only steps were skipped as platform-inapplicable.
- `rust-macos-latest`: format, Clippy, full tests, property qualification, bounded fuzz smoke, Unix IPC, authenticated daemon IPC, adversarial authority qualification, daemon build, and strict-local external network observation all succeeded; Windows-only steps were skipped as platform-inapplicable.
- `rust-windows-latest`: format, Clippy, full tests, property qualification, bounded fuzz smoke, Windows IPC, authenticated daemon IPC, adversarial authority qualification, daemon build, and strict-local external network observation all succeeded; Unix-only steps were skipped as platform-inapplicable.

No waiver was taken.

## T005-035..040 boundary evidence

Phase D retains the following fail-closed properties on the qualified implementation head:

- authorized filesystem observation resolves bounded targets and excludes unsupported/protected identities;
- regular-file reads, directory observation/walk, and literal search use explicit byte/count/depth/duration bounds;
- unqualified opened-directory/content-read platform cases fail closed rather than falling back to weaker path semantics;
- literal L0 search is Golam-owned and in-process: `local_search.rs` composes `LocalFsResolver`, `walk_directory`, `stat_regular_file`, and `read_regular_file`; it exposes no child-process or shell launch path;
- the admitted L0 compiler route set contains only user-selected artifacts, file reads, in-process search, Git, canonical evidence, and managed memory; it contains no process or network route;
- Git read observation remains read-only and separately bounded; production process execution remains unavailable.

Therefore an external search executable cannot be reached through the Phase D L0 search implementation while the production native executor remains `native:unqualified`.

## T005-041..043 context adversarial evidence

The L0 Context Compiler on the qualified head includes focused tests proving:

- source routing does not widen the caller's admitted route set;
- route/source-kind mismatches fail closed;
- duplicate evidence identities fail closed;
- stale evidence is rejected and surfaces explicit missing requirements;
- a hostile item with the maximum retrieval score cannot raise its authority class or clear forbidden taint;
- lower-ranked evidence with valid authority/permission/freshness/taint can satisfy the requirement instead;
- missing evidence yields an explicit bounded replan;
- replan attempts beyond the configured finite bound are rejected;
- capsule construction is deterministic and managed-memory references remain explicit projections rather than canonical-source replacement.

## Phase transition

This evidence file is documentation-only relative to the already qualified implementation head. Its own exact head MUST still pass the ordinary Windows/macOS/Ubuntu CI before Phase D is treated as closed for dependency ordering.

`T005_044=QUALIFIED_PENDING_EVIDENCE_HEAD_CI`
`PHASE_D=QUALIFIED_PENDING_EVIDENCE_HEAD_CI`
`NEXT_TASK=T005-045`
`PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO`
`EXTERNAL_SEARCH_EXECUTABLE_ADMITTED=NO`
`WAIVER_TAKEN=NO`
