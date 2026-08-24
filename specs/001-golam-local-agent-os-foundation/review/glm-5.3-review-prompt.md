# GLM 5.3 External Architecture Review Prompt

Use this prompt with an actual GLM-5.3 session. Do not substitute another model while claiming GLM review.

---

You are GLM-5.3 acting as an independent principal architect and adversarial reviewer for Golam.

Repository: `TheHalfMoon/Golam`

Goal: review Spec Kit package `specs/001-golam-local-agent-os-foundation/` before implementation tasks are generated.

Golam is intended to become a Rust-first, local-first autonomous Agent OS for the user's computer: Desktop + CLI/TUI, local models, durable memory, coding/research/browser/general assistant work, full authorized computer control, and secure remote access through GolamConnect. It targets functional parity with publicly documented Grok Bot features/skills through independent implementation.

IMPORTANT REVIEW RULES:
- Do not implement code.
- Do not redesign just to be different.
- Treat `.specify/memory/constitution.md` as binding.
- Be skeptical: look for hidden cloud dependencies, confused-deputy paths, capability bypasses, replay/idempotency flaws, secret leakage, unsafe remote-control semantics, impossible cross-platform assumptions, over-large trusted computing base, premature abstractions, licensing contamination, benchmark gaming, and roadmap sequencing problems.
- Distinguish a true architecture blocker from a preference.
- Preserve Rust/local-first unless you prove a specific requirement cannot be met under those constraints.

READ IN THIS ORDER:
1. `.specify/memory/constitution.md`
2. `specs/001-golam-local-agent-os-foundation/spec.md`
3. `specs/001-golam-local-agent-os-foundation/clarification-closeout.md`
4. `specs/001-golam-local-agent-os-foundation/research.md`
5. `specs/001-golam-local-agent-os-foundation/plan.md`
6. `specs/001-golam-local-agent-os-foundation/data-model.md`
7. every file under `specs/001-golam-local-agent-os-foundation/contracts/`
8. `specs/001-golam-local-agent-os-foundation/quickstart.md`
9. `specs/001-golam-local-agent-os-foundation/checklists/implementation-readiness.md`

REVIEW THESE AREAS EXPLICITLY:
A. Is the privileged Rust kernel boundary minimal enough and non-bypassable?
B. Are Session/Event Ledger + Goal Ledger semantics sufficient for long-horizon crash recovery and context resets?
C. Is the Effect transaction model safe across crash windows, network ambiguity, retries, at-most-once operations and irreversible actions?
D. Can identity/capability leases/Cedar-style policy avoid confused deputy and self-authority expansion across workers, skills, MCP, channels and Connect devices?
E. Is the secret-broker and taint/information-flow design sufficient against prompt injection/tool poisoning/exfiltration?
F. Is `ExecutionProfile` the correct abstraction for local model + backend + quantization + harness + context/cache strategy? What is missing?
G. Does local inference architecture preserve strict-local operation across Windows/macOS/Linux without making a single backend a hard lock-in?
H. Is the Context Compiler layered correctly for general + coding work without making graphs/vector DB mandatory?
I. Is Markdown canonical memory + SQLite operational state a durable and scalable boundary? Identify consistency/supersession problems.
J. Is the Agent Skills/MCP/ACP strategy interoperable without allowing third-party packages to become authority?
K. Is semantic-first computer control realistic cross-platform? Identify platform-specific blockers and secure-desktop limitations.
L. Is GolamConnect's Iroh/P2P + relay + remote-control design safe? Review pairing, replay protection, short-lived grants, reconnect, per-message authorization, human takeover, emergency stop, file/clipboard transfer and third-party messaging bridges.
M. Does the plan cover all major publicly documented Grok Bot capability domains while staying independent from proprietary source/assets?
N. Is the follow-on spec sequence ordered correctly? Identify dependencies that must move earlier/later.
O. Which proposed donors should be direct dependencies, adapters, selective ports, or reference-only?
P. What benchmarks could create false confidence? Propose missing Golam-native qualification scenarios.
Q. What are the top ten ways this architecture could fail in production despite all current tests passing?

REQUIRED OUTPUT FORMAT:

# GLM-5.3 Review

## Final Recommendation
One of:
- APPROVE_FOR_TASK_GENERATION
- APPROVE_WITH_MANDATORY_CHANGES
- BLOCK

## BLOCKER Findings
For each: ID, affected artifact/section, failure scenario, evidence/reasoning, required correction.

## MAJOR Findings
Same fields.

## MINOR Findings
Same fields.

## KEEP — Decisions That Should Not Be Reopened
List strong decisions that should remain frozen.

## Missing Requirements / Contracts
List exact additions needed.

## Donor Strategy Corrections
Only evidence-backed changes.

## Roadmap/Spec Sequencing Corrections
Provide exact proposed order if changed.

## Verification Gaps
Concrete tests/benchmarks/attack scenarios.

## Final Gate Checklist
State whether each blocker is resolvable without violating the constitution.

Do not output implementation code. Do not claim a source was inspected unless you actually inspected it.
