# Feature Specification: Identity, Policy, Secrets & Sandbox

**Feature Branch**: `spec/003-identity-policy-secrets-sandbox`  
**Base**: `main@a04756f242e48faeda802e5b3fd99a0c8d52f53e`  
**Created**: 2026-08-27  
**Status**: PLANNING_COMPLETE_PENDING_REVIEW  
**Planning rule**: NO PRODUCT IMPLEMENTATION OR DEPENDENCY ADMISSION IN THIS PR

## Purpose

Spec 003 turns the stable Spec 002 authorization seam into a complete local authority system without widening the privileged kernel or introducing models, broad tools, Desktop, Connect, or real external integrations.

It implements the Spec 001 security contracts for explicit identity, capability leases, policy evaluation, step-up approvals, protected resources, secret handles/vault behavior, taint information flow, egress authorization, and declared sandbox profiles.

The stable authorization call remains:

```text
Authorize(principal, action, resource, context) -> Allow | Deny(reason)
```

Cedar is the policy-engine candidate. Golam continues to own identities, actions, resources, context construction, capabilities, hard denials, approvals, protected-resource classes, secret semantics, taint rules, egress semantics, audit records, and failure behavior.

## Product slice

At the end of Spec 003 implementation, the existing authenticated local daemon/CLI spine can prove that:

- protected requests are evaluated by a fail-closed layered authorization pipeline;
- kernel-minted leases cannot self-expand and are checked for scope, expiry and revocation at use time;
- policy bundles are validated, versioned, hash-bound and atomically activated;
- policy errors never become allows;
- approvals are scoped, fresh, durable and consumption-safe;
- protected-state mutations cannot bypass typed KernelApi/effect/audit boundaries;
- taint propagates through derived local artifacts and cannot be self-cleared;
- secret values are represented by opaque handles and brokered where possible;
- unbrokerable secret use follows a bounded isolated fallback with redaction and no argv/ambient inheritance;
- strict-local egress remains an unconditional hard denial for Golam-managed network creation;
- declared sandbox profiles compile to bounded launch/admission plans and unsupported containment claims fail closed.

## User stories

### US1 — Explicit least-privilege authorization

As a local owner, I can know exactly which authenticated principal is asking to perform which action on which resource and why it was allowed or denied.

Acceptance:
- every protected decision uses explicit principal/action/resource/context;
- a hard kernel denial cannot be overridden by Cedar, a capability lease, an approval, or downstream code;
- Cedar/schema/evaluation errors deny and are audited;
- decisions bind the active policy bundle and relevant lease/approval evidence;
- model output or untrusted input cannot instantiate authority.

### US2 — Narrow, revocable capability leases

As a local owner, I can delegate a bounded capability without granting broader ambient authority.

Acceptance:
- only privileged kernel APIs mint authority-bearing leases;
- child lease authority is a subset/intersection of its parent;
- expiry and revocation are checked at action execution;
- a principal cannot edit or widen its own lease;
- stale/revoked/widened/replayed leases fail closed and are auditable.

### US3 — Safe step-up approvals

As a local owner, I can approve a specific risky action or bounded run without creating an unlimited "always allow" bypass.

Acceptance:
- ONCE, SESSION_SCOPED, TIME_BOXED, OPERATION_PATTERN and RUN_PREAUTHORIZATION are supported;
- approval scope binds action/resource/risk/context and material taint;
- freshness is checked immediately before the protected action/effect executes;
- ONCE use cannot be double-consumed under concurrency or crash/retry;
- unattended IRREVERSIBLE work requires bounded per-run preauthorization.

### US4 — Secrets remain security state, not prompt text

As a local owner, I can authorize credential use without routinely exposing raw credentials to model context, logs, history, or arbitrary child processes.

Acceptance:
- normal callers receive opaque secret handles, not plaintext;
- vault records are encrypted at rest using an implementation-qualified key-protection design;
- brokerable use keeps plaintext at the trusted boundary;
- unbrokerable use requires explicit bounded approval and isolated non-argv injection;
- ambient inheritance is denied;
- deterministic secret canaries are absent from prompts, durable logs/events/errors and unauthorized child output;
- user-pasted secrets are redacted/tombstoned at ingestion rather than persisted as plaintext canonical history.

### US5 — Taint survives derivation

