# Tencent Source Adoption Review — 2026-09

**Status**: `RESEARCH_AND_PROGRAM_PLANNING_ONLY`

**Authority**: This artifact does not admit product dependencies, source code, model weights, datasets, services, network access, cloud backends, scanners, skills, MCP servers, or runtime authority. Every later implementation must re-fetch the exact source state and complete the owning Source Foundry / benchmark / dataset-rights / security qualification before use.

**Active Spec 006 PR #24 widened by this artifact**: `NO`

## Source pins reviewed

```text
Tencent/WeKnora@647848f3954dae34473b8a8d0e0eef5e0fb3a58e
Tencent/RoMem@39ac1417b4db41ea729e5c3be71ac20de54da993
Tencent/SkillHone@7d565839fb4dc74f9c77f09ace660e1c0484e048
Tencent/LoopForge@09c765286f549624dd95434e1e6ef2249657cbeb
Tencent/AI-Infra-Guard@e4e622af3ad2b8228ce82dd62b01415dd8ce2b9c
```

## Program-level conclusion

These sources strengthen five different Golam layers. They must not be collapsed into one donor architecture:

1. **WeKnora** → knowledge workspace, revisioned retrieval artifacts, caller-bound memory UX, sandbox/backend product patterns and worker-pool observability.
2. **RoMem** → temporal-memory research: explicit valid time, relation volatility and non-destructive contradiction handling at retrieval time.
3. **SkillHone** → governed whole-skill evolution, private evaluation separation and persistent decision history.
4. **LoopForge** → resumable delivery graphs, complexity-adaptive staged execution and evidence artifacts.
5. **AI-Infra-Guard** → pre-admission Skill/MCP/agent security scanning, static pre-scan, smuggling detection, behavior-alignment checks and security-result interchange.

Golam's protected kernel, capability leases, Effect Gate, taint/provenance, strict-local policy, canonical memory and verification receipts remain architecture authority. Donor popularity or benchmark performance never grants authority.

---

# 1. Tencent/WeKnora

## Exact source state

```text
repository=Tencent/WeKnora
commit=647848f3954dae34473b8a8d0e0eef5e0fb3a58e
license=MIT_FOR_PROJECT_WITH_LISTED_THIRD_PARTY_COMPONENTS_UNDER_THEIR_OWN_TERMS
```

The reviewed source describes an enterprise knowledge/agent framework combining RAG, autonomous tool use, MCP, a skill catalog, session-persistent sandbox backends, revisioned Wiki/knowledge artifacts, long-term memory, connectors, scoped principals/API keys, task queues and worker-pool governance.

The reviewed memory API contains several patterns worth preserving in Golam:

- personal memory identity is derived from the authenticated caller rather than accepted as an arbitrary subject identifier from the client;
- scoped integration keys do not automatically inherit a person's memory;
- inferred memories remain `pending` until confirmation before prompt injection;
- rejected inferred memories leave a tombstone to reduce immediate re-extraction of the same rejected claim;
- memory can be exported, consolidated and individually deleted;
- knowledge/Wiki artifacts have revision history and rollback.

The reviewed sandbox design also exposes a shared backend protocol across Docker/E2B/Cube-style environments with session persistence and configurable resource/network policy. This is relevant to Golam's future `ExecutionBackend` work, but the donor's backend or cloud assumptions are not authority.

## Golam adoption candidate

Create a **Knowledge Workspace** layer separate from canonical personal memory and protected authority state:

```text
KnowledgeArtifact
KnowledgeRevision
KnowledgeSourceBinding
RetrievalProjection
CitationEvidence
KnowledgeWorkspaceId
ConnectorIngestReceipt
```

Required properties:

- source documents and extracted/retrieval derivatives remain distinct;
- every generated Wiki/summary/graph node carries exact source lineage;
- manual edits create immutable revisions rather than silently replacing provenance;
- rollback is a new governed revision/effect, not history erasure;
- retrieval chunks are derivatives and never outrank live authoritative sources merely because they score highly;
- user/private memory is not implicitly shared with knowledge-workspace API clients;
- automatic memory extraction produces candidates, not immediately trusted active memory;
- connector ingestion records source identity, sync position, deletion semantics and stale-source behavior;
- session sandbox state is execution state, not canonical Golam memory or authority.

