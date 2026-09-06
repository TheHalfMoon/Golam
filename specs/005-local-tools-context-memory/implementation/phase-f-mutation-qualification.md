---
task_ids: [T005-060, T005-061, T005-062, T005-063, T005-064, T005-065, T005-066, T005-067]
outcome: PASS
qualified_implementation_head: fc1b6b25aa4e25f11a4eeea043b89c414185fe92
qualification_date: 2026-09-05
official_ci:
  workflow: ci
  run_id: 33960421949
  run_number: 1152
  conclusion: success
  platforms: [windows-latest, macos-latest, ubuntu-latest]
scope:
  filesystem_mutation: bounded_effect_bound_descriptor_relative
  git_mutation: linux_only_bounded_add_commit_branch_create
  destructive_git: explicitly_unavailable_to_ordinary_authority
  reconciliation: protected_exact_receipt_required_for_terminal_resolution
  protected_state: excluded_from_generic_file_and_git_roots
  process_launch: none
  shell_launch: none
  network_widening: none
waiver_taken: false
---

# Phase F consequential filesystem/Git mutation qualification

## Verdict

T005-060 through T005-067 are qualified on implementation head `fc1b6b25aa4e25f11a4eeea043b89c414185fe92` by official CI #1152 / run `33960421949`, which completed successfully on Windows, macOS and Ubuntu.

This evidence does not itself satisfy T005-068 because committing this record changes the branch head. T005-068 requires fresh exact-head CI on the evidence-bearing head.

## T005-060 — bounded file create/write/replace

The Phase F file mutation provider is bound to a Kernel-prepared consequential Effect and exact action/resource/precondition/payload hashes. Mutation expectations bind:

- authorized root and requested operation;
- expected parent identity;
- expected target existence/kind/identity;
- expected content digest and size where applicable;
- exact replacement payload digest.

Unix mutation uses the admitted `nix 0.31.3` `fs` feature and descriptor-relative primitives. Create is no-overwrite. Existing target replacement is conditioned on exact retained identity/content state. A successful result requires deterministic post-operation read-back of target identity and content; an ambiguous commit boundary is not reported as success.

Qualified adversarial coverage includes stale content, stale parent identity, unexpected target existence, identity changes and read-back mismatch.

## T005-061 — rename/delete

Rename/delete are distinct governed mutation operations rather than generic path writes. Preconditions bind source/destination target and parent identities. Rename refuses an unexpected destination and delete refuses stale target/parent state. Terminal success requires exact post-operation observation.

Qualified coverage includes normal rename/delete plus destination collision and stale-parent denial.

## T005-062 — TOCTOU mutation boundary

Adversarial tests prove that the checked filesystem identity must survive to the mutation boundary. The qualification corpus includes:

- symlink substitution after preparation;
- source inode replacement after preparation;
- parent/target stale-state changes;
- protected-root overlap;
- preservation of the original user file when the checked identity no longer matches.

No cross-platform equivalence is inferred. Unsupported platform/provider semantics remain explicit denial states.

## T005-063 — bounded Git mutation

The admitted ordinary Git mutation vocabulary is exactly:

- `git.add`;
- `git.commit`;
- `git.branch.create`.

The implementation is Golam-owned and does not invoke `git`, a shell, another process or the network. The first mutation profile is Linux-only and fails closed elsewhere.

Every mutation is bound to a sealed `GitStatusObservation` and a stable `GitMutationExpectation` containing repository identity, expected HEAD, index checksum and status/worktree digest. Qualification proves:

- stale HEAD after preparation denies before mutation;
- stale valid index after preparation denies before mutation;
- stale worktree content denies add before index mutation;
- nested Git add binds the actual parent identity;
- a prepared branch Effect cannot be rebound to another branch;
- branch creation uses no-overwrite semantics;
- an existing branch cannot be moved;
- loose-object content-address collisions are read back and mismatches fail closed;
- add/commit/branch success is read-back verified.

The Golam-owned stored-zlib writer remains within the previously admitted `miniz_oxide 0.9.1` `default-features=false` boundary; this phase adds no feature expansion.

## T005-064 — destructive Git remains outside ordinary authority

`golam-core::git_authority` freezes ordinary authority to `Add`, `Commit` and `BranchCreate` only. The following operations are explicit typed destructive states whose `ordinary_authority()` is `None`:

- force push;
- forced local ref movement;
- branch overwrite;
- rebase;
- shared-history rewrite;
- destructive equivalents.

The bounded mutation implementation contains no force-ref, rebase, remote-push, network or shell/process path. Existing local branches are never overwritten or moved by ordinary branch-create authority.

