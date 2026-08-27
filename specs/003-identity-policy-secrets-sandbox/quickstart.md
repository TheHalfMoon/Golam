# Quickstart — Spec 003 Planned Behavior

**This is a planning target, not currently implemented CLI behavior.** The Spec 003 implementation PR must update this document to exact shipped commands/interfaces and must not claim PASS from this planning text.

## Existing baseline

Spec 002 already provides authenticated local `golamd`/`golam`, protected authority storage, durable effects/recovery, and strict-local no-egress proof.

## Planned local authority workflow

Implementation should expose the smallest testable/admin surface needed to exercise authority without introducing broad product tools. Exact CLI syntax is intentionally deferred, but the following behaviors must be demonstrable:

```text
policy validate <bundle>
policy stage <bundle>
policy activate <bundle-id>       # protected + approved
lease issue <narrow-scope>
lease derive <parent> <narrowing>
lease revoke <lease>
approval grant <class> <scope>
approval revoke <approval>
secret create-canary <classification>
secret rotate-canary <handle>
authority explain <decision>
sandbox profile validate <profile>
```

No real production secret or external provider is required to qualify these paths.

## Expected strict-local behavior

Even with a policy rule, lease, approval and egress permit that otherwise match, external network access in strict-local mode remains denied before the policy permit can become effective.

## Expected secret behavior

Normal diagnostic/listing/explain APIs display opaque handles and metadata only. A deterministic canary secret must not appear in durable logs/events/errors or model-visible history.

## Expected sandbox behavior

A profile that requires a containment feature unavailable on the current platform is rejected before process launch. A profile definition alone is not reported as sandbox proof.

## Evidence commands

Implementation retains the pinned Rust exact-head gates:

```bash
cargo +1.98.0 fmt --all -- --check
cargo +1.98.0 clippy --locked --workspace --all-targets -- -D warnings
cargo +1.98.0 test --locked --workspace --all-targets
```

It must add focused policy/lease/approval/taint/secret/egress/sandbox qualification without weakening the existing property/fuzz/IPC/adversarial/no-network gates.
