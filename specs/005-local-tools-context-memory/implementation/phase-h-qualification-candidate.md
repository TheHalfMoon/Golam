# Spec 005 Phase H Qualification Candidate

This artifact binds the completed Phase H focused/adversarial qualification evidence to the next founder-authored exact-head CI candidate. It does not claim final implementation closure, mergeability, release readiness, or successor authority.

## Canonical predecessor

- `main`: `4bd23b218304663349fb2f703cedd40c7a3038af`
- implementation PR: `#21`
- implementation branch: `impl/005-local-tools-context-memory`

## Phase H focused evidence

- `T005-089=PASS`
  - exact-head three-OS CI run: `34037746425`
  - qualified head: `8bc2f6b14710435995a7f0a8131de8c123755743`
- `T005-090=PASS_FOCUSED`
  - governed local MCP process launch reuses the admitted Linux x86_64 `process.stage` / `process.execute` boundary and performs immediate MCP binding revalidation.
- `T005-091=PASS_FOCUSED`
  - remote MCP remains a non-emitting preflight binding in Phase H and requires strict-local release plus explicit endpoint/network/credential/secret/redirect/proxy bindings.
- `T005-092=PASS_FOCUSED_ADVERSARIAL`
  - MCP advertisements and nested schemas remain untrusted data and cannot mint capability, approval, Effect or mapping authority; stale/revoked/replaced/mismatched dispatch state fails closed.
- `T005-093=PASS_FOCUSED`
  - ACP derives its client binding only from an authenticated `ServerLifecycle::Ready` state and exact enrolled-client identity; the adapter exposes no privileged `KernelApi`.
- `T005-094=PASS`
  - adversarial qualification workflow run: `34040262705`
  - qualified predecessor head: `e0a178c3525d05faa3da527f9d4d7b92c0609147`
  - suites covered Skills, MCP, ACP, local-client authentication, Effect handling, Kernel authority and daemon protocol boundaries, including stale/revoked/replaced/mismatched state and disconnect/unknown-outcome markers.

## Exact-head Phase H gate

`T005_095=PENDING_EXACT_HEAD_CI`

The next admissible evidence is ordinary PR CI on the exact commit that contains this artifact. Windows, macOS and Ubuntu jobs must all complete successfully. CI from earlier heads must not be reused.

`WAIVER_TAKEN=NO`
`SPEC_005_IMPLEMENTATION_COMPLETE=NO`
`SPEC_005_CLOSED_CANONICAL=NO`
