# Cross-Artifact Analysis — Spec 003

**Date**: 2026-08-27  
**Result**: QODO_REPAIR_RECONCILED_PENDING_REQUALIFICATION

## Constitution ↔ spec

PASS_AFTER_REPAIR. The spec directly implements constitutional principles II-V, IX-XI while preserving local ownership, the smaller privileged kernel, durable effects, explicit secrets/taint and verification discipline. Qodo's three material security findings were accepted and the planning artifacts were strengthened rather than waived.

## Spec 001 ↔ Spec 003 scope

PASS_AFTER_REPAIR. Frozen program decomposition assigns Cedar/capabilities, protected resources, approvals, taint algebra, secret fallback/redaction, egress policy and sandbox profiles to Spec 003. No Spec 004+ product surface is pulled forward.

## Spec 002 ↔ Spec 003 seam

PASS_AFTER_REPAIR. Spec 002 intentionally froze `Authorize(principal, action, resource, context)` and a bootstrap deny-by-default evaluator for replacement in Spec 003. The plan preserves KernelApi, authenticated IPC, protected state, effect FSM/reconciliation, security integrity and strict-local hard denial.

## Policy consistency

PASS_AFTER_REPAIR. Cedar remains an evaluator candidate only. Golam owns hard guards, schemas, capabilities, protected resources, approvals and final semantics. Policy errors deny; candidate policy cannot authorize its own activation.

## Lease/approval consistency

PASS_AFTER_REPAIR. Child lease authority narrows only. Expiry/revocation/freshness are use-time gates. Approval activates/narrows existing authority rather than creating a bypass. ONCE consumption has explicit concurrency/crash semantics.

## Taint/secret consistency

PASS_AFTER_REPAIR. Taint is monotonic provenance. Self-clear is impossible. `SECRET_DERIVED` cannot enter canonical long-term memory; a deterministic secret-elimination sanitizer may create a separately evidenced non-secret representation. Secret handles/brokered use and bounded unbrokerable fallback match constitution/Spec 001 requirements.

Qodo repair: the explicit user-designated secret-entry path now treats the complete submitted value as secret regardless of format recognition and persists only handle/tombstone/redaction marker plus non-secret metadata before model-visible durable append. Recognized-format detection in unrestricted free text is explicitly defense in depth and no longer defines the guarantee.

## Egress/sandbox consistency

PASS_AFTER_REPAIR. Strict-local denial remains above all permits and applies to all Golam-managed processes. A hostname permit does not authorize arbitrary resolved/private targets. Every changed effective destination caused by DNS resolution, redirect, rebinding, protocol/port change, or private/link-local/loopback transition requires mandatory reauthorization before connect/follow, otherwise deny.

Qodo repair: the current daemon-PID-only external observer is explicitly insufficient once managed children exist. Upgrading observation to the complete managed process tree or an equivalent descendant-capturing sinkholed/network boundary is now a predecessor to launching any network-capable managed child and remains an exact-head closeout gate.

## Donor/source consistency

PASS_AFTER_REPAIR. Exact `Golam-Research` snapshot is recorded and classified `REFERENCE_ONLY`; no code/dependency is admitted. Cedar/Wasmtime/secret backends are implementation-time qualified.

## Data model ↔ tasks

PASS_AFTER_REPAIR. Every new protected entity has implementation, integrity, adversarial/fault and closeout tasks. `authority-security` coverage is mandatory for new authority source rows. Security verification tasks now explicitly cover unknown-format explicit secret entry, changed-destination reauthorization, and descendant egress observation.

## Qodo repair reconciliation

1. `Destination changes bypass reauthorization` — ACCEPTED. Contract/spec/plan/tasks now require mandatory effective-destination reauthorization; prior hostname authority cannot transfer implicitly.
2. `Child egress escapes qualification` — ACCEPTED. Planning now requires full managed-process-tree or equivalent descendant-capturing external observation before network-capable managed child execution; the existing single-daemon-PID observer is not treated as sufficient future evidence.
3. `Unknown secrets persist plaintext` — ACCEPTED. Explicit user-designated secret entry now guarantees whole-value secret handling independent of detector recognition; detectors remain defense in depth for unrestricted text.

No waiver was taken. Because these repairs mutate the planning branch, CI #253 and the prior Qodo review are historical evidence only and do not qualify the repaired head.

## Risks carried into implementation

1. exact Cedar version/resource behavior must be qualified;
2. cryptographic vault/key-protection backend differs by platform and must not create a lowest-common-denominator security claim;
3. native sandbox containment differs substantially across Windows/macOS/Linux;
4. automatic secret discovery in unrestricted free text remains bounded, while the explicit secret-entry guarantee is independent of detection;
5. DNS/redirect/rebinding enforcement must remain coupled to actual network-capability creation with mandatory changed-target reauthorization;
6. the process-tree/descendant external egress observer must be upgraded before network-capable managed child execution;
7. authority schema migration must preserve existing Spec 002 canonical history/integrity.

Each risk has an explicit implementation task/fail-closed gate.

## Planning gate

```text
UNRESOLVED_BLOCKERS=REQUALIFICATION_REQUIRED
UNRESOLVED_MAJOR_INCONSISTENCIES=0_AFTER_REPAIR
SPEC_002_CLOSED_CANONICAL=YES
QODO_REPAIR_FINDINGS=3_ACCEPTED_AND_RECONCILED
FINAL_EXACT_HEAD_CI=PENDING_AFTER_REPAIR
FINAL_POST_CI_QODO=PENDING_AFTER_REPAIR
DONOR_CODE_ADMITTED=NO
PRODUCT_IMPLEMENTATION_IN_PLANNING_PR=NO
TASKS_GENERATED=YES
NEXT_GATE=REPAIRED_EXACT_HEAD_CI_THEN_FRESH_AUTHORIZED_QODO
```
