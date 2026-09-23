# Golam OpenMuse Bounded Donor Integration Plan — 2026-09-23

**Status:** PLANNING ONLY / BOUNDED IMPLEMENTATION DONOR
**Source:** `CopilotKit/openmuse`
**Reviewed pin:** `bb7ce4e1c6e523bf282a655c63621e3ed9e75150`
**Repository license observed:** MIT
**Founder permission:** explicitly reaffirmed 2026-09-23

## 1. Decision

OpenMuse is a high-value bounded implementation donor for Golam's Experience/Harness/Computer surfaces.

Golam SHOULD NOT wholesale fork OpenMuse or adopt its trust model. Each selected component is independently mapped into Golam's canonical Task / Effect / Authority / Evidence / Context contracts.

Important upstream boundary:

- repository source is MIT at the reviewed pin;
- CopilotKit Intelligence is a separately configured service and is explicitly not bundled under the repository's MIT license;
- Golam must not make that hosted/service boundary mandatory merely to reuse OpenMuse UI or workflow code.

## 2. High-value bounded donor areas

### 2.1 Conversation queue and run failure semantics

OpenMuse provides useful behavior:

- composer stays usable while a run is active;
- follow-ups queue visibly;
- one active AG-UI run at a time;
- Stop preserves draft;
- failure pauses the queue;
- failed message is never silently resent.

Golam mapping:

```text
OpenMuse conversation queue behavior
-> T167/T221 canonical steer/queue/branch semantics
-> Golam-owned durable identities and failure receipts
```

The client queue itself is not canonical Task truth.

### 2.2 Rich thread / task artifact projection

Useful pattern:

- thread stores stable task/artifact IDs;
- current task/artifact state is hydrated from authenticated canonical endpoints;
- expiring signed URLs are generated on demand rather than stored in messages;
- task/browser/PDF results render as rich cards.

Map to T185/T200/T209/T216.

```text
THREAD_MESSAGE != ARTIFACT_TRUTH
SIGNED_URL != DURABLE_ARTIFACT_IDENTITY
UI_CARD != TASK_AUTHORITY
```

### 2.3 Durable task worker and lease/recovery behavior

High-value patterns:

- durable tasks/plans/steps;
- SQL lease coordination;
- expired-lease recovery;
- pause/resume/cancel/retry;
- saved action/result receipts;
- uncertain external writes are not automatically replayed;
- an already-dispatched external request may finish after cancellation.

Map to T149/T150/T167/T205.

Do not port a second task ledger or Effect state machine.

### 2.4 Reviewed external actions

OpenMuse verification includes review binding to ownership, content hash/version, expiry, account changes/disconnect, concurrent decisions and uncertain writes.

These are high-value test/UX patterns for Golam approvals, but Golam's T174/T199 Effect Gate remains authoritative.

### 2.5 Persistent browser + human takeover

Useful bounded donor components/patterns:

- persistent Chromium profiles;
- session reopen by stable ID;
- screenshots/read/input;
- live takeover console;
- profile persistence after restart;
- bounded downloads;
- public-destination checks;
- signed console/file access.

Map to Spec 006 / T151 / T173 / T179 / T185.

The Playwright worker is not a kernel security boundary.

### 2.6 Isolated Linux computer

Strong donor patterns:

- non-root user;
- read-only root filesystem;
- dropped capabilities;
- no added privileges;
- no host-directory mounts;
- no Docker socket inside the workload;
- no model/provider/Google/API secrets;
- network disabled;
- bounded memory/CPU/PIDs/tmp;
- persistent dedicated workspace volume;
- fixed Docker CLI argv at trusted host boundary;
- no fallback to host shell;
- bounded command duration/output;
- saved exit/output receipts;
- stop/restart recovery with uncertain outcomes;
- symlink/special-file rejection.

Map to T151/T168/T173/T205/T213/T216.

Docker isolation is not treated as hostile-tenant VM isolation.

### 2.7 Goals, monitors, ideas and background updates

Useful product patterns:

- durable Goals + milestones;
- recurring public-page monitors;
- deduplicated alerts/backoff;
- evidence-backed Ideas;
- background task updates linked to canonical task IDs.

