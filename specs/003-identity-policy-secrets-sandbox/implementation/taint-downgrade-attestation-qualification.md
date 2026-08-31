# T003-043 — Taint Downgrade Attestation Qualification

**Task**: T003-043  
**Qualified implementation head**: `2f8655b5bdddd17bb9e6eab7bf00f11a210896cb`  
**CI**: #388 / run `33154505847`  
**Result**: PASS on Windows, macOS and Ubuntu

## Qualified behavior

- Normal taint downgrade creates a new `TaintAttestation`/derived artifact evidence record; source artifact IDs and source-label bytes are retained and source provenance is never rewritten in place.
- Preparation requires non-empty source labels, a strict result-label subset, at least one removed label, bounded unique source artifact IDs, a distinct result artifact ID, canonical principal input and non-empty evidence.
- Human downgrade requires an exact current `taint.downgrade` allow decision, exact authorized at-most-once protected effect and exact ONCE approval bound to effect/action/resource/risk/source-taint digest. Approval consumption is committed atomically with the attestation and receives authority-security coverage.
- Deterministic verifier downgrade requires the same protected decision/effect plus an active registered `deterministic_verifier` rule whose authority-source binding matches exactly and whose canonical allowed-downgrade set covers every removed label.
- Protected verifier-rule downgrade sets are decoded fail-closed through strict canonical `TaintSet` decoding; wrong domain/order/duplicates/unknown codes/trailing bytes are rejected rather than normalized.
- Deterministic verifier authority crosses a typed `DeterministicVerifierEvidence` boundary instead of loosely ordered positional authority fields.
- Unregistered, inactive, wrong-kind, wrong-binding or insufficient verifier authority fails closed.
- Human and deterministic normal downgrade paths reject removal of `SECRET_DERIVED`; that label is reserved for the separately authorized deterministic secret-elimination sanitizer path in T003-045.
- Newly inserted taint attestations receive authority-security authenticated integrity before transaction commit and the authority-security chain is reverified transactionally.

## Qualification history

- CI #381 / run `33153295456`: stopped at rustfmt; formatter-only repair applied.
- CI #384 / run `33153791707`: formatting passed; Clippy identified an eight-argument public deterministic-verifier constructor.
- The public API was repaired structurally with typed `DeterministicVerifierEvidence`; the lint was not suppressed.
- CI #387 / run `33154138384`: Format/Clippy passed; the deterministic-verifier test exposed a fixture migration-order defect (`verifier_rules` table not initialized).
- The fixture was repaired to initialize `AuthorityStore` migrations before direct verifier-row insertion.
- CI #388 / run `33154505847`: full SUCCESS on Windows, macOS and Ubuntu, including format, Clippy, workspace tests, property qualification, bounded fuzz smoke, authenticated IPC, adversarial authority qualification and strict-local external observation.

## Scope boundary

This task does not implement a canonical long-term-memory product sink, secret-elimination sanitizer execution, secret vault/broker, egress permits or sandbox profiles. Those remain owned by later tasks in `tasks.md`.

```text
T003_043=PASS
T003_043_QUALIFIED_HEAD=2f8655b5bdddd17bb9e6eab7bf00f11a210896cb
T003_043_CI_RUN=33154505847
NEXT_TASK=T003-044
```
