# Quickstart — Spec 004 Harness & Local Intelligence

This quickstart describes the acceptance flow the implementation must make possible. It is not authorization to implement from the planning branch.

## Planning read order

1. `.specify/memory/constitution.md`
2. canonical Spec 001 architecture/tasks and `contracts/execution-profile-contract.md`
3. canonical Spec 002 closeout evidence
4. canonical Spec 003 package and implementation closeout evidence
5. `specs/004-harness-local-intelligence/spec.md`
6. `clarification-closeout.md`
7. `research.md`
8. `donor-qualification.md`
9. `plan.md`
10. `data-model.md`
11. all Spec 004 `contracts/`
12. this quickstart
13. `checklists/implementation-readiness.md`
14. `tasks.md`
15. `analysis.md`

## Deterministic harness acceptance flow

The mandatory baseline uses a scripted backend and no downloaded model.

### 1. Build an exact local profile

Create a test/fixture `ExecutionProfile` with:
- `backend=SCRIPTED`;
- `locality=LOCAL`;
- exact tokenizer/template fixture identities;
- bounded context/output/time/retry budgets;
- explicit `NATIVE_TOOLS`, `GRAMMAR_CONSTRAINED`, or `TEXT_PROTOCOL_FALLBACK` test mode;
- no network permission.

Assert that serialization/content identity is stable and a material field change yields a new profile identity.

### 2. Run a successful streamed turn

Script backend events:

```text
TEXT_DELTA("hel")
TEXT_DELTA("lo")
USAGE(...)
STOP(COMPLETE)
```

Verify:
- request series/attempt evidence is durable;
- accepted output becomes `hello` with exact attempt/source references;
- later model-visible context derives from canonical evidence;
- no authority/effect record is created merely by generation.

### 3. Cancel mid-stream

Script:

```text
TEXT_DELTA("partial")
<wait for cancel>
TEXT_DELTA("late")
```

Cancel after the first accepted delta.

Verify:
- cancellation is recorded;
- the accepted `partial` prefix remains attributable interrupted evidence;
- disallowed late data is not silently appended as completed output;
- terminal state is `CANCELLED`, not success;
- no protected effect is replayed/cancelled by inference cancellation.

### 4. Exercise bounded retry

Attempt 1 returns a transient backend error. Attempt 2 succeeds.

Verify:
- both attempts remain visible;
- attempt 2 references the retry parent/reason;
- retry budget is enforced;
- a deterministic failure does not enter the transient retry path.

### 5. Exercise context overflow and compaction

Return `FAILED_CONTEXT_OVERFLOW`, run a compaction projection over exact source evidence, then dispatch a new attempt.

Verify:
- canonical source history is unchanged;
- compaction has explicit start/result/end or equivalent durable lifecycle evidence;
- projection binds source refs/digest and taint;
- Goal/non-negotiable constraints are reintroduced independently of summary text;
- new attempt binds the new projection identity.

### 6. Normalize all tool-call modes

Produce equivalent candidate calls through:
- native structured tool output;
- grammar-constrained output;
- text-protocol fallback.

Verify all three normalize to equivalent bounded candidate semantics and none executes automatically.

Then feed malformed/oversized/unknown/duplicate calls and verify fail-closed rejection.

### 7. Prove strict-local routing

Provide:
- one compatible local scripted profile;
- one unavailable local profile;
- one otherwise high-ranked `EXPLICIT_CLOUD` fixture profile.

In strict-local mode, force local failure and verify cloud is never selected as fallback.

### 8. Calibrate with fixtures

Use fixture HardwareProfiles to prove:
- compatible device selection;
- unsupported backend/device state remains explicit;
- recommendation changes are reversible;
- calibration cannot change privacy/network class.

### 9. Produce separated benchmark evidence

Run the scripted workload and record:
- backend timing fixture metrics separately from
- harness retry/cancel/compaction/tool-call correctness metrics.

Verify a benchmark record binds exact code revision, ExecutionProfile, HardwareProfile and workload fixture.

## Optional real-backend qualification

Only after a Source Foundry admission record exists:

### mistral.rs candidate

Use an exact locally available model artifact and the admitted minimal crate/features. Disable/exclude automatic downloads and backend-owned agent/tool/MCP/shell/code execution features. Run external no-egress observation for strict-local qualification.

### llama.cpp candidate

Launch the exact admitted sidecar build under Spec 003 supervision with:
- local model path;
- explicit offline mode;
- no URL/HF/RPC options;
- authenticated/private local transport;
- bounded environment/resources/output.

Run cancellation/process-failure/no-egress tests.

## Required ordinary CI

Ordinary Windows/macOS/Ubuntu CI must pass without:
- model downloads;
- cloud credentials;
- GPU/accelerator requirement;
- external AI provider access.

The deterministic scripted backend is the required CI oracle for harness semantics.
