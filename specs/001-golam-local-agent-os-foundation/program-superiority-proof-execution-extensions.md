# Program Superiority — Proof, Execution Fabric and Extension Architecture

**Status**: PROGRAM-LEVEL DESIGN OVERLAY — NO PRODUCT IMPLEMENTATION AUTHORITY

**Parent**: `program-superiority-2026.md`

This document sharpens the parts of the superiority roadmap where current agents most often trade safety/reproducibility for convenience: self-certifying completion, self-modifying learning, execution-backend coupling, extension sprawl and implicit cost/privacy fallback.

## 1. Proof-carrying completion

Golam must distinguish:

```text
ACTOR_CLAIMED_DONE
EVIDENCE_COLLECTED
VERIFIER_PASSED
VERIFIED_COMPLETE
```

The same model/worker that performed a task may propose completion but does not receive automatic authority to certify it.

### Verification obligation

Every bounded goal or consequential task may declare a versioned `VerificationObligation` containing:

- goal/task identity;
- exact success criteria;
- required verifier classes;
- authoritative sources or expected-state observations;
- negative conditions that must remain false;
- freshness requirements;
- environment/platform identity requirements;
- artifact/effect refs;
- required independent readback after side effects;
- acceptable unavailable/unsupported handling;
- whether the user may explicitly accept an unverified result.

### Verifier order

Prefer the strongest applicable verifier:

1. deterministic invariant/schema/test/hash/signature check;
2. authoritative API/database/filesystem/source-of-truth readback;
3. independent environment observation;
4. specialized deterministic evaluator;
5. independent model critique/judge as supporting untrusted evidence only.

A model judge must not override deterministic failure or missing authoritative evidence.

### Verification receipt

A successful verifier emits an immutable `VerificationReceipt` bound to:

- obligation ID/version/digest;
- verifier identity/version;
- exact input/artifact/effect/source refs;
- observed environment identity;
- observation time/freshness semantics;
- result + bounded diagnostics;
- integrity chain / canonical ledger reference.

`VERIFIED_COMPLETE` requires all mandatory receipts. Missing evidence means `UNVERIFIED`, `PARTIALLY_VERIFIED`, `BLOCKED`, or `UNSUPPORTED` rather than optimistic success.

### Independence

Where a stronger independent readback exists, the actor that performed a consequential effect cannot be the sole verifier of its own outcome. Examples:

- Git commit creation -> verify repository/head/tree through fresh Git readback;
- file write -> fresh identity/digest/readback;
- API mutation -> fresh API resource read;
- desktop action -> re-observe semantic target/state;
- schedule/routine -> verify durable run/effect records and external postcondition;
- document generation -> parse/validate resulting artifact, not only generation success.

## 2. Verified planning and self-correction

Long-running agents should maintain a durable proof graph:

```text
Goal
 -> SuccessCriteria
 -> PlanNode
 -> EvidenceRequirement
 -> Action/Effect
 -> Observation
 -> VerificationReceipt
 -> Proven/Unproven/Contradicted
```

The planner may replan when an obligation is unmet, but cannot delete contradictory evidence merely to make a plan appear successful.

Required behavior:

- explicit unresolved assumptions;
- explicit contradiction nodes;
- source freshness and authority ranking;
- verification budget reserved before spending the entire task budget on generation/action;
- premature-stop evaluation in GolamBench;
- post-action verification failures create new work or an honest blocker, not silent completion.

## 3. Safe experiential learning versus self-rewriting agents

Persistent/self-evolving agents such as Letta Code demonstrate the product value of durable identity, git-backed memory, skills and harness customization. Golam should retain that value while refusing to let model self-modification silently become trusted runtime behavior.

Separate four layers:

```text
ExperienceEvidence
MemoryCandidate
SkillOrRoutineCandidate
RuntimeExtensionCandidate
```

Activation paths differ:

- memory candidate -> existing governed memory promotion/reconciliation;
- skill/routine candidate -> provenance + taint + capability inference + tests/replay + approval/policy -> immutable active version;
- runtime extension/mod candidate -> Source Foundry + extension conformance + sandbox/permission declaration + independent qualification -> immutable admitted extension version.

Agent identity may evolve in user-owned memory, but identity text cannot grant capabilities, change kernel policy, bypass approvals or make a source trusted.

## 4. Replaceable ExecutionBackend fabric

Agent reasoning must not be coupled to one execution environment.

Proposed unprivileged contract:

```text
ExecutionBackendDescriptor
ExecutionWorkspaceIdentity
ExecutionLaunchPlan
ExecutionSession
ExecutionSnapshotRef
ExecutionObservation
ExecutionTerminalEvidence
```

Candidate backend classes:

- existing Golam governed native local executor;
- local container/VM/microVM when separately admitted;
- local dedicated desktop/browser environment;
- remote self-hosted worker;
- optional remote sandbox provider.

SWE-ReX is a high-value reference for separating agent logic from local/remote shell execution. E2B is a useful remote sandbox/snapshot lifecycle reference. Neither defines Golam authority.

### Authority rules

- backend selection follows current capability/locality/privacy/resource policy;
- strict-local excludes remote sandbox providers;
- remote sandbox credentials use brokered secret handles;
- backend process/filesystem/network surfaces are explicit capabilities;
- remote/provider API success does not prove the in-sandbox effect succeeded;
- snapshots are execution-state artifacts, not canonical Golam memory/effect authority;
- reconnect/adoption requires backend/session identity revalidation;
- backend substitution after a prepared consequential effect requires reconciliation/new effect identity as applicable.

