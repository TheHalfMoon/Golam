# Tasks — Spec 003 Identity, Policy, Secrets & Sandbox

**Status**: PLANNED — IMPLEMENTATION BLOCKED UNTIL PLANNING PR MERGES CANONICAL  
**Planning base**: `main@a04756f242e48faeda802e5b3fd99a0c8d52f53e`

Legend:
- `[ ]` is future implementation work authorized only after this planning package is reviewed/merged.
- Planning completion does not mean product implementation exists.
- No PASS transfers across a branch mutation.

## Phase A — Exact-main bootstrap and dependency gates

- [ ] **T003-001** Re-read exact canonical `main` after planning merge; create implementation branch from that exact commit.
- [ ] **T003-002** Re-read constitution, Spec 001 authority contracts, canonical Spec 002 closeout and the merged Spec 003 package before code mutation.
- [ ] **T003-003** Qualify exact Cedar source/crate/version/features/license/transitives/unsafe/resource behavior before adding it.
- [ ] **T003-004** Qualify exact cryptographic/vault/key-protection dependencies and Windows/macOS/Linux backing-store boundaries before handling secret values.
- [ ] **T003-005** Qualify Wasmtime/WASI only if a bounded WASM-profile implementation task actually requires it; otherwise record `NOT_ADMITTED_NOT_NEEDED`.
- [ ] **T003-006** Record Source Foundry evidence before any donor source reuse; default `Golam-Research=REFERENCE_ONLY`.

## Phase B — Schema, hard guards and policy lifecycle

- [ ] **T003-010** Extend protected authority schema for policy bundles, active policy, leases/revocations, approvals/consumption, taint attestations/verifier rules, secret records/versions, egress permits and sandbox profiles/admissions.
- [ ] **T003-011** Extend `authority-security` canonical coverage and startup verification for every new protected source record.
- [ ] **T003-012** Freeze bounded canonical principal/action/resource/context normalization and policy input test vectors.
- [ ] **T003-013** Implement hard-kernel-guard stage ahead of policy evaluation, preserving strict-local and other monotonic denials.
- [ ] **T003-014** Implement candidate policy bundle parse/schema/policy validation with bounded diagnostics and fail-closed behavior.
- [ ] **T003-015** Implement immutable bundle hashing/versioning/staging and atomic active-policy activation under current authority plus approval.
- [ ] **T003-016** Implement startup active-policy integrity verification and fail-closed recovery behavior; no permissive bootstrap fallback after normal activation.
- [ ] **T003-017** Implement stable authorization-decision evidence binding hard guard, lease, policy bundle/rule and approval state without secret plaintext.

## Phase C — Capability leases

- [ ] **T003-020** Implement sealed kernel-minted capability lease types with no public authority constructor.
- [ ] **T003-021** Implement action/resource/context scope normalization and subset-only child derivation.
- [ ] **T003-022** Implement expiry, revocation, generation and principal binding checks at protected action execution.
- [ ] **T003-023** Implement protected lease issuance/revocation as typed elevated effects with security integrity.
- [ ] **T003-024** Add property/adversarial tests for widening, self-grant, stale generation, expiry, revocation and replay.

## Phase D — Approvals

- [ ] **T003-030** Implement ONCE, SESSION_SCOPED, TIME_BOXED, OPERATION_PATTERN and RUN_PREAUTHORIZATION records/scopes.
- [ ] **T003-031** Bind approvals to action/resource/effect/pattern, risk, taint/context digest, parent decision, expiry and limits.
- [ ] **T003-032** Revalidate approval freshness/scope immediately before protected execution.
- [ ] **T003-033** Implement durable atomic ONCE reservation/consumption with concurrency and crash/retry safety.
- [ ] **T003-034** Enforce bounded RUN_PREAUTHORIZATION for unattended irreversible effects and deny generic always-allow behavior.
- [ ] **T003-035** Add approval expiry/revocation/replay/double-use/scope-overreach/taint-mismatch tests.

## Phase E — Taint and verifier state

- [ ] **T003-040** Implement baseline taint-label set and deterministic canonical encoding.
- [ ] **T003-041** Implement monotonic union propagation for derived artifacts/authority context.
- [ ] **T003-042** Implement protected verifier/sanitizer registry; tainted sources cannot register their own downgrade rule.
- [ ] **T003-043** Implement human/deterministic-verifier downgrade attestations as new evidence rather than in-place source mutation.
- [ ] **T003-044** Enforce `SECRET_DERIVED` rejection at long-term-memory admission boundary reserved for later memory integration tests.
- [ ] **T003-045** Implement deterministic secret-elimination sanitizer evidence path for creating a separately non-secret-derived artifact.
- [ ] **T003-046** Add multi-hop/self-clear/unregistered-verifier/SECRET_DERIVED property and adversarial tests.

## Phase F — Secret vault and broker

