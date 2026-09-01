# Contract — Tool Execution & Context

**Spec**: 005 Local Tools, Context & Memory

## 1. Authority boundary

A tool is a mechanism, never an authority root.

```text
TOOL_DESCRIPTOR != CAPABILITY
TOOL_CALL_CANDIDATE != EFFECT_AUTHORIZATION
TOOL_RESULT != VERIFIED_SUCCESS
PATH_STRING != TARGET_IDENTITY
CONTEXT_RANK != SOURCE_AUTHORITY
```

The model/harness may propose a `ToolCallCandidate`. Golam validates the candidate into a bounded `ToolRequest`; the Kernel independently evaluates policy/capability authority. Consequential operations become durable Effect Gate transactions before external or canonical mutation.

No tool, donor helper, MCP server, skill, model or adapter may set or honor a semantic equivalent of `skipApproval`.

## 2. Tool descriptor

Every executable or read-only tool surface MUST expose an immutable versioned descriptor containing:

- stable `ToolId` and `ToolVersion`;
- operation class;
- explicit input/output byte and count bounds plus finite duration bounds;
- required Golam capability class;
- effect semantics (`READ_ONLY`, `IDEMPOTENT_AT_LEAST_ONCE`, `AT_MOST_ONCE`, `COMPENSATABLE`, `IRREVERSIBLE` as applicable);
- network posture;
- sandbox/containment requirement;
- target-identity rules;
- reconciliation and verification policy.

Material execution/validation changes require a new `ToolVersion`.

## 3. Request binding

Before protected execution, the request MUST bind:

```text
initiating principal
exact tool id/version
candidate provenance
requested operation
authorized resource class
resolved target identity or explicit target-resolution plan
capability/lease context
taint/provenance context
idempotency material
current preconditions
```

User/model strings do not become authority by being present in the request. Once a protected request is durably prepared, its authority-relevant binding is immutable. A retry, materially changed target, changed operation, changed precondition set or changed authority context requires a new request/effect identity rather than mutation of the prepared request.

## 4. Filesystem target contract

### 4.1 Root authority

Filesystem authority is an explicit set of platform-resolved authorized roots plus allowed operation classes. Generic filesystem authority excludes protected Golam resources regardless of lexical containment.

### 4.2 Resolution

The protected action boundary MUST evaluate:

```text
requested path
-> bounded lexical normalization
-> platform-aware parent/target resolution
-> symlink/reparse/junction/mount/alias chain observation
-> protected-resource exclusion
-> operation-specific authorization
-> identity-preserving action
```

A lexical prefix match alone is never sufficient.

### 4.3 Special files and bounds

Ordinary file tools MUST deny unsupported special files including device nodes, sockets, FIFOs, proc-like magic handles or platform equivalents. Reads/lists/walks require byte/count/depth/time bounds.

### 4.4 Mutations and TOCTOU

For create/write/rename/delete operations, policy MUST distinguish target and parent authority. Race-sensitive mutation MUST use identity-preserving handles/primitives when the platform supports them. If checked identity cannot be preserved through commit, the action fails closed.

Optional `FileMutationExpectation` values (existence, kind, identity, digest, size, parent identity) are stale-evidence guards; mismatch denies the mutation.

Rejected or failed operations MUST preserve user data and MUST NOT destructively consume the input artifact solely because validation failed.

## 5. Git contract

Read-only status/diff/log/tree/blob inspection may feed context within repository authorization.

Git mutation is consequential and MUST bind:

- repository identity;
- expected HEAD/ref identity;
- relevant index/worktree preconditions;
- exact requested operation;
- normal Effect Gate authorization;
- post-operation read-back verification.

Force push, force ref movement, destructive history rewrite, rebase of shared history, or bypass of repository governance is not part of ordinary Spec 005 tool authority.

## 6. Process/shell contract

### 6.1 Admission gate

`native:unqualified` is a denial state. Shell/process execution MUST remain unavailable unless the requested platform/profile has an exact `ADMITTED` production containment implementation.

### 6.2 Launch plan

A permitted launch MUST bind:

