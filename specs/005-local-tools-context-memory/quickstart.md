# Quickstart — Spec 005

This quickstart is a qualification guide, not a promise that every feature is already implemented.

## Canonical prerequisite

Implementation begins only after Spec 005 planning is `CLOSED_CANONICAL` and branch creation occurs from the exact post-merge canonical `main` that passed push-triggered CI.

## Safe implementation order

1. Implement/qualify pure tool/context/memory types and deterministic validation.
2. Implement bounded read-only filesystem inspection and L0 context evidence.
3. Implement bounded Git read evidence and context integration.
4. Implement canonical managed Markdown + SQLite operational memory state, durable Effect Gate mutation lifecycle, single writer and user-edit reconciliation.
5. Add consequential filesystem mutations through the existing Effect Gate with target identity/precondition verification.
6. Qualify the exact L0 search mechanism before adding a dependency/binary.
7. Qualify an exact production native containment profile before enabling shell/process or local executable MCP/skills.
8. Add process-backed tools only on admitted profiles.
9. Add MCP/ACP/Agent Skills interoperability within their contracts.
10. Add bounded browser/network tooling only under explicit egress authority and credential-safe authenticated transport rules.
11. Run adversarial convergence, full exact-head CI, independent semantic review, reconciliation, expected-head merge and post-merge canonical-main CI.

## Fail-closed examples

The following MUST fail rather than silently degrade:

```text
shell requested + native:unqualified
remote MCP requested + strict-local
filesystem write target becomes symlink/reparse escape
memory promotion from model text without promotion authority
SECRET_DERIVED candidate promotion
attempted SECRET_DERIVED taint downgrade
managed-memory mutation lacks durable PREPARED Effect Gate intent
managed-memory prior effect remains UNKNOWN_OUTCOME
user-edited Markdown conflicts with last managed version
stale expected Git HEAD/index/worktree
MCP-advertised capability broader than local mapping
unsupported protocol feature
credential-bearing redirect changes origin without fresh endpoint/scope authorization
derivative-dependent operation + index missing and governed rebuild cannot complete
```

A missing derivative index does **not** block canonical Markdown/SQLite memory startup or ordinary canonical memory reads. A derivative-dependent operation first triggers the governed rebuild rule; if the required generation cannot be rebuilt and qualified, only that derivative-dependent operation fails closed.

## Ordinary CI

Ordinary CI remains hermetic and does not require:

- model downloads;
- cloud credentials;
- external services;
- Docker;
- remote MCP servers;
- vector databases;
- specialized accelerators.

Use deterministic local fixtures and scripted adversarial inputs.

## Minimum closeout proof

Before Spec 005 implementation can close:

- repository read/search/context path works with exact provenance;
- at least one consequential file edit goes through durable Effect Gate evidence and deterministic read-back verification;
- path/protected-resource adversarial corpus passes on supported platforms;
- strict-local external observation passes;
- canonical memory survives restart, handles user edits/conflicts, and respects live-state precedence;
- managed-memory mutation proves current authorization, durable PREPARED-before-canonical-mutation evidence, integrity-chained terminal outcome/read-back verification, and `UNKNOWN_OUTCOME` reconciliation/dependent-mutation blocking;
- FORGET/REDACT invalidates and rebuilds every enabled derivative without treating partial multi-store completion as success;
- executable features are either exactly qualified on their advertised platforms or remain explicitly unavailable;
- credential-bearing network redirects cannot leak/replay secrets to an unauthorized or downgraded hop;
- malicious MCP/skill/memory input cannot mint authority or clear taint;
- exact-head Windows/macOS/Ubuntu CI is green;
- substantive independent semantic review is clean on the same head;
- guarded expected-head merge succeeds;
- push-triggered canonical-main CI succeeds.
