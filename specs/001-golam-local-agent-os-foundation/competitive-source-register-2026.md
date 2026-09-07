# Competitive Source Register — 2026-09

**Status**: RESEARCH / REFERENCE REGISTER — NO AUTOMATIC SOURCE OR DEPENDENCY ADMISSION

This register records the exact source states or public product evidence used to strengthen the Golam program. Each owning implementation spec must independently pin, qualify and admit any selected code/dependency/runtime through Source Foundry before use.

## Persistent agents and autonomous teammates

### Grok Bot — public behavior reference

Public official product evidence reviewed:

- `https://x.ai/news/introducing-grok-bot` — 2026-08-11;
- `https://docs.x.ai/grok-bot/overview` — current 2026-09 public overview;
- `https://docs.x.ai/grok-bot/skills-routines-and-automations` — public skills/routines behavior;
- `https://x.ai/news/grok-bot-more-plans` — 2026-08-26.

Useful behavior:

- persistent named teammates;
- persistent computer/workspace state;
- multi-Bot parallelism and handoff/group behavior;
- app/browser/terminal work;
- learned workflows/routines from demonstration;
- approval-oriented unattended work;
- desktop/mobile continuity.

Golam differentiation requirement:

- local canonical authority rather than mandatory cloud state;
- isolated worker identities/capability leases instead of shared computer state being treated as security isolation;
- durable Effect Gate and ambiguous-outcome reconciliation;
- explicit provenance/governed memory and skill/routine activation;
- strict-local execution profile.

Disposition: `PUBLIC_BEHAVIOR_REFERENCE_ONLY`.

### NousResearch/hermes-agent

Pinned research state:

```text
repository=NousResearch/hermes-agent
commit=03f3b09222b8f03becb203a6ebb9bac1f927b8b6
```

Useful mechanisms/behavior to study:

- persistent memory and session search;
- progressive-disclosure Agent Skills;
- autonomous skill candidate creation/improvement behavior;
- cron/scheduled jobs including no-agent execution;
- Telegram/Discord/Slack/WhatsApp/Signal gateway behavior;
- provider/model portability and fallback routing;
- isolated subagents and lifecycle UX;
- multiple execution backends.

Golam rules:

- learned content is candidate data until governed activation;
- fallback providers cannot violate explicit locality/privacy requirements;
- message/channel identity does not grant machine authority;
- any code reuse requires exact Source Foundry admission.

Disposition: `HIGH_VALUE_REFERENCE_AND_SOURCE_FOUNDRY_CANDIDATE`.

### letta-ai/letta-code

Pinned research state:

```text
repository=letta-ai/letta-code
commit=a501f8c4557e49812d66c7b801eff0ade5516343
version_at_pin=0.31.13
```

Useful mechanisms/behavior to study:

- durable named agent identity across conversations/models/computers;
- git-backed projected memory/context state;
- recall across conversations;
- agent-owned skill learning;
- memory/context evolution and rollback visibility;
- subagents and persistent experience;
- local harness customization through mods/hooks/permissions.

Golam differentiation requirement:

- memory, skills, prompts or harness behavior that affects production decisions cannot become trusted merely because the agent rewrote them;
- learning produces versioned candidate artifacts first;
- authority analysis, taint/provenance checks, tests/replay and required approval precede activation;
- active skill/routine behavior cannot silently rewrite itself in place;
- secrets remain brokered/protected rather than stored in ordinary git-backed memory;
- canonical user-owned evidence and protected kernel authority remain distinct from context projection.

Disposition: `HIGH_VALUE_PERSISTENT_AGENT_REFERENCE_AND_SOURCE_FOUNDRY_CANDIDATE`.

## Developer/coding agents

### openai/codex

Pinned research state:

```text
repository=openai/codex
commit=4f1a2bb5ffed8fd28518925c1d4085ead158e304
```

Useful areas:

