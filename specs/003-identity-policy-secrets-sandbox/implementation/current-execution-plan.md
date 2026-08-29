# Spec 003 — Live Implementation Execution Plan

**Status**: IMPLEMENTATION_ACTIVE — PHASE_F_COMPLETE — PHASE_G_COMPLETE — PHASE_H_ACTIVE  
**Canonical base**: `main@82de7084384009ff3a00522f4e0aef09bf549529`  
**Implementation branch**: `impl/003-identity-policy-secrets-sandbox`  
**Current task**: `T003-080`

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

Required boundaries:

- injected pre-commit failures must roll back secret record/version/handle and approval-consumption state atomically;
- crash/restart evidence must show only fully committed protected transitions survive;
- rotation must leave exactly one active current version while retired versions remain immutable and stale handles deny;
- revocation must immediately block broker/fallback use without deleting prior encrypted versions or weakening audit integrity;
- no failure mode may acknowledge success while leaving old secret authority usable as current.

Phase F and Phase G are closed at their task-qualified heads above. Phase H is active at T003-073.

### Remaining Phase F ordering

T003-053 -> T003-054 -> T003-055 -> T003-056 -> T003-057.

## Phase G — Egress permits

`COMPLETE`

### T003-060 — COMPLETE

Qualified at exact implementation head `50941a6d68a7920aca3666eb786f05d8b2c145b2` by CI #502 / run `33196075286`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/strict-local-hard-guard-qualification.md`.

The qualified regression proves strict-local external egress denies before downstream policy/permit evaluation, produces no grant, and records no policy evidence because the downstream permit-like policy is never invoked.

### T003-061 — COMPLETE

Qualified at exact implementation head `94d1482f8963ea4d5630a1ba4d2bdaba0e12e7ef` by CI #514 / run `33198112325`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/egress-permit-qualification.md`.

The qualified boundary implements protected permit issuance/revocation, exact scope and lifetime checks, active-policy and parent-lease validation, atomic bounded use accounting, schema-v4 `uses_consumed`, and `authority-security-v2` snapshots while preserving strict-local dominance.

### T003-062 — COMPLETE

Qualified at exact implementation head `4d730e894ebde948185597f5fe4296a142fd9ac6` by CI #525 / run `33200261387`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/egress-effective-destination-qualification.md`.

The qualified boundary binds normalized effective authority, resolved IP endpoint, protocol/port, address class, permit identity and original destination-scope digest into fresh authorization evidence before connect/follow. Redirect, rebinding and private-target changes reject reused decisions, while protocol/port widening denies. No DNS or socket operation is added to the trusted authority store.

### T003-063 — COMPLETE

Qualified at exact implementation head `a2486acc1b4dab47207ca9becdad09afe27fefe1` by CI #532 / run `33234466919`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/egress-context-qualification.md`.

The qualified effective-use boundary requires exact runtime taint and optional opaque secret-handle context to match the protected permit and binds both into durable authorization-decision context evidence without secret plaintext.

### T003-064 — COMPLETE

Qualified at exact implementation head `d77cd2b8e78a41153085c279fea698bb794d2d4e` by CI #538 / run `33234807292`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/strict-local-process-tree-qualification.md`.

The qualified independent observer computes the transitive managed process-tree closure on Unix and Windows and fails if any observed Golam-owned PID holds an Internet socket. No native managed-child executor is claimed here; T003-074 remains the first task permitted to add one and must reuse this descendant-capturing observer.

## Phase H — Sandbox profiles/admission

`ACTIVE`

### T003-070 — COMPLETE

Qualified at exact implementation head `9704318742c7622cb5304fa24f22b1c5bb35e22e` by CI #545 / run `33235986709`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/sandbox-profile-qualification.md`.

The qualified boundary establishes deterministic bounded SandboxProfile validation and protected immutable profile-version registration under exact current decision/effect/ONCE-approval authority with atomic authority-security evidence. No admission plan or executor is claimed.

### T003-071 — COMPLETE

Qualified at exact implementation head `5f6ea41fc8372273065376f0a5ab546d18100e43`, tree `986ad7b04264339f79ea7399d839bfd4904d5ecb`, by CI #558 / run `33237313243`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/sandbox-plan-qualification.md`.

