# Contract: GolamConnect

## Purpose

GolamConnect lets authenticated users/devices reach a local Golam and, when policy permits, request autonomous actions or participate in a live remote-control session.

## Trust boundaries

1. Local clients use authenticated local IPC only.
2. Remote devices use signed/replay-protected GolamConnect envelopes and current device/lease state.
3. Relay infrastructure is an untrusted metadata observer; payload authorization and secrets never live there.
4. Third-party messaging providers are untrusted content transports even for a bound sender.
5. The model/worker may request Connect effects but cannot pair devices, mint grants, or widen leases.

## Separation of concerns

- `GolamConnect Core`: device identity, pairing, signed event envelopes, encryption, replay protection, routing.
- `Transport`: Iroh/QUIC candidate, direct P2P preferred, encrypted relay fallback.
- `Remote Control`: screen/media, input, clipboard, files, multi-monitor, reconnect and control arbitration.
- `Channel Bridges`: Telegram/WhatsApp/Slack/Discord/etc. command/notification adapters.

A third-party channel is not the native encrypted remote-control plane.

## Pairing and device trust

Pairing is a local user-authorized elevated effect and records device public key, owner principal, trust tier, device capabilities, creation state and revocation state. Device revocation is checked before every protected remote action and again on reconnect.

## Connect event envelope

A protected event includes event ID, sender principal/device, target, timestamp, nonce, payload hash, requested capability, current lease ID/generation, signature and encrypted payload as appropriate. Every protected message is checked host-side against the current pairing registry, revocation state and lease.

## Remote-control leases and arbitration

- control leases are short-lived, resource/action scoped and revocable;
- leases carry generation counters; a newer accepted generation invalidates prior input streams;
- transport success is never authorization;
- reconnect performs full device authentication plus lease/revocation/generation revalidation and is not a new auth path;
- input is rate/bounds checked at the host before OS injection;
- human takeover suspends conflicting agent/remote input at the lease layer;
- returning control to the agent requires explicit authorized re-grant, never automatic resumption after takeover.

## Remote-control safety

- visible non-stealth local indicator during interactive remote control;
- immediate local emergency stop that kills input/media and revokes/suspends the active control lease;
- clipboard read/write and file transfer are independently scoped capabilities;
- file transfers are path constrained and policy checked;
- sensitive application/resource blocking occurs before observation/streaming where feasible;
- secure desktop/UAC/TCC/OS permission boundaries are never bypassed;
- camera/microphone are deny-by-default and separate from screen/audio-output capabilities.

## Relay metadata

Iroh relay payloads are expected to remain encrypted end-to-end, but relays may observe endpoint identifiers, timing and network addresses. Golam MUST document this metadata exposure. Custom relay implementation is not a P0 requirement; configurable/self-hostable relay selection may be supported later.

## Messaging bridges

A channel identity maps to a Golam principal only through the separate `channel-binding-contract.md` using provider-stable identifiers, never display names/usernames. Group/unbound senders hold zero machine authority by default. Cross-channel equivalence is not inferred. All bridge content remains channel-untrusted for taint purposes unless separately verified.

## Verification

Tests cover replay, revoked devices, expiry, reconnect, generation arbitration, emergency stop, human takeover, file/clipboard scoping, channel impersonation, and relay-only/NAT-loss conditions.
