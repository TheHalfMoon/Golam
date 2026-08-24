# Founder Source Permission Attestation

**Recorded**: 2026-08-24  
**Scope**: Spec 001 research/source universe  
**Status**: `FOUNDER_PERMISSION_ATTESTED`

## Attestation

The founder states that permission has been obtained for:

1. all source projects and repositories explicitly supplied by the founder during Golam research; and
2. all source projects and repositories introduced/recommended during Spec 001 research.

This includes `Golam-Research` / the Grok Bot 0.18 reconstruction as source material that may be seriously evaluated for bounded reuse, porting, or implementation guidance.

## What this attestation changes

A source MUST NOT be rejected solely because Spec 001 previously lacked evidence that permission had been obtained. Such sources are eligible to enter Source Foundry qualification.

`Golam-Research` is therefore classified as:

- `HIGH_VALUE_IMPLEMENTATION_EVIDENCE` for architecture, protocols, runtime boundaries, tool/skill behavior, tests, and product behavior;
- `AUTHORIZED_SOURCE_CANDIDATE` for bounded code reuse/porting subject to the admission rule below.

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

Permission is a rights gate, not a technical-quality gate. Golam may still port a donor into Rust, isolate it as an adapter, or reject it for architecture/security/maintenance reasons.

## Admission state machine

```text
REFERENCE
  -> VERIFIED_SOURCE_STATE
  -> PERMISSION_RECORDED
  -> TECHNICALLY_QUALIFIED
  -> ADMITTED
```

No source code enters the trusted product path before `ADMITTED` for the exact bounded component.

## Golam-Research special handling

The repository's own README/NOTICE/PROVENANCE states that it is a working, source-oriented reconstruction grounded in pinned Grok Bot 0.18 artifacts, while also noting that it is not Anysphere's original monorepo and historically did not assert an upstream source-code license.

The founder attestation changes the rights posture for Golam planning from "reference-only by default" to "permission asserted; eligible for admission." It does not erase provenance distinctions. Golam MUST preserve attribution/evidence boundaries and MUST NOT present reconstructed code as original Anysphere source.
