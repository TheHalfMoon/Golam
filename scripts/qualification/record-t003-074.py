from pathlib import Path

QUALIFIED_HEAD = "0d28681c971b3ae1f08504c7eb2448789ac8ed6e"
QUALIFIED_TREE = "32379e52964faca7db84b8354947be825ba2ef64"
CI_RUN = "33247996101"
FOCUSED_RUN = "33247892761"

root = Path("specs/003-identity-policy-secrets-sandbox")
implementation = root / "implementation"

evidence = implementation / "sandbox-native-executor-qualification.md"
if evidence.exists():
    raise SystemExit("T003-074 final evidence already exists")
evidence.write_text(
    f"""# T003-074 Minimum Native Sandbox Executor Qualification

**Status**: PASS  
**Qualified implementation head**: `{QUALIFIED_HEAD}`  
**Qualified tree**: `{QUALIFIED_TREE}`  
**Official qualification**: CI #592 / run `{CI_RUN}` — SUCCESS on Windows, macOS, and Ubuntu.  
**Focused native-executor qualification**: run `{FOCUSED_RUN}` — SUCCESS.

## Qualified boundary

T003-074 reuses the descendant-aware strict-local managed-process-tree observer already qualified by T003-064 and establishes only the minimum native untrusted-process test profile/executor required by the frozen contract.

The Rust qualification profile is test-only. It is intersected through the T003-072 enforcement descriptor, requires strict-local `DenyAll` networking, clears the environment, exposes only explicit filesystem/device rights, allows only managed descendants, and carries the exact platform requirement `platform:linux-x86_64-bwrap-seccomp-test-v1`.

The production T003-073 capability baseline remains `native:unqualified` with zero containment controls. The test profile therefore cannot become a production launch authority and continues to fail closed through normal production capability resolution.

## Linux x86_64 executor evidence

The durable qualification script `scripts/qualification/t003-074-native-executor.sh` proves the bounded OS boundary used by the test profile:

- trusted qualification setup creates mount, PID, IPC, UTS, session and parent-death boundaries;
- payload execution is reduced to uid/gid 65534;
- capability inheritable, permitted, effective, bounding and ambient sets are all empty;
- `no_new_privs` is set before untrusted payload execution;
- ambient environment is cleared;
- host paths outside explicit runtime mounts are not visible;
- runtime roots are read-only and sandbox-local `/tmp` is the only qualified writable root;
- `/dev/null` is the only explicitly exposed device;
- a fixed x86_64 seccomp filter denies socket/connect/listen/bind/accept/socketpair syscalls before payload execution;
- a managed descendant executes under the same no-network/no-ambient-environment boundary.

The GitHub-hosted Ubuntu runner does not support the attempted unprivileged user/network namespace paths. User-namespace and network-namespace isolation are therefore explicitly not claimed. The host network namespace is shared only inside this test harness while seccomp removes socket capability before payload execution.

## Honesty boundaries

- Bubblewrap and sudo are qualification-harness dependencies only; they are not admitted as Golam product runtime dependencies.
- macOS, Windows, non-x86_64 Linux, external-network profiles, arbitrary requester executable paths and universal native isolation remain unsupported by this concrete executor.
- No network-capable Golam-managed native child was launched.
- No platform containment claim transfers from this bounded test harness to the production `native:unqualified` baseline.
- `WASMTIME_DISPOSITION=NOT_ADMITTED_NOT_NEEDED` remains unchanged.

```text
T003_074=PASS
T003_074_QUALIFIED_HEAD={QUALIFIED_HEAD}
T003_074_QUALIFIED_TREE={QUALIFIED_TREE}
T003_074_CI_RUN={CI_RUN}
T003_074_FOCUSED_RUN={FOCUSED_RUN}
MANAGED_PROCESS_TREE_OBSERVER=QUALIFIED
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
USER_NAMESPACE_ISOLATION_CLAIMED=NO
NETWORK_NAMESPACE_ISOLATION_CLAIMED=NO
UNIVERSAL_NATIVE_SANDBOX_CLAIMED=NO
NETWORK_CAPABLE_MANAGED_CHILD_LAUNCHED=NO
NEXT_TASK=T003-075
```
"""
)

tasks = root / "tasks.md"
text = tasks.read_text()
old_task = "- [ ] **T003-074** Before launching any native Golam-managed child process with network capability, upgrade the external strict-local observer from daemon-PID-only inspection to complete managed-process-tree observation or an equivalent descendant-capturing sinkholed boundary; then implement the minimum native untrusted-process test executor/profile required to prove the contract without claiming unsupported universal isolation."
new_task = f"- [x] **T003-074** Before launching any native Golam-managed child process with network capability, upgrade the external strict-local observer from daemon-PID-only inspection to complete managed-process-tree observation or an equivalent descendant-capturing sinkholed boundary; then implement the minimum native untrusted-process test executor/profile required to prove the contract without claiming unsupported universal isolation. Exact-head qualification: CI #592 (`{CI_RUN}`) SUCCESS at `{QUALIFIED_HEAD}`, tree `{QUALIFIED_TREE}`, on Windows/macOS/Ubuntu; focused native-executor run `{FOCUSED_RUN}` SUCCESS. Evidence: `implementation/sandbox-native-executor-qualification.md`."
if text.count(old_task) != 1:
    raise SystemExit(f"T003-074 task anchor count {text.count(old_task)}")