- Rust local agent client architecture;
- sandbox/permission/approval configuration;
- multi-agent runtime/session/fork semantics;
- MCP and skills lifecycle/product surfaces;
- terminal/TUI/IDE/desktop separation;
- developer experience around long-running tasks and steering.

Golam rules:

- provider-specific semantics do not define Golam kernel authority;
- no approval/sandbox mechanism replaces Golam capability/effect/taint rules;
- code reuse requires Source Foundry.

Disposition: `HIGH_VALUE_REFERENCE_AND_SOURCE_FOUNDRY_CANDIDATE`.

### OpenHands/OpenHands

Pinned research state:

```text
repository=OpenHands/OpenHands
commit=f7fb0c4b21f5ed726edbba8a6309634ef434b004
```

Useful areas:

- control surface separated from agent execution backend;
- local/VM/cloud backend topology;
- backend switching and automation-server separation;
- agent canvas concepts for conversations/files/terminal/browser/workspaces.

Golam rules:

- reachable backend != authenticated/authorized principal;
- cloud backend remains optional capability, not local canonical authority;
- execution/workspace isolation must integrate Golam leases/effects.

Disposition: `REFERENCE_AND_SOURCE_FOUNDRY_CANDIDATE`.

### SWE-agent / SWE-ReX / SWE-bench

Pinned SWE-ReX research state:

```text
repository=SWE-agent/SWE-ReX
commit=5c995c365dfb1fd5bc56fda688be5d8538f9931f
```

Use SWE-agent/SWE-bench as Agent-Computer Interface and coding-evaluation references. Use SWE-ReX as an execution-backend architecture reference: agent logic can remain stable while execution moves across local processes, containers or remote infrastructure, and interactive shell sessions remain explicit runtime objects.

Golam rules:

- execution backend substitution cannot widen capability, network, secret or filesystem authority;
- remote execution requires explicit non-strict locality/network authority and exact backend identity;
- remote sandbox state is not canonical Golam authority or memory merely because a worker ran there;
- credentials for remote execution remain brokered secret handles;
- benchmark convenience cannot become a production security boundary without independent Golam qualification.

Disposition: `BENCHMARK_EXECUTION_FABRIC_AND_ARCHITECTURE_REFERENCE`.

### e2b-dev/E2B

Pinned research state:

```text
repository=e2b-dev/E2B
commit=e2612237c1e0f638943fa30a94317de1180423a0
```

E2B provides an open-source SDK/control surface for isolated cloud sandboxes and separate infrastructure for VM-oriented execution. It is useful as a reference for snapshot/resume, remote execution lifecycle and agent-compute product UX.

Golam rules:

- E2B/cloud sandbox use, if ever admitted, is an explicit optional execution capability unavailable to strict-local profiles;
- cloud sandbox reachability or provider identity never becomes Golam authority;
- every remote filesystem/process/network capability remains bounded by Golam authority and evidence;
- snapshots are execution-state artifacts, not replacements for Golam's canonical event/effect/memory state;
- no E2B dependency/service is admitted by this register.

Disposition: `OPTIONAL_REMOTE_EXECUTION_REFERENCE_CANDIDATE`.

### block/goose

Use as local/extensible model-agnostic MCP/tooling reference. Resolve the current canonical repository location and exact commit in the owning research pass before any code/dependency qualification.

Disposition: `REFERENCE_CANDIDATE_REQUIRES_EXACT_PIN`.

## Browser and application agents

### W3C WebDriver / WebDriver BiDi

Official standards reference for the browser semantic/protocol control route.

Golam use:

- provider architecture and protocol semantics;
- capability discovery and exact browser-version qualification;
- stronger route before raw desktop input.

Disposition: `OFFICIAL_ARCHITECTURE_REFERENCE`.

### ServiceNow/BrowserGym

Pinned research state:

```text
repository=ServiceNow/BrowserGym
commit=9e779f087de9a65668b6974d11f9ce9816026e96
```

