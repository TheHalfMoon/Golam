# Golam Agent Kernel Roadmap v2

**Status**: STRATEGIC PLAN CANDIDATE — NOT CANONICAL UNTIL REVIEWED AND MERGED

**Planning base**: `main@4bd23b218304663349fb2f703cedd40c7a3038af`

**Purpose**: strengthen the Golam program so it becomes materially more capable, trustworthy, durable, extensible, and inspectable than contemporary agent runtimes and public Grok Bot behavior, without replacing reproducible evidence with marketing claims.

## Product thesis

Golam is not another chatbot, shell wrapper, or agent dashboard.

Golam is a **local-first Agent Operating Kernel**:

> Models may propose actions, but models never own authority. Every consequential action is bounded, attributable, durable, policy-governed, and verifiable.

The founder goal is extreme product superiority over Grok Bot and peer agent systems. Golam MUST pursue that goal through architectural leverage and benchmarked evidence, not unsupported numeric claims. No claim such as "100x" or "a million times better" may be published without a defined metric, reproducible benchmark, exact versions, and retained evidence.

## Strategic reference corpus

The following systems are high-value design references. They are reference inputs only unless a later per-source Source Foundry record reaches `ADMITTED`.

| Reference | Primary lesson for Golam | Adoption posture |
|---|---|---|
| `semantica-agi/semantica` | provenance, temporal knowledge, causal/decision relationships, explainable evidence graphs | adopt semantics; defer graph infrastructure |
| `multica-ai/multica` | agent operations, task ownership, execution visibility, human review, reusable skills | adopt primitives later; reject org-chart coupling in core |
| `different-ai/openwork` | capability discovery/execution abstraction, governed connected services, MCP control plane | adopt capability-plane abstraction |
| `openclaw/openclaw` | gateway, typed client/node protocol, device pairing, durable local daemon UX | adopt protocol lessons under stronger authority isolation |
| `koala73/worldmonitor` | product packaging, desktop/web/API/MCP/SDK surfaces, agent discoverability | adopt distribution lessons after kernel maturity |
| `paperclipai/paperclip` | durable work leasing, budgets, heartbeats, auditability, review gates | adopt generic work primitives; reject company metaphor in core |
| `PrimeIntellect-ai/prime-agent` | persistent sessions, supervisor/worker split, goals, background execution, long-running agents | adopt continuity; keep Python/runtime kernels untrusted |
| `deepseek-ai/deepseek-harness` | typed plugin/event seams, durable session-event source of truth, composable providers | adopt governed capability seams; reject unrestricted privileged plugins |
| `zeroclaw-labs/zeroclaw` | Rust local runtime, supervised autonomy, sandboxing, tool receipts | adopt and exceed receipt/security model |
| `FoundationAgents/OpenManus` | simple general-agent UX, browser/MCP baseline, clear agent loop | retain simplicity as a UX benchmark |
| `every-app/open-seo` | agent-native vertical workflows, MCP + skills + human UI | adopt dual human/agent product-surface principle |

Any future code reuse, dependency, vendoring, port, binary, model, script, or redistribution path still requires exact Source Foundry qualification under the Golam Constitution.

## Six-plane architecture

Golam should be explained and evolved through six planes. These are ownership and reasoning boundaries, not an instruction to create empty crates.

```text
┌─────────────────────────────────────────────────────────────┐
│                    EXPERIENCE PLANE                         │
│ CLI · TUI · Desktop · IDE · MCP · ACP · Channels · SDKs    │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                     HARNESS PLANE                           │
│ Sessions · Goals · Compaction · Workers · Schedules         │
│ Subagents · Recovery · Long-running execution               │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                   CAPABILITY PLANE                          │
│ Discover → Bind → Authorize → Execute → Verify              │
│ Files · Git · Process · Browser · MCP · Skills · Devices    │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                    CONTEXT PLANE                            │
│ Evidence routing · ContextCapsules · Memory                 │
│ Contradictions · Provenance · Evidence requirements         │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                    EVIDENCE PLANE                           │
│ Event ledger · Effects · Receipts · Readbacks               │
│ Causal links · Replay · Verification · Decisions            │
└─────────────────────────────┬───────────────────────────────┐
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                    AUTHORITY PLANE                          │
│ Privileged Kernel · Identity · Policy · Leases              │
│ Approval · Secrets · Egress · Protected State               │
└─────────────────────────────────────────────────────────────┘
```

