# Spec 003 — Live Implementation Execution Plan

**Status**: IMPLEMENTATION_ACTIVE — PHASE_H_COMPLETE — PHASE_I_ACTIVE  
**Canonical base**: `main@82de7084384009ff3a00522f4e0aef09bf549529`  
**Implementation branch**: `impl/003-identity-policy-secrets-sandbox`  
**Current task**: `T003-083`

## Authority

This file is an implementation-time execution companion to the canonical Spec Kit package. Live GitHub truth, constitution, `spec.md`, `plan.md`, contracts, `data-model.md`, and `tasks.md` override this file on conflict.

For every task:

1. re-read the relevant normative sources and live branch state;
2. implement only the bounded eligible task;
3. add focused deterministic/adversarial evidence;
4. require fresh exact-head CI before task PASS;
5. record qualification inside the repository;
6. immediately start the next eligible task unless a real governance blocker exists.

No force-push, rebase, destructive history rewrite, or PASS/merge/closeout claim without the required exact evidence.

## Completed phases

### Phase A — Exact-main bootstrap and dependency gates

`COMPLETE`

- T003-001..T003-006 complete.
- Cedar admitted exactly at `4.12.0`.
- secret cryptography and OS key-protection dependencies qualified at their recorded exact versions/platform boundaries.
- Wasmtime remains `NOT_ADMITTED_NOT_NEEDED`.
- `Golam-Research` remains `REFERENCE_ONLY`; no donor code is admitted.

### Phase B — Schema, hard guards and policy lifecycle

`COMPLETE`

T003-010..T003-017 complete.

### Phase C — Capability leases

`COMPLETE`

T003-020..T003-024 complete.

### Phase D — Approvals

`COMPLETE`

T003-030..T003-035 complete at their task-recorded qualified heads.

### Phase E — Taint and verifier state

`COMPLETE`

- T003-040: `cb69d638107ca4fe0118c9a61f143ac3ba65a2d3`, CI `33150969442`.
- T003-041: `76e1addf35c92a22d2c5826ca429278cacd598b3`, CI `33151556481`.
- T003-042: `67f74c9b9b75e43b9fa00069050c97c041567184`, CI `33152187952`.
- T003-043: `2f8655b5bdddd17bb9e6eab7bf00f11a210896cb`, CI `33154505847`.
- T003-044: `1a9fcddff4c4dd6a6161547cf89a502750f9bc71`, CI `33155122088`.
- T003-045: `e3b91dcecf0048b183c4c333cd9afda43ee25671`, CI `33155929307`.
- T003-046: `890571fe705f36f42c1c20acff3a8a2c4fa3498e`, CI `33157139728`.

## Phase F — Secret vault and broker

`COMPLETE`

### T003-050 — COMPLETE

Qualified at exact implementation head `9dc77f9ff565f0540b21feb4706e25cc36087be1` by CI #416 / run `33160722873`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/secret-interface-qualification.md`.

The qualified boundary provides protected `SecretRecord` / `SecretVersion` metadata and opaque protected `SecretHandle` loading without generic plaintext reads, ciphertext access, or production secret mutation.

### T003-051 — COMPLETE

Qualified at exact implementation head `92acf59670024004e5fca1658021a99e3e7df913` by CI #424 / run `33161883864`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/secret-vault-storage-qualification.md`.

The qualified boundary provides AES-256-GCM encrypted-at-rest secret-value storage, fresh 96-bit nonces with hard nonce-reuse failure, associated-data binding across secret/version/classification/security-metadata/vault-format identity, and fail-closed OS-backed key protection on the exact dependency/platform set admitted by T003-004. No plaintext master-key fallback or generic public plaintext secret-read API is introduced.

### T003-052 — COMPLETE

Qualified at exact implementation head `b70689c1e4836f1540541f45d66cdd5a3f514dec` by CI #434 / run `33163396509`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/secret-mutation-qualification.md`.

The qualified boundary implements atomic Golam-owned create, immutable rotation/versioning, and monotonic revocation transitions with exact durable authorization/effect/ONCE-approval binding, encrypted-before-persist storage, authenticated security snapshots, stale-version/replay rejection, and rollback on vault/key-protection failure.

### T003-053 — COMPLETE

Qualified at exact implementation head `6f24182e1d810bc4fe6e437117149c4479241034` by CI #448 / run `33165034463`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/secret-broker-qualification.md`.

