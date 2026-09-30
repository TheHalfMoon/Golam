# Golam Source Permission Attestation Supplement — 2026-09-30

**Status:** FOUNDER PERMISSION INPUT / PLANNING ONLY

The founder supplied the following sources for Golam planning and states permission to copy, use, modify, port and selectively reuse their source code as needed for the project:

```text
kunchenguid/firstmate
kunchenguid/no-mistakes
kgoedecke/doop
mrmps/classifier-dev / classifier.dev
caio0452/jev_search
unreallabsai/unreal-agent
```

The latter three sources (`classifier.dev`, `jev_search`, `unreal-agent`) were already permission-recorded in prior Golam planning. This supplement reaffirms that permission and records the newly supplied Firstmate / no-mistakes / Doop source set.

## Exact reviewed source states

```text
kunchenguid/firstmate
  23e5584714e6765cc223a1740d385d0e85f8ad8e
  public license observed: MIT

kunchenguid/no-mistakes
  a1c06cdaefcefa7cbc6902ac507f68ce9eed14ad
  public license observed: MIT

kgoedecke/doop
  77cb306aad47b9c901979c9d246105958458d7fb
  public license observed: AGPL-3.0
```

Detailed technical/source disposition is recorded in:

- `source-adoption-firstmate-no-mistakes-doop-2026-09-30.md`

## Permission boundaries

This attestation is an input to the existing Source Foundry. It does not itself:

- admit code into the privileged TCB;
- admit a dependency/runtime/service;
- clear third-party/transitive dependency obligations;
- admit model weights/datasets/provider accounts;
- grant trademark/brand rights;
- replace required NOTICE/source-offer obligations;
- authorize a hosted service or provider terms;
- make a donor's identity/task/effect/session state canonical Golam truth.

For Doop specifically, the public repository is AGPL-3.0. If Golam relies on the founder's separate permission to reuse a component under terms different from the public AGPL path, the exact permission evidence/scope must be attached to that exact Source Foundry component record before distribution/admission. If that evidence is not available, prefer behavior reimplementation or comply with the public license as applicable.

For `jev_search`, prior review observed no public license file at the reviewed pin. Founder permission remains recorded, but exact permission scope and third-party dependency obligations must be bound before copied code is distributed/admitted.

```text
FIRSTMATE_PERMISSION_ATTESTED_2026_09_30=YES
NO_MISTAKES_PERMISSION_ATTESTED_2026_09_30=YES
DOOP_PERMISSION_ATTESTED_2026_09_30=YES
CLASSIFIER_DEV_PERMISSION_REAFFIRMED_2026_09_30=YES
JEV_SEARCH_PERMISSION_REAFFIRMED_2026_09_30=YES
UNREAL_AGENT_PERMISSION_REAFFIRMED_2026_09_30=YES

PERMISSION_TO_COPY != TECHNICAL_ADMISSION
PERMISSION_TO_COPY != TRANSITIVE_RIGHTS_CLOSURE
PUBLIC_LICENSE != ONLY_POSSIBLE_PERMISSION_SOURCE
HOSTED_SERVICE_TERMS != REPOSITORY_SOURCE_PERMISSION
```
