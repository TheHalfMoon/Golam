# AutoClaw / Z.AI Adoption Review — 2026-09-22

**Status:** PROGRAM RESEARCH / SOURCE FOUNDRY INPUT — PLANNING ONLY  
**Target:** Golam Local/Private Verified Agent OS  
**Founder permission:** founder states Z.AI granted permission to copy/use available Z.AI AutoClaw-related source code  
**Authority:** this document does not admit code, models, cloud services or implementation scope

## 1. Exact reviewed product/source universe

### Official AutoClaw product material

Reviewed 2026-09-22:

- https://autoclaw.z.ai/
- https://autoclaw.z.ai/blog/product/what-is-autoclaw/
- https://autoclaw.z.ai/blog/product/autoclaw-cluster-mode-professional-team/
- https://autoclaw.z.ai/blog/product/multi-agent-work-life-isolation/
- https://autoclaw.z.ai/blog/product/hermes-self-evolution/
- https://autoclaw.z.ai/blog/product/autoclaw-v1-9-0-glm-5-2-auto-design/
- https://autoclaw.z.ai/models/
- https://autoclaw.z.ai/changelog/
- https://autoclaw.z.ai/download/

Observed product claims/features include:

- local desktop app for Windows/macOS;
- 50+ built-in skills;
- browser/file/script/tool execution;
- Discord/Telegram/WhatsApp/Lark-style IM control;
- model switching across GLM/DeepSeek-class providers;
- long-running work;
- multi-agent isolation;
- Cluster Mode;
- Hermes self-evolution;
- Auto Design / PRD-to-product workflows;
- professional document/data/content deliverables;
- local machine execution with model calls requiring network in the official product.

The official AutoClaw blog states that AutoClaw is built on the OpenClaw open-source framework.

### Public code reviewed