The qualified read-only compiler binds exact protected profile, latest launch decision, active lease generation, active policy and optional egress permit into a deterministic non-authority-bearing plan. Trusted locality belongs to the compiler boundary; requesters cannot downgrade strict-local mode. Compilation writes no admission, consumes no egress use and launches no process.

### T003-072 — COMPLETE

Qualified at exact implementation head `241f36a6fbbaad93d46aed1970353a9270b05431`, tree `37d65e8dc72f0c9ed4935ac997d2de06e653b101`, by CI #565 / run `33237725527`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/sandbox-enforcement-qualification.md`.

The qualified boundary starts from a cleared environment, rejects filesystem/environment/device/IPC/inherited-handle widening, preserves strict-local network dominance, prevents spawn widening, intersects resource limits to the stricter bound, and explicitly does not claim OS/platform containment or launch a process.

### T003-073 — COMPLETE

Qualified at exact implementation head `39b5989b18457785f0a02a952c1f6d69bb123e60`, tree `7b70ae25afcf6a601b376d76ea5dd99ba9f6cce4`, by CI #574 / run `33246660545`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/sandbox-executor-capability-qualification.md`.

The qualified boundary resolves every rights-derived and exact profile platform requirement against a trusted current-platform capability manifest and fails closed before launch if any required control is unsupported. The production T003-073 baseline advertises zero native containment controls (`native:unqualified`), preventing capability self-assertion and preserving honest fail-closed behavior until T003-074 qualifies concrete native execution. No process is launched and no platform containment is claimed.

### T003-074 — COMPLETE

Qualified at exact implementation head `0d28681c971b3ae1f08504c7eb2448789ac8ed6e`, tree `32379e52964faca7db84b8354947be825ba2ef64`, by CI #592 / run `33247996101`, SUCCESS on Windows/macOS/Ubuntu. Focused native-executor qualification run `33247892761` also completed SUCCESS.

Evidence: `implementation/sandbox-native-executor-qualification.md`.

The qualified boundary reuses the T003-064 descendant-aware observer and proves a Linux x86_64 test-only native executor/profile using explicit mount/PID/IPC/UTS/session controls, cleared environment, unprivileged payload identity, empty capability sets, `no_new_privs`, bounded device/filesystem exposure and pre-payload seccomp network denial. Production remains `native:unqualified`; user/network namespace isolation, macOS/Windows parity, external-network profiles and universal native isolation are not claimed. No network-capable managed child was launched.

### T003-075 — COMPLETE

Qualified at exact human-authored implementation head `6d3bf98c51ba2c44d187ff07d24bd804a3026bdd`, tree `ae2463fa36240fc801accdfeb5b39adf7fccde10`, by CI #597 / run `33249678974`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/t003-075-wasm-profile-disposition.md`.

T003-005 was not reopened. `WASMTIME_DISPOSITION=NOT_ADMITTED_NOT_NEEDED` remains authoritative; no Wasmtime/WASI executor, runtime/JIT/hostcall surface or dependency was admitted, and runtime authority did not change.

### T003-076 — COMPLETE

Qualified at exact human-authored implementation head `758476315cebf48c31a8c43f84b5d9859f8e3342`, tree `57d4930af50f8d6c259f87332884de4f641a8261`, by CI #606 / run `33250188127`, SUCCESS on Windows/macOS/Ubuntu. Focused adversarial run `33250097386` also completed SUCCESS.

Evidence: `implementation/t003-076-qualification-candidate.md`.

The task repaired a material deny-all capability gap: empty filesystem/device/IPC/inherited-handle allowlists now still require explicit executor enforcement controls. Adversarial coverage also proves forbidden rights widening, strict-local network dominance, spawn denial, independent resource-control support, unsupported-platform fail-closed behavior, and preservation of the bounded T003-074 OS containment harness. Production remains `native:unqualified`; no universal native containment claim was added.

### T003-080 — ACTIVE

Replace `BootstrapPolicy` in the normal authority-serving path with Cedar-backed active-policy evaluation while preserving the stable `Authorize(principal, action, resource, context)` contract and keeping initial/recovery bootstrap administration narrowly bounded to the local owner. Runtime evaluator errors, malformed stored bundle/source/context and missing active policy must fail closed for ordinary product effects.

