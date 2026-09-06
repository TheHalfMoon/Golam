# Spec 005 Process v2 Requalification Evidence

Status: `PENDING_EXACT_HEAD_CI`

This document is an evidence manifest for T005-080. It is not an admission, success claim, waiver, or substitute for GitHub Actions evidence.

## Exact qualification surface

The exact process-tool head must run the repository CI matrix on Windows, macOS and Ubuntu. Linux x86_64 additionally runs the governed process v2 end-to-end qualification through the production `golamd` process boundary.

The dedicated Linux qualification must prove all of the following on the exact head:

- a real immutable `PreparedToolRequest` is bound to a live sealed capability lease;
- capability lease issuance traverses current authorization, an exact ONCE approval, an authorized capability-lease mutation Effect, and the ledger's sealed lease commit path;
- executable staging uses the governed staging Effect and preserves exact file identity/content evidence;
- execution creates a distinct `process.execute` Tool Effect and revalidates authorization, lease evidence, staged executable identity, helper identity, cwd and filesystem roots immediately before spawn;
- the qualification payload is static ELF with no interpreter and launches only through the trusted sibling helper;
- ambient environment is empty inside the payload and the parent secret canary is never injected into argv or environment;
- strict-local networking is denied from inside the admitted process boundary;
- descendant process creation is denied and terminal reconciliation observes zero descendants;
- normal exit reaches terminal `SUCCEEDED` only after exact terminal process-tree reconciliation;
- wall-time exhaustion terminates and reconciles as `TIMED_OUT`;
- combined stdout/stderr exhaustion terminates and reconciles as `OUTPUT_LIMIT_EXCEEDED` without accepting bytes beyond the bound;
- cancellation terminates and reconciles as `CANCELLED`;
- a restart after a prepared/executing process Effect first becomes `UNKNOWN_OUTCOME`, then an unreconcilable result enters manual review rather than success.

## Canonical implementation references

- `crates/golamd/src/process_dispatch_v2.rs`
- `crates/golamd/src/process_execution_v2.rs`
- `crates/golamd/src/native_containment_v2.rs`
- `crates/golamd/src/native_process_supervisor_v2.rs`
- `crates/golamd/src/process_secret_evidence.rs`
- `crates/golamd/src/bin/golam-native-exec-helper-v2.rs`
- `crates/golamd/tests/process_v2_qualification.rs`
- `scripts/qualification/process-v2-payload.rs`
- `.github/workflows/ci.yml`

## Evidence acceptance rule

`T005_080=PASS` may be recorded only after the exact-head CI run containing the dedicated governed process v2 step succeeds, together with the normal repository CI gates. Any branch mutation invalidates that evidence and requires requalification.

`WAIVER_TAKEN=NO`
