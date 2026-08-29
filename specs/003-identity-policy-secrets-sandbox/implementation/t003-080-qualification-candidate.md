# T003-080 Runtime Policy Qualification Candidate

**Status**: QUALIFICATION_CANDIDATE — NOT YET PASS

## Focused qualification

Focused workflow `t003-080-runtime-policy-qualification` run `33252055912` completed SUCCESS and produced clean implementation commit `612d697dfc46b4e4e322d49852963452fb5fcdbc`, tree `5178725b9a46bcb4d649a8b2d572898be8e7ed22`.

The focused gate ran with Rust `1.98.0` and completed:

- active-policy integrity/material loading tests: PASS;
- runtime Cedar authority-policy tests: PASS;
- authorization ordering/evidence tests: PASS;
- daemon routing/authentication tests: PASS;
- `clippy -D warnings`: PASS.

The temporary qualification workflow and preparation helper self-deleted before the clean implementation commit.

## Qualified candidate behavior

The candidate replaces `BootstrapPolicy` in the normal `golamd` product-authority path with `RuntimeAuthorityPolicy` while preserving the stable `Authorize(principal, action, resource, context)` contract.

The runtime authority path now:

- preserves hard kernel guards above policy evaluation;
- denies unauthenticated principals before policy evaluation;
- reads the active immutable policy bundle through one read-only SQLite snapshot after active-policy integrity verification;
- revalidates the exact stored Cedar policy/schema bundle before evaluation;
- maps normalized Golam principal/action/resource/context values into Cedar-owned evaluation inputs without granting Cedar authority to mint leases or approvals;
- records bounded active policy bundle ID/hash and matched Cedar rule IDs in authorization evidence;
- maps malformed bundles, schema warnings, request/schema mismatch and Cedar evaluation diagnostics to DENY;
- re-reads the protected active policy pointer for each authorization decision so a successfully activated policy takes effect without daemon restart;
- permits only narrow local-owner bootstrap administration before the first active policy (`policy.stage`, `policy.activate`, `approval.issue`, `recovery.status.read`);
- denies ordinary product effects while the authority store is still in bootstrap policy state.

The separate authenticated IPC enrollment bootstrap path remains narrow and does not replace the normal product-authority evaluator.

No hard guard was moved below Cedar, no lease/approval constructor was exposed, no secret plaintext path was added, no network-capable child was launched, and no containment claim changed.

## Remaining qualification gate

Official repository CI on this exact human-authored candidate must complete SUCCESS on Windows, macOS and Ubuntu before `T003_080=PASS` may be recorded.

```text
T003_080=NOT_YET_PASS
T003_080_FOCUSED_RUN=33252055912
T003_080_IMPLEMENTATION_HEAD=612d697dfc46b4e4e322d49852963452fb5fcdbc
T003_080_IMPLEMENTATION_TREE=5178725b9a46bcb4d649a8b2d572898be8e7ed22
NORMAL_PRODUCT_AUTHORITY_POLICY=CEDAR_ACTIVE_POLICY
HARD_GUARDS_ABOVE_CEDAR=YES
UNAUTHENTICATED_PRINCIPAL_PRE_POLICY_DENY=YES
BOOTSTRAP_PRODUCT_EFFECTS_ALLOWED=NO
ACTIVE_POLICY_RELOAD_WITHOUT_DAEMON_RESTART=YES
OFFICIAL_THREE_PLATFORM_CI_REQUIRED=YES
NEXT_TASK=T003-080
```
