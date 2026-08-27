# T003-003 — Cedar Dependency Qualification

**Decision**: `ADMITTED_EXACT_VERSION`  
**Qualified dependency**: `cedar-policy = "=4.12.0"` with `default-features = false`

## Exact source

- upstream: `https://github.com/cedar-policy/cedar`
- release tag: `v4.12.0`
- tag commit: `fdcbaed32bdb8c8d13e4eaf2b58db5555e9fb8c5`
- release date: 2026-07-28
- crate: `cedar-policy 4.12.0`
- exact internal crate dependencies include `cedar-policy-core =4.12.0` and `cedar-policy-formatter =4.12.0`
- license: Apache-2.0
- upstream MSRV: Rust 1.89
- Golam workspace Rust: 1.98

## Feature decision

Golam admits the base Cedar authorizer/parser/validator only:

```toml
cedar-policy = { version = "=4.12.0", default-features = false }
```

Do not enable:

- `experimental` or any experimental sub-feature;
- `tpe` / partial evaluation;
- protobuf serialization;
- tolerant AST parsing;
- WASM bindings;
- heap profiling/corpus timing.

The default `ipaddr`, `decimal`, and `datetime` extension set is intentionally disabled. Golam owns egress/IP normalization and does not delegate hard network semantics to a Cedar extension function.

## Security / authority boundary

Upstream Cedar 4.12.0 declares `unsafe_code = "forbid"` at its workspace level. Cedar policies are effect-free and cannot perform filesystem/network I/O. This is appropriate for an evaluator inside the trusted Rust path, but Cedar is not Golam's authority owner.

Golam retains ownership of:

- authenticated principal construction;
- normalized action/resource/context construction;
- hard kernel denials;
- capability leases and revocation;
- approval semantics;
- protected-resource classification;
- egress and secret rules;
- durable decision/audit semantics.

No untrusted text may directly construct a Golam authority-bearing type.

## Critical semantic adaptation: fail closed on Cedar diagnostics

Cedar authorization is default-deny and forbid-overrides-permit, but its documented policy-evaluation behavior is **skip-on-error**: a policy that errors is omitted from the final Cedar decision and errors are returned in diagnostics.

Golam deliberately strengthens this boundary. For every authorization evaluation:

1. construct a bounded normalized Cedar request from trusted Golam types;
2. require a validated active schema/policy bundle;
3. evaluate Cedar;
4. inspect authorization diagnostics;
5. if **any** policy evaluation error/diagnostic exists, map the Golam result to `DENY` with a bounded error reason;
6. only accept a Cedar `Allow` when diagnostics are error-free and all earlier Golam gates already passed.

Thus Cedar skip-on-error can never widen Golam authority.

## Input/resource bounds

Cedar itself does not impose application policy/schema/request size limits. Golam must bound before parsing/evaluation:

- policy bundle bytes and policy count;
- schema bytes;
- entity count and serialized size;
- principal/action/resource identifier lengths;
- context field count/depth/string/collection sizes;
- diagnostic count/text retained in audit.

Malformed, oversized, unsupported, or validation-failing input is denied before activation/use.

## Supply-chain posture

Direct dependency admission is exact-version only. Cargo.lock changes must be reviewed for the complete resolved transitive closure. No build/runtime network is authorized by this dependency. Any later Cedar version change reopens this qualification task.

```text
T003_003=PASS
CEDAR_POLICY_VERSION=4.12.0
CEDAR_RELEASE_COMMIT=fdcbaed32bdb8c8d13e4eaf2b58db5555e9fb8c5
CEDAR_DEFAULT_FEATURES=OFF
CEDAR_EXPERIMENTAL_FEATURES=OFF
CEDAR_EVALUATION_ERRORS=GOLAM_DENY
```
