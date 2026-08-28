# T003-040 — Taint Baseline Qualification

**Task**: T003-040  
**Qualified head**: `cb69d638107ca4fe0118c9a61f143ac3ba65a2d3`  
**CI**: #359 / run `33150969442`  
**Result**: PASS

## Implemented scope

T003-040 implements only the Spec 003 baseline taint-label representation and deterministic canonical encoding.

Implemented in `crates/golam-core/src/taint.rs`:

- exact closed baseline labels:
  - `USER_TRUSTED`
  - `LOCAL_TRUSTED`
  - `LOCAL_UNVERIFIED`
  - `WEB_UNTRUSTED`
  - `CHANNEL_UNTRUSTED`
  - `MCP_UNTRUSTED`
  - `PLUGIN_UNVERIFIED`
  - `MODEL_GENERATED`
  - `SECRET_DERIVED`
- frozen numeric codes `1..=9`;
- reverse code lookup;
- bounded `TaintSet` representation;
- duplicate-insensitive construction;
- deterministic fixed-code ordering;
- domain-separated canonical bytes under `golam:taint-label-set:v1`;
- explicit empty-set representation.

This task intentionally does **not** implement propagation, downgrade, verifier registration, sanitizer authority, long-term-memory admission, or taint persistence. Those remain owned by T003-041..T003-046.

## Focused verification

Unit tests prove:

- every normative baseline label has the frozen name and code;
- invalid label codes are rejected by lookup;
- duplicate/caller-order differences do not alter the set;
- canonical bytes are order/duplicate invariant;
- empty encoding is explicit;
- the full baseline remains bounded to nine labels.

## Exact-head CI evidence

CI #359 / run `33150969442` completed SUCCESS at exact head `cb69d638107ca4fe0118c9a61f143ac3ba65a2d3`.

All supported-platform jobs completed successfully:

- Windows: format, clippy, tests, property qualification, bounded fuzz smoke, Windows IPC, authenticated daemon IPC, adversarial authority qualification, daemon build, strict-local Windows external observation;
- macOS: format, clippy, tests, property qualification, bounded fuzz smoke, Unix IPC, authenticated daemon IPC, adversarial authority qualification, daemon build, strict-local Unix external observation;
- Ubuntu: format, clippy, tests, property qualification, bounded fuzz smoke, Unix IPC, authenticated daemon IPC, adversarial authority qualification, daemon build, strict-local Unix external observation.

Platform-inapplicable Unix/Windows mirror steps were skipped as expected.

## Gate conclusion

```text
T003_040=PASS
T003_040_QUALIFIED_HEAD=cb69d638107ca4fe0118c9a61f143ac3ba65a2d3
T003_040_CI_RUN=33150969442
NEXT_TASK=T003-041
```

Any later branch mutation makes CI #359 historical task evidence only; it is not final Spec 003 exact-head closeout evidence.