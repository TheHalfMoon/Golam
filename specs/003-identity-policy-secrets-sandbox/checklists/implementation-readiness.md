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
- [x] User-pasted secret redaction/tombstone behavior is explicit.
- [x] Qualification uses deterministic canaries, not real credentials.

## Egress / sandbox

- [x] Strict-local hard denial remains above all permits.
- [x] Non-strict permit scope includes destination/purpose/time/taint/secret context.
- [x] DNS/redirect/rebinding is included.
- [x] Authorization and sandboxing are distinct.
- [x] Sandbox profile declarations and unsupported-platform fail-closed semantics are explicit.
- [x] Wasmtime/WASI is candidate-only and not a universal native sandbox.

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
- [x] Secret canary durable/log/output gates defined.
- [x] Strict-local external no-egress gate retained.
- [x] Sandbox unsupported/enforcement gates defined.
- [x] Windows/macOS/Linux exact-head CI and post-CI Qodo gates defined.

## Planning lifecycle

- [ ] Exact-head CI succeeds on the final planning candidate.
- [ ] Fresh authorized Qodo review after CI reports zero unresolved material findings.
- [ ] Planning PR is merged to canonical `main`.
- [ ] Exact post-merge main is reread before implementation branch creation.

```text
SPEC_002_CLOSED_CANONICAL=YES
SPEC_003_PLANNING_PACKAGE=COMPLETE_PENDING_REVIEW
PRODUCT_IMPLEMENTATION_IN_PLANNING_PR=NO
DONOR_CODE_ADMITTED=NO
CEDAR_ADMITTED=NO
WASMTIME_ADMITTED=NO
REAL_SECRETS_USED=NO
SPEC_003_IMPLEMENTATION_AUTHORIZED=NO_UNTIL_PLANNING_MERGE
CODEX_REVIEW_GATE=EXCLUDED_BY_FOUNDER_DIRECTION
```
