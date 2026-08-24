# Implementation Readiness Checklist

## Specification

- [x] Product North Star is explicit.
- [x] Rust-first trusted-path constraint is explicit.
- [x] Strict-local behavior is explicit.
- [x] Desktop + CLI shared-daemon model is explicit.
- [x] Full computer-control requirement is explicit.
- [x] GolamConnect native remote-control requirement is explicit.
- [x] Third-party channel trust boundary is explicit.
- [x] Grok public feature/skill parity is explicit.
- [x] Memory ownership model is explicit.

## Architecture

- [x] Session/Harness/Sandbox are separated.
- [x] Trusted kernel boundary is identified.
- [x] Event/Goal ledgers are defined.
- [x] Effect transaction/idempotency model is defined.
- [x] Identity/capability/policy model is defined.
- [x] Secret broker boundary is defined.
- [x] Taint/information-flow requirement is defined.
- [x] ExecutionProfile model is defined.
- [x] Context Compiler model is defined.
- [x] Markdown/SQLite memory split is defined.
- [x] Skills supply-chain lifecycle is defined.
- [x] Semantic-first computer-control hierarchy is defined.
- [x] Connect pairing/transport/control boundary is defined.
- [x] Program decomposed into bounded follow-on specs.

## Donor/research governance

- [x] Golam-Research is reference-only by default.
- [x] Donor qualification process is defined.
- [x] Reciprocal-license projects are reference-only by default.
- [x] Generic framework discovery has a stop rule.
- [ ] Exact qualification records exist for dependencies selected by implementation specs.

## Verification

- [x] Unit/property/fuzz/integration/platform strategy defined.
- [x] Long-horizon/recovery/idempotency benchmarks required.
- [x] Exact-head evidence rule defined.
- [ ] GLM 5.3 external architecture review completed.
- [ ] All GLM BLOCKER findings resolved.
- [ ] GLM MAJOR findings incorporated or founder-waived.
- [ ] Founder freezes Spec 001.
- [ ] `tasks.md` generated from frozen artifacts.
- [ ] Spec Kit `analyze` reports no critical inconsistency before implementation.

## Current decision

**NOT READY FOR IMPLEMENTATION**

Blocking gate: external GLM 5.3 review requested by founder is not yet completed.