The qualified boundary provides protected metadata-only `BrokerSecretUse` authorization with exact handle/purpose/destination, strict-local, current decision, active policy, complete lease-chain and optional approval binding. Approval-bound uses are atomically consumed. No generic plaintext/ciphertext read API, egress authority or fallback injection is exposed.

### T003-054 — COMPLETE

Qualified at exact implementation head `f2cef061f4a6847a2eedf7b867b8b94e4ccadd8f` by CI #464 / run `33166659638`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/secret-fallback-qualification.md`.

The qualified boundary provides a bounded trusted stdin-only fallback consuming exact pre-existing admission/policy/lease/effect/ONCE-approval authority, with a cleared environment, no secret argv/environment, no ambient descendant inheritance, callback-only vault plaintext exposure and exact-value output/error redaction. It creates no sandbox or egress authority and uses deterministic canaries only.

### T003-055 — COMPLETE

Qualified at exact implementation head `b434da7c675fecf41d3f3aed2104559f959e8310` by CI #473 / run `33167705734`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/secret-entry-qualification.md`.

The qualified boundary treats the entire explicit submitted value as secret without detector dependence, routes raw bytes only into protected vault mutation, creates an authenticated opaque `SecretHandle`, and emits only a dedicated redacted canonical projection with bounded non-secret metadata before model-visible history.

### T003-056 — COMPLETE

Qualified at exact implementation head `b1a47898515dc06237a0d71cea00e81b19cddd0a` by CI #484 / run `33169107060`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/secret-canary-qualification.md`.

The qualified suite exercises recognized and deliberately unknown-format deterministic canaries through the same explicit-entry path, scans durable authority files and canonical model-visible event payloads, checks rendered errors, and proves an environment-cleared unauthorized subprocess cannot recover the plaintext from durable authority bytes. Free-text recognition remains bounded defense in depth only.

### T003-057 — COMPLETE

Qualified at exact implementation head `3621b502854fd46d7b45b28f7e8d0ca071b08b68` by CI #495 / run `33195054055`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/secret-fault-qualification.md`.

The qualified suite uses the real secret mutation transaction path with test-only pre-commit pause injection, OS process termination before rotation/revocation commit, bounded SQLite `SQLITE_FULL` rotation/revocation faults, approval-consumption rollback checks, restart verification, and successful-commit controls. Only fully committed transitions change durable secret authority.

## Phase G — Egress permits

`COMPLETE`

### T003-060 — COMPLETE

Qualified at exact implementation head `50941a6d68a7920aca3666eb786f05d8b2c145b2` by CI #502 / run `33196075286`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/strict-local-hard-guard-qualification.md`.

The qualified regression proves strict-local external egress denies before downstream policy/permit evaluation, produces no grant, and records no policy evidence because the downstream permit-like policy is never invoked.

### T003-061 — COMPLETE

Qualified at exact implementation head `94d1482f8963ea4d5630a1ba4d2bdaba0e12e7ef` by CI #514 / run `33198112325`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/egress-permit-qualification.md`.

### T003-062 — COMPLETE

Qualified at exact implementation head `4d730e894ebde948185597f5fe4296a142fd9ac6` by CI #525 / run `33200261387`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/egress-effective-destination-qualification.md`.

### T003-063 — COMPLETE

Qualified at exact implementation head `a2486acc1b4dab47207ca9becdad09afe27fefe1` by CI #532 / run `33234466919`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/egress-context-qualification.md`.

### T003-064 — COMPLETE

Qualified at exact implementation head `d77cd2b8e78a41153085c279fea698bb794d2d4e` by CI #538 / run `33234807292`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/strict-local-process-tree-qualification.md`.

## Phase H — Sandbox profiles/admission

`COMPLETE`

### T003-070 — COMPLETE

Qualified at exact implementation head `9704318742c7622cb5304fa24f22b1c5bb35e22e` by CI #545 / run `33235986709`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/sandbox-profile-qualification.md`.

### T003-071 — COMPLETE

Qualified at exact implementation head `5f6ea41fc8372273065376f0a5ab546d18100e43` by CI #558 / run `33237313243`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/sandbox-plan-qualification.md`.

### T003-072 — COMPLETE

Qualified at exact implementation head `241f36a6fbbaad93d46aed1970353a9270b05431` by CI #565 / run `33237725527`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/sandbox-enforcement-qualification.md`.

### T003-073 — COMPLETE

