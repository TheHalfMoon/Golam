# Quickstart — Spec 005

This quickstart is a qualification guide, not a promise that every feature is already implemented.

## Canonical prerequisite

Implementation begins only after Spec 005 planning is `CLOSED_CANONICAL` and branch creation occurs from the exact post-merge canonical `main` that passed push-triggered CI.

## Safe implementation order

1. Implement/qualify pure tool/context/memory types and deterministic validation.
2. Implement bounded read-only filesystem inspection and L0 context evidence.
3. Implement bounded Git read evidence and context integration.
4. Qualify and implement the Phase D L0 text-search path **in-process only**: either Golam-owned bounded search or an exactly admitted Rust crate surface. Do not launch an external search binary while production remains `native:unqualified`.
5. Implement canonical managed Markdown plus the exact dedicated memory-operational SQLite store, promotion-authority validation, immutable `MemoryMutationIntent` target/digest/version/store/effect bindings, durable Effect Gate PREPARED-before-mutation, content-only Markdown/front-matter authority separation, commit-time conditional compare-and-replace, three-surface read-back/reconciliation, single-writer attribution and user-edit reconciliation.
6. Add consequential filesystem mutations through the existing Effect Gate with target identity/precondition verification.
7. Qualify an exact production native containment profile before enabling any child-process-backed tool, shell/process, external search binary, or local executable MCP/skill.
8. Add process-backed tools only on admitted profiles. If a pinned external search binary is still useful, qualify it here against the exact admitted containment profile; otherwise keep the binary path not applicable.
9. Add Agent Skills, MCP and ACP interoperability only with explicit exact dispatch bindings: reviewed package/binding digest, version, reviewed local capability/mapping identity and current lifecycle state must be revalidated immediately before every activation/dispatch, and stale queued/prepared/cached/approved state must fail closed.
10. Add bounded browser/network tooling only under explicit egress authority and credential-safe authenticated transport rules.
11. Run adversarial convergence, full exact-head CI, independent semantic review, reconciliation, expected-head merge and post-merge canonical-main CI.

## Fail-closed examples

The following MUST fail rather than silently degrade:

```text
shell requested + native:unqualified
external search binary requested + native:unqualified
remote MCP requested + strict-local
filesystem write target becomes symlink/reparse escape
memory writer enabled before promotion-authority validator qualification
memory promotion from model text without promotion authority
SECRET_DERIVED candidate promotion
attempted SECRET_DERIVED taint downgrade
managed-memory mutation lacks durable PREPARED Effect Gate intent
managed-memory intent omits exact Markdown target/digest/version or exact memory-store binding
memory-operational store aliases the authority database
commit-time Markdown target identity or digest/version differs from prepared intent
authority-bearing Markdown/front-matter field attempts to mint scope/taint/provenance/approval/authorization/effect state
authority journal + Markdown + memory-SQLite read-back disagree
wrong memory-operational-store binding
managed-memory prior effect remains UNKNOWN_OUTCOME
process cancellation observed without terminal descendant reconciliation
user-edited Markdown conflicts with last managed version
stale expected Git HEAD/index/worktree
skill activation uses deprecated/revoked/replaced/version-digest-mapping-mismatched dispatch binding
MCP dispatch uses deprecated/revoked/replaced/version-digest-mapping-mismatched binding
queued/prepared/cached/approved skill or MCP decision survives binding revocation/replacement
MCP-advertised capability broader than local mapping
unsupported protocol feature
credential-bearing redirect changes origin without fresh endpoint/scope authorization
derivative-dependent operation + index missing and governed rebuild cannot complete
```

A missing derivative index does **not** block canonical Markdown/memory-operational-SQLite startup or ordinary canonical memory reads. A derivative-dependent operation first triggers the governed rebuild rule; if the required generation cannot be rebuilt and qualified, only that derivative-dependent operation fails closed.

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
- Phase D proves its text-search path is in-process and cannot spawn an external utility while production native execution is unadmitted;
- any later external search binary is either exactly qualified under an admitted containment profile or remains unavailable;
- at least one consequential file edit goes through durable Effect Gate evidence and deterministic read-back verification;
- path/protected-resource adversarial corpus passes on supported platforms;
- strict-local external observation passes;
- admitted process-backed tools prove descendant supervision plus terminal process-tree reconciliation; cancellation alone is not terminal proof;
- canonical memory survives restart, handles user edits/conflicts, and respects live-state precedence;
- promotion-authority validation is qualified before the governed memory writer is enabled;
- every managed mutation proves an immutable exact target/digest/version/store/effect binding that survives PREPARED unchanged;
- the dedicated memory-operational SQLite store remains separate from authority state and every effect-owned row binds the exact store identity/schema, Effect identity and intent digest;
- Markdown body/front matter cannot mint authority-bearing state and malicious reserved fields enter explicit reconciliation/quarantine;
- commit-time Markdown replacement revalidates target identity + digest/version and uses conditional identity-preserving replacement;
- every committed managed memory version preserves initiating/creating principal, governed writer identity and exact mutation Effect attribution through restart reconciliation;
- terminal managed-memory success requires deterministic read-back/reconciliation across authority journal, exact Markdown identity/digest/version and exact memory-operational-store rows; wrong-store and split-store cuts remain failed/reconciling or `UNKNOWN_OUTCOME`;
- FORGET/REDACT invalidates and rebuilds every enabled derivative without treating partial multi-store completion as success;
- exact reviewed skill and MCP dispatch bindings are revalidated immediately before every activation/dispatch and stale queued/prepared/cached/approved state cannot survive deprecation/revocation/replacement/version/digest/mapping changes;
- executable features are either exactly qualified on their advertised platforms or remain explicitly unavailable;
- credential-bearing network redirects cannot leak/replay secrets to an unauthorized or downgraded hop;
- malicious MCP/skill/memory input cannot mint authority or clear taint;
- exact-head Windows/macOS/Ubuntu CI is green;
- substantive independent semantic review is clean on the same unchanged head;
- guarded expected-head merge succeeds;
- push-triggered canonical-main CI succeeds.
