# Contract: Channel Binding and Impersonation Resistance

Third-party messaging content is untrusted even when the sender is known.

## Identity binding

- A channel account binds to a Golam principal only through an explicit local user-authorized action.
- Binding keys MUST use provider-stable identifiers (for example Telegram numeric user ID, WhatsApp Business Platform identifiers, Slack/Discord account IDs), never display names, handles, usernames, or avatar/name similarity.
- Bind/unbind/revoke actions are elevated effects and are audited.
- Revocation takes effect immediately for new requests and is checked again before consequential effects.

## Groups and unbound senders

- Group-chat participants have zero machine authority by default, including participants whose display name resembles a bound owner.
- Unbound senders may at most create low-trust notification/input events subject to policy; they cannot inherit a bound principal.
- Cross-channel identity equivalence is never inferred automatically.

## Replay and provenance

Normalized channel events include provider, stable sender ID, chat/conversation ID, message ID, timestamp, binding record/version, and raw-provider provenance. Cross-channel replay does not preserve authority.

## Verification gate

Tests cover released/reused usernames, spoofed display names, group injection, cross-channel replay, stale binding versions, and revoked accounts.