```text
admitted containment profile
exact executable identity
argv
cwd identity
cleared ambient environment
explicit environment values
secret-handle bindings
filesystem rights
network rights
device rights
resource limits
inherited-handle rules
timeout/cancellation policy
descendant supervision
```

Unbrokerable secrets follow the canonical secret fallback contract and never become ambient process environment by default.

### 6.3 Shell syntax

Command strings are content, not parsed authority. If shell syntax is supported, command graph, redirections, substitutions and executable identities MUST be explicit or the request is rejected as ambiguous. Model/donor parsing claims cannot waive authorization.

### 6.4 Evidence

PREPARED/authorized durable effect evidence precedes launch. Output, exit state, timeout/cancellation, descendants, redaction and reconciliation evidence are attributable to the exact request/attempt.

## 7. Browser/network tool contract

Spec 005 browser scope is bounded document/web tooling, not Desktop/computer control.

External network actions require explicit egress authority and bind method, URL/origin, redirect policy, request-body source, secret brokerage, target class, output/download bounds and taint/provenance. Redirects are revalidated against egress policy.

A credential-bearing hop MUST additionally satisfy credential transport and origin binding before any brokered secret or sensitive authorization material is attached:

- use an authenticated encrypted transport with endpoint identity/certificate validation (for HTTP, HTTPS with valid TLS peer/hostname validation; protocol-specific equivalents require their own exact qualification);
- bind the brokered credential to the authorized origin/endpoint and permitted operation scope independently of general egress permission;
- never automatically forward authorization headers, cookies, brokered secrets or secret-bearing request bodies across an origin change, redirect, proxy transition or protocol change;
- on every redirect or endpoint/protocol change, strip sensitive material, revalidate egress and endpoint identity, re-evaluate credential scope, and re-broker only when the new hop is explicitly authorized;
- deny credential-bearing transport downgrade or any hop whose authenticated transport/credential scope cannot be proven.

Egress authorization alone never authorizes credential disclosure.

Strict-local external network denial is absolute. Local capability failure never creates cloud/browser/remote-MCP fallback permission.

## 8. Context evidence contract

Every context item MUST retain:

```text
source identity and kind
exact version/observation identity
content digest/reference
authority class
taint set
permission scope
freshness policy
observed time
conflict/supersession relationships when known
```

Retrieval score, semantic similarity, model confidence or frequency never raises authority or clears taint.

## 9. Context compiler

The bounded pipeline is:

```text
intent
-> evidence requirements
-> source routing
-> retrieve
-> permission/authority/freshness/taint filter
-> rank
-> sufficiency decision
-> bounded replan
-> ContextCapsule
```

L0 sources are direct files, bounded search, Git evidence, exact user-selected artifacts, canonical Golam evidence and permitted managed memory. L1 structural sources are conditional on measured need and exact dependency admission. L2 graph/dataflow/vector/runtime indexing is deferred unless separately justified.

A `ContextCapsule` is an attributable projection. It never replaces or rewrites canonical source evidence.

## 10. Freshness/conflict rule

When a remembered or cached/contextual claim conflicts with fresher authoritative repository/filesystem/device/external state, the live authoritative state wins and the conflict is surfaced. Golam MUST NOT silently promote stale memory because it ranked higher.

## 11. Verification

Consequential tool completion is not satisfied by process success or model prose alone. The descriptor's verification policy determines required read-back/deterministic evidence. Verification evidence is independently attributable and cannot be fabricated by the same tool result when the contract requires an external/read-back check.

## 12. Required adversarial corpus

Qualification includes:

- symlink/reparse/junction/root escapes;
- protected-resource aliases;
- path swap/rename races;
- oversized/special files;
- stale mutation expectations;
- Git stale HEAD/index/worktree;
- shell parsing/metacharacter/redirection ambiguity;
- ambient environment/secret leakage;
- descendant escape/cancellation;
- unauthorized network and redirect widening;
- credential forwarding across redirect/origin/protocol changes;
- unauthenticated TLS/endpoint identity and credential-bearing downgrade attempts;
- malicious tool output trying to mint authority or clear taint;
- stale context outranking fresher authoritative state.
