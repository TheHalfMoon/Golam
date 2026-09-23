# Founder Source Permission Attestation

**Recorded**: 2026-08-24  
**Reaffirmed**: 2026-09-23  
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

## 2026-09-13 reaffirmation

The founder again explicitly reaffirmed permission to use source code from the source universe already recorded in Golam and from the newly supplied 2026-09-13 review set, including the materially reviewed CopilotKit, TinyFish, Desktop Commander, Perplexity, OpenRAG, VoiceStudio, MiMo Code, Ripwire and MarkItDown source families.

This additional reaffirmation changes no technical admission rule. Exact components, transitive dependencies, assets, model artifacts, datasets, services, trademarks and redistribution obligations still require component-level reconciliation and Source Foundry qualification before admission.

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



## 2026-09-22 owner-portfolio reaffirmation

The founder explicitly reaffirmed permission for Golam to evaluate, copy, modify, port and selectively reuse source from the founder-owned GitHub portfolio, including public and private repositories, together with the externally supplied source universe already recorded by Golam.

This public attestation deliberately does not enumerate private repository names or contents. A private repository/component may be considered during confidential planning, but any actual code admission must still create the same exact-component Source Foundry record and separately decide what provenance may be published without disclosing private material.

The reaffirmation is not a blanket technical or legal admission. Third-party code, generated/vendored code, model artifacts, datasets, assets, trademarks, service credentials and transitive dependencies inside an owner repository retain their own rights/NOTICE/security/runtime obligations.

```text
OWNER_REPOSITORY_PERMISSION != COMPONENT_ADMISSION
PRIVATE_SOURCE_ACCESS != PUBLICATION_AUTHORITY
FOUNDER_PERMISSION != TRANSITIVE_THIRD_PARTY_RIGHTS
```



## 2026-09-22 voice/source reaffirmation

The founder reaffirmed that Golam may evaluate, copy, modify, port and selectively reuse the code/source/model material the founder has supplied or explicitly identified as permission-granted for this voice/audio pass, including the newly supplied `AlexWortega/openjev` candidate and the founder-owned GitHub portfolio.

This attestation does not infer permission for proprietary source code merely because a public product or architecture was studied. Meta Muse is currently recorded as public behavior/security reference material only; no Meta proprietary source-code grant is asserted by this record.

Exact model weights, tokenizers, datasets, generated training material, native libraries, assets and transitive dependencies remain separate Source Foundry / Model Artifact Foundry objects with their own rights, NOTICE, integrity, runtime and security qualification.

```text
FOUNDER_PERMISSION_TO_REUSE_SUPPLIED_SOURCE=YES
OPENJEV_PERMISSION_ATTESTED=YES
MUSE_PUBLIC_REFERENCE != MUSE_SOURCE_CODE_PERMISSION
MODEL_CARD_LICENSE != TRANSITIVE_ARTIFACT_CLEARANCE
```



## 2026-09-22 Z.AI / AutoClaw source permission reaffirmation

The founder states that Z.AI granted permission to copy, use, modify, port and selectively reuse **all Z.AI source code made available under that permission** for AutoClaw-related work.

This founder attestation is sufficient to move Z.AI-owned/authorized available source into Golam Source Foundry evaluation without rejecting it merely for lack of permission evidence. It does not by itself identify a public source repository for the proprietary AutoClaw desktop application, and it does not assert rights over independent upstream or third-party code merely because AutoClaw depends on or interoperates with it.

Reviewed Z.AI public repositories include:

```text
zai-org/GLM-skills
zai-org/Open-AutoGLM
zai-org/ZCode
zai-org/Synapse
```

Independent `openclaw/openclaw` is governed by its own MIT license/third-party notices and is not treated as covered by the Z.AI permission assertion.

For any non-public AutoClaw source later supplied by Z.AI/the founder, the exact component still requires a private Source Foundry record before Golam code admission.

```text
ZAI_AVAILABLE_SOURCE_PERMISSION_ATTESTED=YES
ZAI_PERMISSION != INDEPENDENT_UPSTREAM_RIGHTS
ZAI_PERMISSION != THIRD_PARTY_DEPENDENCY_CLEARANCE
AUTOCLAW_PRODUCT_ACCESS != AUTOCLAW_SOURCE_REPO_IDENTITY
PERMISSION_TO_COPY != TECHNICAL_ADMISSION
```



## 2026-09-23 OpenMuse / Laya / OpenJev permission reaffirmation

The founder explicitly states permission to copy, use, modify, port and selectively reuse all available source/model material supplied from:

```text
CopilotKit/openmuse
convaiinnovations/laya
AlexWortega/openjev
```

The founder also reaffirms the previously recorded Z.AI permission for source code made available under the AutoClaw-related grant.

This permission input makes the named source/model material eligible for exact-component/model Source Foundry / Model Artifact Foundry evaluation. It does not automatically admit code, weights, hosted services, transitive dependencies, datasets, trademarks, provider credentials or external service terms.

Specific boundaries:

- `CopilotKit/openmuse` source may be evaluated for bounded copy/port/adaptation. CopilotKit Intelligence is a separately configured service boundary and is not assumed to be covered merely because the OpenMuse repository source is reusable.
- `convaiinnovations/laya` and `AlexWortega/openjev` model artifacts still require T175 exact-file/runtime/base-model/transitive-rights qualification.
- the two unrelated "Laya" sources (`aayushch/laya` and `convaiinnovations/laya`) remain separate legal/technical source identities.
- Z.AI permission applies to source made available under that permission; a non-public AutoClaw desktop source component must still be pinned and recorded when/if supplied.

```text
OPENMUSE_PERMISSION_ATTESTED=YES
LAYA_DECISION_MODEL_PERMISSION_ATTESTED=YES
OPENJEV_PERMISSION_REAFFIRMED_2026_09_23=YES
ZAI_AVAILABLE_SOURCE_PERMISSION_REAFFIRMED_2026_09_23=YES

PERMISSION_TO_COPY != TECHNICAL_ADMISSION
REPOSITORY_LICENSE != HOSTED_SERVICE_RIGHTS
MODEL_PERMISSION != TRANSITIVE_DATASET_CLEARANCE
SOURCE_DISPLAY_NAME != SOURCE_IDENTITY
```

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

`SOURCE_PERMISSION_REAFFIRMED_2026_09_13=YES`

`SOURCE_PERMISSION_REAFFIRMED_2026_09_22=YES`

`SOURCE_PERMISSION_REAFFIRMED_2026_09_23=YES`

`AUTOMATIC_CODE_ADMISSION=NO`

`WAIVER_TAKEN=NO`
