# Quickstart for Future Implementers

This is a planning quickstart, not authorization to implement yet.

## 1. Check the gate

Read `review/finalization-status.md`. Do not create product code while status is `PENDING_EXTERNAL_GLM_5_3_REVIEW`.

## 2. Read authoritative artifacts

Follow root `AGENTS.md` read order. The constitution is a hard gate.

## 3. After review approval

The next planning action is to incorporate accepted GLM findings, mark the package `READY_FOR_TASK_GENERATION`, then generate `tasks.md` using current Spec Kit. Run Spec Kit analysis before implementation.

## 4. First implementation target

Do not start with Desktop UI, remote control, Graphify, or model marketplace. Start with follow-on Spec 002: Rust workspace + deterministic kernel/session/effect spine.

A minimal first executable proof should eventually demonstrate:

```text
local client
  -> golamd
  -> create session
  -> persist event
  -> set goal
  -> propose synthetic effect
  -> authorize/deny deterministically
  -> persist receipt
  -> restart daemon
  -> replay same canonical state
```

No model is required to prove the first durability/security spine.

## 5. Default Rust quality gates

When implementation is authorized:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Add `cargo deny`, fuzz/property/platform gates once dependencies and protocol surfaces exist.

## 6. Source admission

Never copy a donor implementation directly from a research note. Open a qualification record first, pin exact source state, and prove license/dependency/security fit.
