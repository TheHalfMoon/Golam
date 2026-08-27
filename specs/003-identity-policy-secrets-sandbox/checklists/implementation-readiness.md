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
- [x] Cedar is candidate-only pending exact dependency qualification.
- [x] Wasmtime is candidate-only pending a bounded task.
- [x] Secret crypto/key-protection/platform backends remain implementation-time qualified.

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

- [x] Finding 1 accepted: optional destination revalidation was too weak; changed effective destinations now require mandatory reauthorization or deny.
- [x] Finding 2 accepted: daemon-PID-only locality observation is insufficient once managed child execution exists; observer upgrade is now a predecessor gate to network-capable child launch.
- [x] Finding 3 accepted: recognized-format-only secret ingestion was too weak; explicit user-designated secret entry now treats the whole value as secret independent of detection.

## Planning lifecycle

- [ ] Exact-head CI succeeds on the repaired final planning candidate.
- [ ] Fresh authorized Qodo review after repaired exact-head CI reports zero unresolved material findings.
- [ ] Planning PR is merged to canonical `main`.
- [ ] Exact post-merge main is reread before implementation branch creation.

```text
SPEC_002_CLOSED_CANONICAL=YES
SPEC_003_PLANNING_PACKAGE=REPAIRED_PENDING_REQUALIFICATION
QODO_REPAIR_FINDINGS=3_ACCEPTED_AND_RECONCILED
FINAL_EXACT_HEAD_CI=PENDING_AFTER_REPAIR
FINAL_POST_CI_QODO=PENDING_AFTER_REPAIR
PRODUCT_IMPLEMENTATION_IN_PLANNING_PR=NO
DONOR_CODE_ADMITTED=NO
CEDAR_ADMITTED=NO
WASMTIME_ADMITTED=NO
REAL_SECRETS_USED=NO
SPEC_003_IMPLEMENTATION_AUTHORIZED=NO_UNTIL_PLANNING_MERGE
CODEX_REVIEW_GATE=EXCLUDED_BY_FOUNDER_DIRECTION
```
