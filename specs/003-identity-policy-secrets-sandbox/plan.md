# Implementation Plan — Spec 003 Identity, Policy, Secrets & Sandbox

**Branch**: `spec/003-identity-policy-secrets-sandbox`  
**Base**: `main@a04756f242e48faeda802e5b3fd99a0c8d52f53e`  
**Status**: PLANNED — NO PRODUCT IMPLEMENTATION IN THIS PLANNING PR

## Summary

Extend the closed-canonical Spec 002 kernel spine with explicit identity/capability leases, a fail-closed policy evaluator, durable step-up approvals, taint information-flow state, an encrypted secret-handle vault/broker boundary, policy-governed egress permits beneath the existing strict-local hard denial, and declared sandbox profiles/admission plans.

No new authority architecture is introduced. The work fills the authority families reserved by Spec 001 and preserves Spec 002 durability, audit, IPC and effect semantics.

## Constitution check

| Gate | Result |
|---|---|
| Local ownership / strict-local | PASS_SPEC — authority is local; strict-local hard denial remains above policy |
| Rust trusted path | PASS_SPEC — all authority product code remains Rust-first |
| Small privileged kernel | PASS_SPEC — uses existing KernelApi, no framework-owned authority |
| Explicit least privilege | PASS_SPEC — principal/action/resource/context + narrowing leases |
| Protected resources | PASS_SPEC — typed elevated mutation only |
| Durable effects | PASS_SPEC — authority mutations integrate existing effect/audit invariants |
| Secret safety | PASS_SPEC — handles/broker/fallback/redaction/vault contracts frozen |
| Taint | PASS_SPEC — monotonic labels and auditable downgrade/sanitization |
| Sandbox honesty | PASS_SPEC — profile != containment, unsupported enforcement fails closed |
| Source governance | PASS — no donor/dependency admitted by planning |
| Verification | PASS_SPEC — adversarial/property/crash/canary/no-egress gates enumerated |

## Preserve the seven-package spine

Spec 003 starts inside the existing package structure:

```text
crates/
  golam-core
  golam-ledger
  golam-effects
  golam-ipc
  golam-kernel
  golamd
  golam
```

Do not create `golam-policy`, `golam-secrets`, or `golam-sandbox` merely because the architecture names those domains. Split only if implementation evidence demonstrates a real independent ownership/testing boundary that does not widen authority.

## Layered authorization pipeline

```text
request
  |
  v
hard kernel guards
  | deny => durable DENY
  v
authenticated principal
  | missing/invalid => DENY
  v
capability lease
  | expired/revoked/out-of-scope => DENY
  v
policy evaluator (Cedar candidate)
  | invalid/error/forbid/no permit => DENY
  v
step-up approval (when required)
  | stale/mismatch/consumed => DENY
  v
typed protected mutation / existing Effect Gate
```

Rules:
- denial is monotonic;
- policy cannot override strict-local or other hard safety guards;
- approval cannot manufacture authority not present in lease/policy;
- every layer has a bounded reason class and durable security evidence where appropriate;
- external/model/parser data is data, never authority material.

## Policy architecture

### Golam-owned schema

Golam owns normalized principal/action/resource/context types. The evaluator receives only bounded canonical input assembled by trusted code.

### Bundle lifecycle

```text
candidate -> parse -> schema validate -> policy validate -> hash -> stage
        -> protected approval/effect -> atomic activate -> active
```

- immutable bundle ID/version/hash;
- active pointer is protected state;
- activation records prior and new active bundle;
- startup verifies active pointer, bundle hash/schema and required security-audit coverage;
- invalid/corrupt active policy enters fail-closed recovery behavior, never a permissive bootstrap fallback;
- initial/recovery bootstrap authority is narrowly defined local-owner administration only.

## Capability leases

Lease authority is explicit sets/predicates over action/resource/context plus expiry/revocation/generation.

Derivation:

```text
child_authority = parent_authority ∩ requested_narrowing
```

Any requested widening fails. Use-time validation checks principal binding, parent/revocation chain, expiry, context and action/resource scope.

Exact token cryptographic representation is deferred to implementation qualification; security semantics do not depend on exposing a bearer string to untrusted components.

## Approvals

Approval classes:
- ONCE;
- SESSION_SCOPED;
- TIME_BOXED;
- OPERATION_PATTERN;
- RUN_PREAUTHORIZATION.

Approval scope includes action/resource or exact effect/pattern, risk, relevant context digest and taint summary, issue/expiry, limits and parent decision context.

ONCE consumption uses a durable atomic reservation/consumption record so concurrent or retrying callers cannot execute twice.

## Protected mutations

New protected families:
- policy/schema/active bundle;
- principal metadata beyond Spec 002 enrollment identity;
- capability leases/revocations;
- approvals/consumption;
- secret records/versions/redaction keys;
- taint verifier/sanitizer registrations/downgrade attestations;
- egress mode/permits;
- sandbox profile definitions.

Every mutation is typed KernelApi work, authorized under current authority, durably journaled and included in `authority-security` integrity coverage. Generic filesystem access cannot mutate these records.

