---
task_ids:
  - T005-073
  - T005-074
  - T005-075
  - T005-076
  - T005-077
outcome: IMPLEMENTED_PENDING_FINAL_EXACT_HEAD_CI_AND_INDEPENDENT_REVIEW
recorded_on: 2026-09-05
candidate_profile: platform:linux-x86_64-landlock-v4-seccomp-v2
production_profile_admitted: false
waiver_taken: false
---

# Phase G qualification convergence and live closeout policy

## Purpose

This record converges the mutable Phase G implementation before the T005-077 exact-head gate. It deliberately does not mirror a future CI/review result into branch content after qualification. Once the branch reaches its final Phase G candidate head, live GitHub CI, review, PR comments and lifecycle metadata are authoritative for T005-077 PASS/FAIL so that recording PASS does not itself invalidate the exact-head evidence.

A branch mutation after qualification invalidates CI/review evidence bound to the prior head. If a material finding requires code or governance repair, the head must advance forward-only and both exact-head CI and independent review must be repeated.

## T005-073 — secret-safe process evidence boundary

`crates/golamd/src/process_secret_evidence.rs` provides the pre-launch evidence boundary required before T005-078 can create a governed process executor:

- brokered secret use is represented by opaque handle/use/lease/decision/approval references and never by plaintext argv/environment values;
- an unbrokerable fallback requires an exact admitted-profile reference, stdin-only injection attestation, at-most-once Effect semantics and exact authority evidence;
- explicit environment is rejected if a known secret value is present;
- argv is rejected if a known secret value is present;
- stdout/stderr evidence is value-aware redacted before it leaves the trusted boundary;
- stored evidence contains redaction counts/flags and bounded redacted bytes, not the secret value;
- the boundary validates evidence only and does not mint launch authority.

The repository-wide `cargo test --workspace --all-targets` gate is the authoritative exact-head test carrier for these unit tests and the existing Spec 003 broker/fallback/canary regression suite.

## T005-074 — external descendant-aware strict-local observation

The applicable v2 qualification is `scripts/qualification/native-containment-hostile.sh` executed against `golam-native-containment-hostile-probe-v2` in Linux x86_64 CI.

The harness externally:

- observes the live owned process tree with `ps` and requires exactly the root process because v2 denies spawn;
- observes Internet sockets for every managed PID with `lsof` and requires zero sockets;
- requires the contained child to prove socket creation/connect denial and local `socketpair` denial;
- requires exact v2 profile markers and rejects missing evidence.

The later daemon-level strict-local observation remains regression evidence only and is not substituted for this v2-specific process-tree observation.

## T005-075 — exact platform claim boundary

`phase-g-platform-boundary.md` freezes the only eligible candidate as:

```text
platform:linux-x86_64-landlock-v4-seccomp-v2
```

No generic Linux, macOS, Windows, namespace-equivalence or other-architecture containment claim is inferred. Unsupported platforms remain explicit denial states. v1 remains historical evidence and is not admission-eligible.

## T005-076 — hostile corpus

The v2 hostile probe/harness covers the currently frozen first-profile claims:

- non-empty ambient environment denial;
- inherited non-stdio descriptor denial;
- empty Linux inherited/permitted/effective/ambient capability sets;
- exact identity-bound regular-file data roots and rejection of directory/device/special-file donation;
- strict-local network denial and local socket IPC creation denial;
- process creation denial;
- forbidden filesystem write and device access denial;
- bounded wall time;
- one combined stdout/stderr budget;
- cancellation as a non-terminal request followed by exact terminal observation;
- zero descendant persistence for the spawn-denied profile;
- fail-closed parent supervisor binding to the hardened containment receipt.

No production process launcher is admitted by these qualification binaries.

## T005-077 live evidence policy

The final Phase G admission decision requires all of the following on one unchanged exact candidate head:

1. `cargo fmt --check` success;
2. Clippy with warnings denied;
3. full workspace/all-targets tests;
4. existing property/fuzz/IPC/authentication/adversarial qualification;
5. Linux x86_64 v2 hostile containment execution with all hardened evidence markers;
6. external process-tree/no-socket observation for the exact v2 probe;
7. Windows/macOS/Ubuntu repository CI success for the exact head;
8. a fresh substantive independent repository-integrated semantic/security review on that same exact head;
9. reconciliation of every material review finding.

A summary-only, status-only, billing-blocked, unavailable, stale-head, or self-authored review does not satisfy T005-077.

If all gates are clean, record T005-077 PASS and the exact admitted profile in GitHub PR metadata/comments without mutating the branch merely to mirror PASS. T005-078 may then consume that live admission evidence and add the governed process-execution implementation; that later implementation is a new head and must be independently requalified under T005-080.

Until the live gate closes:

```text
T005_073=IMPLEMENTED_PENDING_FINAL_EXACT_HEAD_CI
T005_074=IMPLEMENTED_PENDING_FINAL_EXACT_HEAD_CI
T005_075=IMPLEMENTED_PENDING_FINAL_EXACT_HEAD_CI
T005_076=IMPLEMENTED_PENDING_FINAL_EXACT_HEAD_CI
T005_077=PENDING_FINAL_EXACT_HEAD_CI_AND_INDEPENDENT_REVIEW
PROFILE=platform:linux-x86_64-landlock-v4-seccomp-v2
PRODUCTION_PROFILE_ADMITTED=NO
PROCESS_LAUNCH_ENABLED=NO
SHELL_ENABLED=NO
WAIVER_TAKEN=NO
```