## T005-065 — deterministic post-operation verification and reconciliation

Filesystem and Git providers produce deterministic read-back receipts only after verifying the exact post-operation state. Ambiguous completion is routed through the existing Effect lifecycle rather than being converted to success by provider output.

A protected operational evidence store now persists exact mutation intent and verified receipt evidence bound to:

- `EffectId`;
- action;
- resource;
- preconditions hash;
- payload hash;
- provider identity;
- immutable bounded intent bytes;
- one-shot verified terminal receipt and integrity hash.

Intent replay is idempotent only for byte-identical evidence. Receipt replay is idempotent only for the exact same binding/status/bytes. Rebinding under the same Effect is rejected.

`resolve_tool_reconciliation(... Succeeded|Failed)` does not trust caller-supplied evidence bytes. It requires the Kernel-owned protected evidence store to contain a matching provider-verified receipt with the exact requested terminal classification, and the Effect transition records only that verified receipt integrity hash. Missing or mismatched evidence cannot become success or failure. Remaining ambiguity escalates through the existing `UNKNOWN_OUTCOME -> reconciling -> manual_review` lifecycle.

This store is operational evidence, not a new authority source: recording and reading remain behind current Kernel authorization and exact Effect binding.

## T005-066 — protected Golam resources remain unreachable

Generic file and Git roots are rejected when they overlap protected Golam runtime/authority state. Protection is enforced when the authorized root is created, before target-level mutation logic can run.

Qualification covers generic filesystem and Git roots and the Kernel generic path-admission boundary. Ordinary mutation authority therefore cannot be used to rewrite policy, capability lease, approval, secret, Effect or audit state through a path alias.

## T005-067 — restart, UNKNOWN_OUTCOME, idempotency and stale state

The existing Effect Gate restart semantics remain authoritative: interrupted at-most-once/irreversible attempts are never blindly redispatched.

Phase F adds mutation-specific restart qualification proving:

1. an exact protected verified receipt survives Kernel reopen;
2. reconciliation after restart preserves the original attempt, precondition and payload binding;
3. verified success can resolve after restart without a second dispatch;
4. attempt count remains exactly one;
5. when only the intent survives and no verified receipt exists, caller-provided bytes cannot claim success;
6. the Effect remains `reconciling` and unresolved rather than silently succeeding;
7. stale HEAD/index/worktree/identity state is rejected before a new mutation can commit.

## Exact qualification evidence

Official CI #1152 / run `33960421949` on `fc1b6b25aa4e25f11a4eeea043b89c414185fe92` completed SUCCESS for all three jobs:

- `rust-ubuntu-latest`: SUCCESS;
- `rust-macos-latest`: SUCCESS;
- `rust-windows-latest`: SUCCESS.

The workflow completed format, Clippy, workspace tests, property qualification, bounded fuzz smoke, platform-applicable IPC qualification, authenticated daemon IPC qualification, adversarial authority qualification, daemon build and platform-applicable strict-local external network observation.

Relevant qualified test families include:

- file create/write/replace exact precondition and read-back tests;
- rename/delete conditional mutation tests;
- symlink and inode-swap mutation adversarial tests;
- stale Git HEAD/index/worktree tests;
- exact Git Effect rebinding denial;
- Git loose-object collision/read-back tests;
- branch no-overwrite tests;
- destructive Git authority-denial tests;
- protected file/Git root overlap tests;
- protected mutation-evidence immutability and terminal-status tests;
- restart with and without verified mutation receipts;
- existing no-blind-redispatch Effect restart tests.

## Honesty boundaries

- `PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO` remains unchanged.
- No shell/process execution is enabled by Phase F.
- No remote Git operation or network authority is implemented.
- Git mutation is not claimed on Windows or macOS; unsupported platforms deny explicitly.
- Filesystem platform claims remain limited to the exact implementation/CI boundaries and do not infer Windows descriptor-relative equivalence.
- Caller/model/protocol output is not terminal reconciliation authority.
- No waiver is taken.

```text
T005_060=PASS
T005_061=PASS
T005_062=PASS
T005_063=PASS
T005_064=PASS
T005_065=PASS
T005_066=PASS
T005_067=PASS
PHASE_F_IMPLEMENTATION_HEAD=fc1b6b25aa4e25f11a4eeea043b89c414185fe92
PHASE_F_IMPLEMENTATION_CI_RUN=33960421949
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
PROCESS_LAUNCH_ENABLED=NO
SHELL_LAUNCH_ENABLED=NO
NETWORK_WIDENING=NO
WAIVER_TAKEN=NO
NEXT_TASK=T005-068
```
