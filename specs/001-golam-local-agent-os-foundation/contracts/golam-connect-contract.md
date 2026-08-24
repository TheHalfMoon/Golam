# Contract: GolamConnect

## Purpose

GolamConnect lets authenticated users/devices reach a local Golam and, when policy permits, request autonomous actions or take part in a live remote-control session.

## Separation of concerns

- `GolamConnect Core`: identity, pairing, signed event envelopes, encryption, replay protection, routing.
- `Transport`: Iroh/QUIC candidate, direct P2P preferred, encrypted relay fallback.
- `Remote Control`: screen/media, input, clipboard, files, multi-monitor, reconnect.
- `Channel Bridges`: Telegram/WhatsApp/Slack/Discord/etc. command and notification adapters.

A third-party channel is not the native encrypted remote-control plane.

## Pairing

Pairing must establish device identity without trusting display names/usernames. Pairing records:
- device public key;
- owner principal;
- trust tier;
- allowed device-level capabilities;
- creation/revocation state.

## Connect event envelope

A Connect event includes event ID, sender principal/device, target, timestamp, nonce, payload hash, requested capability, signature and encrypted payload as appropriate.

## Remote-control safety

- host remains final authorization authority;
- control lease is short-lived and revocable;
- per-message capability check;
- visible local indicator during interactive remote control;
- immediate local emergency stop;
- human takeover suspends conflicting agent input;
- reconnect revalidates the existing authorization/grant and is not a new auth path;
- clipboard/file transfer can be independently disabled;
- sensitive application/resource blocklist supported;
- secure desktop/UAC/OS permission boundaries are not bypassed.

## Messaging bridges

Bridge messages are normalized into a `GolamRequest`/ConnectEvent with sender/channel provenance. A Telegram/WhatsApp identity maps to a Golam principal only through explicit binding. Group participants receive no machine authority by default.