## Explicit non-adoption

Do not copy the following as Golam trust rules:

- multi-tenant RBAC as a replacement for Golam capability/lease authority;
- remote sandbox reachability as authentication;
- Langfuse/remote observability as a default runtime dependency;
- cloud/provider fallback that can violate strict-local policy;
- autonomous Wiki or memory generation as canonical truth without provenance/verification;
- the donor's entire dependency closure merely to obtain RAG or sandbox behavior.

## Source disposition

```text
WEKNORA_ARCHITECTURE_AUTHORITY=NONE
WEKNORA_BEHAVIOR_REFERENCE=HIGH_VALUE
WEKNORA_CODE_REUSE=REQUIRES_FILE_LEVEL_SOURCE_FOUNDRY_AND_THIRD_PARTY_LICENSE_REVIEW
WEKNORA_PRODUCT_DEPENDENCY_ADMISSION=NO
PREFERRED_USE=SELECTIVE_REIMPLEMENTATION_OF_KNOWLEDGE_WORKSPACE_AND_MEMORY_UX_PATTERNS
```

---

# 2. Tencent/RoMem

## Exact source state

```text
repository=Tencent/RoMem
commit=39ac1417b4db41ea729e5c3be71ac20de54da993
repository_license_metadata=NULL
root_LICENSE=NOT_PRESENT_AT_REVIEWED_REVISION
```

The reviewed repository presents temporal-memory research using continuous temporal scoring, a semantic volatility/speed gate and query-time geometric shadowing so temporally obsolete facts can be down-ranked without destructive deletion. It includes evaluation paths around MultiTQ, LoCoMo, DMR-MSC and FinTMMBench plus bundled baselines/checkpoints.

Because GitHub reports no repository license and no root `LICENSE` exists at the reviewed revision, **no RoMem source code, model checkpoint, dataset, baseline bundle or generated asset is admitted for copying or redistribution** by this planning review.

## Golam research adoption candidate

Golam should add first-class temporal semantics independently of RoMem's implementation:

```text
TemporalClaim {
  observed_at,
  valid_from,
  valid_until,
  source_time_basis,
  volatility_class,
  supersedes,
  contradicts,
  provenance
}

TemporalQueryContext {
  query_time,
  wall_clock_evidence,
  timezone_identity,
  uncertainty
}

TemporalRetrievalEvidence {
  candidate_version,
  temporal_fit,
  source_authority,
  freshness,
  contradiction_state
}
```

Rules:

- time-aware ranking never mutates or deletes canonical history;
- temporal relevance cannot raise source authority;
- a lower-authority recent memory cannot outrank a live authoritative source solely because its timestamp is newer;
- static/dynamic or volatility labels are ranking hints, not authority;
- unknown validity intervals surface uncertainty rather than inventing expiry;
- user corrections create new governed memory versions and contradiction lineage;
- query-time temporal filtering remains deterministic/replayable where possible;
- any learned volatility model is replaceable evidence and cannot define truth.

## Evaluation candidate

Reproduce the research question independently using properly qualified benchmark datasets and Golam-native temporal fixtures. At minimum test:

- current-vs-historical fact questions;
- future/unknown-time abstention;
- conflicting observations from different authorities;
- stale memory versus live source precedence;
- recurring facts and discontinuous validity intervals;
- user correction without destructive history deletion;
- clock/timezone ambiguity;
- temporal retrieval quality versus non-temporal Golam memory.

## Source disposition

```text
ROMEM_ARCHITECTURE_AUTHORITY=NONE
ROMEM_RESEARCH_REFERENCE=HIGH_VALUE
ROMEM_CODE_REUSE=DENIED_PENDING_CLEAR_ROOT_LICENSE_AND_FILE_PROVENANCE
ROMEM_CHECKPOINT_REUSE=DENIED_PENDING_RIGHTS_AND_MODEL_PROVENANCE_QUALIFICATION
ROMEM_DATASET_REUSE=DENIED_PENDING_PER_DATASET_RIGHTS_QUALIFICATION
PREFERRED_USE=INDEPENDENT_TEMPORAL_MEMORY_DESIGN_AND_BENCHMARK_REPRODUCTION
```

