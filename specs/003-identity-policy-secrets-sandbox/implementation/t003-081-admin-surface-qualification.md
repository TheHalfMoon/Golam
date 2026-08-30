# T003-081 Admin Surface Qualification

**Status**: SOURCE_CANDIDATE_PASS — RECORDING_HEAD_REQUALIFICATION_REQUIRED

## Exact source qualification identity

- Qualified source candidate: `877f8e45f8f8ba4ba4a98af036d51032b3fba684`
- Qualified source tree: `1808de4cce60669d89a5c3a11f2c8f47b6608b2f`
- Official CI: #630 / run `33264029849`
- Official platforms: Windows, macOS, Ubuntu

CI #630 completed SUCCESS on the exact T003-081 source candidate after GitHub-hosted runners became available again. Every supported platform allocated a real runner and completed the applicable pinned formatting, `clippy -D warnings`, full workspace tests, property qualification, bounded fuzz smoke, IPC transport qualification, authenticated daemon IPC qualification, adversarial authority qualification, daemon build, and external strict-local observation.

This evidence file is repository-owned qualification recording. Because adding this file moves the branch head, the recording head itself requires fresh exact-head CI before T003-081 may be treated as stably closed and T003-082 implementation may mutate the branch.

## Qualified boundary

The qualified T003-081 source candidate:

- exposes only the minimum authenticated local CLI/admin/test surface for `policy validate`, authority qualification for lease/approval/secret-canary/sandbox-profile, and `authority explain`;
- routes daemon requests through the existing authenticated local IPC lifecycle and rejects requests before authentication;
- admits enrolled bootstrap clients only to the explicit read/qualification actions introduced by T003-081; policy staging/activation and protected authority mutation remain outside this CLI surface;
- authorizes every qualification/explain request before the target read/qualification work and durably appends that allow/deny decision as attribution evidence;
- does not treat qualification/explain as storage-side-effect-free, but does not mutate the target policy/lease/approval/secret/sandbox authority object being inspected or qualified;
- preserves typed protected KernelApi/ledger mutation boundaries and introduces no shell or raw-SQL authority bypass;
- resolves `authority explain` by exact primary-key lookup (`WHERE decision_id = ?1 LIMIT 1`) instead of materializing the monotonically growing authorization-decision table;
- preserves global authorization-ledger integrity because `AuthorizationAuditLog::open()` opens the canonical `AuthorityStore`, whose startup verification covers `AuthorizationDecisionV2` through `authority-security-v2`; the selected record is additionally decoded and validated before explanation;
- returns bounded non-secret authorization metadata and never raw authorization context or secret plaintext;
- keeps the deterministic unknown-format secret canary internal to the designated-secret preparation path, never commits/returns/formats it, and drops the protected plaintext owner through the existing zeroizing path;
- preserves strict-local external egress hard denial and the previously qualified descendant-aware external observation gate;
- adds no public capability/approval/sandbox authority constructor and makes no new platform-containment claim.

## Review disposition

Bounded source review after the NFR-004 lookup repair found no additional material T003-081 defect. A pre-existing Spec 002 `KernelApi::authorization_decision_count()` implementation still obtains a test/diagnostic count through full-record materialization; it is not introduced by T003-081 and remains for the legitimate later regression/closeout phase rather than widening this task.

```text
T003_081_SOURCE_CANDIDATE=PASS
T003_081_QUALIFIED_SOURCE_HEAD=877f8e45f8f8ba4ba4a98af036d51032b3fba684
T003_081_QUALIFIED_SOURCE_TREE=1808de4cce60669d89a5c3a11f2c8f47b6608b2f
T003_081_CI_RUN=33264029849
T003_081_RECORDING_HEAD_REQUALIFICATION_REQUIRED=YES
NEXT_TASK_AFTER_RECORDING_HEAD_PASS=T003-082
WAIVER_TAKEN=NO
```