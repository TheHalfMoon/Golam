# Quickstart — Spec 002 Implemented Behavior

This document describes the bounded Spec 002 CLI/daemon surface implemented on PR #3. It does not imply later product ergonomics, models, broad tools, or external effects.

## Start the daemon

```bash
golamd --foreground
```

The daemon starts privileged service only when authority recovery state is `Normal`. `RecoveryOnly` and `Quarantined` startup states fail closed and the daemon reports the recovery mode/issues instead of exposing a privileged recovery bypass.

## First local CLI enrollment

Choose a decimal client ID and pre-create/use the protected local credential through the CLI bootstrap flow:

```bash
golam client enroll 1
```

First enrollment requires explicit foreground approval by `golamd`; later connections authenticate through the enrolled Ed25519 credential plus OS-local peer checks.

## Session operations

Create a model-free session using explicit decimal IDs, an ISO-like recorded-at string, and a bounded payload:

```bash
golam session create 100 1000 2026-08-26T10:00:00Z root
```

List/open sessions:

```bash
golam sessions
golam session open 100
```

Append a typed goal version:

```bash
golam goal append 2000 200 1001 100 1 0 2026-08-26T10:01:00Z prove-durable-session-recovery
```

Spec 002 intentionally does **not** expose a generic caller-selected `session event append --type ...` primitive. Reserved canonical event families are emitted by their owning typed KernelApi domain operations so a client cannot forge checkpoint/effect/goal lifecycle evidence without the corresponding protected record.

## Checkpoint, replay, and fork

```bash
golam checkpoint create 300 1002 100 2 2026-08-26T10:02:00Z
golam checkpoint verify 300 100 2
golam replay 100 2
golam session fork 101 1003 100 2 2026-08-26T10:03:00Z
```

## Synthetic effect qualification

The only Spec 002 effects are deterministic local simulators. Supported semantics are:

```text
read-only
idempotent-at-least-once
at-most-once
compensatable
irreversible
```

Example:

```bash
golam effect simulate 400 100 irreversible
golam effect reconcile 400
```

No real external action or network provider is invoked.

## Recovery / health report

While the daemon is in normal serving mode:

```bash
golam doctor
```

`golam doctor` crosses authenticated local IPC and returns the kernel recovery report. If startup is blocked in `RecoveryOnly` or `Quarantined`, the privileged daemon does not expose an unauthenticated diagnostic control plane; the startup failure/report is the fail-closed recovery evidence for that state.

## Required evidence commands

The CI matrix runs the pinned Rust toolchain equivalents of:

```bash
cargo +1.98.0 fmt --all -- --check
cargo +1.98.0 clippy --locked --workspace --all-targets -- -D warnings
cargo +1.98.0 test --locked --workspace --all-targets
```

It also runs the deterministic property corpus, bounded fuzz smoke, platform IPC qualification, authenticated-daemon/adversarial probes, and externally observed strict-local no-network gate defined by `tasks.md`.

Never treat this document as proof by itself; exact-head GitHub Actions evidence is required for PASS claims.
