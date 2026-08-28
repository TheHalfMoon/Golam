# T003-053 Secret Broker Authorization Boundary

Status: `ACTIVE`

This note records the implementation boundary for T003-053 before exact-head qualification. It is not qualification evidence and does not claim PASS.

## Authorized scope

T003-053 implements authorization and protected evidence for `BrokerSecretUse` around an existing opaque secret handle. A successful broker authorization may return only bounded internal authority metadata: use ID, handle ID, secret ID, selected immutable version, lease ID/generation, authorization decision ID, and optional approval ID.

The broker must revalidate, inside one `BEGIN IMMEDIATE` protected transaction:

- canonical request fields and strict-local locality;
- the authenticated secret handle, exact purpose scope, and expiry;
- active/non-revoked secret state and the selected current immutable version;
- the exact durable `secret.use` authorization decision and its freshness;
- the decision's active policy bundle/hash binding;
- the complete bounded capability-lease parent chain, generation, status, revocation, freshness, action scope, and exact resource scope;
- approval binding, freshness, scope, risk class, taint digest, and usage limit when the decision carries an approval;
- canonical and authority-security integrity before and after the protected mutation.

A successful use appends only a metadata `secret_use_records` row with `mode = 'brokered'`, appends its authenticated `authority-security-v2` snapshot, and atomically consumes the bound approval when one is present.

## Explicit exclusions

T003-053 does not expose a generic plaintext read/decrypt API. It does not return ciphertext or plaintext to callers, inject secrets into argv/environment/stdin/files, grant network authority, create an egress permit, or implement the unbrokerable fallback. Those concerns remain owned by T003-054 and Phase G.

Under the current strict-local contract, broker destinations must be explicitly local `process:` or `service:` identities. External destinations fail closed before a use record is created.

## Qualification requirements

Exact-head CI must compile the production-linked broker on Windows, macOS, and Ubuntu and pass the repository's pinned format, clippy, workspace tests, property, fuzz, IPC, adversarial authority, and strict-local observation gates.

The T003-053 test surface must cover at least:

- successful metadata-only broker authorization;
- strict-local external destination denial before durable use evidence;
- stale handle version denial;
- revoked secret denial;
- resource binding sensitivity to purpose/destination;
- authenticated `SecretUseRecord` coverage.

No real secret values are permitted in qualification.