text = text.replace(old_task, new_task, 1)
if text.count("NEXT_TASK=T003-074") != 1:
    raise SystemExit(f"tasks NEXT_TASK T003-074 count {text.count('NEXT_TASK=T003-074')}")
text = text.replace(
    "NEXT_TASK=T003-074",
    f"T003_074=PASS\nT003_074_QUALIFIED_HEAD={QUALIFIED_HEAD}\nT003_074_QUALIFIED_TREE={QUALIFIED_TREE}\nT003_074_CI_RUN={CI_RUN}\nT003_074_FOCUSED_RUN={FOCUSED_RUN}\nNEXT_TASK=T003-075",
    1,
)
tasks.write_text(text)

plan = implementation / "current-execution-plan.md"
text = plan.read_text()
if text.count("**Current task**: `T003-074`") != 1:
    raise SystemExit("current task anchor missing")
text = text.replace("**Current task**: `T003-074`", "**Current task**: `T003-075`", 1)
old_section = """### T003-074 — ACTIVE

Reuse the already-qualified descendant-aware strict-local observer, then implement the minimum native untrusted-process test executor/profile necessary to prove the contract without claiming unsupported universal isolation. Any network-capable managed child remains forbidden until complete managed-process-tree external observation is active."""
new_section = f"""### T003-074 — COMPLETE

Qualified at exact implementation head `{QUALIFIED_HEAD}`, tree `{QUALIFIED_TREE}`, by CI #592 / run `{CI_RUN}`, SUCCESS on Windows/macOS/Ubuntu. Focused native-executor qualification run `{FOCUSED_RUN}` also completed SUCCESS.

Evidence: `implementation/sandbox-native-executor-qualification.md`.

The qualified boundary reuses the T003-064 descendant-aware observer and proves a Linux x86_64 test-only native executor/profile using explicit mount/PID/IPC/UTS/session controls, cleared environment, unprivileged payload identity, empty capability sets, `no_new_privs`, bounded device/filesystem exposure and pre-payload seccomp network denial. Production remains `native:unqualified`; user/network namespace isolation, macOS/Windows parity, external-network profiles and universal native isolation are not claimed. No network-capable managed child was launched.

### T003-075 — ACTIVE

Re-read the canonical T003-005 Wasmtime disposition. Because Wasmtime remains `NOT_ADMITTED_NOT_NEEDED` and no bounded Phase H requirement has reopened dependency qualification, complete T003-075 as explicit deferred/not-admitted evidence without adding Wasmtime."""
if text.count(old_section) != 1:
    raise SystemExit(f"T003-074 plan section count {text.count(old_section)}")
text = text.replace(old_section, new_section, 1)
if text.count("Remaining Phase H: T003-074..T003-076.") != 1:
    raise SystemExit("remaining Phase H anchor missing")
text = text.replace("Remaining Phase H: T003-074..T003-076.", "Remaining Phase H: T003-075..T003-076.", 1)
if text.count("NEXT_TASK=T003-074") != 1:
    raise SystemExit(f"plan NEXT_TASK T003-074 count {text.count('NEXT_TASK=T003-074')}")
text = text.replace(
    "NEXT_TASK=T003-074",
    f"T003_074=PASS\nT003_074_QUALIFIED_HEAD={QUALIFIED_HEAD}\nT003_074_QUALIFIED_TREE={QUALIFIED_TREE}\nT003_074_CI_RUN={CI_RUN}\nT003_074_FOCUSED_RUN={FOCUSED_RUN}\nNEXT_TASK=T003-075",
    1,
)
plan.write_text(text)

candidate = implementation / "t003-074-qualification-candidate.md"
ctext = candidate.read_text()
if "**Status**: QUALIFICATION_CANDIDATE — NOT YET PASS" not in ctext:
    raise SystemExit("candidate status anchor missing")
ctext = ctext.replace(
    "**Status**: QUALIFICATION_CANDIDATE — NOT YET PASS",
    "**Status**: SUPERSEDED BY QUALIFIED EVIDENCE",
    1,
)
ctext = ctext.replace(
    "Official Windows/macOS/Ubuntu repository CI on this candidate is still required before `T003_074=PASS` may be recorded.",
    "Official Windows/macOS/Ubuntu CI #592 completed SUCCESS. Canonical task evidence is now `implementation/sandbox-native-executor-qualification.md`.",
    1,
)
candidate.write_text(ctext)

assert "T003_074=PASS" in tasks.read_text()
assert "NEXT_TASK=T003-075" in tasks.read_text()
assert "### T003-075 — ACTIVE" in plan.read_text()
assert evidence.exists()
