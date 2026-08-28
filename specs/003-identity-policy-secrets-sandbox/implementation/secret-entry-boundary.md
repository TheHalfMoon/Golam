# T003-055 Designated Secret Entry Boundary

Status: `ACTIVE`

This note records the implementation boundary for T003-055 before exact-head qualification. It is not qualification evidence and does not claim PASS.

## Authorized scope

T003-055 implements an explicit user-designated secret-entry path whose guarantee does not depend on syntax recognition or secret detectors.

The path is deliberately two-stage:

1. `prepare_designated_secret_entry` transfers the complete submitted byte value directly into the already-qualified protected `PreparedSecretCreate` path and exposes only the resulting non-plaintext authorization resource/digest for the existing protected mutation workflow.
2. `SecretEntryStore::commit` consumes the exact authorized secret-create decision/effect/ONCE approval, persists encrypted secret state, mints an opaque protected `SecretHandle`, and only then appends a model-visible canonical redacted projection.

The canonical projection contains only:

- a domain/version marker;
- the opaque handle ID;
- the literal tombstone marker `<redacted-secret>`;
- bounded non-secret classification/purpose metadata;
- immutable version metadata;
- optional handle expiry metadata.

It does not contain the submitted value, secret ID, ciphertext, associated-data hash, value commitment, keyed commitment, detector output or any plaintext-derived digest intended for model-visible history.

The new canonical `SecretEntryRedacted` event kind has a distinct stable event code and is security-critical. Raw input never becomes the `AppendEvent.payload` for this explicit entry path.

## Protected handle issuance

`SecretHandle` is protected authority state. T003-055 promotes its existing `authority-security-v2` snapshot writer to production use and mints the handle internally with fresh randomness only after the created secret is revalidated as active at the exact created version. There is still no public handle constructor.

Handle issuance binds:

- the protected secret ID internally;
- an exact immutable version constraint;
- bounded purpose scope;
- optional expiry.

Only the opaque handle ID is projected into model-visible history.

## Explicit exclusions

- Recognized-format/free-text detectors are not consulted by the explicit entry guarantee. Their defense-in-depth tests remain T003-056 work.
- T003-055 does not expose plaintext/ciphertext reads.
- T003-055 does not weaken the T003-052 decision/effect/ONCE-approval requirements.
- Crash/disk-full coupling and half-transition qualification remain T003-057 work.
- No real credential is required or used in qualification.

## Qualification requirements

Fresh exact-head Windows/macOS/Ubuntu CI must pass the repository's pinned gates with the production-linked entry path.

Focused deterministic evidence must prove at minimum:

- a deliberately unknown-format designated byte value is accepted without detector recognition;
- the protected secret version is encrypted and the plaintext value is absent from stored ciphertext;
- an authenticated opaque handle is minted and reloadable through `SecretCatalog`;
- the canonical `session_events.payload_bytes` projection contains the tombstone and handle but not the designated value;
- the projection uses the distinct `SecretEntryRedacted` event kind;
- canonical and `authority-security-v2` integrity remain valid after the complete entry.