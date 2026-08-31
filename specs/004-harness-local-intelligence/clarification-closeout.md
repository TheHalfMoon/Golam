# Clarification Closeout — Spec 004

**Date**: 2026-08-31  
**Decision**: CLARIFIED_FOR_RESEARCH_AND_PLAN

## C1 — Is the model backend the agent runtime?

**Decision**: No. Golam owns the harness, canonical session/request evidence, cancellation/retry semantics, compaction policy, tool-call normalization, authority/effect boundaries and completion truth. A backend is a replaceable inference service behind a bounded adapter.

`MODEL_BACKEND != HARNESS_OR_AUTHORITY_ROOT`

## C2 — Does model output execute tools?

**Decision**: No. Model output may produce a `ToolCallCandidate`. Golam normalizes and validates it, then later owning tool/runtime layers submit typed requests through existing authority and Effect Gate semantics. A backend-owned agent loop, MCP executor, shell, code execution feature, approval mechanism or tool callback is not an authority path for Golam.

`MODEL_TOOL_CALL != AUTHORITY_OR_EFFECT_COMMIT`

## C3 — What is canonical during a model turn?

**Decision**: the existing durable Golam event/session spine remains canonical. Model-visible context is a projection. Accepted user/model-visible input and accepted model output are attributable to canonical request/session evidence, subject to Spec 003 secret-ingestion redaction/tombstone rules.

## C4 — Can compaction rewrite history?

**Decision**: No. Compaction creates a derived context artifact/projection that references the canonical source range/events and records method/profile, digest, taint/provenance and invalidation conditions. Canonical history remains recoverable. Goal/non-negotiable constraint state stays independently durable outside ordinary conversation compaction.

`COMPACTION != HISTORY_DELETION`

## C5 — How is interrupted streaming represented?

**Decision**: cancellation/timeout is explicit. Any accepted prefix exposed to the user/model-visible history is recorded as interrupted/partial evidence. It is not silently discarded, silently completed, or converted into verified Task success. Backend cancellation acknowledgement and harness terminal state are distinct observations.

## C6 — What is a retry?

**Decision**: a retry is a new attributable request attempt within a bounded request series. It never rewrites the failed attempt. Retry policy distinguishes transient backend/transport failure, context-overflow recovery and semantic/tool-call repair. Existing Effect Gate `UNKNOWN_OUTCOME` and no-blind-retry rules dominate model retry behavior.

## C7 — How are tool-call modes unified?

**Decision**: native tool calls, grammar-constrained output and text-protocol fallback all normalize to one bounded Golam-owned candidate representation before validation. Unknown schema, malformed arguments, ambiguous framing, duplicate IDs or size-limit violations reject without execution.

## C8 — Is `mistral.rs` admitted by planning?

**Decision**: No. `mistral.rs@d348a88833c5da8403e2320997c28cdd02ae4f4b` (workspace version 0.9.2, MIT) is the primary local-backend candidate. Its current workspace includes network-capable/model-download paths and broad agentic/tool features. Implementation may admit only an exact minimal crate/feature surface after Source Foundry qualification proves license/notices, transitive/native/unsafe boundaries, offline behavior and feature closure. Golam will not delegate its agent loop, tools, MCP, code execution, shell, approvals or authority to mistral.rs.

## C9 — How may `llama.cpp` be integrated?

**Decision**: `llama.cpp@010be9683afabe14ce299197b38c329f94bae568` (MIT) is a compatibility candidate. Unsafe/native C/C++ FFI stays outside `golamd`. The default architecture is a supervised sidecar with an authenticated/private local transport boundary. A generic unauthenticated localhost HTTP control surface is not acceptable. Local-file/offline invocation is required for strict-local profiles; model URL/Hugging Face download and RPC options are not silently enabled.

## C10 — Is localhost authentication optional for a model sidecar?

**Decision**: No. Spec 001/002 local-control rules still apply. If a compatibility sidecar uses loopback HTTP, Golam must add authentication and the required local web protections, or use a stronger private OS-local transport. Process locality alone is not identity or authority.

## C11 — What is `ExecutionProfile` identity?

**Decision**: an immutable/versioned or content-addressed definition covering the full frozen Spec 001 contract. A material field change produces a distinct identity. Benchmark/calibration evidence binds the exact identity and cannot silently carry over to a changed profile.

## C12 — What is `HardwareProfile`?

**Decision**: bounded local capability evidence needed for routing/calibration: OS/architecture, CPU class/count where needed, available RAM, accelerator/backend-visible devices, available VRAM/device memory where measurable, backend feature support and calibration observations. It is not a general hardware fingerprinting/telemetry surface.

## C13 — Can calibration change privacy/locality automatically?

**Decision**: No. Calibration may recommend among profiles already permitted by policy. It cannot create authority, enable egress, or move strict-local work to an explicit-cloud profile.

## C14 — What must ordinary CI prove without model weights?

**Decision**: deterministic scripted/mock backends prove request/stream state machines, cancellation, timeout, retry, compaction projection, profile identity/routing and tool-call normalization. Real-model benchmarks are separately reproducible qualification evidence and must not make normal CI depend on network downloads, cloud credentials or specialized accelerators.

## C15 — What do researched harnesses contribute?

**Decision**: behavior/mechanism evidence only unless separately admitted.

- `Golam-Research` demonstrates routing/stream/transcript behaviors but directly couples provider routing to tool execution; Golam rejects that authority coupling.
- `grok-build` demonstrates explicit compaction sampler/retry/error classification/cancellation/telemetry seams worth adapting conceptually.
- Goose demonstrates explicit Rust state-machine, cancellation/retry/compaction/tool-operation separation and persisted message identity.
- DeepSeek Harness demonstrates durable session events, turn/step/request reconstruction, cancellation outcomes, tool-call result accounting and compaction as a separately replaceable capability.

No donor code is admitted by this clarification.

## C16 — Does Spec 004 implement broad tools, context retrieval or memory?

**Decision**: No. Only deterministic fixture-tool schemas/candidates are needed to prove harness conformance. Product filesystem/shell/git/browser, MCP/ACP product integration, context retrieval and long-term memory remain Spec 005.

## C17 — Planning/implementation discipline

**Decision**: the complete Spec 004 planning package must pass its own exact-head CI/convergence/review lifecycle and merge before any Spec 004 product implementation begins. The implementation branch must start from the exact canonical main produced by planning closeout.
