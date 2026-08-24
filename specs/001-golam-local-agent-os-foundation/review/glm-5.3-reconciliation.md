# GLM-5.3 Reconciliation Ledger

**Review recommendation**: APPROVE_WITH_MANDATORY_CHANGES  
**Founder waivers**: NONE  
**Policy**: all BLOCKER, MAJOR, and useful MINOR findings are accepted unless explicitly marked deferred below.

## Finding disposition

| Finding | Disposition | Owning correction |
|---|---|---|
| BLK-001 | RESOLVED_SPEC | `kernel-boundary-privilege-contract.md`; constitution/spec/plan reconciliation |
| BLK-002 | RESOLVED_SPEC | `local-ipc-auth-contract.md`; spec/plan requirements |
| MAJ-001 | RESOLVED_SPEC | `effect-handler-contract.md`; event/effect integration |
| MAJ-002 | RESOLVED_SPEC | kernel protected-resource list + elevated authority-mutation effects |
| MAJ-003 | RESOLVED_SPEC | `taint-information-flow-contract.md` |
| MAJ-004 | RESOLVED_SPEC | secret fallback/redaction requirements in reconciled spec/plan |
| MAJ-005 | RESOLVED_SPEC | `memory-governance-contract.md` |
| MAJ-006 | RESOLVED_SPEC | `ledger-replay-contract.md` |
| MAJ-007 | RESOLVED_SPEC | `channel-binding-contract.md` |
| MAJ-008 | RESOLVED_SPEC | `approval-step-up-contract.md` |
| MIN-001 | ACCEPTED | donor verification register/research correction |
| MIN-002 | ACCEPTED | `sandbox-profile-contract.md` |
| MIN-003 | ACCEPTED | ExecutionProfile contract amendment |
| MIN-004 | ACCEPTED | GolamConnect generation arbitration requirement |
| MIN-005 | ACCEPTED | incremental benchmark gates in Specs 002–005 |
| MIN-006 | ACCEPTED | relay metadata disclosure/self-host config requirement |
| MIN-007 | ACCEPTED | explicit Linux control capability tiers |
| MIN-008 | ACCEPTED | clipboard read separate; camera/mic deny-by-default |
| MIN-009 | ACCEPTED | parity ledger domains expanded |
| MIN-010 | ACCEPTED | llama.cpp preferred out-of-process sidecar |
| MIN-011 | ACCEPTED | backup/restore and disk-full fail-closed |
| MIN-012 | ACCEPTED | authorization interface required in Spec 002 |

## Simplifications accepted

- Start implementation with at most eight real Rust crates; the larger workspace diagram is a target decomposition, not an empty-crate mandate.
- Skills-as-instructions may precede executable skill scripts; executable untrusted extensions require the sandbox profile gate.
- L2 graph/deep-code intelligence is justification-gated and not mandatory for P0 Spec 005.
- Do not build a custom relay in P0; use configurable Iroh relay behavior.
- Groups/collaboration and teach-by-demonstration remain late 008/009 work.
- Voice, native mobile client, A2A federation, and media generation are deferred through Spec 010 unless a later reviewed spec pulls them in.
- GolamBench BS-1/BS-2/BS-10 begin early; Spec 010 remains full release qualification.

## Residual note

The founder-supplied GLM output is truncated only in the redundant tail of the final gate checklist. The recommendation and complete mandatory-change list are present. Reconciliation therefore uses the explicit finding set and mandatory list, while preserving the truncation fact in the review record.
