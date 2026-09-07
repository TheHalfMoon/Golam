# Founder Source Permission Attestation

**Recorded**: 2026-08-24  
**Reaffirmed**: 2026-09-08  
**Scope**: Golam research/source universe recorded in the repository and sources supplied/reviewed during current program research  
**Status**: `FOUNDER_PERMISSION_ATTESTED_AND_REAFFIRMED`

## Attestation

The founder states that permission has been obtained for:

1. all source projects and repositories explicitly supplied by the founder during Golam research; and
2. all source projects and repositories introduced/recommended during Spec 001 and later Golam program research.

This includes `Golam-Research` / the Grok Bot 0.18 reconstruction as source material that may be seriously evaluated for bounded reuse, porting, or implementation guidance.

## 2026-09-08 reaffirmation

The founder explicitly reaffirmed that Golam may copy, use, modify, port and selectively reuse code/source material from the sources supplied during the current research pass and sources already recorded in the repository, subject to exact component-level rights and technical admission.

This reaffirmation means a source should not remain `REFERENCE_ONLY` merely because the repository previously lacked founder reuse permission evidence. It may advance to reuse evaluation when its exact component is otherwise eligible.

The reaffirmation does **not** assert that third-party license, NOTICE, attribution, trademark, model-weight, dataset, service, export, patent or redistribution obligations disappear. It also does not convert an ambiguous/unlicensed upstream component into automatically reusable code when the applicable rights for that exact component remain unclear.

```text
PERMISSION_TO_COPY != TECHNICALLY_ADMITTED
PERMISSION_TO_COPY != LICENSE_OBLIGATIONS_DISAPPEAR
PERMISSION_TO_COPY != DONOR_TRUST_MODEL_ADOPTED
FOUNDER_PERMISSION != PROOF_OF_UNSTATED_THIRD_PARTY_RIGHTS
```

Every owning implementation spec MUST select and justify one primary reuse mode for the exact component:

```text
COPY_AS_IS
SELECTIVE_COPY
PORT_TO_RUST
ADAPTER
REIMPLEMENT_BEHAVIOR
BENCHMARK_ONLY
REJECT
```

The selection should minimize attack surface and maintenance burden while preserving the desired product behavior. The fact that direct copying is permitted does not make direct copying the preferred architecture.

## What this attestation changes

A source MUST NOT be rejected solely because Golam planning previously lacked evidence that founder permission had been obtained. Such sources are eligible to enter Source Foundry qualification.

`Golam-Research` is therefore classified as:

- `HIGH_VALUE_IMPLEMENTATION_EVIDENCE` for architecture, protocols, runtime boundaries, tool/skill behavior, tests, and product behavior;
- `AUTHORIZED_SOURCE_CANDIDATE` for bounded code reuse/porting subject to the admission rule below.

Other recorded sources may similarly become `AUTHORIZED_SOURCE_CANDIDATE` when the applicable exact component rights are compatible with the intended use.

## What this attestation does NOT change

This record is not a claim that every possible use is automatically covered. Before code admission, each implementation spec MUST record the exact permission scope/evidence reference for the selected source/component, including where relevant:

- source repository/artifact and exact commit/tree/version;
- permission grant/evidence reference and permitted acts (use/copy/modify/redistribute/sublicense where applicable);
- license/NOTICE obligations that continue to apply;
- trademarks/branding scope;
- binary/installer/renderer/assets scope;
- model weights/datasets/service credentials or provider terms when applicable;
- vendored/generated code and dependency closure;
- reciprocal/copyleft obligations;
- telemetry/network/secrets behavior;
- selected files/crates and modifications;
- independent Golam security/tests/benchmarks.

Permission is a rights gate, not a technical-quality gate. Golam may still port a donor into Rust, isolate it as an adapter, reimplement only the behavior, keep it benchmark-only, or reject it for architecture/security/maintenance reasons.

## Admission state machine

```text
REFERENCE
  -> VERIFIED_SOURCE_STATE
  -> PERMISSION_RECORDED
  -> RIGHTS_AND_OBLIGATIONS_RECONCILED
  -> TECHNICALLY_QUALIFIED
  -> ADMITTED
```

No source code enters the trusted product path before `ADMITTED` for the exact bounded component.

Implementation admission must additionally preserve:

- exact source provenance;
- exact dependency/runtime closure;
- Golam authority-boundary analysis;
- Source Foundry evidence;
- independent security/correctness qualification appropriate to the component;
- third-party notices/source-offer obligations where applicable.

## Golam-Research special handling

The repository's own README/NOTICE/PROVENANCE states that it is a working, source-oriented reconstruction grounded in pinned Grok Bot 0.18 artifacts, while also noting that it is not Anysphere's original monorepo and historically did not assert an upstream source-code license.

The founder attestation changes the rights posture for Golam planning from "reference-only by default" to "permission asserted; eligible for admission." It does not erase provenance distinctions. Golam MUST preserve attribution/evidence boundaries and MUST NOT present reconstructed code as original Anysphere source.

## Source-specific ambiguity remains fail closed

A broad founder permission record does not override a source-specific ambiguity already recorded by Source Foundry. For example, if a repository/component has no identifiable applicable license or contains third-party material whose redistribution rights are unclear, code reuse remains blocked until the exact rights are reconciled or separate permission evidence clearly covers that material.

Research, behavioral comparison and clean-room reimplementation may remain available where legally and technically appropriate, but the repository must record the selected posture explicitly.

`SOURCE_PERMISSION_REAFFIRMED=YES`

`AUTOMATIC_CODE_ADMISSION=NO`

`WAIVER_TAKEN=NO`
