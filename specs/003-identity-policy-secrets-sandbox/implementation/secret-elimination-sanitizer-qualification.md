# T003-045 — Deterministic Secret-Elimination Sanitizer Qualification

**Task**: T003-045  
**Qualified implementation head**: `e3b91dcecf0048b183c4c333cd9afda43ee25671`  
**CI**: #398 / run `33155929307`  
**Result**: PASS on Windows, macOS and Ubuntu

## Qualified behavior

- `TaintDowngradeMechanism` now includes the data-model-reserved `secret_elimination_sanitizer` mechanism without a schema migration.
- Sanitizer execution has a distinct protected action, `taint.secret_eliminate`, rather than inheriting normal downgrade authority implicitly.
- `SecretEliminationSanitizerEvidence` is a typed authority boundary carrying exact registered rule ID, authority-source binding and evidence hash.
- Sanitizer preparation requires source provenance containing `SECRET_DERIVED`, a distinct result artifact, a strict subset result label set and a result that no longer contains `SECRET_DERIVED`.
- Human and normal deterministic-verifier downgrade paths still cannot remove `SECRET_DERIVED`.
- Commit requires a current exact allow decision and exact authorized at-most-once protected effect for the sanitizer action/resource/intent.
- The registered rule must be active, kind `secret_elimination_sanitizer`, exactly source-bound, and its canonical allowed-downgrade set must cover every removed label.
- A normal deterministic verifier cannot impersonate a sanitizer, and a sanitizer rule cannot silently remove unrelated labels outside its registered downgrade scope.
- The source artifact/labels remain unchanged and auditable in the attestation; only the separately evidenced result artifact may be non-secret-derived.
- A successful result passes the T003-044 canonical long-term-memory sink guard because its own provenance no longer contains `SECRET_DERIVED`; the original source remains rejected.
- The attestation continues to receive authority-security authenticated integrity before transaction commit.

## Qualification history

- CI #397 / run `33155748946` stopped at rustfmt before Clippy/tests; formatter-only changes were applied.
- CI #398 / run `33155929307` completed SUCCESS on Windows, macOS and Ubuntu, including format, Clippy, workspace tests, property qualification, bounded fuzz smoke, authenticated IPC, adversarial authority qualification and strict-local external observation.

## Scope boundary

This task adds only the deterministic secret-elimination evidence path. It does not implement a vault, plaintext secret handling, memory product, broad sanitizer framework or later Phase F secret broker.

```text
T003_045=PASS
T003_045_QUALIFIED_HEAD=e3b91dcecf0048b183c4c333cd9afda43ee25671
T003_045_CI_RUN=33155929307
NEXT_TASK=T003-046
```