---

# 3. Tencent/SkillHone

## Exact source state

```text
repository=Tencent/SkillHone
commit=7d565839fb4dc74f9c77f09ace660e1c0484e048
license=MIT
```

The reviewed source treats the unit of skill evolution as the **entire skill folder**, not only `SKILL.md`; it separates the skill under improvement from held-out evaluation through code/filesystem boundaries; records decisions as persistent Git artifacts; and uses role-separated improvement/evaluation flows with regression validation.

## Golam adoption candidate

Strengthen existing T122/T123 safe learning into a **Whole-Skill Evolution Protocol**:

```text
SkillRevisionCandidate {
  base_skill_version,
  skill_md_delta,
  script_deltas,
  reference_deltas,
  asset_deltas,
  diagnosis_refs,
  training_evidence_refs,
  proposed_by,
  candidate_digest
}

SkillEvaluationReceipt {
  candidate_digest,
  private_eval_fixture_digest,
  evaluator_identity,
  regression_results,
  safety_results,
  contamination_checks,
  outcome
}
```

Required properties:

- active skills are immutable versions;
- the entire skill bundle may evolve atomically when necessary;
- helper scripts/references/assets are part of the governed candidate surface;
- private/held-out evaluation material is inaccessible to the improver except through bounded result summaries;
- train/dev/eval contamination is detectable and release-blocking;
- evaluator identity and fixture digest are recorded;
- a regression can reject or supersede a candidate before activation;
- activation is a separate governed effect after evaluation; merge/commit success alone does not activate a skill;
- every diagnosis → candidate → evaluation → activation/rejection outcome remains replayable persistent decision history;
- skill improvement never grants new capability, network, secret or filesystem authority automatically.

## Explicit non-adoption

- no Claude/Codex/OpenClaw/Hermes bypass mode becomes a Golam production policy;
- arbitrary local subprocess execution is not admitted because a donor workflow uses it;
- the improving model cannot approve its own authority widening;
- evaluation success does not waive Source Foundry for new dependencies/scripts.

## Source disposition

```text
SKILLHONE_ARCHITECTURE_AUTHORITY=NONE
SKILLHONE_METHOD_REFERENCE=VERY_HIGH_VALUE
SKILLHONE_CODE_REUSE=SOURCE_FOUNDRY_CANDIDATE
SKILLHONE_RUNTIME_BYPASS_BEHAVIOR=REJECTED
PREFERRED_USE=GOLAM_NATIVE_WHOLE_SKILL_EVOLUTION_WITH_STRONGER_AUTHORITY_AND_VERIFIER_SEPARATION
```

---

# 4. Tencent/LoopForge

## Exact source state

```text
repository=Tencent/LoopForge
commit=09c765286f549624dd95434e1e6ef2249657cbeb
license=MIT_WITH_THIRD_PARTY_NOTICES
noted_third_party=obra/superpowers_MIT_MODIFIED_BY_TENCENT
```

The reviewed source structures coding work as a complexity-sensitive delivery workflow: understand/clarify → design → implement → independent review → test → deliver, while persisting requirements, design, code/review, E2E evidence, reusable knowledge and workflow summary artifacts so interrupted work can resume.

## Golam adoption candidate

Define a general **DeliveryGraph** above individual agent loops:

```text
DeliveryGraph
DeliveryStage
StageEntryPredicate
StageExitObligation
DeliveryCheckpoint
ResumeCursor
ArtifactManifest
StageVerifier
```

Required properties:

