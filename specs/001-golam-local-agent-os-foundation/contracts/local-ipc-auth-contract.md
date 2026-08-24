# Contract: Local IPC and Client Authentication

## Purpose

`golamd` controls high-authority local capabilities. Localhost or same-user-machine location is not authentication.

## Required properties

- Every local client MUST authenticate before it can create/resume sessions or request protected actions.
- Windows local IPC SHOULD use named pipes restricted to the interactive user's SID and validate the connecting client identity.
- Unix local IPC SHOULD use Unix-domain sockets with owner-only permissions (`0600`) plus peer-credential validation where supported.
- IDE/ACP or other delegated clients MUST receive explicit per-client credentials/tokens through a user-approved enrollment flow; tokens are scoped, revocable, rotated, and never equal the user's full authority.
- Authentication failures and revoked-client attempts MUST be audited without leaking credentials.

## Network binding

- `golamd` MUST NOT bind a non-loopback control listener by default.
- `golamd` MUST NOT expose an unauthenticated HTTP/WebSocket control surface, including on loopback.
- Any loopback HTTP surface MUST authenticate, reject hostile `Origin`/`Host` patterns, protect against CSRF and DNS rebinding, and bind explicitly to loopback only.
- GolamConnect/channel traffic uses its own signed/authenticated remote path and MUST NOT tunnel into the local IPC trust plane as an already-authenticated client.

## Client attribution

Every accepted IPC connection maps to a concrete client principal/device/process identity and that identity is included in downstream authorization context and audit events.

## Verification gate

GolamBench MUST prove: an unauthenticated local process cannot issue commands; forged/revoked client credentials are rejected and audited; a listener scan shows no unexpected control listener.