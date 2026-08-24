# Implementation Readiness Checklist — Spec 002

## Scope

- [x] Model-free durable kernel slice is explicit.
- [x] <=7 initial crates/binaries defined.
- [x] Out-of-scope future systems are explicit.
- [x] Golam-Research is treated as serious implementation evidence without forcing TypeScript architecture.

## Security boundary

- [x] Privileged KernelApi is process-splittable.
- [x] Protected state enumeration exists.
- [x] Local IPC transport + cryptographic enrollment are specified.
- [x] No localhost HTTP control plane.
- [x] Bootstrap authorization is deny-by-default.
- [x] Strict-local egress interface is deny-by-default.

## Durability

- [x] Global/per-session ordering defined.
- [x] Fork semantics defined.
- [x] Goal versions append.
- [x] Checkpoints do not replace canonical history.
- [x] Effect handler/reconciler contract defined.
- [x] Intent-before-dispatch defined.
- [x] Unknown outcome/dependency blocking defined.
- [x] Authority corruption fails closed.
- [x] Disk-full ambiguity is addressed/testable.

## Donor/provenance

- [x] Exact planning snapshots recorded for Golam-Research, grok-build, DeepSeek Harness and Goose.
- [x] Founder permission posture recorded.
- [x] No donor code admitted by planning package.
- [ ] Per-file Source Foundry admission record exists for any source actually copied/ported during implementation.

## Verification plan

- [x] Unit/property/fuzz/fault/platform tests defined.
- [x] Unauthenticated local process probe defined.
- [x] Kernel-boundary compromise probe defined.
- [x] Duplicate-effect crash suite defined.
- [x] No-listener/no-egress checks defined.
- [x] Corruption/checkpoint/disk-full tests defined.

## Gate

**READY_FOR_TASK_EXECUTION_AFTER_PLANNING_PR_REVIEW/MERGE**

No Rust implementation belongs in this planning PR.
