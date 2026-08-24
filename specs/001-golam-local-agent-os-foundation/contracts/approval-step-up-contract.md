# Contract: Approval and Step-Up Authorization

Approvals narrow and temporarily activate existing policy authority; they do not create unconstrained authority.

## Approval classes

At minimum:

- `ONCE`: one exact effect intent.
- `SESSION_SCOPED`: bounded actions/resources for one session until expiry.
- `TIME_BOXED`: bounded actions/resources until a short expiry.
- `OPERATION_PATTERN`: a narrowly defined operation pattern with resource and quantitative limits.
- `RUN_PREAUTHORIZATION`: explicit bounded scope for unattended work.

Every approval records approver principal/device, exact scope, risk class, taint/provenance summary, issued/expiry time, usage limits, and parent authorization context.

## Rules

- Approval freshness is checked at effect execution, not only at proposal time.
- Expired/revoked approvals fail closed.
- Related low-risk effects may be batched only when the batch scope is shown and bounded.
- IRREVERSIBLE effects in unattended mode require an explicit per-run preauthorization with an upper bound; no generic "always allow irreversible" mode exists.
- An effect outside preauthorized scope waits for a fresh approval or is denied.
- Safety denial is monotonic and survives approval expiry/retry.
- Approval UI MUST surface material taint/provenance when a request is derived from untrusted content.

## Verification gate

Tests cover expiry, revocation, batch overreach, unattended irreversible actions, replay of used ONCE approvals, and tainted-request disclosure.