## Taint algebra

Baseline set from Spec 001:

```text
USER_TRUSTED
LOCAL_TRUSTED
LOCAL_UNVERIFIED
WEB_UNTRUSTED
CHANNEL_UNTRUSTED
MCP_UNTRUSTED
PLUGIN_UNVERIFIED
MODEL_GENERATED
SECRET_DERIVED
```

Derivation defaults to set union. Downgrade is an explicit attested transformation, not label deletion in place.

`SECRET_DERIVED` is special: long-term memory admission rejects it. A registered deterministic secret-elimination sanitizer may produce a new artifact whose evidence proves no secret-derived content remains; the source remains unchanged/secret-derived.

## Secret vault and broker

### Interface

```text
SecretHandle -> BrokerSecretUse(handle, purpose, destination/process, authority context)
```

Callers should not receive raw values when the kernel/broker can perform the use at the trusted boundary.

### Durable design

- opaque secret ID + versions;
- encrypted value bytes only in durable vault;
- AEAD associated data binds secret identity/version/classification metadata;
- protected key hierarchy/backing store selected only after platform dependency qualification;
- rotation creates a new version; revocation prevents future use;
- unavailable/corrupt key material fails closed;
- audit stores handle/version/use metadata, not plaintext.

### Unbrokerable fallback

- explicit approval;
- sandbox/process profile must allow the exact injection channel;
- never argv;
- environment cleared first; inject only the approved variable/handle/channel;
- minimize plaintext lifetime and zeroize owned buffers where the chosen libraries support it;
- output/error/log redaction is value-aware using deterministic canary tests;
- no ambient inheritance to grandchildren unless separately authorized.

## Secret ingestion

A designated ingestion path detects explicit/recognized secret input before durable model-visible append. It writes a redacted/tombstone representation plus non-secret audit metadata. Tests prove known deterministic canaries are removed; the product does not claim perfect arbitrary-secret detection.

## Egress

### Strict-local

The Spec 002 hard guard remains first. No external network permit is effective in strict-local mode.

### Non-strict permit

Permit scope includes:
- principal/process;
- action/purpose;
- destination/port/protocol class;
- time/usage bound;
- taint labels;
- optional secret handle;
- parent lease/decision.

Resolution/redirect/rebinding checks are part of the execution authorization path.

## Sandbox profiles

A profile is protected configuration compiling to a launch/admission plan:
- filesystem roots/read/write;
- network class/permit requirement;
- environment allowlist;
- spawn/child policy;
- CPU/memory/time/output limits;
- device access;
- IPC endpoints;
- inherited handles/capabilities;
- platform executor requirement.

Profile classes reserve:
- pure WASM/WASI extension;
- native untrusted subprocess;
- MCP server;
- skill helper;
- browser/protocol helper;
- local model sidecar.

Spec 003 need not implement later product integrations. It must implement/test enough profile/admission mechanics to prove deny-by-default inheritance and unsupported-platform failure. A Wasmtime executor is implementation-task-gated and may be deferred if no executable WASM path is needed to satisfy the bounded slice.

## Data migration

Extend the protected authority SQLite schema through forward-only migrations. No destructive reset. Existing Spec 002 authorization/effect/audit records remain valid.

Migration must be crash-safe and startup verification must reject future/inconsistent authority schema.

## Dependency qualification

Before implementation code adds Cedar, crypto/vault/keychain, Wasmtime or platform isolation dependencies:
1. pin exact version/source;
2. record license/notices;
3. inspect transitive dependencies and generated/vendored code;
4. document unsafe/FFI/JIT/process/platform boundary;
5. verify no hidden network/telemetry/secrets behavior;
6. write focused adversarial tests;
7. keep dependency outside authority ownership semantics.

## Test strategy

- policy schema/bundle malformed/error/forbid/no-permit tests;
- hard-deny dominance properties;
- lease subset/expiry/revocation/replay properties;
- approval scope/freshness/ONCE concurrency/crash tests;
- protected-state hostile-adapter tests;
- policy activation/lease/approval/secret rotation disk-full and crash fault injection;
- taint union/downgrade/sanitizer properties;
- `SECRET_DERIVED` memory-sink rejection;
- deterministic secret canary log/event/error/durable-vault tests;
- egress permit + strict-local dominance tests;
- external no-egress observation retained;
- sandbox profile env/FS/network/spawn/resource/unsupported-platform tests;
- Windows/macOS/Linux exact-head CI;
- fresh authorized Qodo after exact-head CI.

## Exit gate

Spec 003 implementation may close only when:
- all implementation tasks are complete;
- exact-head Windows/macOS/Ubuntu CI succeeds;
- hard-denial/lease/approval/taint/secret/egress/sandbox adversarial evidence exists;
- canary-secret and external no-egress gates pass;
- convergence finds no material divergence from constitution/spec/contracts/tasks;
- fresh authorized post-CI Qodo review has zero unresolved material findings;
- implementation PR is reviewed/merged and post-merge canonical evidence is green before Spec 004 starts.
