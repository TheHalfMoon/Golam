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

A designated ingestion boundary redacts/tombstones recognized secret material before durable model-visible canonical text is committed. Audit may retain non-secret metadata that redaction occurred. Golam does not claim perfect detection of arbitrary unknown secret formats.

## Testing

Use deterministic canary values only. Tests prove canaries are absent from durable event/audit payloads, ordinary errors, prompts/model-visible history, unauthorized subprocess output and raw durable vault bytes.
