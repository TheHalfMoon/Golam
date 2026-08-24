# Contract: Authenticated Local IPC

## Goal

Locality is transport location, not authority. Every privileged client must authenticate.

## Transport

- Windows: named pipe scoped to current interactive user SID; capture peer PID/token metadata when supported.
- macOS/Linux: Unix-domain socket in user-private runtime dir; parent 0700, socket 0600; read peer UID/PID where supported.
- no HTTP/TCP control listener in Spec 002.

## Protocol lifecycle

```text
Client -> HELLO { protocol_version, client_id, client_nonce }
Server -> CHALLENGE { protocol_version, server_epoch, server_nonce, limits }
Client -> AUTHENTICATE { key_id, signature(transcript), client_nonce }
Server -> READY { connection_id, server_epoch, limits }
...
REQUEST | CANCEL | EVENT | REPLY
...
SHUTDOWN { reason }
```

Transcript signature binds protocol version, client ID, client nonce, server nonce and server epoch.

## Required frame classes

- lifecycle: hello/challenge/authenticate/ready/shutdown;
- request: request_id + method + typed payload;
- cancel: request_id;
- reply: request_id + typed success/failure;
- event: typed family + payload.

## Rejection rules

Connection closes on:
- malformed frame;
- unsupported protocol version;
- repeated/out-of-order lifecycle frame;
- invalid/revoked/unknown client key;
- invalid signature/nonce;
- oversized frame;
- request before ready;
- impossible server/client-direction frame;
- request concurrency/resource limit breach beyond documented error policy.

## Limits

Implementation must define and test:
- max frame bytes;
- max decoded payload bytes;
- max pending requests per connection;
- max connections;
- handshake deadline;
- request cancellation semantics;
- bounded audit detail so malicious frames cannot fill disk.

## Enrollment

Enrollment is an explicit local action. Credential storage uses an OS-protected facility where qualified; any filesystem fallback is user-private and documented as lower assurance.

Peer OS identity + client cryptographic credential are both inputs. Neither alone is treated as complete authority.

## Threat claim

Spec 002 protects against browser/localhost attacks, stray scripts, unenrolled processes, replay and accidental exposure. It does not claim to defeat an attacker that already has arbitrary code execution with full access to the user's OS account and credential store.
