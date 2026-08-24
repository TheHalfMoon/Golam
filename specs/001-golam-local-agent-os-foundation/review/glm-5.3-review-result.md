# GLM-5.3 Independent Architecture Review — Received Result Record

**Received**: 2026-08-24  
**Reviewed head reported by GLM**: `adfcb73`  
**Recommendation**: `APPROVE_WITH_MANDATORY_CHANGES`

## Source integrity note

The founder supplied the GLM-5.3 review output as a pasted-text artifact. The supplied artifact contains the complete recommendation, scorecard, 2 BLOCKER findings, 8 MAJOR findings, 12 MINOR findings, KEEP decisions, missing requirements/contracts, corrected TCB, GolamConnect/computer/memory/ExecutionProfile reviews, donor matrix, parity review, roadmap review, GolamBench scenarios, production failure modes, simplification opportunities, and the 11-item Final Mandatory Changes list. The supplied artifact ends during section 21 `Final Gate Checklist` immediately after `CLEAN_ROOM_BOUNDARY`; the missing tail is not invented here.

This file is a normalized finding ledger, not a claim of verbatim transcription. The raw founder-supplied artifact remains the source of record for review wording.

## BLOCKER findings

- `BLK-001` — privileged kernel had no runtime enforcement mechanism; trusted Rust path and authority-bearing kernel were conflated.
- `BLK-002` — local client-to-`golamd` IPC authentication and daemon network-binding rules were unspecified.

## MAJOR findings

- `MAJ-001` — effect FSM lacked a normative handler/executor/reconciler interface.
- `MAJ-002` — policy/principal/lease/lock state was not protected from generic filesystem writes/self-modification.
- `MAJ-003` — taint downgrade semantics were undefined; artifacts also need taint.
- `MAJ-004` — unbrokerable-secret and accidental secret-ingestion paths were underspecified.
- `MAJ-005` — memory ADD/UPDATE/SUPERSEDE/CONTRADICT/MERGE/EXPIRE/FORGET/REDACT and concurrency semantics were missing.
- `MAJ-006` — ledger forking, cross-session causality, mandatory integrity chaining, artifact retention/GC were missing.
- `MAJ-007` — channel binding key had to be provider-stable ID, never username/display name.
- `MAJ-008` — approval classes/freshness and unattended irreversible preauthorization were undefined.

## MINOR findings accepted

`MIN-001` donor verification-status accuracy; `MIN-002` MCP/skill script isolation; `MIN-003` ExecutionProfile fields; `MIN-004` remote-control generation arbitration; `MIN-005` incremental benchmark gates; `MIN-006` relay metadata disclosure; `MIN-007` Linux X11/Wayland matrix; `MIN-008` clipboard/camera/mic capability separation; `MIN-009` additional parity domains; `MIN-010` prefer llama.cpp sidecar over in-process FFI; `MIN-011` backup/disk-exhaustion behavior; `MIN-012` Spec 002 authorization interface.

## Final Mandatory Changes from GLM

1. Add Kernel Boundary & Privilege Separation contract and distinguish trusted path from privileged kernel.
2. Add local IPC authentication/network-binding requirements.
3. Add Effect Handler contract, durable intent-before-execution, no blind retry, and UNKNOWN_OUTCOME blocking.
4. Protect kernel-owned resources and make authority/policy/lock changes elevated effects.
5. Specify taint downgrade rules, artifact taint, and secret-derived-never-in-memory.
6. Specify unbrokerable-secret fallback and accidental-secret ingestion handling.
7. Enumerate memory governance/conflict/promotion/FORGET semantics and single-writer/external-edit reconciliation.
8. Add ledger forking, cross-session ordering, mandatory integrity chaining, artifact retention/GC.
9. Require provider-stable identifiers for channel bindings.
10. Define approval classes/freshness/unattended irreversible preauthorization.
11. Require a mechanized strict-local egress choke point.

## KEEP decisions

Local-first/no hidden fallback; Rust trusted path with untrusted TS renderer; effect UNKNOWN_OUTCOME model; Goal Ledger outside compaction; core invariants; deny-by-default/monotonic denial/capability narrowing; Markdown+SQLite split; ExecutionProfile; semantic-first computer control; Cedar/Wasmtime roles; mistral.rs + llama.cpp behind adapters; native GolamConnect separate from channel bridges; RASystem+Iroh direction; Source Foundry/clean-room separation; bounded Specs 002–010; evidence-based Grok parity.

## Roadmap conclusion

GLM recommended `KEEP CURRENT ORDER` for Specs 002–010 with three conditions: Spec 002 defines the authorization interface with deny-by-default bootstrap; Specs 002–005 have incremental GolamBench exit gates; Specs 007 and 008 may swap if substrate progress warrants.
