from pathlib import Path

HEAD = "758476315cebf48c31a8c43f84b5d9859f8e3342"
TREE = "57d4930af50f8d6c259f87332884de4f641a8261"
RUN = "33250188127"
FOCUSED = "33250097386"
root = Path("specs/003-identity-policy-secrets-sandbox")

candidate = root / "implementation/t003-076-qualification-candidate.md"
text = candidate.read_text()
text = text.replace("**Status**: QUALIFICATION_CANDIDATE — NOT YET PASS", "**Status**: PASS", 1)
text = text.replace(
    "Official Windows/macOS/Ubuntu repository CI on this candidate remains required before `T003_076=PASS` and Phase H completion may be recorded.",
    f"Official CI #606 / run `{RUN}` completed SUCCESS on Windows, macOS and Ubuntu at exact human-authored head `{HEAD}`, tree `{TREE}`. Phase H may therefore close and T003-080 becomes the next canonical task.",
    1,
)
old = """T003_076=NOT_YET_PASS
T003_076_FOCUSED_RUN=33250097386
T003_076_IMPLEMENTATION_HEAD=60a3c2119e8c7101e0810fb7efb755dd9c094344
T003_076_IMPLEMENTATION_TREE=d4e70ade4cc6162178146330ffdbe0231e604f8a"""
new = f"""T003_076=PASS
T003_076_QUALIFIED_HEAD={HEAD}
T003_076_QUALIFIED_TREE={TREE}
T003_076_CI_RUN={RUN}
T003_076_FOCUSED_RUN={FOCUSED}
T003_076_IMPLEMENTATION_HEAD=60a3c2119e8c7101e0810fb7efb755dd9c094344
T003_076_IMPLEMENTATION_TREE=d4e70ade4cc6162178146330ffdbe0231e604f8a"""
if old not in text:
    raise SystemExit("candidate marker block missing")
text = text.replace(old, new, 1).replace("OFFICIAL_THREE_PLATFORM_CI_REQUIRED=YES\nNEXT_TASK=T003-076", "OFFICIAL_THREE_PLATFORM_CI_REQUIRED=SATISFIED\nNEXT_TASK=T003-080", 1)
candidate.write_text(text)

tasks = root / "tasks.md"
text = tasks.read_text()
old_line = "- [ ] **T003-076** Add escape/inheritance/forbidden-FS/network/spawn/resource/unsupported-platform tests."
new_line = f"- [x] **T003-076** Add escape/inheritance/forbidden-FS/network/spawn/resource/unsupported-platform tests. Exact-head qualification: CI #606 (`{RUN}`) SUCCESS at `{HEAD}`, tree `{TREE}`, on Windows/macOS/Ubuntu; focused adversarial run `{FOCUSED}` SUCCESS. Empty allowlists now require explicit executor enforcement and resource controls fail closed. Evidence: `implementation/t003-076-qualification-candidate.md`."
if text.count(old_line) != 1:
    raise SystemExit(f"T003-076 task anchor count {text.count(old_line)}")
text = text.replace(old_line, new_line, 1)
if text.count("PHASE_H_ACTIVE=YES") != 1:
    raise SystemExit("tasks PHASE_H_ACTIVE anchor missing")
text = text.replace("PHASE_H_ACTIVE=YES", "PHASE_H_COMPLETE=YES\nPHASE_I_ACTIVE=YES", 1)
if text.count("NEXT_TASK=T003-076") != 1:
    raise SystemExit("tasks NEXT_TASK anchor missing")
text = text.replace(
    "NEXT_TASK=T003-076",
    f"T003_076=PASS\nT003_076_QUALIFIED_HEAD={HEAD}\nT003_076_QUALIFIED_TREE={TREE}\nT003_076_CI_RUN={RUN}\nT003_076_FOCUSED_RUN={FOCUSED}\nNEXT_TASK=T003-080",
    1,
)
tasks.write_text(text)

plan = root / "implementation/current-execution-plan.md"
text = plan.read_text()
if text.count("**Current task**: `T003-076`") != 1:
    raise SystemExit("plan current task anchor missing")
text = text.replace("**Current task**: `T003-076`", "**Current task**: `T003-080`", 1)
old_section = """### T003-076 — ACTIVE

Add consolidated adversarial sandbox coverage for escape/inheritance, forbidden filesystem/network/device/IPC/handle/spawn rights, bounded resources and unsupported-platform denial. Reuse the T003-074 bounded Linux x86_64 test harness without widening it into a product executor or claiming unsupported platform parity."""
new_section = f"""### T003-076 — COMPLETE

Qualified at exact human-authored implementation head `{HEAD}`, tree `{TREE}`, by CI #606 / run `{RUN}`, SUCCESS on Windows/macOS/Ubuntu. Focused adversarial run `{FOCUSED}` also completed SUCCESS.

Evidence: `implementation/t003-076-qualification-candidate.md`.

The task repaired a material deny-all capability gap: empty filesystem/device/IPC/inherited-handle allowlists now still require explicit executor enforcement controls. Adversarial coverage also proves forbidden rights widening, strict-local network dominance, spawn denial, independent resource-control support, unsupported-platform fail-closed behavior, and preservation of the bounded T003-074 OS containment harness. Production remains `native:unqualified`; no universal native containment claim was added.

### T003-080 — ACTIVE

Replace `BootstrapPolicy` in the normal authority-serving path with Cedar-backed active-policy evaluation while preserving the stable `Authorize(principal, action, resource, context)` contract and keeping initial/recovery bootstrap administration narrowly bounded to the local owner. Runtime evaluator errors, malformed stored bundle/source/context and missing active policy must fail closed for ordinary product effects."""
if text.count(old_section) != 1:
    raise SystemExit("plan T003-076 section missing")
text = text.replace(old_section, new_section, 1)
text = text.replace("- Remaining Phase H: T003-076.\n- Phase I: T003-080..T003-084.", "- Phase H: COMPLETE through T003-076.\n- Phase I: ACTIVE at T003-080; remaining T003-080..T003-084.", 1)
if text.count("PHASE_H_ACTIVE=YES") != 1:
    raise SystemExit("plan PHASE_H_ACTIVE anchor missing")
text = text.replace("PHASE_H_ACTIVE=YES", "PHASE_H_COMPLETE=YES\nPHASE_I_ACTIVE=YES", 1)
if text.count("NEXT_TASK=T003-076") != 1:
    raise SystemExit("plan NEXT_TASK anchor missing")
text = text.replace(
    "NEXT_TASK=T003-076",
    f"T003_076=PASS\nT003_076_QUALIFIED_HEAD={HEAD}\nT003_076_QUALIFIED_TREE={TREE}\nT003_076_CI_RUN={RUN}\nT003_076_FOCUSED_RUN={FOCUSED}\nNEXT_TASK=T003-080",
    1,
)
plan.write_text(text)

assert "PHASE_H_COMPLETE=YES" in tasks.read_text()
assert "NEXT_TASK=T003-080" in tasks.read_text()
assert "### T003-080 — ACTIVE" in plan.read_text()