Map to T208/T209/T212/T221. These remain projections over canonical state.

## 3. Reuse modes

| Area | Preferred reuse mode |
| --- | --- |
| conversation queue / run-error handling | SELECTIVE_COPY or PORT |
| rich task/artifact cards | SELECTIVE_COPY / ADAPT |
| background update UX | ADAPT |
| durable worker/lease algorithms | REIMPLEMENT_BEHAVIOR or SELECTIVE_PORT after authority mapping |
| review/receipt tests | PORT_TEST_PATTERN |
| browser worker | BOUNDED_ADAPTER / SELECTIVE_PORT |
| takeover console | SELECTIVE_COPY / ADAPT |
| Linux computer hardening/test fixtures | PORT / REIMPLEMENT_BEHAVIOR |
| CopilotKit Intelligence dependency | DO_NOT_REQUIRE |
| shared-access-key single-owner auth | REFERENCE_ONLY; do not adopt as Golam identity root |

Every selected file/component still receives a Source Foundry record.

## 4. Required exact-component port matrix

Before implementation, T236 must record for each selected component:

```text
source_path
source_commit
source_blob_digest
reuse_mode
target_package
target_contract_owner
dependency_closure
rights/NOTICE
secrets/network behavior
authority ceiling
Effect mapping
TCB delta
tests to port
tests to add
rollback/removal path
```

No whole-app admission.

## 5. Tests worth adapting

At minimum evaluate porting or reproducing:

- follow-up queue pauses on SDK/run error;
- failed queue item not implicitly replayed;
- draft survives Stop/navigation;
- task lease race + expiry recovery;
- cancel/pause/resume restart semantics;
- uncertain external write does not replay;
- review invalid after account/version/hash change;
- persistent browser profile reopen;
- failed first navigation cleanup;
- signed access renewal;
- download failure visibility;
- browser destination/DNS/egress checks;
- Linux computer never falls back to host shell;
- stale executor restart fencing;
- interrupted command recovery without replay;
- filesystem symlink/special-file rejection;
- output/time/resource caps.

## 6. Boundaries not to import

```text
OPENMUSE_TASK_STORE != GOLAM_TASK_AUTHORITY
OPENMUSE_REVIEW != GOLAM_APPROVAL_AUTHORITY
OPENMUSE_WORKER_TOKEN != GOLAM_PRINCIPAL
COPILOTKIT_THREAD != GOLAM_CANONICAL_TASK
PLAYWRIGHT_CONTAINER != KERNEL_SECURITY_BOUNDARY
DOCKER_CONTAINER != HOSTILE_TENANT_VM
SIGNED_URL != OBJECT_AUTHORITY
```

Also do not import:

- mandatory CopilotKit Intelligence service dependency;
- provider-specific identity as canonical Golam identity;
- application-level public-web checks as a substitute for T179 egress enforcement;
- one shared access key as future multi-user authentication;
- hidden external retry semantics.

## 7. OpenMuse vs Golam ownership

```text
OpenMuse UI / queue / task cards
  -> Golam Experience Plane

OpenMuse durable worker patterns
  -> Golam T167/T205 execution contracts

OpenMuse browser/computer
  -> Golam Capability/Execution Plane

OpenMuse reviews
  -> Golam Approval UX inputs

Golam Authority Kernel / Effect Gate / Verification
  -> remains sole protected authority
```

## 8. Acceptance

A bounded port is acceptable only if:

- exact source component is pinned;
- transitive dependencies are known;
- target contract owner is explicit;
- no second canonical Task/Effect/Approval/Evidence state is created;
- source tests are ported or replaced by equal/stronger fixtures;
- strict-local and secret boundaries are preserved or strengthened;
- donor can be removed without corrupting canonical user state;
- UI projection can rebuild from canonical state;
- runtime failures preserve honest uncertainty.

## 9. Disposition

```text
OPENMUSE_BOUNDED_DONOR=YES
OPENMUSE_WHOLESALE_FORK=NO
OPENMUSE_COPILOTKIT_INTELLIGENCE_MANDATORY=NO
OPENMUSE_CODE_ADMITTED=NO
OPENMUSE_RUNTIME_DEPENDENCY_ADMITTED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
```