The privileged kernel remains smaller than the Rust trusted path. Dynamic extension MUST NOT make the entire daemon privileged.

## Program invariants

Existing constitutional invariants remain binding. The roadmap emphasizes the following product-defining rules:

```text
MODEL_VISIBLE => LOGGED
MODEL_OUTPUT != AUTHORITY
MEMORY != TRUTH
RETRIEVAL_SCORE != SOURCE_AUTHORITY
MODEL_CONFIDENCE != EVIDENCE_STRENGTH
SKILL != AUTHORITY
PLUGIN != AUTHORITY
CHANNEL != AUTHORITY
LOCALHOST != AUTHENTICATION
NO_EXTERNAL_EFFECT_WITHOUT_EFFECT_GATE
NO_GOLAM_MANAGED_EGRESS_WITHOUT_EGRESS_GATE
EVERY_WRITE_IS_ATTRIBUTABLE
EVERY_LONG_RUN_IS_CRASH_RESUMABLE
UNKNOWN_EFFECT_OUTCOME_BLOCKS_DEPENDENT_EFFECTS
SAFETY_DENIAL_IS_MONOTONIC
PROTECTED_AUTHORITY_STATE_IS_NOT_GENERIC_FILESYSTEM_STATE
```

## Strategic upgrade 1 — Capability Plane

All model-facing and automation-facing powers should converge on one governed abstraction instead of growing as unrelated tool families.

Conceptual contract:

```text
CapabilityDescriptor {
    capability_id
    provider_id
    version
    operations
    input_schema
    output_schema
    authority_requirements
    target_identity_rules
    sandbox_requirements
    network_posture
    secret_requirements
    effect_semantics
    verification_policy
    provenance
}
```

Conceptual lifecycle:

```text
search_capabilities(intent)
    -> inspect_capability(id)
    -> bind_capability(provider, exact_version, mapping)
    -> prepare_capability(request)
    -> authorize(kernel)
    -> execute(bound_request)
    -> reconcile_if_needed
    -> verify(readbacks)
    -> receipt
```

Requirements:

1. discovery metadata never grants authority;
2. capability provider identity/version/content digest is exact;
3. stale bindings, mappings, approvals, leases, or queued decisions fail closed;
4. filesystem, Git, process, browser, MCP, skills, device actions, and future adapters share the same authority/evidence lifecycle where semantics overlap;
5. providers may be replaceable, but privileged authority types remain kernel-owned;
6. provider output is untrusted evidence until validated according to its class.

This should become one of Golam's most important developer-facing abstractions.

## Strategic upgrade 2 — Evidence Receipts beyond ordinary tool receipts

Golam should not stop at proving that a tool returned a string. A durable `EvidenceReceipt` should bind action, authority, target identity, result, and verification.

Conceptual shape:

```text
EvidenceReceipt {
    receipt_id
    session_id
    goal_id?
    request_attempt_id?
    principal_id
    capability_id
    capability_version
    tool_request_digest
    target_identity?
    effect_id?
    input_digest
    output_digest
    started_at
    terminal_at
    authority_decision_ref
    approval_ref?
    secret_broker_ref?
    result_status
    verifier_refs[]
    readback_refs[]
    previous_receipt_hash?
    integrity_binding
}
```

Security goals:

- model-generated text cannot fabricate a valid runtime receipt;
- receipt proves execution/result binding, not policy correctness by itself;
- consequential effects additionally bind durable Effect Gate state;
- receipts survive restart and are independently inspectable;
- receipt data is minimal and redaction-aware;
- a receipt cannot mint authority;
- failures and blocked actions have auditable terminal evidence even if no success receipt exists.

## Strategic upgrade 3 — Evidence Graph semantics without graph-database lock-in

Golam should record causal/provenance relations now while keeping SQLite/Markdown/event ledgers canonical.

Minimum relation vocabulary should support concepts such as:

```text
DERIVED_FROM
OBSERVED_BY
SUPPORTS
CONTRADICTS
SUPERSEDES
CAUSED
VERIFIED_BY
RECONCILES
RESULT_OF
REQUIRES
BLOCKED_BY
```

