# Research: Golam Local Agent OS Foundation

**Research closeout date**: 2026-08-24  
**Decision**: `DISCOVERY_COMPLETE_FOR_IMPLEMENTATION_SPECS`  
**External architecture review**: `GLM_5_3_REVIEWED_APPROVE_WITH_MANDATORY_CHANGES_RECONCILED`

## 1. Final research conclusion

Golam should not be built by porting the old Grok Bot reconstruction into Rust or by stacking agent frameworks. The final direction is a Rust-first local Agent OS with a small authority-bearing privileged kernel and replaceable harness/model/tool/memory/control/connect services.

Generic framework discovery is closed. Future research is triggered only by a concrete implementation choice, failed qualification, security gap, benchmark result, or material protocol/platform change.

The final GLM-5.3 architecture review validated the core direction and required enforceable kernel/IPC/effect/taint/secret/memory/ledger/channel/approval/egress contracts. Those findings are reconciled in the owning Spec 001 artifacts. See `review/glm-5.3-review-result.md` and `review/glm-5.3-reconciliation.md`.

## 2. Final-gap findings

### 2.1 Spec Kit process

Golam uses the modern Spec Kit loop:

`constitution -> specify -> clarify -> plan -> checklist -> tasks -> analyze -> implement -> converge`.

Planning snapshot inspected during the research cycle:
- `github/spec-kit`
- main `27f50f7e6b618ea14d74dd4037f9e7c60218b16c`
- release line 1.0.1 / 1.0.2.dev0 at the time of inspection.

### 2.2 Semantic-first computer control

The final control hierarchy is:

`Domain/App API -> Native OS automation -> Accessibility/Semantic tree -> Browser DOM/protocol -> Input injection -> Vision`.

Windows UIA sources such as `microsoft/winappCli` and semantic-ref desktop projects support the design direction. Golam owns the Rust `DesktopController` contract and platform truth matrix. Reciprocal-license remote-control sources remain behavior references only by default.

### 2.3 Remote-control substrate

`CasualOffice/RASystem` is strongly aligned with GolamConnect: Rust core, Iroh/QUIC, host-issued grants, per-message checks, consent/emergency stop, input/media, clipboard/files and audit. The inspected planning snapshot was head `494a0883bf1ca6bd120069e8aec6052097051a3f`, tree `7031400a69064775ea13dbce8a215edcedf49fc2`, project-reported Apache-2.0.

`n0-computer/iroh` provides Rust public-key dialing, QUIC, hole punching and relay fallback under MIT/Apache-2.0.

Decision: Iroh is a direct-dependency candidate; RASystem is selective-port candidate. Golam does not build a custom NAT/relay stack in P0 and independently qualifies Windows/Linux remote-control behavior before release claims.

### 2.4 Local inference

`mistral.rs` is the primary Rust-native inference candidate behind Golam's `ExecutionProfile` adapter. `llama.cpp` is the broad compatibility backend and should default to an out-of-process sidecar to keep unsafe C FFI outside `golamd`.

Golam owns harness semantics. Ollama/MLX/vLLM/SGLang remain optional adapters only.

### 2.5 Authorization and sandboxing

Cedar remains the policy-engine candidate, but Golam owns capability schema, protected-resource classes, approval semantics and denial behavior.

Wasmtime/WASI is appropriate for bounded portable extension code but is not a universal sandbox for arbitrary native tools. Executable skills, MCP servers and optional adapters require explicit sandbox profiles.

### 2.6 Local security is mechanized

The GLM review identified that "Rust kernel" and "local daemon" were not enough as security claims. Final research therefore freezes:

- smaller privileged kernel distinct from the broader Rust trusted path;
- protected kernel-owned state;
- authenticated local IPC;
- process-splittable kernel API;
- durable effect handlers/reconciliation;
- explicit taint downgrade rules;
- bounded secret fallback/redaction;
- governed memory operations;
- immutable forks/integrity/artifact lifecycle;
- provider-stable channel binding;
- approval classes/freshness;
- kernel-authorized strict-local egress.

## 3. Donor verification and classification

The canonical planning-status register is `donor-verification-register.md`. Its status is deliberately separate from code admission.

Key current classifications:

| Source | Planning classification |
|---|---|
| xAI Grok Bot | BENCHMARK_ONLY / REFERENCE_ONLY |
| xai-org/grok-build | SELECTIVE_PORT candidate |
| deepseek-ai/deepseek-harness | REFERENCE_ONLY — verified Node/TypeScript/Cordis, not Python |
| aaif-goose/goose | SELECTIVE_PORT candidate |
| CopilotKit/OpenBot | REFERENCE_ONLY patterns |
| CasualOffice/RASystem | SELECTIVE_PORT candidate |
| n0-computer/iroh | DIRECT_DEPENDENCY candidate |
| cedar-policy/cedar | DIRECT_DEPENDENCY candidate |
| bytecodealliance/wasmtime | DIRECT_DEPENDENCY candidate when executable extensions need it |
| EricLBuehler/mistral.rs | DIRECT_DEPENDENCY candidate |
| llama.cpp | ADAPTER / sidecar candidate |
| microsoft/winappCli | REFERENCE_ONLY behavior |
| Graphify/code-graph-rag | OPTIONAL ADAPTER only when L2 evidence justifies it |
| Restate/Temporal | REFERENCE_ONLY durability patterns |
| RustDesk/OpenControl/reciprocal sources | REJECT_CODE / REFERENCE_ONLY behavior by default |
| Golam-Research reconstruction | REJECT_CODE / REFERENCE_ONLY behavioral evidence |

No donor is admitted merely by appearing here.

## 4. Source Foundry admission record

Before code admission, the implementation spec MUST record:

- repository URL;
- exact commit/tree/version;
- license/NOTICE obligations;
- generated/vendored boundaries;
- relevant transitive dependency closure;
- reciprocal-license closure;
- network/telemetry behavior;
- credential/secret handling;
- unsafe Rust/FFI/process boundaries;
- platform support and on-device evidence;
- exact files/crates/API proposed for reuse;
- classification: DIRECT_DEPENDENCY | ADAPTER | SELECTIVE_PORT | REFERENCE_ONLY | REJECT;
- independent Golam tests/benchmarks required for acceptance.

`VERIFIED_SNAPSHOT` in the research register still does not equal admission.

## 5. Frozen architectural conclusions

1. Local-first/no hidden cloud fallback is non-negotiable and strict-local egress is mechanically enforced.
2. Rust trusted path and authority-bearing privileged kernel are distinct.
3. Local clients authenticate; localhost is not authority.
4. Canonical event history is append-oriented; forks never rewrite history; context is a projection.
5. Goal Ledger survives ordinary compaction.
6. Effects have explicit execution semantics plus handler/reconciler contracts and UNKNOWN_OUTCOME behavior.
7. Protected authority state is not generic filesystem state.
8. Secret handles are preferred; fallback secret use is bounded/redacted.
9. Taint survives derivation and can only downgrade through human or registered deterministic verification.
10. Model routing means `ExecutionProfile`, not model name alone.
11. Memory is governed user-owned evidence: Markdown canonical knowledge, SQLite operational state, derivatives rebuildable.
12. Context is tiered evidence planning; no mandatory graph DB.
13. Computer control is semantic-first and honest about OS limitations.
14. GolamConnect native transport is separate from third-party messaging bridges and uses host-side per-message authority.
15. Skills/MCP/ACP are interoperability surfaces, not authorities.
16. Internal workers use typed Rust supervision rather than A2A.
17. Long-horizon evaluation includes crash/resume, duplicate-effect prevention, secret isolation, taint survival, goal retention and premature stopping.
18. Start implementation with <=8 real crates; split only on proven boundaries.
19. Single-worker/local foundation precedes swarm/groups/teach-by-demonstration scope.
20. Voice/native mobile/A2A/media generation/custom relay/CRDT memory sync are deferred through Spec 010 unless a later reviewed spec changes scope.

## 6. Research stop rule

New external research is justified only when:
- an implementation task has an unresolved technical choice;
- a security review identifies an unmitigated threat;
- a dependency/donor qualification fails;
- a benchmark reveals a specific capability gap;
- an external protocol/platform changes materially.

Otherwise proceed from the frozen Spec Kit artifacts rather than reopening generic discovery.
