from pathlib import Path

QUALIFIED_HEAD = "cd721231b498450e984810b4f06c4e14bdc311e1"
QUALIFIED_TREE = "4c5ddb92f8764f0107510ba72e6f697a3225bd56"
CI_RUN = "33252170158"
FOCUSED_RUN = "33252055912"


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing recorder anchor in {path}: {old[:100]!r}")
    p.write_text(text.replace(old, new, 1))


tasks = "specs/003-identity-policy-secrets-sandbox/tasks.md"
replace_once(
    tasks,
    "- [ ] **T003-080** Replace bootstrap policy evaluation in normal authority path while preserving the stable `Authorize` call contract and narrow recovery bootstrap path.",
    "- [x] **T003-080** Replace bootstrap policy evaluation in normal authority path while preserving the stable `Authorize` call contract and narrow recovery bootstrap path. Exact-head qualification: CI #616 (`33252170158`) SUCCESS at `cd721231b498450e984810b4f06c4e14bdc311e1`, tree `4c5ddb92f8764f0107510ba72e6f697a3225bd56`, on Windows/macOS/Ubuntu; focused runtime-policy run `33252055912` SUCCESS. Evidence: `implementation/t003-080-runtime-policy-qualification.md`.",
)
replace_once(
    tasks,
    "T003_076_FOCUSED_RUN=33250097386\nNEXT_TASK=T003-080",
    "T003_076_FOCUSED_RUN=33250097386\nT003_080=PASS\nT003_080_QUALIFIED_HEAD=cd721231b498450e984810b4f06c4e14bdc311e1\nT003_080_QUALIFIED_TREE=4c5ddb92f8764f0107510ba72e6f697a3225bd56\nT003_080_CI_RUN=33252170158\nT003_080_FOCUSED_RUN=33252055912\nNEXT_TASK=T003-081",
)

plan = "specs/003-identity-policy-secrets-sandbox/implementation/current-execution-plan.md"
old_section = """### T003-080 — ACTIVE

Replace `BootstrapPolicy` in the normal authority-serving path with Cedar-backed active-policy evaluation while preserving the stable `Authorize(principal, action, resource, context)` contract and keeping initial/recovery bootstrap administration narrowly bounded to the local owner. Runtime evaluator errors, malformed stored bundle/source/context and missing active policy must fail closed for ordinary product effects.
"""
new_section = """### T003-080 — COMPLETE

Qualified at exact human-authored implementation head `cd721231b498450e984810b4f06c4e14bdc311e1`, tree `4c5ddb92f8764f0107510ba72e6f697a3225bd56`, by CI #616 / run `33252170158`, SUCCESS on Windows/macOS/Ubuntu. Focused runtime-policy run `33252055912` also completed SUCCESS.

Evidence: `implementation/t003-080-runtime-policy-qualification.md`.

The normal `golamd` authority-serving path now uses a Cedar-backed runtime policy loaded from one integrity-verified read-only active-policy snapshot for each authorization decision. Hard guards and unauthenticated-principal denial remain above Cedar; malformed or incompatible policy/schema/context/evaluator state fails closed; bounded bundle/rule evidence is recorded; and pre-activation bootstrap authority is restricted to narrow local-owner administration rather than ordinary product effects.

### T003-081 — ACTIVE

Add the minimum authenticated CLI/admin/test surface required by Spec 003 for policy lifecycle, capability leases, approvals, deterministic canary-secret qualification, authorization-decision explanation and sandbox-profile qualification. Every mutation must route through existing typed protected kernel/ledger authority paths; no shell or raw-SQL authority bypass is permitted.
"""
replace_once(plan, old_section, new_section)
replace_once(
    plan,
    "- Phase I: ACTIVE at T003-080; remaining T003-080..T003-084.",
    "- Phase I: ACTIVE at T003-081; remaining T003-081..T003-084.",
)
replace_once(
    plan,
    "T003_076_FOCUSED_RUN=33250097386\nNEXT_TASK=T003-080",
    "T003_076_FOCUSED_RUN=33250097386\nT003_080=PASS\nT003_080_QUALIFIED_HEAD=cd721231b498450e984810b4f06c4e14bdc311e1\nT003_080_QUALIFIED_TREE=4c5ddb92f8764f0107510ba72e6f697a3225bd56\nT003_080_CI_RUN=33252170158\nT003_080_FOCUSED_RUN=33252055912\nNEXT_TASK=T003-081",
)