The graph is initially a projection over canonical evidence. Neo4j/RDF/SPARQL/vector infrastructure MUST NOT become a baseline dependency or trust root merely because reference systems use graph infrastructure.

Principle:

> Graph semantics now; graph infrastructure only after measured need.

## Strategic upgrade 4 — Decision provenance

Add a first-class, non-chain-of-thought `DecisionRecord` for externally inspectable system decisions.

```text
DecisionRecord {
    decision_id
    goal_ref?
    actor
    decision_class
    options_considered_refs[]
    selected_option_ref?
    evidence_refs[]
    policy_refs[]
    constraints[]
    confidence_class?
    uncertainty_class?
    reason_summary
    downstream_effect_refs[]
    supersedes?
}
```

Examples:

- why a capability was denied;
- why a model/backend was selected;
- why context was insufficient and replanned;
- why a memory candidate was promoted, rejected, contradicted, or expired;
- why a worker stopped or requested escalation;
- why an execution was not retried after ambiguity.

`DecisionRecord` contains inspectable justification references, not hidden model chain-of-thought.

## Strategic upgrade 5 — Evidence Requirements and context sufficiency

Extend the existing Context Compiler with explicit evidence requirements so retrieval optimizes for proof rather than similarity alone.

Conceptual structure:

```text
EvidenceRequirement {
    requirement_id
    question_or_claim_ref
    required_authority_class
    freshness_requirement
    minimum_independent_sources
    contradiction_policy
    acceptable_taint
    permission_scope
    evidence_budget
}
```

The compiler should answer:

1. what must be known;
2. what source classes can authoritatively establish it;
3. whether current evidence is fresh and permitted;
4. whether contradictory evidence exists;
5. whether available evidence is sufficient;
6. whether to replan, abstain, ask for authority, or proceed.

This is intended to make Golam materially stronger than generic RAG systems.

## Strategic upgrade 6 — Evidence-backed memory

Memory should evolve from "remembered notes" to governed knowledge claims with source lineage.

Conceptual projection:

```text
MemoryClaim {
    claim_id
    statement_ref
    provenance_refs[]
    evidence_refs[]
    confidence_class
    authority_class
    observed_at
    valid_from?
    valid_until?
    contradiction_refs[]
    supersession_refs[]
    promotion_status
}
```

Requirements:

- model confidence cannot upgrade source authority;
- remembered state never outranks live authoritative state;
- contradictions remain visible;
- promotion is attributable;
- derived indexes remain rebuildable;
- memory evolution preserves causal and evidence lineage;
- secret-derived content remains ineligible for canonical long-term memory.

## Strategic upgrade 7 — Governed skills as reviewed procedures

Agent Skills should become reusable reviewed procedures rather than prompt files with implicit powers.

Conceptual manifest:

```text
SkillManifest {
    skill_id
    version
    content_digest
    instructions
    declared_capabilities[]
    required_inputs[]
    expected_outputs[]
    effect_classes[]
    sandbox_profile?
    evidence_requirements[]
    verification_procedure
    provenance
    lifecycle_state
}
```

Lifecycle:

```text
DISCOVERED -> REVIEWED -> ACTIVE -> DEPRECATED -> REVOKED
```

Every activation revalidates exact version/digest/mapping and current lifecycle state. Replacement versions do not inherit old authority automatically.

## Strategic upgrade 8 — Governed continual harness improvement

Golam should support self-improvement without self-authority.

```text
Observed failure or inefficiency
    -> HarnessImprovementCandidate
    -> exact evidence package
    -> deterministic evaluation
    -> human or pre-registered verifier approval
    -> new immutable HarnessRevision
    -> A/B qualification
    -> promotion or rejection
    -> rollback remains possible
```

Potential revision classes:

- harness profile;
- skill revision;
- prompt-policy revision;
- context routing policy;
- subagent template;
- compaction policy;
- model routing recommendation.

The model may propose a revision but cannot activate privileged behavioral changes by itself.

## Strategic upgrade 9 — Durable work primitives, not an AI-company metaphor

Later multi-agent work should be built from generic primitives:

```text
Goal
WorkItem
Worker
WorkLease
Dependency
Budget
Checkpoint
Artifact
MailboxMessage
ReviewGate
```

