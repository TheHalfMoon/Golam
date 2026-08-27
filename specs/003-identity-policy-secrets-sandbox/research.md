# Research — Spec 003 Identity, Policy, Secrets & Sandbox

**Date**: 2026-08-27  
**Decision**: RESEARCH_COMPLETE_FOR_PLAN

## 1. Research objective

Identify the smallest security architecture that upgrades the Spec 002 bootstrap authority without framework stacking, authority inversion, secret leakage, or false sandbox claims.

Research is planning evidence only. It does not admit source code or dependency versions.

## 2. Canonical predecessor evidence

Spec 002 is closed canonical on `main@a04756f242e48faeda802e5b3fd99a0c8d52f53e`, tree `33eeff5d624d25ab94d677a81a5e591d0d20330b`.

Existing seams to preserve:
- process-splittable KernelApi;
- authenticated local clients;
- `Authorize(principal, action, resource, context)`;
- protected authority DB/path boundary;
- durable authorization/security audit records;
- effect gate and UNKNOWN_OUTCOME/reconciliation semantics;
- monotonic strict-local egress hard denial;
- Windows/macOS/Linux CI and externally observed no-egress qualification.

## 3. Cedar policy research

Primary sources reviewed: Cedar policy language/authorization documentation and the official Cedar Rust implementation/documentation.

Planning conclusions:
- Cedar is a policy evaluator, not Golam's authority owner.
- Golam owns entity/action/resource schemas and context normalization.
- Policy/schema validation happens before activation.
- Runtime evaluation diagnostics/errors are mapped to fail-closed DENY plus bounded audit evidence.
- Golam hard guards execute before Cedar and cannot be overridden by a `permit`.
- Immutable versioned bundles plus an atomic active pointer provide a tractable durable activation model.
- Policy decisions should record stable Golam reason/rule references rather than exposing unbounded raw evaluator diagnostics as authority semantics.

Unresolved implementation choices, explicitly task-gated:
- exact Cedar crate/version and feature set;
- parser/evaluator resource ceilings and compilation/cache design;
- canonical entity/schema serialization format;
- whether the evaluator remains in `golam-kernel` or is isolated behind a narrower internal module boundary.

## 4. Wasmtime/WASI research

Primary sources reviewed: official Wasmtime security and WASI documentation.

Planning conclusions:
- Wasmtime/WASI is suitable as a candidate for portable bounded extension execution where capabilities are explicitly preopened/provided.
- It is not a universal sandbox for native binaries, browser helpers, MCP servers or language sidecars.
- A declarative sandbox profile must be separate from its platform executor.
- Unsupported containment must deny rather than silently degrade.
- Exact Wasmtime version/dependency is not admitted by planning and should be added only when a bounded implementation task needs the WASM profile.

## 5. Secret-management research

Primary security guidance reviewed includes OWASP Secrets Management guidance plus the binding Golam constitution/contracts.

Planning conclusions:
- minimize plaintext secret lifetime and visibility;
- do not place secrets in command-line arguments;
- avoid ambient inheritance and broad environment propagation;
- log/audit access to handles and policy decisions, not plaintext values;
- version/rotate/revoke secrets through protected state transitions;
- encrypt durable secret values at rest and bind ciphertext to version/identity metadata;
- keep redaction keys and vault key material inside the privileged authority boundary;
- test with deterministic canaries rather than production credentials.

The exact vault cipher, key-encryption-key protection, OS keychain/credential-store abstraction and zeroization crates remain implementation-time dependency/security decisions.

## 6. Taint/information-flow research

The frozen Spec 001 contract is stronger than a conventional trust score: taint is provenance used by policy and survives derivation. Golam therefore uses a monotonic set of labels rather than a scalar confidence value.

Planning conclusions:
- derivation is set-union plus explicitly introduced labels;
- model transformation cannot remove labels;
- deterministic registered authoritative verification may produce a downgrade attestation;
- human approval may authorize a normal downgrade but cannot erase the fact that source material contained a secret;
- `SECRET_DERIVED` requires a deterministic secret-elimination sanitizer to produce a separately evidenced non-secret representation before long-term-memory admission.

## 7. Egress research

The Spec 001 egress contract and Spec 002 no-egress implementation already establish the correct ordering: strict-local is a kernel hard guard, not a policy preference.

Planning conclusions:
- keep the existing hard deny above all policy/lease/approval permit logic;
- non-strict permits are explicit, short/bounded and destination/action/purpose scoped;
- DNS resolution and redirect/rebinding are authorization inputs, not post-authorization implementation details;
- every Golam-managed socket-capable process requires the same kernel authorization contract before receiving network capability.

## 8. Donor search

`TheHalfMoon/Golam-research` live snapshot:
- commit `a9f633e09d49a85829b8236331b9e21f7e612634`;
- tree `b68f24972427952c4934e4364736fec62661044f`.

Repository/code searches for policy/auth/credential/sandbox/security primitives did not reveal a trusted Rust authority substrate appropriate for Spec 003. Its reconstructed Electron/TypeScript behavior may inform interaction/UX later, but it is not an authority implementation donor for this slice.

Decision: `REFERENCE_ONLY`.

## 9. Research stop rule

Research is sufficient for planning. Reopen research only if:
- Cedar fails exact dependency/behavior qualification;
- a secret-storage/key-protection design cannot meet platform requirements;
- a platform sandbox executor cannot enforce a required profile;
- tests reveal a semantic ambiguity not resolved by the frozen contracts.
