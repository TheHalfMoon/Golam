# T003-054 Secret Fallback Qualification

Status: `PASS`

## Exact qualification evidence

- Qualified implementation head: `f2cef061f4a6847a2eedf7b867b8b94e4ccadd8f`
- Official workflow: `ci`
- Run number: `464`
- Run ID: `33166659638`
- Result: `SUCCESS`
- Platforms: Windows, macOS, Ubuntu

The qualified exact head passed the repository's pinned format, Clippy, workspace-test, property, bounded-fuzz, IPC, authenticated-daemon IPC, adversarial-authority, daemon-build and strict-local external-observation gates as applicable on each platform.

## Qualified boundary

T003-054 provides a bounded trusted fallback for a secret use that cannot be brokered directly. It consumes pre-existing protected sandbox/process admission and authority state; it does not create or widen sandbox, lease, policy, approval or egress authority.

The qualified fallback requires:

- an authenticated opaque secret handle and active current immutable secret version;
- an authenticated pre-existing admission bound to the exact resolved launch-plan hash and exact platform executor;
- no egress permit under the current strict-local contract;
- the exact active policy and capability-lease chain represented by the admission decision;
- an exact authorized at-most-once `secret.fallback.use` effect;
- an exact fresh ONCE approval bound to action/resource/effect/risk/taint;
- injector capability evidence for a cleared environment, stdin-only secret delivery, stdin closure, no secret argv/environment, no ambient descendant inheritance, and captured stdout/stderr.

The only admitted secret injection channel in this task is stdin. The vault exposes decrypted bytes only through a crate-internal callback boundary; no generic plaintext-return API was introduced. The value is checked against executable, argv and explicit environment fields before durable `SecretUseRecord` evidence or approval consumption. Captured stdout, stderr and injector errors are exact-value redacted before leaving the trusted boundary.

## Focused evidence

Deterministic tests prove:

- successful stdin-only fallback records metadata-only use evidence and consumes the exact ONCE approval atomically;
- replay fails after approval consumption;
- a deterministic canary embedded in argv is rejected specifically as `SecretPresentInLaunchField` even when admission/effect/lease/approval authority is bound to that exact launch plan, before use evidence or approval consumption;
- weak injector containment capability declarations fail closed;
- admission launch-plan mismatch fails closed;
- launch-plan hashes are sensitive to launch fields while environment clearing, no-secret-argv, no-secret-environment and no-descendant-inheritance invariants remain explicit.

The qualification fixtures use deterministic canary material only. No real credential or user secret is required or used.

## Repair history

Three bounded repairs were required before qualification:

1. the admission verifier's argument list was replaced by a bounded `AdmissionRequest` structure to satisfy the pinned Clippy gate without suppressing the lint;
2. test-only sandbox profile/admission snapshot writers were added to match the already-mandatory `authority-security-v2` coverage rather than weakening integrity verification or implementing the later sandbox lifecycle early;
3. the argv-canary test fixture was bound to the exact secret-bearing launch plan so the test exercises the no-argv invariant directly instead of being denied earlier by plan-hash mismatch.

Temporary implementation helper workflows were removed before the qualified exact head and are not qualification evidence.

## Explicit non-claims

T003-054 does not claim a universal native sandbox executor, does not launch a network-capable managed child, does not grant egress authority, and does not implement the T003-055 explicit user-designated secret-entry path. Those remain governed by their later tasks.