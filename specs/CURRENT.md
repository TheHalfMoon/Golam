# Golam Canonical Current-State Pointer

This file records only **durable canonical program state**. It is not a cache of live pull-request heads, CI conclusions, review comments or mergeability. Those facts MUST be re-fetched from GitHub immediately before any qualification, readiness, merge or successor-authority action.

## Canonical durable state at this proposal base

```text
CANONICAL_MAIN=c85b4b8f0d6ffccb039645803542d75b3bd47f29
SPEC_005_IMPLEMENTATION_COMPLETE=YES
SPEC_005_CLOSED_CANONICAL=YES
SPEC_006_PLANNING_CLOSED_CANONICAL=YES
SPEC_006_PRODUCT_IMPLEMENTATION_AUTHORIZED=YES
ACTIVE_CANONICAL_IMPLEMENTATION_UNIT=SPEC_006_DESKTOP_COMPUTER_CONTROL
ACTIVE_IMPLEMENTATION_PR=24
ACTIVE_IMPLEMENTATION_PR_IS_CANONICAL=NO_UNTIL_MERGED_AND_POST_MERGE_QUALIFIED
PROGRAM_SUPERIORITY_OVERLAY=PLANNING_GOVERNANCE_ONLY
PROGRAM_SUPERIORITY_OVERLAY_IMPL_AUTHORITY=NO
```

The active Spec 006 implementation branch/PR head, checks, reviews, changed files, task frontier and mergeability MUST be read live. Do not hardcode those values here.

The superiority overlay remains planning/governance material whether it is still under review or later becomes canonical. Canonicalization of the overlay does not itself create product implementation authority for Browser/Application Semantic Control, Visual Semantics, Workers/Automations, GolamConnect, release infrastructure or any other future unit. Each such unit still requires exact successor authority and its own bounded Spec Kit lifecycle.

## Authority order

1. exact live GitHub/repository truth for mutable state;
2. `.specify/memory/constitution.md`;
3. canonical Spec 001 program architecture/contracts/tasks;
4. closed canonical predecessor spec packages and closeout evidence;
5. the currently authorized bounded spec package;
6. exact Source Foundry records for every admitted source/dependency/runtime primitive;
7. exact-head CI/review/evidence for the lifecycle action being attempted.

A nonmerged proposal branch, open PR, issue, source candidate, benchmark result, bot summary, stale status block or prior handoff cannot create product authority.

## Current implementation rule

Spec 006 implementation is the only currently authorized product feature unit at this canonical base. Continue it only through `specs/006-desktop-computer-control/tasks.md` and its canonical governance.

The program-superiority roadmap may be researched, reviewed and canonicalized as planning/governance material in a separate PR. It MUST NOT widen PR #24, admit new product dependencies into PR #24, or authorize Browser/Application Semantic Control, Visual Semantics, Workers/Automations, GolamConnect or other future implementation by itself.

## Mutable-state rule

Before every CI qualification, independent review request, Ready transition, merge, post-merge closeout or successor-authorization decision:

```text
FETCH_CURRENT_MAIN
FETCH_CURRENT_PR_HEAD_AND_BASE
FETCH_CURRENT_DIFF
FETCH_CURRENT_CHECKS
FETCH_CURRENT_REVIEWS_AND_THREADS
VERIFY_EXPECTED_HEAD
```

A branch mutation invalidates CI/review evidence bound to the prior head. Canonical predecessor evidence remains valid unless superseded by new canonical truth.

## Updating this file

Update this file only when **durable canonical lifecycle state changes**, normally after a guarded merge and successful post-merge canonical-main qualification. Do not update it for every intermediate implementation commit.