## 5. Workspace snapshots and time travel

Persistent workers require reproducible work state beyond chat history.

Define snapshot evidence for coding/research/workspace operations:

- canonical repository/base commit identity;
- worktree/workspace ID;
- filesystem snapshot/image ID where available;
- dependency/toolchain lock identities;
- active skill/routine/profile identities;
- local artifact refs;
- external effect refs that cannot be rolled back by restoring files;
- secret/session material explicitly excluded or separately governed.

Rollback is a new governed operation. It must state honestly that restoring a workspace does not unsend email, unpublish a post, reverse an API mutation, revoke exposed credentials, or undo another already-committed external effect.

## 6. Quality / Cost / Privacy Governor

Resource routing should be extended into a multi-objective governor while keeping hard constraints non-negotiable.

Decision order:

1. authority compatibility;
2. strict-local/privacy/network compatibility;
3. secret/taint/destination compatibility;
4. capability/format/hardware compatibility;
5. user hard quality/latency/cost constraints;
6. resource availability;
7. evidence-based ranking among remaining options.

Possible dimensions:

- expected verified success probability;
- latency / deadline;
- monetary budget;
- token budget;
- CPU/GPU/RAM/battery/thermal pressure;
- provider/model availability;
- locality/privacy class;
- workload type;
- context-window/tool-call/multimodal needs;
- reliability history bound to exact profile revisions.

No fallback may silently widen privacy, network, secret disclosure or authority merely to finish a task.

Receipts should expose the selected execution/model profile and material budget decisions without leaking sensitive content.

## 7. Extension SDK and conformance kit

Golam needs an ecosystem without making third-party packages part of the TCB.

Provide versioned SDK/contracts for:

- instruction-only Skills;
- executable skill helpers;
- MCP servers/resources/tools;
- ACP clients;
- connectors;
- control-route providers;
- model/inference adapters;
- optional ExecutionBackend adapters;
- verifiers;
- context/retrieval providers.

Every extension declares:

- stable extension identity/version/digest;
- protocol/schema versions;
- requested capability classes;
- filesystem/network/process/device/secrets requirements;
- taint/provenance behavior;
- expected resource limits;
- lifecycle/startup behavior;
- cancellation/restart semantics;
- deterministic conformance fixtures;
- Source Foundry/provenance record.

The conformance kit tests shape and safety behavior but does **not** grant production authority. Installation, technical qualification, review and activation remain separate lifecycle states.

Target lifecycle:

```text
DISCOVERED
-> PINNED
-> SOURCE_REVIEWED
-> CONFORMANCE_TESTED
-> SANDBOX_TESTED
-> TECHNICALLY_QUALIFIED
-> ADMITTED
-> ACTIVE
-> DEPRECATED / REVOKED
```

Version replacement invalidates stale descriptors, queued calls and cached authority decisions exactly as existing Skill/MCP governance requires.

## 8. Local-first extension distribution

A future catalog may help users discover skills/connectors/extensions, but the catalog is recommendation metadata rather than a trust root.

Requirements:

- install from exact immutable source/artifact identity;
- show source/license/provenance/permissions before activation;
- separate install from activation;
- local lockfile of active extension identities;
- local export/import support;
- offline/manual package install path;
- revocation/advisory feed is optional network capability and never a hidden strict-local dependency;
- no popularity score can override security/source policy.

## 9. Human factors and accessibility as safety features

The safest emergency control is useless if users cannot find or operate it.

Release qualification should include:

- keyboard-only emergency pause/stop/takeover;
- screen-reader accessible control labels/state where platform allows;
- no safety-critical distinction based only on color;
- high contrast and reduced motion;
- localized explanatory text without localizing stable machine identifiers;
- approval prompts that state action, resource, destination, duration and consequences plainly;
- measured emergency-stop discoverability/time-to-action in user tests;
- user-visible distinction among local, remote, strict-local, cloud-model and cloud-execution states;
- clear uncertainty/reconciliation UX rather than generic failure/success banners.

## 10. Comparative advantage requirements

Golam should claim superiority only when evidence demonstrates that the combination is better, not just larger.

Desired differentiators:

```text
PERSISTENT_AGENT_WITHOUT_CLOUD_AUTHORITY_ROOT=YES
LEARNING_WITHOUT_SELF_EXPANDING_AUTHORITY=YES
MULTI_WORKER_WITH_ISOLATED_IDENTITIES_AND_HANDOFF_EVIDENCE=YES
LOCAL_AND_REMOTE_EXECUTION_WITH_ONE_AUTHORITY_MODEL=YES
PROOF_CARRYING_COMPLETION=YES
RESTART_SAFE_CONSEQUENTIAL_EFFECTS=YES
STRICT_LOCAL_HARD_DENIAL=YES
HUMAN_VISIBLE_INTERRUPTIBLE_COMPUTER_CONTROL=YES
EXTENSION_ECOSYSTEM_WITHOUT_EXTENSION_TRUST_ROOT=YES
REPRODUCIBLE_REVISION_BOUND_BENCHMARK_CLAIMS=YES
```

None of these values may be marked `YES` in a release claim until the owning specs and exact release artifacts prove them.