## Later phases

- Phase G: COMPLETE through T003-064; strict-local hard denial and descendant-capturing observation remain dominant predecessors.
- Phase H: COMPLETE through T003-076.
- Phase I: ACTIVE at T003-080; remaining T003-080..T003-084.
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
T003_054=PASS
T003_054_QUALIFIED_HEAD=f2cef061f4a6847a2eedf7b867b8b94e4ccadd8f
T003_054_CI_RUN=33166659638
T003_055=PASS
T003_055_QUALIFIED_HEAD=b434da7c675fecf41d3f3aed2104559f959e8310
T003_055_CI_RUN=33167705734
T003_056=PASS
T003_056_QUALIFIED_HEAD=b1a47898515dc06237a0d71cea00e81b19cddd0a
T003_056_CI_RUN=33169107060
T003_057=PASS
T003_057_QUALIFIED_HEAD=3621b502854fd46d7b45b28f7e8d0ca071b08b68
T003_057_CI_RUN=33195054055
T003_060=PASS
T003_060_QUALIFIED_HEAD=50941a6d68a7920aca3666eb786f05d8b2c145b2
T003_060_CI_RUN=33196075286
T003_061=PASS
T003_061_QUALIFIED_HEAD=94d1482f8963ea4d5630a1ba4d2bdaba0e12e7ef
T003_061_CI_RUN=33198112325
T003_062=PASS
T003_062_QUALIFIED_HEAD=4d730e894ebde948185597f5fe4296a142fd9ac6
T003_062_CI_RUN=33200261387
T003_063=PASS
T003_063_QUALIFIED_HEAD=a2486acc1b4dab47207ca9becdad09afe27fefe1
T003_063_CI_RUN=33234466919
T003_064=PASS
T003_064_QUALIFIED_HEAD=d77cd2b8e78a41153085c279fea698bb794d2d4e
T003_064_CI_RUN=33234807292
T003_070=PASS
T003_070_QUALIFIED_HEAD=9704318742c7622cb5304fa24f22b1c5bb35e22e
T003_070_CI_RUN=33235986709
T003_071=PASS
T003_071_QUALIFIED_HEAD=5f6ea41fc8372273065376f0a5ab546d18100e43
T003_071_CI_RUN=33237313243
T003_072=PASS
T003_072_QUALIFIED_HEAD=241f36a6fbbaad93d46aed1970353a9270b05431
T003_072_QUALIFIED_TREE=37d65e8dc72f0c9ed4935ac997d2de06e653b101
T003_072_CI_RUN=33237725527
T003_073=PASS
T003_073_QUALIFIED_HEAD=39b5989b18457785f0a02a952c1f6d69bb123e60
T003_073_QUALIFIED_TREE=7b70ae25afcf6a601b376d76ea5dd99ba9f6cce4
T003_073_CI_RUN=33246660545
T003_074=PASS
T003_074_QUALIFIED_HEAD=0d28681c971b3ae1f08504c7eb2448789ac8ed6e
T003_074_QUALIFIED_TREE=32379e52964faca7db84b8354947be825ba2ef64
T003_074_CI_RUN=33247996101
T003_074_FOCUSED_RUN=33247892761
T003_075=PASS
T003_075_QUALIFIED_HEAD=6d3bf98c51ba2c44d187ff07d24bd804a3026bdd
T003_075_QUALIFIED_TREE=ae2463fa36240fc801accdfeb5b39adf7fccde10
T003_075_CI_RUN=33249678974
T003_076=PASS
T003_076_QUALIFIED_HEAD=758476315cebf48c31a8c43f84b5d9859f8e3342
T003_076_QUALIFIED_TREE=57d4930af50f8d6c259f87332884de4f641a8261
T003_076_CI_RUN=33250188127
T003_076_FOCUSED_RUN=33250097386
NEXT_TASK=T003-080
REAL_SECRETS_USED=NO
SPEC_003_IMPLEMENTATION_COMPLETE=NO
SPEC_003_CLOSED_CANONICAL=NO
PR_READY=NO
```