Conceptual lease:

```text
WorkLease {
    work_id
    worker_id
    expected_revision
    workspace_identity
    authority_subset
    token_budget
    time_budget
    cost_budget
    lease_generation
    expiry
    heartbeat_deadline
}
```

Required properties:

- atomic claim;
- stale generation rejection;
- duplicate-work prevention after restart;
- bounded delegated authority;
- workspace/worktree identity binding;
- crash adoption/reassignment with explicit evidence;
- heartbeats are liveness evidence, not proof of correctness;
- review gates remain independent of worker self-assertion.

A future product may render these primitives as a team, swarm, organization, coding crew, or personal automation system without coupling the trusted core to one metaphor.

## Strategic upgrade 10 — Autonomy Profiles as policy convenience, never authority

Expose understandable user modes while retaining kernel-native leases and approvals underneath.

Suggested product profiles:

```text
OBSERVE
  read-only, no consequential effects

ASSIST
  bounded safe effects require user approval

EXECUTE
  selected effect classes may use explicit scoped preauthorization

AUTONOMOUS_RUN
  goal-scoped + workspace-scoped + capability-scoped + budget-scoped + time-scoped authority
```

`AutonomyProfile` is a configuration convenience that compiles to explicit policy/capability/approval state. It is not itself a capability token.

## Strategic upgrade 11 — Gateway and principal taxonomy

Extend GolamConnect into a typed gateway architecture while preserving stronger trust boundaries than convenience-first runtimes.

Principal classes should include:

```text
OperatorClient
AgentClient
DeviceNode
ChannelBridge
AutomationClient
ExternalAgent
```

Each binds:

```text
identity
protocol_version
capabilities
lease_generation
pairing_state
revocation_generation
```

Requirements:

- loopback is not identity;
- pairing is not unlimited authority;
- idempotency material is required for side-effecting remote requests;
- replay protection and reconnect reauthorization remain explicit;
- clients/nodes do not inherit each other's authority;
- channels never imply machine authority;
- operator takeover invalidates conflicting control leases at the authority layer.

## Strategic upgrade 12 — Agent-native distribution

A superior kernel that nobody can adopt is not a superior product. After the trust/runtime spine is mature, Golam should ship coherent human and agent surfaces:

- one excellent CLI;
- TUI/desktop experience;
- authenticated local API;
- ACP client surface;
- MCP server surface;
- Agent Skills packages;
- stable SDKs where justified;
- machine-readable capability/discovery manifest;
- reproducible installers/releases;
- example vertical workflows;
- clear security posture and audit UX.

Every useful subsystem should be operable by a human and consumable by another authorized agent without creating a second authority path.

## Roadmap evolution

The current Specs 002–005 remain authoritative and must not be retroactively destabilized by this strategic document. Spec 005 should finish according to its canonical task ordering. The roadmap below is a successor-program candidate to be converted into canonical Spec Kit authority only through normal planning/review/merge gates.

### Spec 006 — Desktop & Semantic Computer Control

Preserve existing direction:

- Tauri desktop;
- semantic-first OS control;
- platform capability matrix;
- before/after verification;
- human takeover;
- visibility and emergency stop;
- vision only as late fallback.

Add explicit product integration for `AutonomyProfile` visualization and EvidenceReceipt inspection.

### Spec 007 — Gateway, Devices & GolamConnect

Expand existing Connect plan with:

- typed client/node roles;
- device pairing and revocation generations;
- idempotent side-effect protocol;
- capability discovery scoped by principal;
- resumable session attachment;
- channel bridges as low-authority principals;
- inspectable remote-control leases.

### Spec 008 — Durable Workers, Goals & Automations

Incorporate long-running-agent lessons:

- durable worker lifecycle;
- atomic WorkLease;
- heartbeat/liveness;
- budgets;
- dependency DAG;
- worktree/workspace isolation;
- background sessions;
- schedules/triggers;
- crash adoption;
- checkpoint/review gates;
- goal retention and premature-stop detection.

Single-worker reliability remains a prerequisite for broad multi-agent orchestration.

### Spec 009 — Capability Ecosystem & Interoperability

Promote the capability abstraction into a first-class ecosystem:

- CapabilityDescriptor and provider bindings;
- capability search/inspect/prepare/execute/verify;
- MCP lifecycle;
- ACP lifecycle;
- Agent Skills lifecycle;
- external-agent adapters;
- provider revocation/version locking;
- extension Source Foundry;
- SDK-facing discovery;
- no privileged dynamic plugin authority.

### Spec 010 — Evidence Graph & Decision Intelligence

Build projections and query semantics over existing canonical evidence:

- DecisionRecord;
- EvidenceReceipt query/verification;
- causal/provenance relation projection;
- contradiction/supersession graph;
- evidence-lineage queries;
- action-explanation UX;
- time-travel/replay views;
- decision-impact inspection;
- rebuildability from canonical evidence.

Graph database adoption remains measurement-gated.

### Spec 011 — Multi-Agent Operations

Only after Spec 008 proves reliable worker primitives:

- delegation;
- worker teams;
- task board projection;
- structured handoffs;
- mailboxes;
- bounded concurrency;
- conflict detection;
- shared goal projections;
- independent review workflows;
- no implicit peer authority transfer.

### Spec 012 — Agent-Native Developer Platform

Focus on adoption and ecosystem quality:

- stable public local API;
- SDKs where demanded by measured use;
- MCP server exposure;
- capability discovery manifests;
- examples and vertical starter packs;
- contributor extension cookbook;
- packaging/install/update lifecycle;
- plugin/skill provenance UX;
- excellent docs and diagnostics.

### Spec 013 — GolamBench & Release Qualification

Move final qualification here while keeping incremental gates in every implementing spec.

Grok/public-agent parity becomes a continuous reference benchmark track rather than the central product identity.

## Continuous reference capability matrix

From Spec 005 onward, maintain a scenario-based matrix across Golam and major reference agents.

Each scenario records:

```text
scenario_id
reference_system/version
reference_behavior
Golam_required_outcome
security_delta
durability_delta
evidence_delta
locality_delta
recovery_delta
UX_delta
benchmark_artifacts
```

References may include public Grok Bot behavior and the research corpus above. The purpose is to identify gaps and superior designs, not to clone proprietary internals or turn one competitor into Golam's roadmap owner.

## GolamBench v2 metrics

### Authority

```text
Unauthorized Effect Rate = 0
Privilege Expansion Rate = 0
Stale Lease Acceptance Rate = 0
Protected-State Generic-Write Rate = 0
```

### Durability

```text
Crash Resume Success Rate
Duplicate Consequential Effect Rate = 0
UNKNOWN_OUTCOME Classification Accuracy
Replay Reconstruction Rate
Disk/partial-write Recovery Correctness
```

### Evidence

```text
Evidence Coverage Rate
Receipt Verification Rate
Readback Completion Rate
Unsupported Success-Claim Rate = 0
Decision-to-Evidence Traceability
```

### Context

```text
Evidence Sufficiency Precision/Recall
Stale Evidence Use Rate
Contradiction Detection Rate
Token Cost per Supported Claim
Authority-aware Retrieval Accuracy
```

### Memory

```text
Stale Memory Override Rate = 0
Provenance Retention Rate
Contradiction Preservation
Forget/Redact Correctness
User-Edit Reconciliation Correctness
```

### Long-running autonomy

```text
Goal Retention
Premature Stop Rate
Recovery After Restart
Duplicate Work Claim Rate
Budget Compliance
Human Escalation Precision
```

### Security/privacy

```text
Prompt-Injection Escape Rate
Secret Leakage Rate
Strict-local Egress Violations = 0
Sandbox Boundary Violations = 0
Channel Impersonation Acceptance = 0
```

### Human experience

```text
Approval Burden
Useful Autonomy per Approval
Time to Understand Why
Time to Recover Control
Time to Reproduce an Agent Claim
```

Every benchmark claim binds exact Golam head, platform, model/backend, harness revision, dataset/scenario version, configuration, and retained result evidence.

## Killer demonstration target

Golam's flagship demo should prove trust, durability, autonomy, and inspectability together rather than merely showing chat quality.

Example:

```text
User:
"Inspect this repository, fix the failing tests, prepare the PR,
but do not modify dependencies."
```

Expected behavior:

