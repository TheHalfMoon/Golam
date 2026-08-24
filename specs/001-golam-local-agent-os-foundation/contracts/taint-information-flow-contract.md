# Contract: Taint Labels and Information Flow

## Purpose

Taint is provenance used by policy. It is not a model opinion and cannot be self-cleared by the model.

## Baseline labels

Initial labels include USER_TRUSTED, LOCAL_TRUSTED, LOCAL_UNVERIFIED, WEB_UNTRUSTED, CHANNEL_UNTRUSTED, MCP_UNTRUSTED, PLUGIN_UNVERIFIED, MODEL_GENERATED, and SECRET_DERIVED.

Derived text, structured data, memory candidates, scripts, files, code patches, screenshots/observations, and other artifacts inherit relevant source labels.

## Downgrade rules

A taint label MAY be downgraded only by:

1. explicit human approval identifying the item and resulting authority; or
2. deterministic verification against a pre-registered authoritative source/rule whose verifier is not controlled by the tainted input.

A model, worker, skill, MCP server, channel, or generated verifier statement MUST NOT downgrade its own or upstream taint.

Every downgrade records source labels, verifier/human principal, rule ID, evidence, timestamp, and resulting labels.

## Effect and memory rules

- Authorization context for an effect MUST include relevant taint/provenance of the facts/artifacts motivating the request.
- Executing a generated artifact is an effect carrying the artifact's labels.
- SECRET_DERIVED content MUST NOT enter canonical long-term memory.
- Tainted content may be stored as evidence only with provenance and scope; it must not silently become trusted instruction.

## Verification gate

Adversarial chains web -> summary -> memory -> later worker -> effect MUST preserve taint unless one of the explicit downgrade mechanisms is exercised and audited.