As a local owner, untrusted data cannot silently become trusted instructions or trusted effect evidence after summarization/transformation.

Acceptance:
- derived artifacts inherit source taint by monotonic union;
- model/worker/skill/MCP/channel statements cannot self-clear taint;
- normal downgrade requires human approval or deterministic pre-registered authoritative verification;
- `SECRET_DERIVED` is rejected from canonical long-term memory;
- creation of a non-secret-derived representation requires a deterministic registered sanitizer/verifier with auditable evidence.

### US6 — Network capability is explicit

As a strict-local user, policy configuration or a plugin cannot accidentally re-enable external network access.

Acceptance:
- strict-local denial is enforced before Cedar/lease/approval permits;
- a network permit cannot make external egress usable in strict-local mode;
- non-strict permits are destination/action/purpose/time scoped;
- DNS resolution, redirects and rebinding are represented in the authorization boundary;
- every denied or unexpected attempt is auditable and externally observable qualification remains required.

### US7 — Sandboxes are declared and fail closed

As a local owner, an untrusted subprocess/extension receives only the filesystem/network/environment/process resources explicitly declared for its profile.

Acceptance:
- authorization and sandboxing are separate gates;
- environment begins cleared and only approved values/handles are injected;
- profiles declare filesystem, network, process, resource, device and IPC bounds;
- unsupported profile/platform enforcement fails closed;
- profile approval does not override capability/egress denial;
- Wasmtime/WASI may implement portable bounded extension profiles after dependency qualification, but native tools require native containment evidence.

## Functional requirements

- **FR-001**: Preserve the Spec 002 process-splittable `KernelApi` and stable `Authorize(principal, action, resource, context)` semantics.
- **FR-002**: Implement a layered decision pipeline: hard guards -> authenticated principal -> lease -> policy evaluation -> approval -> protected mutation/effect gate.
- **FR-003**: A denial at any earlier layer MUST be monotonic; downstream layers MUST NOT convert it into allow.
- **FR-004**: Policy/schema load, validation or evaluation failure MUST return DENY with bounded audited diagnostics; no permissive fallback.
- **FR-005**: Policy bundles MUST be immutable/versioned/hash-bound and activated atomically; active-policy corruption or missing required state fails closed.
- **FR-006**: Updating policy/schema/active policy MUST itself be a protected elevated effect evaluated under the currently active authority state plus required approval.
- **FR-007**: Kernel-minted capability leases MUST be action/resource/context scoped, expire/revoke at use time, and support subset-only derivation.
- **FR-008**: Authority-bearing lease/token types MUST remain unforgeable outside privileged kernel modules.
- **FR-009**: Authorization decisions MUST durably bind relevant hard-guard, lease, policy-bundle, rule/reason and approval evidence without secret plaintext.
- **FR-010**: Implement ONCE, SESSION_SCOPED, TIME_BOXED, OPERATION_PATTERN and RUN_PREAUTHORIZATION approval classes.
- **FR-011**: Approval freshness/scope MUST be revalidated at execution; ONCE consumption MUST be durable, atomic and replay-safe.
- **FR-012**: Implement baseline taint labels from Spec 001 and monotonic propagation through derived data/artifacts.
- **FR-013**: Taint downgrade MUST require explicit human approval or deterministic registered authoritative verification, with audit evidence.
- **FR-014**: `SECRET_DERIVED` MUST be denied at canonical long-term-memory admission; ordinary model/human assertions cannot clear it.
- **FR-015**: Implement opaque secret handles and protected secret metadata/version records. Raw values MUST NOT be exposed through generic APIs.
- **FR-016**: Vault storage MUST be encrypted at rest; exact cryptographic/key-protection dependencies are implementation-time qualified and MUST fail closed when unavailable or corrupt.
- **FR-017**: Brokerable secret use MUST avoid model/untrusted plaintext exposure. Unbrokerable use requires bounded approval, isolated non-argv injection, cleared ambient environment and value-aware redaction.
- **FR-018**: User-pasted secret ingestion MUST redact/tombstone plaintext before durable model-visible history is committed.
- **FR-019**: Strict-local external egress MUST remain a kernel hard deny independent of Cedar permits, leases or approvals.
- **FR-020**: Non-strict egress permits MUST bind principal/process, action/purpose, destination, time/usage, taint and optional secret handle; DNS/redirect/rebinding are in scope.
- **FR-021**: Implement declared sandbox profile records and a fail-closed profile-to-launch/admission plan boundary.
- **FR-022**: Sandbox profiles MUST declare FS roots/writes, network, environment, process spawning, CPU/memory/time/output, devices, IPC and inherited handles.
- **FR-023**: Unsupported sandbox enforcement on a platform MUST deny rather than silently execute with weaker isolation.
- **FR-024**: Protected policy/principal/lease/approval/secret/taint-verifier/egress/sandbox definitions MUST NOT be writable through generic filesystem/tool APIs.
- **FR-025**: Every protected mutation added by Spec 003 MUST receive mandatory `authority-security` integrity coverage or equivalently strong authenticated integrity.
- **FR-026**: Spec 003 MUST preserve Spec 002 effect durability/reconciliation invariants and must not introduce real external integrations to prove authority semantics.
- **FR-027**: Implementation dependency/donor admission MUST follow Source Foundry and exact-version unsafe/FFI/platform qualification before code use.
- **FR-028**: `Golam-Research` remains reference-only for this slice unless a later exact bounded admission record explicitly changes that state.

