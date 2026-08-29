# Quickstart — Spec 003 Authority Qualification Surface

The Spec 003 implementation exposes a deliberately small authenticated local CLI/admin/test surface. It is for qualification and explanation; it does **not** create a generic protected-authority mutation console.

## Existing baseline

Spec 002 provides authenticated local `golamd`/`golam`, protected authority storage, durable effects/recovery, and strict-local no-egress proof. Spec 003 keeps all protected mutations behind typed `KernelApi` methods with exact authorization/effect/approval evidence.

## Exact T003-081 CLI surface

```text
golam policy validate <policy-source> <schema-source>
golam authority qualify lease
golam authority qualify approval
golam authority qualify secret-canary
golam authority qualify sandbox-profile
golam authority explain <decision-id-hex>
```

The policy/schema arguments are bounded inline Cedar source strings intended for deterministic admin/test qualification. Shell quoting is required when the source contains spaces or punctuation.

All commands travel over the existing authenticated local IPC path. `policy.validate`, `authority.qualify`, and `authority.explain` are non-mutating. In bootstrap state they are the only new read/qualification actions admitted for an authenticated enrolled local client; policy staging/activation, lease issuance/revocation, approval issuance/revocation, secret mutation, and sandbox-profile registration are **not** granted by this CLI surface.

## Protected mutation boundary

The planning examples below remain implemented as typed kernel lifecycle APIs, not as free-form CLI shortcuts:

- policy stage/activate require current policy authority and exact protected lifecycle evidence;
- capability lease issue/derive/revoke remains kernel-minted, non-self-expanding, and exact-evidence-bound;
- approval issue/revoke/use remains protected and class/scope/effect bound;
- secret create/rotate/revoke remains protected and never accepts production plaintext through this qualification CLI;
- sandbox-profile registration remains protected and requires exact decision/effect/ONCE-approval evidence.

T003-081 intentionally does not fabricate the decision/approval/effect tuples those mutation paths require.

## Qualification behavior

### Policy

`policy validate` runs the same bounded strict Cedar candidate parser/schema validator used before policy staging. It does not stage or activate the candidate.

### Lease

`authority qualify lease` exercises canonical lease-scope normalization and a strict child narrowing. The returned receipt is evidence only and is not a `CapabilityLease`.

### Approval

`authority qualify approval` constructs and canonically digests a bounded ONCE approval scope. It does not issue an approval.

### Secret canary

`authority qualify secret-canary` sends no caller-supplied secret. A fixed unknown-format deterministic canary exists only inside the ledger qualification module, enters the same explicit designated-secret preparation path, is never committed or returned, and is dropped through the zeroizing protected owner. T003-093 remains the full durable leakage suite.

### Decision explain

`authority explain` returns bounded stored authorization evidence: principal, action, resource, context hash, hard-guard result, lease/policy/approval identifiers, matched policy rule IDs, decision/reason, sequence and evidence version. It does not return secret plaintext or raw authorization context.

### Sandbox profile

`authority qualify sandbox-profile` validates and canonicalizes a fixed deny-all, no-spawn, empty-inheritance native profile and returns non-authority intent evidence. It neither registers the profile nor launches a process and is not containment proof.

## Expected strict-local behavior

Even with a policy rule, lease, approval and egress permit that otherwise match, external network access in strict-local mode remains denied before the policy permit can become effective.

## Expected secret behavior

Normal diagnostic/listing/explain APIs display opaque identifiers and metadata only. The deterministic canary must not appear in durable logs/events/errors or model-visible history.

## Expected sandbox behavior

A profile that requires a containment feature unavailable on the current platform is rejected before process launch. A profile definition or qualification receipt alone is not reported as sandbox proof.

## Evidence commands

Implementation retains the pinned Rust exact-head gates:

```bash
cargo +1.98.0 fmt --all -- --check
cargo +1.98.0 clippy --locked --workspace --all-targets -- -D warnings
cargo +1.98.0 test --locked --workspace --all-targets
```

Focused T003-081 qualification additionally covers the command codec, CLI parser, kernel admin boundary, daemon router, unauthenticated/denied ordering and bootstrap mutation denial.