Qualified at exact implementation head `39b5989b18457785f0a02a952c1f6d69bb123e60` by CI #574 / run `33246660545`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/sandbox-executor-capability-qualification.md`.

### T003-074 — COMPLETE

Qualified at exact implementation head `0d28681c971b3ae1f08504c7eb2448789ac8ed6e` by CI #592 / run `33247996101`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/sandbox-native-executor-qualification.md`.

### T003-075 — COMPLETE

Qualified at exact implementation head `6d3bf98c51ba2c44d187ff07d24bd804a3026bdd` by CI #597 / run `33249678974`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/t003-075-wasm-profile-disposition.md`.

### T003-076 — COMPLETE

Qualified at exact implementation head `758476315cebf48c31a8c43f84b5d9859f8e3342` by CI #606 / run `33250188127`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/t003-076-qualification-candidate.md`.

### T003-080 — COMPLETE

Qualified at exact implementation head `cd721231b498450e984810b4f06c4e14bdc311e1`, tree `4c5ddb92f8764f0107510ba72e6f697a3225bd56`, by CI #616 / run `33252170158`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/t003-080-runtime-policy-qualification.md`.

### T003-081 — COMPLETE

Qualified source candidate `877f8e45f8f8ba4ba4a98af036d51032b3fba684`, tree `1808de4cce60669d89a5c3a11f2c8f47b6608b2f`, by CI #630 / run `33264029849`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/t003-081-admin-surface-qualification.md`.

### T003-082 — COMPLETE

Qualified exact implementation head `a312ba3d0b40454dd6bddd8eb1887e481ec5b0d3`, tree `e6123476f352d94ea10dfd2da65cb2bf9c22dc63`, by CI #650 / run `33307009245`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/t003-082-hostile-adapter-qualification.md`.

The qualified hostile-adapter suite proves fabricated authority evidence cannot mint leases, activate policy, forge approvals, self-register verifier authority or register a deliberately weakened sandbox profile; a permissive downstream policy still cannot override strict-local egress; product crates do not link the privileged ledger directly; and plaintext-bearing secret internals remain non-public.

### T003-083 — ACTIVE

Preserve the closed Spec 002 effect FSM/reconciliation, IPC, corruption/integrity, recovery and strict-local gates without regression. This task is regression qualification unless evidence identifies a concrete break: do not introduce parallel authority or effect behavior merely to create new code. Fresh exact-head CI must execute the complete workspace/property/fuzz/IPC/authenticated-daemon/adversarial/strict-local matrix before PASS.

## Later phases

- Phase I: ACTIVE at T003-083; remaining T003-083..T003-084.
- Phase J: T003-090..T003-098, including fresh exact-head multi-platform CI, convergence, authorized Qodo review, merge, and post-merge canonical-main evidence.

## Current invariant set

```text
SPEC_002_CLOSED_CANONICAL=YES
SPEC_003_PLANNING_CLOSED_CANONICAL=YES
SPEC_003_IMPLEMENTATION_AUTHORIZED=YES
PHASE_A_COMPLETE=YES
PHASE_B_COMPLETE=YES
PHASE_C_COMPLETE=YES
PHASE_D_COMPLETE=YES
PHASE_E_COMPLETE=YES
PHASE_F_COMPLETE=YES
PHASE_G_COMPLETE=YES
PHASE_H_COMPLETE=YES
PHASE_I_ACTIVE=YES
T003_080=PASS
T003_080_QUALIFIED_HEAD=cd721231b498450e984810b4f06c4e14bdc311e1
T003_080_CI_RUN=33252170158
T003_081=PASS
T003_081_QUALIFIED_SOURCE_HEAD=877f8e45f8f8ba4ba4a98af036d51032b3fba684
T003_081_QUALIFIED_SOURCE_TREE=1808de4cce60669d89a5c3a11f2c8f47b6608b2f
T003_081_CI_RUN=33264029849
T003_082=PASS
T003_082_QUALIFIED_HEAD=a312ba3d0b40454dd6bddd8eb1887e481ec5b0d3
T003_082_QUALIFIED_TREE=e6123476f352d94ea10dfd2da65cb2bf9c22dc63
T003_082_CI_RUN=33307009245
NEXT_TASK=T003-083
REAL_SECRETS_USED=NO
SPEC_003_IMPLEMENTATION_COMPLETE=NO
SPEC_003_CLOSED_CANONICAL=NO
PR_READY=NO
WAIVER_TAKEN=NO
```