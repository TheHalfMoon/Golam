# T003-046 Taint Adversarial Qualification

## Qualified task

`T003-046` — multi-hop, self-clear, unregistered-verifier, and `SECRET_DERIVED` property/adversarial qualification.

## Exact qualification evidence

- Branch: `impl/003-identity-policy-secrets-sandbox`
- Qualified exact head: `890571fe705f36f42c1c20acff3a8a2c4fa3498e`
- CI workflow: `ci`
- CI run number: `405`
- CI run ID: `33157139728`
- Result: `SUCCESS`
- Platforms: Windows, macOS, Ubuntu

The successful exact-head run includes pinned formatting, Clippy, workspace tests, property qualification, bounded fuzz smoke, platform IPC qualification, authenticated daemon IPC qualification, adversarial authority qualification, daemon build, and supported-platform strict-local external network observation.

## Security properties qualified

The bounded T003-046 test wave is wired into `golam-ledger` only under `#[cfg(test)]` and adds no production authority or dependency.

It proves:

1. the canonical long-term-memory admission guard is exhaustive across all 512 combinations of the nine frozen taint labels and rejects exactly every combination containing `SECRET_DERIVED`;
2. multi-hop derivation preserves the union of upstream and introduced provenance, including `SECRET_DERIVED`, across later transformations;
3. the normal human downgrade and deterministic-verifier downgrade preparation paths cannot clear `SECRET_DERIVED` even when callers supply apparently authoritative evidence;
4. an unregistered secret-elimination sanitizer cannot commit even when an exact protected effect and allow decision exist;
5. a successful registered sanitizer produces a distinct result artifact and leaves the original source artifact and source labels unchanged and still inadmissible to canonical long-term memory;
6. the dedicated `taint.secret_eliminate` protected action cannot be replaced by the normal `taint.downgrade` action;
7. all protected verifier/sanitizer and taint-attestation state continues to pass authority-store integrity and `authority-security` verification after the tested mutations.

## Scope boundary

This task does not add memory product implementation, retrieval, model behavior, new verifier authority, secret vault behavior, egress behavior, or sandbox execution. It closes only the Phase E adversarial/property qualification required by the canonical Spec 003 task order.

## Result

`T003-046=PASS`

With T003-040 through T003-046 all qualified at their recorded exact heads, Phase E is complete. The next canonical eligible task is `T003-050` in Phase F.