candidate = "specs/003-identity-policy-secrets-sandbox/implementation/t003-080-qualification-candidate.md"
replace_once(
    candidate,
    "**Status**: QUALIFICATION_CANDIDATE — NOT YET PASS",
    "**Status**: SUPERSEDED_BY_EXACT_HEAD_QUALIFICATION",
)
replace_once(
    candidate,
    "T003_080=NOT_YET_PASS",
    "T003_080=PASS_RECORDED_IN_FINAL_EVIDENCE",
)

Path("specs/003-identity-policy-secrets-sandbox/implementation/t003-080-runtime-policy-qualification.md").write_text(
    """# T003-080 Runtime Policy Qualification

**Status**: PASS

## Exact qualification identity

- Qualified human head: `cd721231b498450e984810b4f06c4e14bdc311e1`
- Qualified tree: `4c5ddb92f8764f0107510ba72e6f697a3225bd56`
- Official CI: #616 / run `33252170158`
- Focused runtime-policy qualification: run `33252055912`
- Official platforms: Windows, macOS, Ubuntu

CI #616 completed SUCCESS on the exact human-authored candidate across all three supported repository CI platforms. Each platform completed pinned formatting, `clippy -D warnings`, full workspace tests, property qualification, bounded fuzz smoke, IPC qualification, authenticated daemon IPC qualification, adversarial authority qualification, daemon build and the applicable external strict-local network observer.

## Qualified authority behavior

The qualified T003-080 boundary:

- replaces `BootstrapPolicy` in the normal product-authority serving path with `RuntimeAuthorityPolicy`;
- preserves the stable `Authorize(principal, action, resource, context)` contract;
- executes hard kernel guards before Cedar and denies unauthenticated principals before policy evaluation;
- reads the protected active immutable policy bundle from one read-only SQLite snapshot after active-policy integrity verification;
- revalidates exact stored Cedar policy/schema source before every decision;
- maps normalized Golam principal/action/resource/context values into Cedar evaluation inputs without granting Cedar lease/approval authority;
- records bounded active policy bundle ID/hash and matched rule identifiers in authorization evidence;
- maps malformed bundle/schema/context/request/evaluator diagnostics to DENY;
- observes a newly activated policy without daemon restart by loading current protected active-policy state for each authorization decision;
- restricts no-active-policy bootstrap behavior to narrow local-owner administration (`policy.stage`, `policy.activate`, `approval.issue`, `recovery.status.read`);
- denies ordinary product effects during bootstrap state;
- leaves the separate authenticated first-client enrollment bootstrap path narrowly scoped.

No hard denial moved below policy evaluation, no authority constructor became public, no secret plaintext path was introduced, no external egress was enabled and no sandbox-containment claim changed.

```text
T003_080=PASS
T003_080_QUALIFIED_HEAD=cd721231b498450e984810b4f06c4e14bdc311e1
T003_080_QUALIFIED_TREE=4c5ddb92f8764f0107510ba72e6f697a3225bd56
T003_080_CI_RUN=33252170158
T003_080_FOCUSED_RUN=33252055912
NORMAL_PRODUCT_AUTHORITY_POLICY=CEDAR_ACTIVE_POLICY
HARD_GUARDS_ABOVE_CEDAR=YES
UNAUTHENTICATED_PRINCIPAL_PRE_POLICY_DENY=YES
BOOTSTRAP_PRODUCT_EFFECTS_ALLOWED=NO
ACTIVE_POLICY_RELOAD_WITHOUT_DAEMON_RESTART=YES
NEXT_TASK=T003-081
```
"""
)
