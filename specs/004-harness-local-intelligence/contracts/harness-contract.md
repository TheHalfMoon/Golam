# Contract: Harness Lifecycle

The harness is Golam-owned, model/provider independent and unprivileged. It consumes canonical session evidence and an immutable `ExecutionProfile`, dispatches a bounded request through a replaceable backend adapter, records accepted model events, and returns explicit terminal state.

## Core lifecycle

```text
PrepareRequest
  -> PersistAttemptPrepared
  -> Dispatch
  -> Stream/Normalize
  -> PersistAcceptedEvidence
  -> Complete | Cancel | Timeout | Fail
  -> optional bounded Retry as a NEW attempt
```

Every transition is attributable to `RequestSeriesId` + `RequestAttemptId`.

## Request preparation

A request MUST bind:
- canonical session/turn source references;
- exact context projection identity/digest;
- exact `ExecutionProfileId`;
- tool-schema digest when schemas are visible;
- input/output/time budgets;
- request digest.

Request construction MUST NOT expose protected secret plaintext or authority-bearing mutation tokens to a backend.

## Streaming

Adapters may emit bounded normalized events:
- text delta;
- reasoning delta;
- tool-call fragment/complete candidate;
- usage;
- stop;
- warning/error.

The harness MUST validate event order/size/type before accepting it into canonical evidence.

A prefix delivered to a user or accepted as model-visible history remains attributable evidence if later cancellation or failure occurs.

## Terminal states

Exactly one terminal state is recorded per attempt:
- `COMPLETED`;
- `CANCELLED`;
- `TIMED_OUT`;
- `FAILED_TRANSIENT`;
- `FAILED_DETERMINISTIC`;
- `FAILED_CONTEXT_OVERFLOW`.

Backend process exit and harness terminal classification are separate observations.

## Cancellation

Cancellation is explicit:
1. record cancellation request/state;
2. signal the backend;
3. bound/ignore disallowed late events;
4. observe acknowledgement/termination if available;
5. persist accepted partial evidence;
6. record terminal state.

Cancellation never rewrites prior evidence and never implies an external effect was cancelled.

`STOP_MODEL_REQUEST != CANCEL_EXTERNAL_EFFECT`

## Retry

Retry MUST:
- create a new `RequestAttemptId`;
- reference its parent attempt and reason;
- obey a bounded retry budget;
- preserve prior failed/cancelled evidence;
- rebind any changed context/profile identity;
- never blind-replay a protected effect.

Transient backend failure MAY be retryable. Deterministic validation/configuration failure is not automatically retryable. Context overflow MAY trigger bounded compaction/reprojection before a new attempt.

## Compaction

Compaction MUST:
- record an explicit attempt/transaction identity;
- bind exact source event/artifact references and source digest;
- preserve canonical source history;
- retain/inherit taint/provenance;
- record method/profile and output digest;
- validate output before activation;
- leave failed/incomplete attempts visible;
- never replace the independently durable Goal/non-negotiable constraint state.

`COMPACTION != CANONICAL_HISTORY_REWRITE`

## Tool calls

Backend output is normalized to `ToolCallCandidate` before any tool action.

Validation MUST reject:
- malformed or ambiguous framing;
- unknown tool/schema;
- invalid arguments;
- oversized payload;
- duplicate/replayed candidate identity;
- attempts to express capability, approval or protected mutation as model authority.

A validated candidate is still untrusted data and does not execute itself.

`VALID_TOOL_CALL_CANDIDATE != AUTHORIZED_EFFECT`

## Logging invariant

`MODEL_VISIBLE => LOGGED`, subject to Spec 003 secret-ingestion redaction/tombstone rules.

Raw backend diagnostics MAY be bounded/redacted and need not become model-visible, but any accepted content that can influence a later model request must be attributable to canonical evidence.

## Crash/restart

Canonical request/accepted-output/compaction evidence survives restart. Backend process handles, stream buffers, KV cache and warm residency do not become canonical merely because they can be restored.

After restart, incomplete attempts are recovered/reconciled as explicit interrupted/failed state; they are never silently upgraded to success.

## Authority boundary

The harness/backend MUST NOT:
- mint or widen capability leases;
- satisfy step-up approval from model text;
- bypass strict-local egress;
- read generic secret plaintext outside an authorized brokered path;
- mutate protected policy/identity/lease/approval/secret/egress/sandbox state;
- commit an external effect without the existing Effect Gate;
- mark a Task verified merely because generation completed.