- task complexity can select a streamlined or full workflow without bypassing safety-critical obligations;
- requirements/scope/acceptance criteria become immutable versioned inputs to the execution stage;
- design, implementation, review and verification may use distinct worker principals;
- every stage declares exact entry predicates and verification obligations;
- interruption persists a resume cursor only after durable stage state/evidence is committed;
- resume revalidates live repository/environment/authority state before continuing;
- abort records whether any external effects already occurred and cannot pretend filesystem rollback reversed them;
- workflow artifacts are evidence, not authority;
- user confirmation is requested only where Golam governance/risk policy requires it; LoopForge's confirmation cadence does not become a universal Golam requirement.

This architecture should integrate with T127/T134 persistent workers, T149 verification receipts and T153 snapshots rather than become a separate orchestration kernel.

## Source disposition

```text
LOOPFORGE_ARCHITECTURE_AUTHORITY=NONE
LOOPFORGE_WORKFLOW_REFERENCE=HIGH_VALUE
LOOPFORGE_CODE_REUSE=SELECTIVE_SOURCE_FOUNDRY_CANDIDATE
PREFERRED_USE=GOLAM_NATIVE_RESUMABLE_DELIVERY_GRAPH_AND_ARTIFACT_MANIFEST
```

---

# 5. Tencent/AI-Infra-Guard

## Exact source state

```text
repository=Tencent/AI-Infra-Guard
commit=e4e622af3ad2b8228ce82dd62b01415dd8ce2b9c
root_license=Apache-2.0
```

The reviewed project contains AI red-team tooling including Skill scanning, MCP scanning, agent scanning, component/CVE checks, jailbreak evaluation and LLM API poisoning detection.

The reviewed `skill-scan` design adds particularly useful pre-admission ideas:

- deterministic/static pre-scan before model analysis;
- charset/encoding anomaly and smuggling detection;
- `.pyc` bytecode and build/cache/dependency surface flagging;
- SkillTrustBench-style instruction/memory/code-execution/privilege/toolchain/dependency/code-quality risk categories;
- `SKILL.md` versus implementation intent-alignment/hidden-behavior auditing;
- SARIF 2.1.0 output for machine-readable findings.

The reviewed `mcp-scan` design includes risk classes for secret exposure, privilege/scope creep, tool poisoning, supply-chain attack, command injection, prompt injection, insufficient authentication/authorization, audit gaps, shadow servers, context over-sharing, name confusion, rug-pull behavior and tool shadowing. It also combines static pre-scan with optional isolated dynamic analysis.

The `skill-scan` subtree carries a NOTICE requiring downstream attribution when incorporating or deriving from that project. Any later code reuse must preserve all applicable Apache/NOTICE obligations exactly.

## Golam adoption candidate

Create an **Extension Security Admission** gate before a Skill/MCP/connector/control-route provider/execution backend can become active:

```text
ExtensionSecurityCandidate
DeterministicPreScanReceipt
BehaviorAlignmentReceipt
DependencyRiskReceipt
DynamicSandboxScanReceipt
SecurityDisposition
SecurityRuleId
SecurityFindingFingerprint
```

Minimum deterministic pre-scan:

- hidden executable/binary/bytecode inventory;
- encoding/charset anomaly and mixed-decoder checks;
- archive traversal/symlink/device-file checks;
- suspicious download-and-execute/bootstrap patterns;
- command construction and shell interpolation surfaces;
- credential/metadata endpoint access patterns;
- persistence/autostart/install-hook behavior;
- declared versus actual network/filesystem/process requirements;
- dependency source/lock/integrity/advisory/license checks;
- Skill/MCP declared intent versus executable behavior surface;
- duplicate/confusable tool names and shadowing/rug-pull/version-drift risk.

Security policy:

- deterministic checks run before any LLM-based scanner;
- an LLM scanner may discover/support findings but cannot return an authority-bearing `SAFE` token;
- no extension can self-certify;
- dynamic scans execute only inside an isolated disposable environment with synthetic credentials/data and denied production secrets;
- strict-local profiles cannot require a remote scanner/model;
- network-enabled dynamic analysis requires explicit qualification-only egress and never grants product runtime egress;
- findings use stable machine-readable rule IDs/fingerprints, preferably SARIF-compatible output;
- every extension revision invalidates prior security admission and requires re-scan according to risk policy;
- material behavior drift, tool shadowing or dependency-integrity change suspends activation fail closed;
- known-safe scan results do not replace runtime capability/Effect Gate controls.