| Source | Pin | License observed | Golam disposition |
| --- | --- | --- | --- |
| \`openclaw/openclaw\` | \`e9df70639592b85e6c6f15e609b5a4a1c53b1b18\` | MIT | high-value upstream reference/source candidate for channels, automation, sessions/memory, plugins, multi-agent operator lifecycle and completeness rubrics |
| \`zai-org/GLM-skills\` | \`2ecd31c37e75671a4767342ba3a68a84c8f1b848\` | Apache-2.0 | high-value bounded skill-pack source candidate |
| \`zai-org/Open-AutoGLM\` | \`86f55382982fb054e8fc98ca80609dff8a2cdc3c\` | Apache-2.0 | bounded mobile visual-control/action-provider source candidate |
| \`zai-org/ZCode\` | \`872ad960de7ec172591f7e1952f7849229f94521\` | Apache-2.0 | coding-agent harness / browser / artifact / architecture-governance source candidate |
| \`zai-org/Synapse\` | \`651d39d92ff08beff0868f991c36a27c20191726\` | Apache-2.0 | collaboration/workspace/plugin/memory and self-improvement reference/source candidate |

The Z.AI permission attestation applies to Z.AI source code made available to the founder under that permission. Independent upstream OpenClaw remains governed by its own MIT license and third-party notices; Z.AI permission is not treated as the rights source for independent upstream code.

## 2. Product thesis to retain

AutoClaw demonstrates a valuable user-facing simplification:

\`\`\`text
user goal
-> one conversation
-> agent plans/executes
-> tools/files/browser/skills
-> progress
-> finished artifact/result
-> result delivered back into the user's channel
\`\`\`

Golam should preserve this simplicity while retaining stronger internal contracts:

\`\`\`text
user goal
-> TaskContract
-> DeliveryFormation / Worker graph
-> Authority / Capability / Context
-> Effects
-> Evidence / Verification
-> artifact/result
-> channel projection
\`\`\`

The user should experience "tell Golam what you need"; the implementation must remain inspectable and authority-safe.

## 3. Cluster Mode: adopt the delivery discipline

AutoClaw Cluster Mode's strongest lesson is not "use many agents." It is a mandatory delivery process for sufficiently complex tasks:

\`\`\`text
understand
-> plan
-> research
-> parallelize where useful
-> independent audit/review
-> revise
-> deliver to spec
\`\`\`

Golam already has T163 DeliveryGraph, T167 canonical Task/Run/Worker identities, T184 dissent, T216 multidimensional verification and T185 UI projection.

The gap is the adaptive **formation contract** connecting those primitives into one operator-visible mode.

Retain:

- complexity-sensitive solo vs team formation;
- named role responsibilities;
- independent reviewer/auditor where conclusions/code/actionable advice require it;
- parallel work only when dimensions are separable;
- source/evidence traceability;
- progress as real task state rather than textual promises;
- revise-after-review before delivery;
- artifact/deliverable quality profile selected by task need.

Do not retain:

- model consensus as authority;
- self-review as sufficient verification for protected actions;
- fixed agent-count marketing;
- mandatory parallelism for simple work.

## 4. Multi-agent isolation and cross-channel continuity

AutoClaw's product pattern is strong:

- different agents can have separate personality, memory and workspace;
- the same agent can be reached from multiple IM surfaces while preserving continuity.

Golam should make this explicit with one canonical binding model:

\`\`\`text
AgentIdentity
  -> MemoryNamespace
  -> WorkspaceBinding
  -> AccountBindings
  -> Skill/CapabilityProfile
  -> allowed ChannelBindings[]
\`\`\`

Channel identity does not become principal authority.

The same agent across Telegram/WhatsApp/Discord may share the same canonical memory/task state only when the binding is explicit. Two agents never share memory merely because they run on the same device or participate in the same group/thread.

Required separation:

- agent identity;
- workspace;
- memory namespace;
- task/session namespace;
- account/credential bindings;
- channel bindings;
- capability profile;
- privacy profile.

Shared project artifacts must be explicit canonical objects, not ambient cross-agent filesystem or credential leakage.

## 5. Hermes self-evolution: adopt the lifecycle, strengthen the authority

AutoClaw describes two useful learning levels:

1. regular evolution: user preferences/corrections become persistent rules;
2. skill evolution: repeated/complex workflows become reusable skills.

Golam already has T122/T123/T162/T215. The missing product-level contract is the low-friction path from a correction/preference/tool gotcha to a governed candidate.

Target:

\`\`\`text
correction / failure / preference / better pattern
-> LearningObservation
-> candidate type:
     PreferenceRuleCandidate
     ToolKnowledgeCandidate
     AgentWorkflowCandidate
     Skill/WorkflowCandidate
-> provenance + scope + taint
-> conflict detection
-> preview
-> explicit activation policy
-> immutable revision
-> outcome evidence
-> supersede/revoke
\`\`\`

AutoClaw/Synapse-style AGENTS/SOUL/TOOLS promotion is a useful reference, but Golam should not directly mutate active prompt files as protected truth.

Rules:

\`\`\`text
LEARNING != ACTIVE_RULE
CORRECTION != GLOBAL_PREFERENCE
MODEL_SELF_CRITIQUE != VERIFIED_FAILURE
PROMOTED_RULE != AUTHORITY
SKILL_EVOLUTION != IN_PLACE_MUTATION
\`\`\`

A user's explicit "from now on..." instruction can create a strongly supported candidate, but protected behavior still follows the owning policy/activation semantics.

## 6. Skill packs and prerequisites

Z.AI GLM-skills exposes a practical skill packaging pattern with:

- \`SKILL.md\`;
- declared environment/bin prerequisites;
- scripts;
- references;
- task-specific workflows.

Golam should adapt this into its existing T154/T165/T180/T206/T215 contracts rather than invent a new plugin authority.

A \`SkillPackRevision\` should bind:

\`\`\`text
skill_id
version
source_revision
publisher/provenance
manifest_digest
entrypoints
references/assets/scripts
required_bins
required_env_names
required_models/providers
network/egress needs
filesystem/data classes
operation/effect classes
runtime/isolation profile
dependency closure
license/NOTICE
qualification evidence
activation/revocation state
rollback predecessor
\`\`\`

Secrets are referenced by handles; environment variable names in a manifest do not authorize secret release.

Progressive disclosure is preferred: catalog metadata first, full instructions/assets only when selected.

## 7. IM as a control surface, not an authority root

AutoClaw's IM UX is valuable: assign work from a chat and receive progress/files/results in the same thread.

Golam already has T169 and GolamConnect/channel bridges. Strengthen the product contract so a channel can project:

- task creation;
- progress;
- blocker/clarification;
- approval request UI where the channel is qualified;
- artifact/result delivery;
- next-step proposal.

But:

\`\`\`text
CHANNEL_ACCOUNT != GOLAM_PRINCIPAL
DISPLAY_NAME != IDENTITY
MESSAGE_CONTENT != OWNER_PRESENCE
CHANNEL_THREAD != TASK_AUTHORITY
CHANNEL_APPROVAL_UI != APPROVAL_AUTHORITY
\`\`\`

Provider-stable IDs plus current Golam identity/pairing/approval rules remain required.

## 8. Open-AutoGLM: bounded mobile visual-control donor

The reviewed public source implements a simple model loop:

\`\`\`text
capture screenshot/current app
-> model response
-> parse bounded action
-> execute tap/type/swipe/etc.
-> repeat
\`\`\`

Useful bounded components/patterns:

- Android ADB / HarmonyOS HDC / iOS XCTest adapters;
- normalized coordinate mapping;
- screenshot acquisition;
- app launch/input/swipe/back/home primitives;
- takeover callback;
- sensitive-action confirmation hook;
- multi-device IDs;
- explicit model client separation.

Golam strengthening:

- provider action never dispatches directly from model output to device;
- parse output into T199 Operation/Effect candidate;
- target identity and consequence classification happen before dispatch;
- protected lease/takeover generation is revalidated;
- confirmation hook becomes canonical Golam approval/OwnerPresence path rather than a local callback;
- screenshot/model output remains tainted evidence;
- route order remains constitutional;
- post-action verification is mandatory where the operation requires it.

Disposition: **bounded adapter/port candidate**, not agent-loop authority.

## 9. Auto Design / professional deliverables

AutoClaw's dedicated design/product experience and GLM-skills PRD-to-app/document workflows reinforce an existing Golam principle:

- choose specialized workflow/provider contracts for artifacts;
- deliver editable/inspectable artifacts rather than only prose;
- render and visually verify outputs;
- preserve semantic source plus rendered derivative;
- support iterative refinement.

Map into T193 Semantic Artifact Provider, T215 WorkflowIR and Experience projections. No separate AutoClaw-style design authority is required.

## 10. OpenClaw completeness rubrics as an external parity harness

OpenClaw's current repository contains detailed completeness rubrics covering:

- multi-agent orchestration;
- session/memory/context;
- channels;
- automation/cron/hooks/tasks;
- plugins;
- security/auth/pairing/secrets;
- browser/sandbox;
- voice;
- platform clients;
- providers;
- observability.

These are valuable as an external **operator-lifecycle parity checklist**, not as Golam's architecture authority.

Golam should periodically map each relevant rubric item to:

\`\`\`text
SUPPORTED
PLANNED
INTENTIONALLY_DIFFERENT
OUT_OF_SCOPE
BLOCKED
UNKNOWN
\`\`\`

and require evidence links for SUPPORTED.

This prevents a technically sophisticated core from shipping with obvious operator lifecycle gaps such as setup without removal, run without recovery, install without rollback, or channel send without health/status diagnostics.

## 11. Source-admission strategy

Recommended reuse modes:

| Source | Default reuse mode |
| --- | --- |
| AutoClaw proprietary/product-only behavior | REIMPLEMENT_BEHAVIOR / REFERENCE unless exact source is separately supplied and admitted |
| OpenClaw | SELECTIVE_COPY / ADAPTER / REIMPLEMENT_BEHAVIOR by component |
| Z.AI GLM-skills | SELECTIVE_COPY / ADAPTER for bounded skills |
| Open-AutoGLM | PORT_TO_RUST or ADAPTER for device primitives; do not copy agent authority loop |
| ZCode | SELECTIVE_COPY / REIMPLEMENT_BEHAVIOR for browser/artifact/governance tooling |
| Synapse self-improvement | REIMPLEMENT_BEHAVIOR / SELECTIVE_COPY for learning-candidate tooling |

Every exact component still passes Source Foundry, T165 security admission, T197 TCB budget and T180 provenance/revocation.

## 12. What AutoClaw does not change

AutoClaw does not justify weakening:

- Rust-first protected runtime;
- one canonical Effect path;
- strict-local dominance;
- explicit egress;
- exact secret/account binding;
- independent verification;
- current-authority checks;
- source admission;
- no in-place autonomous active-skill mutation;
- voice/channel non-authentication rules.

## 13. Integration disposition

\`\`\`text
AUTOCLAW_STUDIED=YES
AUTOCLAW_PRODUCT_BEHAVIOR_REFERENCE=YES
ZAI_SOURCE_PERMISSION_ATTESTED_BY_FOUNDER=YES
AUTOCLAW_PUBLIC_DESKTOP_SOURCE_REPO_IDENTIFIED=NO
OPENCLAW_UPSTREAM_SOURCE_CANDIDATE=YES
ZAI_GLM_SKILLS_SOURCE_CANDIDATE=YES
OPEN_AUTOGLM_SOURCE_CANDIDATE=YES
ZCODE_SOURCE_CANDIDATE=YES
SYNAPSE_SOURCE_CANDIDATE=YES
AUTOCLAW_MONOLITHIC_COPY=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_IMPLEMENTATION_AUTHORITY_GRANTED=NO
ACTIVE_SPEC_006_WIDENED=NO
\`\`\`