## Non-functional requirements

- **NFR-001 Rust trusted path**: policy/capability/approval/secret/egress authority remains Rust-first and privileged code forbids unsafe code unless a separately isolated reviewed boundary is required.
- **NFR-002 Fail closed**: malformed/corrupt/missing policy, lease, approval, vault, taint verifier, egress or sandbox authority never widens access.
- **NFR-003 Deterministic decisions**: identical normalized authority inputs and the same active authority state produce identical allow/deny semantics and stable reason classes.
- **NFR-004 Bounded data**: policy/context/lease/approval/profile/secret metadata and diagnostics have explicit size/count limits.
- **NFR-005 Durability**: activation, revocation, consumption and secret-version transitions are transactional and crash-safe.
- **NFR-006 Locality**: no cloud/service/model dependency is required for Spec 003 authority.
- **NFR-007 Portability**: Windows, macOS and Linux are qualification targets; unsupported containment capabilities are explicit and fail closed.
- **NFR-008 TCB control**: no broad framework may become the authority owner; parser/config surfaces remain bounded and isolated from authority construction.
- **NFR-009 No false sandbox claim**: a declarative profile is not evidence of OS containment; containment claims require implemented platform executors and tests.
- **NFR-010 Secret hygiene**: implementation tests use deterministic canaries only; no real user/service secret is required for acceptance.

## Success criteria

- **SC-001**: adversarial tests prove policy errors, stale/revoked/widened leases and stale/mismatched approvals deny without authority widening.
- **SC-002**: property tests prove child leases never exceed parent authority and hard denials dominate all other decision layers.
- **SC-003**: concurrent/restart tests prove ONCE approval cannot be consumed twice.
- **SC-004**: policy activation and protected authority mutations remain atomic under injected crash/disk-full failures.
- **SC-005**: taint property tests prove derivation preserves source labels and self-clear attempts fail.
- **SC-006**: `SECRET_DERIVED` memory admission tests fail closed unless a registered deterministic secret-elimination sanitizer produces a separately evidenced non-secret artifact.
- **SC-007**: canary-secret tests show no canary in durable logs/events/errors/prompts or unauthorized subprocess output, and vault durable bytes do not contain plaintext canary.
- **SC-008**: strict-local tests prove no policy/lease/approval/egress permit can enable external network; externally observed no-egress remains green.
- **SC-009**: sandbox profile tests deny unsupported enforcement and reject inherited env/forbidden FS/network/process rights.
- **SC-010**: exact-head Windows/macOS/Ubuntu CI and authorized post-CI Qodo review contain no unresolved material findings before implementation closeout.

## Out of scope

- model inference, model/provider routing, harness/context compilation (Spec 004);
- broad filesystem/shell/git/browser product tools and memory product implementation (Spec 005);
- Desktop/computer control (Spec 006);
- GolamConnect/channels (Spec 007);
- workers/scheduler/automations (Spec 008);
- real production credentials, payments, email, deployments, cloud providers, remote effects;
- a universal native sandbox abstraction that claims parity without platform evidence;
- new unauthenticated HTTP/TCP control surfaces.
