from pathlib import Path

HEAD = "6d3bf98c51ba2c44d187ff07d24bd804a3026bdd"
TREE = "ae2463fa36240fc801accdfeb5b39adf7fccde10"
RUN = "33249678974"
root = Path("specs/003-identity-policy-secrets-sandbox")

tasks = root / "tasks.md"
text = tasks.read_text()
old = "- [ ] **T003-075** If Wasmtime is later admitted by reopened T003-005, implement the bounded WASM/WASI profile through the qualified executor; otherwise keep it deferred with explicit evidence."
new = f"- [x] **T003-075** If Wasmtime is later admitted by reopened T003-005, implement the bounded WASM/WASI profile through the qualified executor; otherwise keep it deferred with explicit evidence. Deferred/not-admitted qualification: CI #597 (`{RUN}`) SUCCESS at `{HEAD}`, tree `{TREE}`, on Windows/macOS/Ubuntu. `WASMTIME_DISPOSITION=NOT_ADMITTED_NOT_NEEDED`; no runtime authority or dependency surface changed. Evidence: `implementation/t003-075-wasm-profile-disposition.md`."
if text.count(old) != 1:
    raise SystemExit(f"T003-075 task anchor count {text.count(old)}")
text = text.replace(old, new, 1)
if text.count("NEXT_TASK=T003-075") != 1:
    raise SystemExit(f"tasks NEXT_TASK count {text.count('NEXT_TASK=T003-075')}")
text = text.replace(
    "NEXT_TASK=T003-075",
    f"T003_075=PASS\nT003_075_QUALIFIED_HEAD={HEAD}\nT003_075_QUALIFIED_TREE={TREE}\nT003_075_CI_RUN={RUN}\nNEXT_TASK=T003-076",
    1,
)
tasks.write_text(text)

plan = root / "implementation/current-execution-plan.md"
text = plan.read_text()
if text.count("**Current task**: `T003-075`") != 1:
    raise SystemExit("current task T003-075 anchor missing")
text = text.replace("**Current task**: `T003-075`", "**Current task**: `T003-076`", 1)
old_section = """### T003-075 — ACTIVE

Re-read the canonical T003-005 Wasmtime disposition. Because Wasmtime remains `NOT_ADMITTED_NOT_NEEDED` and no bounded Phase H requirement has reopened dependency qualification, complete T003-075 as explicit deferred/not-admitted evidence without adding Wasmtime."""
new_section = f"""### T003-075 — COMPLETE

Qualified at exact human-authored implementation head `{HEAD}`, tree `{TREE}`, by CI #597 / run `{RUN}`, SUCCESS on Windows/macOS/Ubuntu.

Evidence: `implementation/t003-075-wasm-profile-disposition.md`.

T003-005 was not reopened. `WASMTIME_DISPOSITION=NOT_ADMITTED_NOT_NEEDED` remains authoritative; no Wasmtime/WASI executor, runtime/JIT/hostcall surface or dependency was admitted, and runtime authority did not change.

### T003-076 — ACTIVE

Add consolidated adversarial sandbox coverage for escape/inheritance, forbidden filesystem/network/device/IPC/handle/spawn rights, bounded resources and unsupported-platform denial. Reuse the T003-074 bounded Linux x86_64 test harness without widening it into a product executor or claiming unsupported platform parity."""
if text.count(old_section) != 1:
    raise SystemExit(f"T003-075 plan section count {text.count(old_section)}")
text = text.replace(old_section, new_section, 1)
if text.count("Remaining Phase H: T003-075..T003-076.") == 1:
    text = text.replace("Remaining Phase H: T003-075..T003-076.", "Remaining Phase H: T003-076.", 1)
elif text.count("Remaining Phase H: T003-076.") != 1:
    raise SystemExit("remaining Phase H anchor missing")
if text.count("NEXT_TASK=T003-075") != 1:
    raise SystemExit(f"plan NEXT_TASK count {text.count('NEXT_TASK=T003-075')}")
text = text.replace(
    "NEXT_TASK=T003-075",
    f"T003_075=PASS\nT003_075_QUALIFIED_HEAD={HEAD}\nT003_075_QUALIFIED_TREE={TREE}\nT003_075_CI_RUN={RUN}\nNEXT_TASK=T003-076",
    1,
)
plan.write_text(text)

assert "T003_075=PASS" in tasks.read_text()
assert "NEXT_TASK=T003-076" in tasks.read_text()
assert "### T003-076 — ACTIVE" in plan.read_text()