## Explicit non-adoption

- the full AI-Infra-Guard web platform is not proposed as a Golam runtime component;
- a scanner service lacking product authentication cannot become a trusted Golam control plane;
- default remote LLM/API endpoints are not acceptable for strict-local qualification;
- benchmark F1 or scanner popularity does not make model-generated findings authoritative;
- red-team tooling cannot be allowed to access real user secrets merely to improve coverage.

## Source disposition

License/provenance guardrail: see `tencent-source-license-findings-2026.md`; the `mcp-scan` selective-port path remains blocked until the exact subtree and selected files complete file-level license and provenance reconciliation.

```text
AI_INFRA_GUARD_ARCHITECTURE_AUTHORITY=NONE
AI_INFRA_GUARD_SECURITY_REFERENCE=VERY_HIGH_VALUE
SKILL_SCAN_SELECTIVE_PORT=SOURCE_FOUNDRY_CANDIDATE_WITH_NOTICE_ATTRIBUTION
MCP_SCAN_SELECTIVE_PORT=BLOCKED_PENDING_FILE_LEVEL_LICENSE_RECONCILIATION
MCP_SCAN_BEHAVIOR_AND_RISK_TAXONOMY=REFERENCE_ONLY_UNTIL_THEN
FULL_PLATFORM_DEPENDENCY=NOT_PROPOSED
PREFERRED_USE=GOLAM_NATIVE_EXTENSION_SECURITY_ADMISSION_PLUS_OPTIONAL_SANDBOXED_SCANNER_ADAPTER
```

---

# Cross-source synthesis

The sources combine into one stronger Golam lifecycle without creating donor trust roots:

```text
Knowledge/experience arrives
        |
        v
Canonical provenance + temporal semantics
        |
        +--> Knowledge Workspace revisions (WeKnora reference)
        |
        +--> Temporal ranking/contradiction evidence (RoMem research reference)
        |
        v
Skill/Routine candidate generation
        |
        v
Whole-bundle candidate + private eval separation (SkillHone reference)
        |
        v
Extension Security Admission (AI-Infra-Guard reference)
        |
        v
Independent verification + authority analysis
        |
        v
Governed activation
        |
        v
Resumable DeliveryGraph / worker execution (LoopForge reference)
        |
        v
Effect Gate + verification receipts + durable outcome evidence
```

## Non-negotiable Golam differences

```text
DONOR_MODEL_OUTPUT_IS_AUTHORITY=NO
DONOR_SCANNER_SAFE_VERDICT_IS_AUTHORITY=NO
MEMORY_RECENCY_CAN_RAISE_SOURCE_AUTHORITY=NO
SKILL_SELF_MODIFICATION_CAN_SELF_ACTIVATE=NO
REMOTE_SANDBOX_IS_CANONICAL_AUTHORITY=NO
WORKFLOW_ARTIFACT_IS_AUTHORITY=NO
BENCHMARK_SCORE_CAN_COMPENSATE_FOR_UNAUTHORIZED_EFFECT=NO
CODE_REUSE_WITHOUT_EXACT_SOURCE_FOUNDRY=NO
UNLICENSED_ROMEM_CODE_REUSE=NO
ACTIVE_SPEC_006_SCOPE_WIDENED=NO
WAIVER_TAKEN=NO
```

## Program task mapping

This review adds program tasks T161–T165 in `program-superiority-tasks.md`:

- T161 — Temporal Memory Semantics and temporal contradiction qualification;
- T162 — Whole-Skill Evolution and eval isolation;
- T163 — Resumable DeliveryGraph and stage evidence;
- T164 — Revisioned Knowledge Workspace and connector-ingest provenance;
- T165 — Extension Security Admission and Skill/MCP security qualification.

These are future program tasks only. Their owning implementation specs must be authorized separately after the current Spec 006 lifecycle reaches canonical closeout.