- [ ] **T003-050** Implement protected opaque SecretHandle/SecretRecord/SecretVersion interfaces without generic plaintext reads.
- [ ] **T003-051** Implement qualified encrypted-at-rest vault storage and key-protection abstraction with fail-closed corruption/unavailability behavior.
- [ ] **T003-052** Implement secret create/version/rotate/revoke protected transitions with atomic security evidence.
- [ ] **T003-053** Implement `BrokerSecretUse` authorization around handle, purpose, destination/process, lease/policy/approval and locality state.
- [ ] **T003-054** Implement bounded unbrokerable fallback: no argv, cleared environment, exact injection scope, no ambient child inheritance, minimized lifetime and redaction.
- [ ] **T003-055** Implement user-pasted secret redaction/tombstone ingestion boundary for deterministic recognized canary classes.
- [ ] **T003-056** Add canary tests proving no plaintext in durable vault bytes, event/audit/log/error/prompt paths or unauthorized subprocess output.
- [ ] **T003-057** Add crash/disk-full/rotation/revocation tests and prove no acknowledged half-transition exposes stale secret authority.

## Phase G — Egress permits

- [ ] **T003-060** Keep strict-local external egress as an unconditional hard guard before policy/permit evaluation.
- [ ] **T003-061** Implement non-strict EgressPermit scope and protected issuance/revocation/use accounting.
- [ ] **T003-062** Implement DNS resolution/redirect/rebinding/private-target revalidation semantics.
- [ ] **T003-063** Bind relevant taint and secret-handle context into egress authorization/evidence.
- [ ] **T003-064** Add strict-local dominance and external sinkhole/no-egress qualification proving permits cannot bypass the hard guard.

## Phase H — Sandbox profiles/admission

- [ ] **T003-070** Implement protected SandboxProfile records and deterministic profile validation.
- [ ] **T003-071** Compile profiles to bounded launch/admission plans intersected with active lease/policy/egress authority.
- [ ] **T003-072** Enforce cleared environment and explicit FS/network/spawn/resource/device/IPC/handle inheritance rules.
- [ ] **T003-073** Implement platform-executor capability checks and fail closed when a required containment control is unsupported.
- [ ] **T003-074** Implement the minimum native untrusted-process test executor/profile required to prove the contract without claiming unsupported universal isolation.
- [ ] **T003-075** If admitted by T003-005, implement bounded WASM/WASI profile via the qualified executor; otherwise keep it deferred with explicit evidence.
- [ ] **T003-076** Add escape/inheritance/forbidden-FS/network/spawn/resource/unsupported-platform tests.

## Phase I — Kernel/CLI integration and adversarial qualification

- [ ] **T003-080** Replace bootstrap policy evaluation in normal authority path while preserving the stable `Authorize` call contract and narrow recovery bootstrap path.
- [ ] **T003-081** Add minimal authenticated CLI/admin/test surface for policy, lease, approval, canary secret, decision explain and sandbox-profile qualification.
- [ ] **T003-082** Extend hostile-adapter tests: no capability minting, policy activation, approval forging, vault plaintext read, egress bypass, verifier self-registration or profile weakening.
- [ ] **T003-083** Preserve Spec 002 effect FSM/reconciliation, IPC, corruption and strict-local gates without regression.
- [ ] **T003-084** Fault-inject every coupled authority mutation around SQLite transaction/commit/restart boundaries.

## Phase J — Exact-head closeout

- [ ] **T003-090** Run pinned fmt/clippy/workspace tests on Windows/macOS/Linux exact head.
- [ ] **T003-091** Run policy/lease/approval/taint/secret/egress/sandbox property/adversarial qualification.
- [ ] **T003-092** Run bounded fuzz for newly introduced policy/profile/authority input parsers where applicable.
- [ ] **T003-093** Run deterministic secret-canary leakage suite.
- [ ] **T003-094** Run external strict-local no-egress observation across supported CI platforms.
- [ ] **T003-095** Re-run Spec Kit convergence and repair material constitution/spec/plan/contracts/tasks divergence.
- [ ] **T003-096** Obtain fresh authorized Qodo review only after exact-head CI; repair every material finding and repeat qualification after mutation.
- [ ] **T003-097** Prepare exact-head closeout evidence; no Ready/merge/Spec 004 claim without repository-authorized lifecycle evidence.
- [ ] **T003-098** After merge, require canonical `main` post-merge CI success before `SPEC_003_CLOSED_CANONICAL=YES` or starting Spec 004.

## Planning gate

```text
SPEC_002_CLOSED_CANONICAL=YES
SPEC_003_TASKS_PLANNED=YES
PRODUCT_IMPLEMENTATION_IN_PLANNING_PR=NO
DONOR_CODE_ADMITTED=NO
DEPENDENCY_ADMISSION_PENDING_IMPLEMENTATION_TASKS=YES
SPEC_003_IMPLEMENTATION_AUTHORIZED=NO_UNTIL_PLANNING_MERGE
```
