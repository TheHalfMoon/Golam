# T003-056 — Secret Canary Leakage Qualification

## Result

`PASS`

Qualified exact implementation head:

`b1a47898515dc06237a0d71cea00e81b19cddd0a`

Official CI:

- workflow: `ci`
- run number: `#484`
- run id: `33169107060`
- Windows: `SUCCESS`
- macOS: `SUCCESS`
- Ubuntu: `SUCCESS`

## Qualified coverage

The qualification exercises both a deterministic recognized-format canary and a deliberately unknown-format deterministic canary through the same explicit user-designated secret-entry path. The explicit-entry guarantee does not consult the free-text detector.

For each canary the suite proves:

- raw submitted bytes enter the protected encrypted secret-create path;
- every file under the durable authority root is scanned after a full WAL checkpoint and contains no plaintext canary;
- every canonical `session_events.payload_bytes` value remains free of the canary, covering the current Spec 003 durable model-visible history boundary;
- ordinary rendered secret-entry error text does not contain submitted secret material;
- an environment-cleared unauthorized subprocess that can read only durable authority bytes cannot emit the plaintext canary on stdout or stderr;
- canonical integrity and `authority-security-v2` verification remain valid after the protected mutation.

The current Spec 003 product slice has no model runtime or prompt compiler. Later Spec 004 prompt/model integration must preserve this qualified boundary rather than weakening it.

## Detector defense in depth

A separate bounded free-text recognizer detects a small explicit set of common credential shapes and returns only a kind, never matched value material or offsets. It has a 64 KiB input ceiling. The deliberately unknown-format canary is intentionally not detected, demonstrating that automatic free-text recognition is defense in depth rather than the source of the explicit-entry guarantee.

## Gate

T003-056 is complete. Continue directly to T003-057 for crash/disk-full/rotation/revocation qualification and proof that no acknowledged partial transition leaves stale secret authority usable.
