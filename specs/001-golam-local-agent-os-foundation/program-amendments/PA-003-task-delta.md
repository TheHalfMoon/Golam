# PA-003 Task Delta — Product Spine and Core Alpha

**Status**: PROPOSED_FOR_REVIEW  
**Parent amendment**: `PA-003-product-spine-golden-loop.md`  
**Purpose**: make the PA-003 execution implications explicit without rewriting or renumbering the frozen historical Spec 001 task graph in place.

These are **additional future owning-spec requirements**, not authorization to execute them now.

---

## Spec 004 additions — Runtime/product spine

Before Spec 004 closes, its bounded Spec Kit package MUST include tasks that prove:

- **PA003-004-A — Durable Task identity**: define `Task`, `Session`, `Run`, `Worker`, `TaskContract`, and Goal Ledger relationships with persistence/restart tests.
- **PA003-004-B — In-flight control**: implement/verify Pause, Stop, Steer, AddConstraint, Inspect, and Resume semantics at the canonical harness/runtime layer. Desktop/mobile-native UI is not required yet.
- **PA003-004-C — Capability truth**: publish machine-readable model/harness/provider capability descriptors and conformance evidence; unsupported features fail honestly.
- **PA003-004-D — Locality projection**: expose strict-local/local-preferred/cloud-allowed style user posture without hidden provider fallback.
- **PA003-004-E — Run failure taxonomy**: preserve blocked-policy, blocked-environment, verification failure, unknown-effect, budget exhaustion, interruption, and crash/recovery distinctions.

**Spec 004 product-spine exit gate**: a Task survives model switch and process restart; a user can inspect and steer a running task; resume revalidates live/protected state; provider claims are conformance-backed.

---

## Spec 005 additions — Golam Core Alpha

Before Spec 005 closes, its bounded Spec Kit package MUST include tasks that prove:

- **PA003-005-A — Trust Receipt**: project task/run result evidence, changed artifacts, external effects, egress destinations/classes, provider/tool use, approvals, unknowns, and learning candidates from canonical records.
- **PA003-005-B — UserModel baseline**: separate compact governed stable user preferences from general memory; retain provenance/supersession; no silent sensitive profiling.
- **PA003-005-C — Migration staging**: safely detect/import supported portable Markdown memory from selected external assistants into quarantined provenance-preserving staging; no credentials, protected authority, or silent auto-promotion.
- **PA003-005-D — Export/portability**: export user-owned Markdown and stable machine-readable task/evidence/receipt records where the owning spec freezes a format.
- **PA003-005-E — CLI/TUI Golden Loop UX**: expose current task/goal, evidence, running state, pause/stop/steer/inspect/resume, approvals, unresolved blockers, and final Trust Receipt without requiring Desktop.
- **PA003-005-F — Core Alpha repository scenario**: inspect/edit/verify a real local repository through governed tools and report exact evidence.
- **PA003-005-G — Core Alpha research scenario**: combine permitted local/web evidence with provenance/taint and produce an attributable result/artifact.
- **PA003-005-H — Core Alpha filesystem/document scenario**: transform local artifacts without destructive source mutation unless authorized; produce exact receipt.
- **PA003-005-I — Core Alpha cross-session memory scenario**: approved memory persists through restart/new session and loses to fresher authoritative live state when conflicting.
- **PA003-005-J — Core Alpha interrupt/recovery scenario**: pause/steer/stop/resume/restart a non-trivial task without losing goal/evidence or duplicating protected effects.
- **PA003-005-K — Core Alpha strict-local scenario**: useful end-to-end task with externally observed zero unauthorized external egress and no hidden remote model/vector/eval/telemetry fallback.
- **PA003-005-L — Core Alpha report**: report baseline task quality, false-success/verification failure, time to first useful action, interruption/recovery, approval repetition, tokens/resources, egress, unresolved outcomes and memory correctness without inventing target thresholds before measurement.

**Mandatory product checkpoint**: Spec 006 MUST NOT become the next release blocker until the owning program review records whether **Golam Core Alpha** passed or failed on the exact Spec 005 close head.

A failed Core Alpha gate does not authorize skipping required architecture/security work; it requires fixing the Golden Loop before expanding product breadth unless the founder explicitly accepts a documented exception.

---

## Spec 006 additions — Desktop projection

- **PA003-006-A**: Desktop projects the same Task/Run/Goal/Trust state; it does not maintain a separate agent truth.
- **PA003-006-B**: expose Inspect/Pause/Steer/Stop/TakeOver/Resume in the desktop experience.
- **PA003-006-C**: Trust Center baseline shows devices, active leases/approvals, egress/provider posture, sandbox posture, memory/learning changes, scheduled work and recent Trust Receipts without exposing secret plaintext.

---

## Spec 007 additions — Everywhere continuity

- **PA003-007-A**: Native Mobile continues an existing Task/Run rather than cloning a conversation-only state.
- **PA003-007-B**: channel messages can navigate/create/steer tasks only through stable binding and normal authority rules; channel identity never equals Task identity.
- **PA003-007-C**: mobile/channel receipt views preserve what changed/data-egress/approval/unknown state appropriate to the surface.

---

## Spec 008 additions — Proactivity without interruption abuse

- **PA003-008-A — InitiativePolicy**: define when Golam may discover/propose/start bounded proactive work.
- **PA003-008-B — AttentionBudget**: define notify/quiet/batch/dedupe/urgency/surface-routing controls independently from effect authority.
- **PA003-008-C — Proactive receipt**: unattended work produces the same causal/effect/evidence/receipt trail as attended work.
- **PA003-008-D — User feedback**: mute/defer/reduce-priority/never-notify-for-class feedback changes future attention behavior without silently altering unrelated capability authority.

---

## Spec 009 additions — Superset posture

- **PA003-009-A**: public parity scenarios explicitly score Golam Trust Receipts, locality, in-flight control, governed memory and recovery as candidate `VERIFIED_SUPERSET` dimensions where supported by evidence.
- **PA003-009-B**: no parity claim may be closed by bypassing Golam security/durability/product-spine invariants.

---

## Spec 010 additions — Golden Loop release qualification

- **PA003-010-A**: evaluate process and outcome separately for long-horizon tasks.
- **PA003-010-B**: measure false-success rate and fabricated verification/shortcut behavior.
- **PA003-010-C**: add representative hybrid CLI + GUI/browser/computer tasks once Spec 006 capabilities exist.
- **PA003-010-D**: score controllable agent failures separately from external/environment blockers.
- **PA003-010-E**: test user steering/takeover/recovery in trajectory, not only terminal artifact state.
- **PA003-010-F**: verify Trust Receipt completeness against canonical event/effect/evidence state.
- **PA003-010-G**: verify capability truth matrices against real claimed provider/platform behavior.
- **PA003-010-H**: produce exact-head product ladder evidence for Core Alpha, Desktop, Everywhere, Persistent Team, Parity/Superset and release qualification claims.

---

## Explicit non-blockers for Core Alpha

Unless the owning Spec Kit package demonstrates measured necessity, Core Alpha does not wait for:

- native Desktop;
- iOS/Android;
- WhatsApp/WeChat/Telegram/Slack/Discord/Matrix breadth;
- multi-agent groups/swarms;
- custom relay infrastructure;
- mandatory Qdrant/graph database;
- hosted observability/evaluation;
- A2A federation;
- image/video generation;
- marketplace/discovery ecosystem;
- generalized autonomous self-modification.
