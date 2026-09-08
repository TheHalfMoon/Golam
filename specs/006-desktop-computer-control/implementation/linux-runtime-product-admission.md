# Spec 006 Linux Bounded Runtime Product Admission Evidence

Status: admitted product topology materialized; exact-head regular CI qualification pending.

## Authority and independent admission

The previously proposed design in which `golamd` directly depended on Tokio remains rejected and is not reclassified by this record.

A revised Source Foundry candidate structurally isolated Tokio behind the internal `golam-bounded-linux-async` facade. The exact reviewed candidate head was:

`a44836244670a03f56b8411aa4330e628b28eb84`

Exact-head qualification on that candidate succeeded:

- regular CI run `34244975893` — SUCCESS;
- Linux runtime closure Source Foundry run `34244970989` — SUCCESS;
- bounded facade structural Source Foundry run `34244970867` — SUCCESS.

Fresh independent Source Foundry review on PR #24, comment `5588918398`, admitted the revised topology with these controlling constraints:

`ADMIT_SPEC006_LINUX_BOUNDED_ASYNC_DRIVER=YES`
`TOKIO_DIRECT_VERSION=1.53.1`
`TOKIO_DIRECT_OWNER=GOLAM_BOUNDED_LINUX_ASYNC_FACADE_ONLY`
`GOLAMD_DIRECT_TOKIO_DEPENDENCY=NO`
`FACADE_PUBLIC_TOKIO_REEXPORT=NO`
`FACADE_ALLOWED_TOKIO_REFERENCES=NEW_CURRENT_THREAD_AND_TIMEOUT_ONLY`
`CONSUMER_TOKIO_NAME_RESOLUTION=COMPILE_FAIL`
`DIRECT_PRODUCT_TOKIO_API_SURFACE_STRUCTURALLY_RESTRICTED_TO_BOUNDED_DRIVER=YES`
`DIRECT_RUNTIME_PACKAGE_CLOSURE_CHANGED=NO`
`DIRECT_RUNTIME_EXTERNAL_LOCK_PACKAGE_CLOSURE_CHANGED=NO`
`PRODUCT_MANIFEST_MUTATION=AUTHORIZED_MINIMUM_FACADE_TOPOLOGY_ONLY`
`T006_016_T006_033_IMPLEMENTATION_CORRECTNESS=NOT_PREAPPROVED`
`FINAL_SPEC006_REVIEW=NOT_PREAPPROVED`
`WAIVER_TAKEN=NO`

## Controlled product materialization

Helper-only commit:

`23c02fd7850c608d2fe23f67cff8c1945eb3f3f2` — `ci(006): materialize admitted bounded runtime facade`

The helper workflow verified that its parent was exactly the admitted candidate head and that no product manifest or lockfile had changed before materialization.

Materialization workflow run:

`34255719996` — SUCCESS

The workflow used Rust/Cargo 1.98.0 to generate the product `Cargo.lock`; the lockfile was not hand-edited. It compiled the facade and `golamd`, then proved:

`BOUNDED_FACADE_EXTERNAL_LOCK_CLOSURE_CHANGED=NO`
`BOUNDED_FACADE_LOCAL_PACKAGE_ADDED=golam-bounded-linux-async`
`GOLAMD_LOCAL_DEPENDENCY_ADDED=golam-bounded-linux-async`
`TOKIO_DIRECT_OWNER=GOLAM_BOUNDED_LINUX_ASYNC_FACADE_ONLY`
`TOKIO_DIRECT_VERSION=1.53.1`
`GOLAMD_DIRECT_TOKIO_DEPENDENCY=NO`
`FACADE_PUBLIC_TOKIO_REEXPORT=NO`
`FACADE_ALLOWED_TOKIO_REFERENCES=NEW_CURRENT_THREAD_AND_TIMEOUT_ONLY`
`DIRECT_PRODUCT_TOKIO_API_SURFACE_STRUCTURALLY_RESTRICTED_TO_BOUNDED_DRIVER=YES`

The self-removing helper produced product commit:

`c0ab68c942c000e483bb9011df34adee1e5c56c6` — `build(006): admit bounded Linux async facade`

The product topology added only:

- workspace membership for `crates/golam-bounded-linux-async`;
- the internal facade crate with exact Tokio `1.53.1`, `default-features = false`, features `rt,time`;
- a Linux-target-only path dependency from `golamd` to the facade;
- the Cargo-generated lockfile delta.

The temporary product-lock helper removed itself in the same materialization commit.

## Non-executed bot-authored CI attempt

The product commit caused pull-request CI run `34255904010` to be created with conclusion `action_required`. GitHub returned no jobs for that run (`jobs=[]`). Its actor and triggering actor were both `github-actions[bot]`.

This record classifies run `34255904010` only as a platform-level non-execution. It is neither a CI success nor an implementation/test failure, and it must not satisfy any exact-head qualification gate.

This human-authored evidence commit intentionally creates a new forward-only head so normal pull-request CI can execute on a non-bot-authored SHA. It is not a rerun of `34255904010` and does not erase or reclassify that historical non-execution.

## Remaining gate

Before Phase D product implementation continues, the new exact head containing this evidence record and the admitted facade topology must complete regular Windows/macOS/Ubuntu CI successfully. Any failure must be preserved and repaired forward-only.

`PRODUCT_FACADE_ADMISSION=YES`
`PRODUCT_FACADE_MATERIALIZED=YES`
`PRODUCT_LOCK_CARGO_GENERATED=YES`
`EXACT_HEAD_REGULAR_CI=PENDING`
`T006_016_T006_033_IMPLEMENTATION_CORRECTNESS=NOT_PREAPPROVED`
`FINAL_SPEC006_REVIEW=NOT_PREAPPROVED`
`PR_READY=NO`
`MERGE_AUTHORIZED=NO`
`SPEC_006_IMPLEMENTATION_COMPLETE=NO`
`SPEC_006_CLOSED_CANONICAL=NO`
`WAIVER_TAKEN=NO`
