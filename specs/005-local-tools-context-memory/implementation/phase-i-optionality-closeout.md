# Spec 005 Phase I Optionality Closeout

Status: QUALIFIED

## Decision

The bounded Spec 005 product slice does not require an HTTP/document-fetch transport, Tree-sitter/LSP structural indexing, or a dense/vector retrieval dependency.

- `T005-100=NOT_APPLICABLE_NOT_REQUIRED_BY_SELECTED_OUTCOME`
- `T005-101=PASS_STRICT_LOCAL_DOMINATES`
- `T005-102=PASS_SCOPE_EXCLUDED`
- `T005-103=PASS_NO_MATERIAL_L0_STRUCTURAL_GAP`
- `L1=DEFER_NOT_NEEDED`
- `DENSE_VECTOR_INDEX=DEFER_NOT_NEEDED`
- `T005-106=PASS_POLICY_GATE_ONLY_NO_NETWORK_TRANSPORT_SELECTED`
- `WAIVER_TAKEN=NO`

These are measured deferrals, not implicit admissions and not claims that the deferred surfaces are qualified.

## Exact evidence

Initial representative L0 optionality qualification:

- head: `6178bab7c542bb7260416381990df65e6c4c0dae`
- workflow: `spec005-i-optionality`
- run: `34040833971`
- result: `SUCCESS`

The run qualified bounded context compilation, local read/walk/in-process literal search, Git read/observe/status, the remote-MCP authority gate, and the complete `golamd` library test surface. It also verified that the selected dependency closure contains no exact `tree-sitter`, `qdrant`, `qdrant-client`, `reqwest`, or `hyper` package and that Spec 006 UI/control dependency candidates are absent from Spec 005 manifests.

Credential/redirect/proxy drift qualification after the permanent adversarial test was added:

- final Phase I qualification head: `772eb538f0d63eeb7721ebacf45e585900b8c202`
- workflow: `spec005-i-optionality`
- run: `34041594575`
- result: `SUCCESS`
- permanent test: `crates/golamd/tests/phase_i_network_optionality.rs`

The exact run proved that:

- strict-local denial occurs before any future remote transport;
- unencrypted transport, unauthenticated endpoint state, and missing egress authority fail closed;
- endpoint identity, network policy, and secret policy drift fail closed;
- credential-scope, redirect-policy, proxy-policy, and egress-authority drift invalidate a prepared remote dispatch;
- `remote_network_emission_implemented()` remains false;
- the permanent Phase I network-optionality test passed `2 passed; 0 failed`;
- the complete `golamd --lib` sweep passed `73 passed; 0 failed` on that qualification head.

A prior qualification attempt, run `34041472488`, stopped at `cargo fmt --check` before semantic execution. The only repair was the formatter-prescribed source layout change; run `34041594575` is the superseding evidence.

## Boundary interpretation

The current L0 context compiler already routes bounded user-selected artifacts, file reads, in-process search, Git evidence, canonical evidence, and managed memory. Ranking does not raise authority or clear taint, stale evidence produces bounded replan behavior, and the resulting context capsule is deterministic.

The current remote-MCP surface is an authority/freshness gate only. It does not implement HTTP or emit remote traffic. Adding a network client solely to make the optional Phase I task non-empty would widen the attack surface and contradict the measured minimality decision.

Tree-sitter/LSP and dense/vector retrieval remain eligible for a later bounded spec only if representative evidence demonstrates a material gap and the exact dependency/source closure is independently qualified before admission.

## Phase I closeout

`PHASE_I=QUALIFIED`
`NEXT_TASK=T005-110`
