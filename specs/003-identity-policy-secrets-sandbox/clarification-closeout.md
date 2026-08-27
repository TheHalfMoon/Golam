# Clarification Closeout — Spec 003

**Date**: 2026-08-27  
**Decision**: CLARIFIED_FOR_PLAN

## C1 — Does Spec 003 replace KernelApi?

**Decision**: No. The Spec 002 process-splittable KernelApi and `Authorize(principal, action, resource, context)` semantics remain stable. Spec 003 replaces the bootstrap evaluator and fills the reserved authority families without making external policy libraries the trust root.

## C2 — What is the authorization ordering?

**Decision**: fail-closed layered evaluation:

```text
hard kernel guards
 -> authenticated principal
 -> active capability lease
 -> policy evaluator
 -> step-up approval
 -> typed protected mutation/effect gate
```

Any denial is monotonic. An approval can activate/narrow already permitted authority; it cannot overturn hard safety/strict-local denial or manufacture otherwise absent authority.

## C3 — Is Cedar mandatory?

**Decision**: Cedar is the preferred policy-evaluator candidate because Spec 001 already selected it as the candidate and its model fits principal/action/resource/context authorization. Planning does not admit a crate/version. Implementation must qualify the exact dependency, unsafe/FFI/transitive surface and validation behavior. If it fails qualification, Golam retains its owned contract and may select another evaluator without changing external semantics.

## C4 — How are policy failures handled?

**Decision**: fail closed. Invalid schema/policy, missing active bundle, evaluation diagnostics/errors, inconsistent entities, oversized authority input or corrupt active-policy state produce DENY plus bounded audit/recovery evidence. No fallback to bootstrap allow rules after normal policy activation.

## C5 — How is policy changed safely?

**Decision**: bundles are immutable/versioned/hash-bound. Candidate bundle/schema validation occurs before activation. Activation is one protected transaction updating the active pointer and security evidence. Policy mutation is itself an elevated effect authorized under the currently active authority state plus required approval. Initial/recovery bootstrap authority is narrow local-owner-only and cannot become a general policy bypass.

## C6 — What is a capability lease?

**Decision**: kernel-minted bounded authority referencing an authenticated principal and explicit action/resource/context scope. Child derivation is intersection/subset only. Expiry and revocation are checked at the protected action boundary. Exact binary/signature/MAC representation is an implementation-time design/dependency decision, not a planning assumption.

## C7 — How do approvals bind?

**Decision**: approval records bind approver, class, action/resource/effect or operation pattern, risk, relevant context hash/taint, issue/expiry, usage limits and parent authorization context. ONCE usage is atomically reserved/consumed with replay protection. Approval freshness is rechecked immediately before the protected action executes.

## C8 — Can a human clear `SECRET_DERIVED`?

**Decision**: not by an ordinary trust assertion. Human approval may downgrade normal provenance taints where policy permits, but `SECRET_DERIVED` is a non-memory safety label. A non-secret-derived representation requires deterministic registered secret-elimination/sanitization that produces a new artifact plus evidence; the source remains secret-derived.

## C9 — How are secrets stored and used?

**Decision**: callers use opaque handles. Durable vault values are encrypted at rest using an implementation-qualified design; exact OS key protection and cryptographic dependencies are deferred to dependency qualification. Brokered use is preferred. Unbrokerable use requires bounded approval and isolated injection not via argv, with cleared ambient environment, minimal lifetime and value-aware redaction.

## C10 — What happens to pasted secrets?

**Decision**: secret-like user input crossing a designated secret-ingestion boundary is redacted/tombstoned before durable model-visible canonical text is committed. Audit retains non-secret metadata sufficient to explain that redaction occurred. Implementation must use deterministic canaries for tests and avoid a claim that arbitrary secret detection is perfect.

## C11 — Can policy enable network in strict-local mode?

**Decision**: No. Strict-local external egress is a hard kernel denial above policy, leases and approvals. Non-strict egress uses explicit permits and includes DNS/redirect/rebinding semantics. Loopback remains separately scoped.

## C12 — Is a sandbox profile equivalent to containment?

**Decision**: No. The profile is an authority/admission contract. A platform executor must prove actual containment. Unsupported enforcement fails closed. Wasmtime/WASI is a candidate for portable bounded extension profiles only; native tools need native isolation evidence.

## C13 — Does Spec 003 add broad tools or real effects?

**Decision**: No. Authority behavior is proven through bounded local fixtures/canaries and existing synthetic effect mechanisms. Product tools, models, Desktop, Connect and workers remain later-spec work.

## C14 — What is the donor posture?

**Decision**: `Golam-Research@a9f633e09d49a85829b8236331b9e21f7e612634` / tree `b68f24972427952c4934e4364736fec62661044f` is `REFERENCE_ONLY` for Spec 003. It does not provide the trusted Rust policy/secret/sandbox authority substrate. No donor code is admitted by planning.

## C15 — Planning/implementation discipline

**Decision**: this planning package must be reviewed/merged independently. Spec 003 Rust implementation begins only from the exact canonical main produced by that merge. Planning CI/review evidence does not authorize product implementation before merge.
