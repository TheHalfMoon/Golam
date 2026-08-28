# T003-042 — Verifier Registry Qualification

**Task**: T003-042  
**Qualified head**: `67f74c9b9b75e43b9fa00069050c97c041567184`  
**CI**: #373 / run `33152187952`  
**Result**: PASS on Windows, macOS, and Ubuntu

## Scope qualified

T003-042 implements the protected verifier/sanitizer registry without widening into downgrade-attestation execution.

The qualified implementation:

- reuses the existing protected `verifier_rules` schema; no migration or new dependency was required;
- admits only the frozen rule kinds `deterministic_verifier` and `secret_elimination_sanitizer`;
- bounds and canonicalizes rule version, authority-source binding, allowed downgrade labels, principal, rule ID, registration provenance digest, resource, and mutation intent digest;
- rejects empty or malformed authority-source bindings and empty downgrade sets;
- rejects registration when its provenance is empty or contains untrusted/generated/secret-derived labels, preventing a tainted source from manufacturing authority to downgrade itself or upstream provenance;
- requires an exact current durable authorization decision for `verifier.register` and the exact verifier-rule resource;
- requires an exact authorized at-most-once elevated effect with the taint-authority-mutation risk class and exact registration intent digest;
- requires an exact ONCE approval bound to effect, action, resource, risk class, and registration taint digest;
- consumes that ONCE approval atomically with the protected registry mutation;
- appends fresh `authority-security-v2` coverage for the verifier rule and approval consumption before commit;
- verifies canonical ledger and authority-security integrity before and after the mutation;
- does not expose an API that performs a downgrade or mutates source provenance in place.

## Adversarial evidence

Focused tests demonstrate that `LOCAL_UNVERIFIED`, `WEB_UNTRUSTED`, `CHANNEL_UNTRUSTED`, `MCP_UNTRUSTED`, `PLUGIN_UNVERIFIED`, `MODEL_GENERATED`, and `SECRET_DERIVED` registration provenance cannot prepare downgrade authority. Empty registration provenance is also denied.

A trusted registration test proves the accepted path is exact-decision/effect/approval bound, persists the declared allowed downgrade set, consumes the one-shot approval, and remains valid under authority-store integrity verification.

## Exact-head CI evidence

CI #373 / run `33152187952` completed successfully at exact head `67f74c9b9b75e43b9fa00069050c97c041567184`.

All three platform jobs succeeded. The run covered formatting, Clippy, workspace tests, property qualification, bounded fuzz smoke, authenticated IPC, adversarial authority qualification, daemon build, and the platform-appropriate strict-local external network observation. Unix/Windows transport and observation steps were skipped only where not applicable to the runner platform.

## Scope boundary

T003-042 does not create downgrade attestations, alter artifact provenance, implement long-term-memory rejection, or execute the secret-elimination sanitizer. Those remain owned by T003-043 through T003-045.

`T003-042=PASS` is historical task qualification evidence only after subsequent branch mutation; it is not final Spec 003 exact-head closeout evidence.