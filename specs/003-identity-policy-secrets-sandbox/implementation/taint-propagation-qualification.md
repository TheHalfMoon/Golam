# T003-041 — Taint Propagation Qualification

**Task**: T003-041  
**Qualified head**: `76e1addf35c92a22d2c5826ca429278cacd598b3`  
**CI**: #366 / run `33151556481`  
**Result**: PASS

## Implemented scope

T003-041 adds monotonic taint composition without introducing any downgrade authority.

### Core provenance primitives

`golam-core::taint` now provides:

- `TaintSet::union` — bitwise monotonic union;
- `TaintSet::contains_all` — invariant verification;
- `Provenanced<T>` — provenance carried beside a value without changing the value's own identity/canonical bytes;
- `Provenanced::source` — explicit source/trust-boundary construction;
- `Provenanced::derive` — `union(all source labels) ∪ transform-introduced labels`.

There is intentionally no taint removal/downgrade method in this task.

### Derived artifact boundary

Ledger integration tests wrap `ArtifactReceipt` in `Provenanced<ArtifactReceipt>` and prove:

- content hash/path identity remains unchanged by provenance;
- all source labels survive derivation;
- transform-introduced `MODEL_GENERATED` survives;
- source order does not change the resulting taint or canonical taint encoding.

This keeps artifact content identity separate from provenance authority.

### Authority-context boundary

Kernel integration tests wrap `AuthorizationContext` in `Provenanced<AuthorizationContext>` and prove:

- the existing scope/safety identity is not overloaded with taint text;
- source provenance is preserved monotonically;
- transform-introduced provenance is preserved;
- source order does not change canonical taint representation.

Normal policy-engine consumption of this typed provenance remains a later integration responsibility; T003-041 establishes the monotonic carrier and propagation semantics without widening the stable authorization seam.

## Exact-head CI evidence

CI #366 / run `33151556481` completed SUCCESS at exact head `76e1addf35c92a22d2c5826ca429278cacd598b3`.

Windows, macOS and Ubuntu each passed all applicable gates:

- format;
- clippy;
- workspace tests;
- property qualification;
- bounded fuzz smoke;
- platform IPC qualification;
- authenticated daemon IPC qualification;
- adversarial authority qualification;
- daemon build for external locality observation;
- strict-local external network observation.

Platform-inapplicable mirror steps were skipped as expected.

## Gate conclusion

```text
T003_041=PASS
T003_041_QUALIFIED_HEAD=76e1addf35c92a22d2c5826ca429278cacd598b3
T003_041_CI_RUN=33151556481
NEXT_TASK=T003-042
```

Any later branch mutation makes CI #366 historical task evidence only; it is not final Spec 003 exact-head closeout evidence.