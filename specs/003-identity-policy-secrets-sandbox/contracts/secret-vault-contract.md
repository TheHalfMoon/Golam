# Contract — Secret Vault, Handles & Broker

## Principle

Real secrets should remain outside model context and untrusted execution whenever brokered use is possible. The default caller interface is an opaque `SecretHandle`, not plaintext.

## Durable vault

- secret metadata and values are protected authority state;
- durable value bytes are encrypted at rest;
- ciphertext authentication/associated data binds secret identity, version and security metadata;
- key protection/backing store is implementation-qualified per platform;
- unavailable/corrupt key material fails closed;
- rotation creates a new immutable version and revocation blocks future use;
- audit/security records contain handle/version/use metadata, never plaintext.

## Brokered use

`BrokerSecretUse` validates principal, lease, policy, approval where required, purpose, destination/process and locality/egress state. When possible, the trusted boundary performs credential application without returning raw bytes to model/untrusted code.

## Unbrokerable fallback

Requires:
- explicit bounded approval;
- a sandbox/process admission authorizing the exact injection channel;
- no command-line argument injection;
- ambient environment cleared before injection;
- no implicit child/grandchild inheritance;
- minimum plaintext lifetime;
- value-aware redaction of stdout/stderr/log/error paths;
- deterministic canary verification.

## User-pasted secrets

The explicit user-designated secret-entry boundary treats the entire submitted value as secret regardless of whether any detector recognizes its format. Before any durable model-visible canonical append, that path may persist only an opaque handle, tombstone/redaction marker, and non-secret metadata. Raw submitted value bytes may enter only the qualified protected vault/broker mutation path and must never be committed as canonical plaintext.

Recognized-format and deterministic-canary detection on ordinary free-form text is defense in depth, not the source of this guarantee. Detection of arbitrary unknown secret formats is necessarily bounded, so Golam does not claim perfect automatic discovery in unrestricted free text. When the user identifies input as a credential/secret, the explicit secret-entry path is mandatory and its whole-value treatment does not depend on format recognition.

## Testing

Use deterministic canary values only. Tests include recognized and deliberately unknown-format values submitted through the explicit secret-entry path and prove those values are absent from durable event/audit payloads, ordinary errors, prompts/model-visible history, unauthorized subprocess output and raw durable vault bytes. Free-form detector tests are additional defense-in-depth evidence and do not substitute for the explicit-entry guarantee.
