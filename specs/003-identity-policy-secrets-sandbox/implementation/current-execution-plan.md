# Spec 003 — Live Implementation Execution Plan

**Status**: IMPLEMENTATION_ACTIVE — PHASE_F_ACTIVE  
**Canonical base**: `main@82de7084384009ff3a00522f4e0aef09bf549529`  
**Implementation branch**: `impl/003-identity-policy-secrets-sandbox`  
**Current task**: `T003-054`

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

`ACTIVE`

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

### T003-054 — ACTIVE

Implement the bounded unbrokerable fallback only when brokered credential application is impossible.

Required boundaries:

- require explicit bounded approval and an already-authenticated sandbox/process admission authorizing the exact injection channel;
- never inject secret material into argv;
- construct execution from a cleared ambient environment and add only the exact authorized injection value plus explicitly admitted non-secret variables;
- prevent implicit child/grandchild secret inheritance;
- minimize plaintext lifetime and keep any decrypted bytes inside the trusted fallback boundary;
- apply value-aware redaction to captured stdout/stderr/log/error surfaces before they can reach model-visible or durable ordinary paths;
- use deterministic canaries only; no real credentials;
- fail closed when admission, approval, channel, process inheritance control, redaction, key protection or locality state is unavailable/ambiguous;
- do not create sandbox admission or egress authority in this task; those remain owned by their later lifecycle tasks.

After exact-head T003-054 qualification, continue directly to T003-055.

### Remaining Phase F ordering

T003-053 -> T003-054 -> T003-055 -> T003-056 -> T003-057.

## Later phases

- Phase G: T003-060..T003-064, strict-local hard denial remains dominant.
- Phase H: T003-070..T003-076, with descendant-capturing no-egress predecessor before network-capable native managed children.
- Phase I: T003-080..T003-084.
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
PHASE_F_ACTIVE=YES
T003_046=PASS
T003_046_QUALIFIED_HEAD=890571fe705f36f42c1c20acff3a8a2c4fa3498e
T003_046_CI_RUN=33157139728
T003_050=PASS
T003_050_QUALIFIED_HEAD=9dc77f9ff565f0540b21feb4706e25cc36087be1
T003_050_CI_RUN=33160722873
T003_051=PASS
T003_051_QUALIFIED_HEAD=92acf59670024004e5fca1658021a99e3e7df913
T003_051_CI_RUN=33161883864
T003_052=PASS
T003_052_QUALIFIED_HEAD=b70689c1e4836f1540541f45d66cdd5a3f514dec
T003_052_CI_RUN=33163396509
T003_053=PASS
T003_053_QUALIFIED_HEAD=6f24182e1d810bc4fe6e437117149c4479241034
T003_053_CI_RUN=33165034463
T003_054=ACTIVE
NEXT_TASK=T003-054
REAL_SECRETS_USED=NO
SPEC_003_IMPLEMENTATION_COMPLETE=NO
SPEC_003_CLOSED_CANONICAL=NO
PR_READY=NO
```