1. create durable Goal and constraints;
2. compile explicit evidence requirements;
3. discover capabilities;
4. read repository under bounded identity-aware authority;
5. surface insufficiency/contradiction when present;
6. request only the minimum required effect authority;
7. create bounded workspace/work lease;
8. edit with target/precondition binding;
9. survive daemon/process restart mid-task;
10. resume without duplicate effects;
11. verify tests and repository state;
12. prepare the PR through authorized tooling;
13. produce a final inspectable evidence report.

The user should be able to inspect:

```text
Why the change was selected
Files changed
Evidence used
Commands/capabilities executed
Network destinations
Secret handles used
Approvals/leases
Effects
Readbacks
Verification
Receipts
Uncertainty/blockers
```

## Explicit non-goals and rejection list

To remain better by architecture rather than feature-count sprawl, Golam should reject or defer the following until prerequisites are proven:

- no early race to dozens of chat channels;
- no AI-company/org-chart metaphor in the trusted core;
- no Python REPL or Node runtime inside privileged authority;
- no unrestricted "everything is a privileged plugin" architecture;
- no graph database as canonical memory or authority;
- no skill, MCP server, plugin, model, channel, or worker that can self-mint authority;
- no YOLO/autonomy bypass as default UX;
- no cloud fallback hidden behind local failure;
- no benchmark or superiority claim without reproducible evidence;
- no broad ecosystem expansion before Spec 005/006 trust and tool primitives close cleanly.

## Immediate priority order

### P0 — finish current canonical Spec 005

Do not widen the active implementation merely because this roadmap exists.

Finish, in canonical dependency order:

- bounded local tools;
- Git read qualification;
- Context Compiler;
- governed memory;
- effect-backed mutations;
- production executor qualification where authorized;
- skills/MCP/ACP boundaries;
- adversarial convergence;
- exact-head CI and independent review;
- guarded merge and post-merge main proof.

Where naturally compatible with already-authorized Spec 005 contracts, prefer implementations that preserve future `CapabilityDescriptor`, `EvidenceReceipt`, provenance-relation, and evidence-requirement evolution without expanding scope.

### P1 — Spec 006–008

- semantic desktop control;
- gateway/connect;
- durable workers;
- Autonomy Profiles;
- atomic work leases;
- long-running recovery.

### P2 — Spec 009–011

- capability ecosystem;
- evidence graph/decision intelligence;
- governed harness improvement;
- multi-agent operations.

### P3 — Spec 012–013

- developer platform/distribution;
- full benchmark/release qualification;
- ecosystem breadth only after trust primitives are proven.

## Competitive doctrine

Golam should compete on properties that become more valuable as agents gain more autonomy:

1. **Authority separation** — the model cannot silently become the security principal.
2. **Evidence** — claims and actions have attributable proof.
3. **Durability** — long-running work survives failures without duplicating effects.
4. **Target identity** — path/string names are not confused with real protected objects.
5. **Context sufficiency** — retrieval is driven by evidence needs, authority, freshness, and contradiction.
6. **Memory governance** — remembered information remains user-owned evidence rather than automatic truth.
7. **Replaceability** — models/providers/tools can change without redefining the trusted semantics.
8. **Local ownership** — local operation is real architecture, not a UI setting.
9. **Human control** — approvals, takeover, budgets, and explanation remain first-class.
10. **Developer clarity** — a coherent capability/evidence model replaces ad-hoc tool integrations.

## Completion criterion for this strategic roadmap

This document becomes program authority only after normal Golam governance converts its accepted parts into canonical specs/tasks/contracts through exact-head CI, substantive independent review, guarded merge, and post-merge verification.

Until then:

```text
ROADMAP_V2=STRATEGIC_CANDIDATE
CURRENT_SPEC_005_SCOPE_WIDENED=NO
CURRENT_CANONICAL_TASK_ORDER_CHANGED=NO
REFERENCE_SOURCE_CODE_ADMITTED=NO
GROK_SUPERIORITY_NUMERIC_CLAIM=UNPROVEN_AND_NOT_PUBLISHED
FOUNDER_GOAL=MAXIMAL_MATERIALLY_VERIFIABLE_SUPERIORITY
NEXT_EXECUTION_PRIORITY=FINISH_CANONICAL_SPEC_005
```