Use as browser-agent evaluation/reference across WebArena/WorkArena/AssistantBench-style environments and temporal/browser evaluation tooling.

Disposition: `BENCHMARK_CANDIDATE`.

### browser-use/browser-use

Use as browser-agent behavior/tooling and benchmark reference; exact pin required before bounded technical qualification.

Golam must not inherit stealth/proxy/cloud-browser assumptions into strict-local profiles.

Disposition: `REFERENCE_CANDIDATE_REQUIRES_EXACT_PIN`.

### openclaw/openclaw

Use dedicated agent browser/profile design as a behavioral reference. The useful pattern is an agent-controlled browser profile separated from the user's normal browser session.

Golam must add authenticated capability/secret/effect boundaries and must not treat loopback/local gateway reachability as authority.

Disposition: `REFERENCE_CANDIDATE_REQUIRES_EXACT_PIN`.

## Desktop/computer benchmarks

### xlang-ai/OSWorld-V2

Pinned current research state:

```text
repository=xlang-ai/OSWorld-V2
commit=ae70e2b2670c80daa5b7303135b483a859a2d137
```

Use only after GolamBench freezes one exact benchmark fixture version including code/tasks/assets/websites/environment setup. Never claim results from floating `main`.

Disposition: `PRIMARY_DESKTOP_BENCHMARK_CANDIDATE`.

### WindowsAgentArena

Windows-native external evaluation candidate for real application tasks. Exact source/task/environment version must be pinned before use.

Disposition: `PLATFORM_BENCHMARK_CANDIDATE`.

## Memory and long-horizon behavior

### xiaowu0162/LongMemEval-V2

Pinned research state:

```text
repository=xiaowu0162/LongMemEval-V2
commit=2cc8c540bdb87fe6761629b585e727e1c4704520
```

Use for agentic long-term memory evaluation. Add Golam-specific cases for:

- stale-memory resistance;
- authoritative-source precedence;
- contradictions;
- provenance/taint;
- memory abstention;
- user-edited canonical Markdown reconciliation;
- FORGET/REDACT behavior.

Disposition: `PRIMARY_MEMORY_BENCHMARK_CANDIDATE`.

## Repository/context intelligence

### redhat-et/ripwire

Pinned state already tracked in Issue #25:

```text
repository=redhat-et/ripwire
commit=2848e64c16ea09022579d8862d6fe35dd9a1a58b
```

Preferred evaluation order:

`external CLI benchmark -> optional sandboxed sidecar/MCP -> selective port only after measured need`.

Disposition: `OPTIONAL_LOCAL_TOOL_AND_SELECTIVE_PORT_CANDIDATE`.

## Supply chain and release security

Candidates/references to qualify in the release-security owning spec:

- `cargo-vet` — dependency audit evidence;
- `cargo-deny` — advisories/licenses/bans/source policy;
- RustSec advisory database;
- GitHub artifact attestations and SLSA guidance;
- SBOM tooling compatible with exact shipped Rust/npm/Tauri artifacts;
- Tauri signing/updater documentation;
- platform code-signing/notarization mechanisms;
- `cargo-dist` or equivalent reproducible packaging candidate.

These tools do not own Golam source policy; they provide evidence/checking mechanisms under Golam governance.

Disposition: `SOURCE_FOUNDRY_AND_RELEASE_TOOL_CANDIDATES`.

## Source acceptance rule

For every source in this register:

```text
REFERENCE_EVIDENCE != CODE_ADMISSION
BENCHMARK_RESULT != AUTHORITY
POPULARITY != TECHNICAL_QUALIFICATION
LOCAL_EXECUTION != AUTHENTICATION
MODEL_OUTPUT != VERIFIED_FACT
REMOTE_SANDBOX != CANONICAL_AUTHORITY
SELF_MODIFIED_CONTEXT != TRUSTED_POLICY
```

If a later spec wants code, packages, binaries, fixtures, benchmark datasets or generated assets, it must create an exact per-source admission/qualification record first.
