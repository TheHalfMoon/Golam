# Contract — Taint & Information Flow

Taint is durable provenance consumed by policy. It is not model confidence.

## Baseline labels

```text
USER_TRUSTED
LOCAL_TRUSTED
LOCAL_UNVERIFIED
WEB_UNTRUSTED
CHANNEL_UNTRUSTED
MCP_UNTRUSTED
PLUGIN_UNVERIFIED
MODEL_GENERATED
SECRET_DERIVED
```

## Derivation

By default:

```text
result_labels = union(all source labels) ∪ labels introduced by the transform
```

Summarization, model generation, formatting, code generation, file conversion and memory-candidate generation do not remove labels.

## Downgrade

A downgrade creates a new attestation/derived artifact; it never rewrites source provenance.

Allowed mechanisms:
1. explicit human approval for an identified normal provenance downgrade where policy permits;
2. deterministic verification against a pre-registered authoritative source/rule independent of the tainted input;
3. for `SECRET_DERIVED`, only a registered deterministic secret-elimination sanitizer that produces a separately evidenced non-secret representation.

A model, worker, skill, MCP server, channel or generated verifier statement cannot clear its own/upstream taint.

## Sinks

- effect authorization receives relevant taint in context;
- sandbox/egress/secret policy receives relevant taint;
- canonical long-term memory rejects `SECRET_DERIVED`;
- untrusted content may be stored as evidence with provenance but cannot silently become instruction authority.

## Verification

Property tests cover multi-hop union and adversarial web/MCP/channel -> summary -> artifact/memory candidate -> later effect chains. Self-clear and unregistered-verifier attempts deny and are audited.
