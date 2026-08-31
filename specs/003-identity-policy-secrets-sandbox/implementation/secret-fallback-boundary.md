# T003-054 Secret Fallback Boundary

Status: `ACTIVE`

This note records the implementation boundary for T003-054 before exact-head qualification. It is not qualification evidence and does not claim PASS.

Current repaired candidate includes the bounded `AdmissionRequest` verification input refactor required by the repository's pinned Clippy gate, test-only sandbox profile/admission snapshot helpers matching the existing `authority-security-v2` coverage, and an argv-canary fixture whose admission/effect/lease/approval authority is bound to the exact tested launch plan so the non-argv invariant is exercised directly. Qualification remains pending fresh exact-head CI.

## Authorized scope

T003-054 implements a bounded trusted fallback only for a secret use that cannot be brokered directly.

The fallback requires all of the following before plaintext application:

- an authenticated opaque secret handle and active current immutable secret version;
- an already-existing authenticated sandbox/process admission whose exact resolved launch-plan hash and platform executor match the requested fallback plan;
- no egress permit on that admission;
- the admission's exact allow/pass authorization decision, active policy binding and capability-lease chain;
- an exact authorized at-most-once `secret.fallback.use` effect;
- an exact ONCE approval bound to action/resource/effect/risk/taint and fresh at execution;
- an injector that explicitly attests cleared-environment, stdin-only secret delivery, stdin closure, no secret argv/environment, no ambient descendant inheritance, and captured stdout/stderr.

The only secret injection channel admitted by this task is stdin. The launch plan starts from a cleared ambient environment and contains only explicitly admitted non-secret variables. A decrypted secret value is checked against executable, argv and explicit environment fields before any use record or approval consumption is committed.

The vault exposes plaintext only through a crate-internal callback boundary; the generic private decrypt primitive remains private and no plaintext-return API is introduced.

Captured stdout, stderr and injector errors are exact-value redacted before leaving the trusted fallback boundary. Tests use only a deterministic canary.

## Explicit exclusions

T003-054 does not create or widen sandbox profiles, sandbox admissions, capability leases, policy authority, approvals or egress permits. It consumes pre-existing protected authority only.

It does not claim a universal native sandbox implementation or platform containment beyond the executor capabilities explicitly supplied and authenticated by the existing admission. A missing or ambiguous admission/capability fails closed.

It does not launch a network-capable managed child or grant network authority; the admission must carry no egress permit under the current strict-local contract.

## Qualification requirements

Exact-head CI must compile the production-linked fallback on Windows, macOS and Ubuntu and pass all applicable repository gates.

Focused tests must prove at minimum:

- successful stdin-only fallback with exact-value redaction and atomic ONCE approval consumption;
- replay denial after approval consumption;
- deterministic canary rejection when duplicated into argv before any durable use/approval consumption;
- weak injector containment-capability denial;
- launch-plan/admission hash mismatch denial;
- launch-plan hash sensitivity and cleared-environment/no-secret-argv/no-secret-environment/no-descendant-inheritance invariants.

No real secret values are permitted in qualification.