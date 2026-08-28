# Contract: Trajectory Behavior, Evaluation, and Experiment Evidence

**Authority note**: This additive Spec 001 program contract is introduced by `program-amendments/PA-002-memory-retrieval-learning-evals.md`. It defines future GolamBench/evaluation requirements and does not authorize implementation in the active Spec 003 scope.

## 1. Outcome success is not sufficient

Golam evaluation MUST distinguish:

- final outcome/artifact quality; and
- trajectory/process behavior.

A run MAY fail evaluation even when its final output looks correct if the trajectory violated a binding behavior, safety, authority, provenance or verification requirement.

Examples include fabricated PASS claims, stale-evidence use, unauthorized actions, ignored denials, duplicate effects, hidden network fallback, secret exposure or silent memory mutation.

## 2. BehaviorSpec

A `BehaviorSpec` defines expected recurring conduct across a trajectory.

Golam SHOULD support compatibility with the open `BEHAVIOR.md` convention where practical, while preserving Golam-owned metadata/integrity records.

A behavior spec may describe:

- intent;
- required evidence;
- decision discipline;
- execution behavior;
- verification;
- recovery;
- failure modes;
- applicability/scope.

A behavior specification is instruction/evaluation data. It is NOT authority.

`BEHAVIOR_SPEC != POLICY_OR_CAPABILITY`

## 3. Malicious behavior text cannot grant authority

A BehaviorSpec, benchmark case, scorer prompt, reference answer, evaluator plugin or judge output MUST NOT:

- mint capabilities;
- approve protected effects;
- clear taint;
- relax hard denials;
- alter egress policy;
- change protected authority state.

All evaluation content is treated according to its provenance/trust class.

## 4. TrajectoryTrace

Trajectory evaluation SHOULD derive from canonical or integrity-bound runtime evidence rather than model self-report.

A trace can reference:

- session/goal events;
- model requests/responses;
- tool calls/results;
- retrieval evidence;
- worker/graph events;
- approvals/denials;
- EffectIntents and effect records;
- network/egress decisions;
- memory/learning proposals;
- verification/test records;
- receipts;
- interruptions/resumes;
- timestamps/resource usage.

Derived trace views are rebuildable projections and do not replace canonical records.

## 5. EvalCase reproducibility

An `EvalCase` used for meaningful regression/release claims MUST identify enough state to reproduce the evaluation, including as applicable:

- case ID/version;
- dataset/corpus version/hash;
- initial environment or fixture digest;
- task/input;
- expected/reference evidence;
- allowed tools/capabilities;
- model/harness/ExecutionProfile constraints;
- time/token/cost/resource budgets;
- scorer/BehaviorSpec versions;
- environment/platform state;
- random seeds where meaningful.

## 6. Scorer hierarchy

Where the domain permits, prefer evidence in this order:

1. deterministic invariants/assertions;
2. exact structured comparison;
3. executable/environment verification;
4. statistical/heuristic scoring;
5. model-based judging as supplementary evidence.

A model judge MUST NOT replace a deterministic security invariant that can be checked directly.

## 7. LLM judge provenance

Every model-graded result used in a comparison or release claim MUST record at least:

- evaluator/scorer ID and version;
- judge provider/locality;
- model ID/revision where available;
- prompt/rubric/template hash;
- relevant sampling settings/seed where meaningful;
- input/output/reference refs;
- raw score/classification;
- rationale or raw judge artifact ref where retained;
- token/cost/latency metadata where relevant;
- error/retry state;
- repeated-run/variance information when the judgment is materially stochastic.

`LLM_JUDGE_SCORE != RELEASE_AUTHORITY`

## 8. Candidate agent cannot grade itself invisibly

A candidate agent/model MAY participate in self-critique, but self-critique is not independent evaluation.

Release/adoption decisions MUST distinguish:

- candidate self-evaluation;
- deterministic environment evidence;
- independent evaluator/judge evidence.

An ExperimentProgram MUST keep evaluator/guardrail artifacts immutable to the candidate unless a separately reviewed meta-experiment explicitly says otherwise.

## 9. Behavior result vocabulary

Behavior evaluations SHOULD support at least:

- PASS / true;
- FAIL / false;
- NOT_APPLICABLE;
- INCONCLUSIVE / insufficient evidence.

Inconclusive evidence MUST NOT be silently converted to PASS.

## 10. Required long-horizon behaviors

GolamBench SHALL eventually carry explicit behaviors for at least:

- maintain the durable goal and acceptance criteria;
- re-read authoritative/live state after relevant mutations;
- do not use stale memory as current truth;
- gather sufficient evidence before acting/claiming;
- verify consequential work before claiming completion;
- never fabricate tool/test/CI/merge/review evidence;
- respect scope and authority boundaries;
- respect denials and approval freshness;
- avoid premature stopping when authorized work remains;
- recover from tool/model failures without inventing results;
- preserve source/citation provenance;
- handle user interrupt/redirect/stop promptly;
- prevent duplicate or blind-retried irreversible effects;
- preserve secret/taint boundaries;
- abstain/replan when evidence is insufficient.

## 11. Conformance suites for replaceable providers

A replaceable provider surface SHOULD have a common conformance suite before Golam claims support.

Candidate conformance families include:

- model adapters;
- checkpoint providers;
- retrieval/index providers;
- tool providers;
- channel adapters;
- sandbox backends;
- scorer/evaluator providers.

A conformance suite distinguishes required base capabilities from optional advertised capabilities. Unsupported optional capabilities must be explicit rather than silently emulated incorrectly.

## 12. ContextRouteBench

Golam SHALL benchmark evidence-access routes under equal conditions rather than assuming one approach wins universally.

A common corpus/question set SHOULD compare, where applicable:

- direct filesystem tools;
- lexical/ripgrep/FTS;
- structured SQLite/SQL;
- vector retrieval;
- hybrid retrieval;
- graph/entity retrieval;
- Context Compiler dynamic routing.

Task categories include at least exact lookup, aggregation, textual reasoning, cross-entity, multi-hop, temporal, negation, semantic, code/reference navigation and stale/current-state conflict.

Report quality plus tokens, tool calls, latency, errors and resource/provider cost.

## 13. Memory evaluation

Memory benchmarks SHOULD include public suites such as LoCoMo, LongMemEval and BEAM where dataset/license/tooling terms permit, plus Golam-specific adversarial cases.

Golam-specific memory evaluation MUST cover:

- contradiction/supersession;
- temporal validity;
- current live state versus stale memory;
- authority/provenance ranking;
- project/user/worker/channel scope isolation;
- user hand-edit reconciliation;
- FORGET/REDACT and derived-index rebuild;
- secret-derived candidate rejection;
- prompt injection/memory poisoning;
- index corruption/rebuild;
- crash during memory promotion/index update.

Memory claims MUST report retrieval/context token budget and latency with accuracy/success.

## 14. ExperimentRun evidence

An autonomous/manual optimization run records at least:

- ExperimentProgram/version;
- parent/baseline;
- candidate patch/config digest;
- exact code head/environment digest;
- model/harness profile;
- dataset/evaluator versions;
- start/end/status;
- primary metrics;
- guardrail/behavior results;
- tokens/cost/resources;
- logs/artifact refs;
- keep/discard/crash/timeout/rejected decision;
- adoption decision separately.

Experiment evidence is append-oriented; failed/discarded experiments remain useful history and SHOULD not be silently erased from the research ledger.

## 15. Baseline-first and equal-budget comparison

Before claiming improvement, establish a comparable baseline using the same relevant:

- task corpus;
- model/harness envelope;
- time/token budget;
- hardware/environment;
- evaluator versions;
- tool/permission surface.

Differences must be disclosed. A faster/larger/more expensive judge or wider retrieval budget must not be presented as a pure algorithm improvement.

## 16. Multi-objective regression gate

A candidate can regress even if its headline task score improves.

Applicable guardrails include:

- security/invariant failures;
- behavior compliance;
- token/cost increase;
- latency;
- memory/VRAM/RAM/disk;
- network/privacy change;
- complexity/maintainability;
- platform regressions.

Hard safety/integrity failures dominate optimization gains.

## 17. Reference-answer governance

Reference answers/evidence used for release-quality evaluation SHOULD be independently validated and versioned.

When a reference is uncertain or disputed, record confidence/provenance and do not disguise judge consensus as deterministic ground truth.

Changing a reference answer after seeing candidate output is an evaluation mutation and must be attributable/versioned.

## 18. Citation/evidence evaluation

Research/factual evaluation SHOULD distinguish:

- answer correctness;
- source support/faithfulness;
- source authority;
- source freshness;
- citation-location correctness;
- unsupported claims.

A valid-looking URL alone is not evidence that the cited source supports the claim.

## 19. Security evaluation isolation

Evaluation infrastructure handling hostile prompts/plugins/files MUST run with the minimum required capability and must not become a privileged bypass around normal Golam security.

A benchmark that intentionally tests dangerous behavior may use synthetic/disposable fixtures, but it cannot require real secrets or irreversible host damage merely to prove a guardrail.

## 20. Offline/local evaluation

Core GolamBench functionality MUST be runnable locally without mandatory Braintrust/LangSmith or another hosted telemetry/eval service.

Hosted logging/eval integrations MAY be optional explicit adapters when permitted by privacy/egress policy.

Strict-local evaluation MUST not silently send traces, prompts, artifacts or metrics externally.

## 21. Exact-head claim binding

A release/parity/security/performance claim MUST bind to the exact code/config/dataset/evaluator state that produced it.

Any mutation that can affect the result invalidates earlier exact-head qualification until the required gates are rerun.

## 22. Required adversarial tests

The owning eval/bench spec MUST cover at least:

- candidate prompt injection into judge/evaluator;
- malicious BehaviorSpec attempting authority expansion;
- candidate modifying evaluator/reference files;
- benchmark memorization/overfitting where detectable;
- judge inconsistency/variance;
- evaluator network leakage in strict-local mode;
- fabricated test/trace result from the candidate agent;
- stale evaluation artifact reused after a code mutation;
- final-success/trajectory-failure cases;
- scorer exception/timeout/inconclusive handling;
- poisoned reference evidence.

## 23. Release claim rule

No evaluator dashboard, aggregate score, model judge or benchmark leaderboard alone is sufficient evidence for a Golam release/security/parity claim. Claims require the complete applicable exact-head evidence chain defined by the owning spec and constitution.