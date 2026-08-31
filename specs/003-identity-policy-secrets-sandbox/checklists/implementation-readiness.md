# Implementation Readiness Checklist — Spec 003

## Planning scope

- [x] Spec 002 canonical predecessor and exact base are identified.
- [x] Spec 003 scope matches frozen Spec 001 decomposition.
- [x] Planning/implementation are explicitly separated.
- [x] Out-of-scope models/tools/Desktop/Connect/workers are explicit.
- [x] Existing seven-package spine is preserved; no empty crate scaffolding planned.

## Authority architecture

- [x] Stable KernelApi/Authorize seam is preserved.
- [x] Hard-deny -> principal -> lease -> policy -> approval -> effect/mutation ordering is explicit.
- [x] Denial monotonicity is explicit.
- [x] Policy evaluation errors fail closed.
- [x] Policy self-activation under the candidate policy is forbidden.
- [x] Protected-state classes and typed mutation path are explicit.
- [x] Lease subset/expiry/revocation rules are explicit.

## Approvals

- [x] All five frozen approval classes are represented.
- [x] Execution-time freshness is explicit.
- [x] ONCE durable reservation/consumption semantics are planned.
- [x] Unattended irreversible actions require bounded RUN_PREAUTHORIZATION.

## Taint

- [x] Frozen baseline labels are carried forward.
- [x] Derivation uses monotonic union.
- [x] Self-clear is forbidden.
- [x] Human/deterministic-verifier downgrade is auditable.
- [x] `SECRET_DERIVED` long-term-memory denial and deterministic secret-elimination sanitizer semantics are explicit.

## Secrets

- [x] Opaque handles are the default interface.
- [x] Encrypted-at-rest vault semantics are explicit without prematurely selecting a crypto/keychain dependency.
- [x] Brokered use is preferred.
- [x] Unbrokerable fallback forbids argv/ambient inheritance and requires isolated approval/redaction.
- [x] Explicit user-designated secret entry treats the complete submitted value as secret independent of format detection before durable model-visible append.
- [x] Recognized-format detection in ordinary free text is defense in depth rather than the source of the secret-entry guarantee.
- [x] Qualification includes deliberately unknown-format deterministic canaries through the explicit entry path and uses no real credentials.

## Egress / sandbox

- [x] Strict-local hard denial remains above all permits and applies to every Golam-managed process.
- [x] Non-strict permit scope includes destination/purpose/time/taint/secret context.
- [x] DNS resolution, redirects, rebinding, protocol/port changes and private/link-local/loopback target changes require mandatory effective-destination reauthorization before connect/follow.
- [x] Hostname permission never implicitly transfers to an arbitrary changed effective target.
- [x] Authorization and sandboxing are distinct.
- [x] Sandbox profile declarations and unsupported-platform fail-closed semantics are explicit.
- [x] Wasmtime/WASI is candidate-only and not a universal native sandbox.
- [x] External strict-local observation must cover the full Golam-managed process tree or equivalent descendant-capturing network boundary before a network-capable managed child is launched.

## Donor/dependency governance

- [x] `Golam-Research` exact snapshot recorded and classified `REFERENCE_ONLY`.
- [x] No donor source code admitted by planning.
- [x] Cedar was candidate-only during planning; implementation qualification later admitted exactly `cedar-policy 4.12.0` with the bounded selected feature surface.
- [x] Wasmtime remained candidate-only and implementation recorded `NOT_ADMITTED_NOT_NEEDED`.
- [x] Secret crypto/key-protection/platform backends were implementation-time qualified before secret-value handling.

## Verification plan

- [x] Hard-denial and lease narrowing property tests defined.
- [x] Policy malformed/error/corruption gates defined.
- [x] Approval concurrency/crash gates defined.
- [x] Taint/self-clear/SECRET_DERIVED gates defined.
- [x] Recognized and unknown-format explicit-entry secret canary durable/log/output gates defined.
- [x] Mandatory changed-destination egress reauthorization tests defined.
- [x] Strict-local external no-egress gate covers all managed descendants/process-tree or equivalent sinkholed boundary.
- [x] Sandbox unsupported/enforcement gates defined.
- [x] Windows/macOS/Linux exact-head CI and post-CI Qodo gates defined.

## Qodo repair reconciliation

- [x] Finding 1 accepted: optional destination revalidation was too weak; changed effective destinations require mandatory reauthorization or deny.
- [x] Finding 2 accepted: daemon-PID-only locality observation is insufficient once managed child execution exists; observer upgrade is a predecessor gate to network-capable child launch.
- [x] Finding 3 accepted: recognized-format-only secret ingestion was too weak; explicit user-designated secret entry treats the whole value as secret independent of detection.

## Planning lifecycle — closed canonical

- [x] Exact-head CI succeeded on the repaired final planning candidate.
- [x] Fresh authorized Qodo review after repaired exact-head CI reported zero unresolved material findings.
- [x] Planning PR #4 was merged to canonical `main`.
- [x] Exact post-merge `main@82de7084384009ff3a00522f4e0aef09bf549529` was reread and post-merge CI #255 succeeded before implementation branch creation.

The checklist above remains the planning/readiness record. Implementation-time dependency admission and task qualification are recorded under `implementation/` and `tasks.md`; the implementation PR still requires its own final exact-head CI, post-CI Qodo review, lifecycle evidence and post-merge main CI.

```text
SPEC_002_CLOSED_CANONICAL=YES
SPEC_003_PLANNING_PACKAGE=CLOSED_CANONICAL
QODO_REPAIR_FINDINGS=3_ACCEPTED_AND_RECONCILED
PLANNING_EXACT_HEAD_CI=PASS
PLANNING_POST_CI_QODO=PASS
PLANNING_PR_4=MERGED
PLANNING_POST_MERGE_MAIN_CI_255=PASS
PRODUCT_IMPLEMENTATION_IN_PLANNING_PR=NO
DONOR_CODE_ADMITTED=NO
CEDAR_POLICY_ADMITTED_EXACT=4.12.0
WASMTIME_DISPOSITION=NOT_ADMITTED_NOT_NEEDED
REAL_SECRETS_USED=NO
SPEC_003_IMPLEMENTATION_AUTHORIZED=YES
IMPLEMENTATION_FINAL_EXACT_HEAD_CI=PENDING_AFTER_T003_095
IMPLEMENTATION_FINAL_QODO=PENDING_AFTER_EXACT_HEAD_CI
CODEX_REVIEW_GATE=EXCLUDED_BY_FOUNDER_DIRECTION
```
