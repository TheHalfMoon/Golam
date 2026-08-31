# T003-096 Live Closeout Evidence Policy

**Status**: ACTIVE_CLOSEOUT_GOVERNANCE  
**Scope**: Spec 003 final exact-head CI, independent external review, PR lifecycle, and post-merge state

## Problem

The first two substantive replacement-review cycles exposed a self-invalidating documentation pattern. A branch-owned status field such as `FINAL_EXACT_HEAD_CI=PENDING` becomes stale when exact-head CI succeeds. Updating that field to `PASS` necessarily creates a new commit and therefore a new head for which the recorded CI evidence no longer applies.

The same problem applies to recording final external-review PASS inside the reviewed branch: mutating the branch merely to mirror the review result invalidates the reviewed head and requires another review.

## Decision

Branch-owned Spec 003 documents are stable authority, task, historical-evidence, and closeout-rule records. They MUST NOT embed a mutable claim for the latest final exact-head CI, final external-review result, Ready state, merge state, or post-merge canonical-main CI when recording that claim would itself move the qualified head.

For the final closeout cycle, exact live GitHub PR metadata is the authoritative state ledger for:

- current exact PR head;
- final exact-head CI run and platform results;
- final substantive independent external review result;
- unresolved material review threads;
- Draft/Ready state;
- expected-head guarded merge result;
- merge commit identity;
- post-merge canonical-main CI result;
- `SPEC_003_CLOSED_CANONICAL` transition.

The branch documents may record historical predecessor heads/runs/reviews and this evidence-location rule, but use `SEE_LIVE_GITHUB_PR_METADATA` for the mutable final state.

## Qualification rule

1. Any branch mutation invalidates final exact-head CI/review evidence for the previous head.
2. The resulting exact head must pass the required Windows/macOS/Ubuntu CI matrix.
3. A substantive independent external review must then be obtained on that unchanged exact head.
4. If the review is clean, record T003-096 PASS and T003-097 lifecycle evidence in GitHub PR metadata/comments without mutating the branch merely to mirror PASS.
5. Mark Ready and merge only with the expected exact head unchanged.
6. Record T003-098 only after post-merge canonical-main CI succeeds on the actual merge commit.

This policy does not weaken any CI, review, lifecycle, security, or post-merge gate. It makes the evidence ledger compatible with the repository's exact-head rule.

## Provenance

- Head `5ce152a8a3370b3927eb7b9eeaed838a3e0c7dc6`: CI #662 / run `33395230450` passed; substantive CodeRabbit review found stale closeout records and no additional material product correctness/security defect.
- Repair head `862936b4ea3c62ead65b318ba394b49444722944`: CI #663 / run `33405009138` passed; substantive CodeRabbit review again found only mutable closeout status fields stale after CI success and no additional material product correctness/security defect.

The second finding demonstrates that mirroring final dynamic PASS/PENDING state inside the branch is structurally self-invalidating and must be replaced by this live-metadata authority rule.

```text
CLOSEOUT_EVIDENCE_AUTHORITY=LIVE_GITHUB_PR_METADATA
BRANCH_MUTATION_INVALIDATES_PRIOR_FINAL_EVIDENCE=YES
EMBED_LATEST_FINAL_CI_STATUS_IN_QUALIFIED_BRANCH=NO
EMBED_LATEST_FINAL_REVIEW_STATUS_IN_REVIEWED_BRANCH=NO
EXACT_HEAD_CI_REQUIRED=YES
SUBSTANTIVE_EXTERNAL_REVIEW_REQUIRED=YES
QODO=EXCLUDED_BY_FOUNDER_DIRECTION
CODEX=EXCLUDED
EXPECTED_HEAD_GUARDED_MERGE_REQUIRED=YES
POST_MERGE_MAIN_CI_REQUIRED=YES
WAIVER_TAKEN=NO
```