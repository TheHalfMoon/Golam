# Contract: ExecutionProfile and Routing

This contract specializes the frozen Spec 001 `ExecutionProfile` for Spec 004 without weakening or removing any frozen field.

## Required fields

An `ExecutionProfile` binds:
- immutable `profile_id` and schema version;
- exact model identity and revision;
- tokenizer identity;
- chat-template identity;
- backend identity/build/adapter version;
- locality: `LOCAL` or `EXPLICIT_CLOUD`;
- quantization/precision;
- hardware/device mapping;
- harness profile;
- reasoning mode;
- tool-call conformance: `NATIVE_TOOLS`, `GRAMMAR_CONSTRAINED`, or `TEXT_PROTOCOL_FALLBACK`;
- tool-schema mode;
- sampling parameters;
- context policy;
- prompt/prefix cache policy;
- KV-cache policy;
- warm-residency policy;
- workload class: `INTERACTIVE`, `BATCH`, or `BACKGROUND`;
- multimodal capability flags;
- resource, time and token budgets;
- latency/quality budget;
- privacy constraints;
- network constraints;
- explicit load/failure/fallback policy;
- benchmark evidence references;
- canonical content digest.

## Identity

A material field change MUST yield a distinct profile identity/content digest. This includes a changed model revision, template, backend build, quantization, harness policy, tool mode, privacy/network class or fallback rule.

Evidence recorded for profile A MUST NOT be silently reused as evidence for materially changed profile B.

## Selection order

Routing MUST apply hard compatibility before preference/ranking:

```text
requested/pinned constraints
 -> privacy/locality/network compatibility
 -> backend/model availability
 -> hardware compatibility
 -> resource/workload budgets
 -> evidence-based preference
```

A later preference score cannot override an earlier hard incompatibility.

## Strict-local

When strict-local is active:
- only `LOCAL` profiles are eligible;
- backend launch/load behavior must be qualified for no unauthorized external egress;
- model download/update/telemetry/RPC/cloud fallback is not implicitly permitted;
- failure reports clearly rather than widening locality.

## Explicit cloud representation

Spec 004 may define `EXPLICIT_CLOUD` in the schema for completeness but does not require a production cloud adapter. Any later cloud use still requires explicit user/policy authority and an owning egress-capable implementation scope.

## User pinning

A user MAY pin an exact profile within policy. Pinning does not override:
- strict-local hard denial;
- unavailable/incompatible hardware;
- revoked/denied resource/effect authority;
- invalid/corrupt profile definition.

## Fallback

Fallback MUST be an explicit ordered set or policy that names allowed target classes/profiles. A fallback may change model/backend only inside the explicitly allowed privacy/network class and resource constraints.

No hidden "try cloud if local fails" rule exists.

## Profile switching

A profile switch that affects execution is attributable canonical state/event evidence. The next model request binds the new exact `profile_id`.

An in-flight attempt does not silently change its profile; a changed route requires a new attributable attempt.

## Cache semantics

Prompt/prefix/KV/warm-residency caches are performance state, not canonical history.

Cache reuse MUST be invalidated or treated as miss when material profile/template/context identity makes reuse unsafe or semantically stale.

Loss/corruption of a cache must degrade performance explicitly, not change privacy/authority/task truth.

## Hardware mapping

Hardware recommendations are compatibility/performance evidence only. `HardwareProfile` cannot grant device/network authority and cannot silently enable a backend feature excluded by the profile.

## Benchmark binding

Every benchmark record MUST bind:
- exact code revision;
- `profile_id`;
- `HardwareProfileId`;
- workload fixture/version;
- backend build/source identity;
- harness schema/profile identity;
- raw evidence references.

## Invariants

`PROFILE != AUTHORITY`
`PROFILE_COMPATIBILITY != PERMISSION`
`PROFILE_SWITCH != HISTORY_REWRITE`
`CACHE_STATE != CANONICAL_STATE`
`LOCAL_FAILURE != CLOUD_FALLBACK_PERMISSION`
