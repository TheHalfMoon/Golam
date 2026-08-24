# Quickstart — Spec 002 Target Behavior

This is an implementation target, not proof that commands exist yet.

## After implementation

```bash
golamd --foreground
```

First local CLI enrollment:

```bash
golam client enroll
```

Create a model-free session:

```bash
golam session create --goal "prove durable session recovery"
```

Append deterministic test event:

```bash
golam session event append <SESSION> --type TestObservation --json '{"value":1}'
```

Create checkpoint:

```bash
golam session checkpoint <SESSION>
```

Verify replay:

```bash
golam session replay <SESSION> --verify
```

Fork:

```bash
golam session fork <SESSION> --through-seq 3
```

Run synthetic effect fault scenario:

```bash
golam dev effect-sim run irreversible-ambiguous --crash-after remote-accept
```

Restart daemon and reconcile:

```bash
golam effect list --state unknown-outcome
golam effect reconcile <EFFECT>
```

Inspect recovery/ledger health:

```bash
golam doctor --ledger --ipc --recovery
```

## Required evidence commands in implementation PR

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Plus property/fuzz/fault-injection/platform IPC suites defined by `tasks.md`.

Do not claim any of these pass until executed on the exact implementation head.
