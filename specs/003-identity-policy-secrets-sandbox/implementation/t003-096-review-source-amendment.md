# T003-096 Review Source Amendment

**Date**: 2026-08-31  
**Scope**: Spec 003 Phase J external semantic review source only  
**Founder direction**: `skip qodo use others`

## Decision

Qodo is no longer an authorized or required review source for the active Spec 003 closeout sequence. Codex remains excluded. T003-096 continues to require a fresh substantive independent external semantic review after fresh exact-head CI, but the reviewer may be any actually available repository-integrated independent reviewer other than Qodo or Codex.

Eligible examples include CodeRabbit, Cubic, Greptile, or an equivalent independent reviewer when it produces an actual substantive result bound to the exact reviewed head.

The following do not satisfy T003-096:

- self-review;
- status-only or summary-only bot output;
- stale-head review output;
- billing-blocked, rate-limited, skipped, unavailable or failed-to-start responses;
- CI by itself;
- historical Qodo results;
- Codex.

## Rationale

Qodo remained externally blocked by workspace billing/credits despite successful exact-head CI. The founder explicitly changed the reviewer-source direction rather than waiving review. Constitution v1.2.0 does not mandate any named review vendor, so no constitutional amendment is required. The substantive external semantic review gate remains mandatory and exact-head bound.

## Qualification consequences

This governance mutation moves the implementation branch and invalidates CI #661 as final exact-head closeout evidence. Fresh Windows/macOS/Ubuntu CI is required on the resulting exact head before any replacement reviewer request.

If a replacement reviewer reports any material finding, the finding must be repaired without waiver, followed by fresh exact-head three-platform CI and a fresh substantive independent external review on the new exact head.

T003-097, Ready, merge, T003-098, `SPEC_003_CLOSED_CANONICAL=YES`, and Spec 004 remain blocked until T003-096 genuinely passes.

```text
FOUNDER_REVIEW_SOURCE_DIRECTION=SKIP_QODO_USE_OTHERS
QODO=EXCLUDED_FROM_ACTIVE_SPEC_003_CLOSEOUT
CODEX=EXCLUDED
EXTERNAL_SEMANTIC_REVIEW_REQUIRED=YES
REVIEW_MUST_BE_SUBSTANTIVE=YES
REVIEW_MUST_BIND_EXACT_HEAD=YES
SUMMARY_OR_STATUS_ONLY_COUNTS=NO
WAIVER_TAKEN=NO
```
