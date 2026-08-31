# T003-062 Effective Destination Reauthorization Qualification

Status: PASS

## Exact qualified implementation head

- Commit: `4d730e894ebde948185597f5fe4296a142fd9ac6`
- Tree: `0aad61fc8bf1782557627d9c998c14aab0a2cc61`
- Official CI: #525
- Workflow run: `33200261387`
- Windows: SUCCESS
- macOS: SUCCESS
- Ubuntu: SUCCESS

The qualified head contains no temporary T003-062 helper workflow.

## Qualified behavior

The implementation adds a protected effective-destination boundary without performing DNS resolution or socket operations inside the authority store.

`EffectiveDestination` deterministically binds:

- normalized authority/hostname;
- effective IP address;
- protocol;
- port;
- explicit external/private/link-local/loopback classification;
- exact effective resource identity.

Every connect/follow authorization uses a context hash bound to the permit identifier, the original authorized destination-scope digest, the effective authority, exact effective resource and address class. Reusing a prior decision after a redirect, DNS rebinding, changed resolved endpoint or private-target transition therefore fails closed unless a fresh exact authorization decision is present.

Protocol/port changes outside the permit's explicit protocol/port scope deny rather than widening the permit.

The effective-use boundary revalidates the current authorization decision, active policy, parent capability-lease chain, permit state/lifetime/use limit and authenticated authority state before atomically consuming one use. A failed changed-endpoint attempt does not consume permit usage.

## Focused qualification

Before exact-head CI, the clean implementation passed focused Rust 1.98.0 qualification covering:

- canonical endpoint normalization and decision-context determinism;
- explicit private/link-local/loopback classification and unsupported-address denial;
- hostname-to-IP first-use authorization;
- redirect decision non-transferability;
- DNS rebinding decision non-transferability;
- private-target transition reauthorization;
- protocol/port scope change denial;
- unchanged existing EgressPermit lifecycle regressions;
- `cargo +1.98.0 clippy -p golam-ledger --all-targets -- -D warnings`.

## Boundary statement

This task does not add networking, DNS resolution, redirect following, socket creation, a new dependency, donor code, real secrets or a strict-local override. The T003-060 kernel hard guard remains independent and dominant.
