---
task_id: T005-070
outcome: PASS
recorded_on: 2026-09-05
predecessor_phase_f_head: 2bf274be7120ab9304f40fdfa21f0f8fc6665923
predecessor_phase_f_ci_run: 33960737678
production_native_executor_admitted_before_task: false
implementation_selected_by_this_task: false
process_launch_enabled_by_this_task: false
waiver_taken: false
next_task: T005-071
---

# Phase G production containment requirements freeze

## Purpose

T005-070 re-reads the canonical Spec 003 sandbox/executor evidence and freezes the production native-containment requirements that T005-071 and later Phase G work must satisfy. This task does not select or admit an implementation and does not enable process launch.

## Canonical predecessor truth

The following Spec 003 facts remain binding:

1. `current_platform_executor_capabilities()` exposes the production baseline as `native:unqualified` with zero containment controls. Any launch plan requiring containment therefore fails closed before launch.
2. T003-073 qualified trusted capability-manifest resolution only. Requesters cannot self-assert executor capability, platform mismatch denies, and missing required controls deny.
3. T003-074 qualified a Linux x86_64 **test-only** harness using a `platform:linux-x86_64-bwrap-seccomp-test-v1` requirement. It did not admit a production executor.
4. The T003-074 harness established bounded evidence for cleared environment, explicit filesystem/device exposure, empty capability sets, `no_new_privs`, socket-syscall denial and managed descendants. It explicitly did **not** claim user-namespace isolation, network-namespace isolation, macOS support, Windows support, non-x86_64 Linux support or universal native isolation.
5. Bubblewrap and sudo in T003-074 were qualification-harness dependencies only, not product-runtime admissions.
6. T003-076 established that empty filesystem/device/IPC/inherited-handle allowlists mean explicit deny-all and still require matching executor enforcement primitives. Ambient authority is never implied by an empty list.
7. Canonical secret fallback requires a cleared ambient environment; secret plaintext must not appear in executable identity, argv or explicit environment, and the qualified fallback channel is stdin-only with exact-value redaction and no ambient descendant inheritance.
8. Existing Effect Gate durability/reconciliation remains authoritative. Cancellation is a request, not terminal process-tree proof.

## Frozen production requirements

Any first production native containment profile admitted by Spec 005 MUST satisfy every applicable requirement below before it can become launch authority.

### Trusted profile and executor identity

- A production profile has an exact immutable profile identity/version and exact current-platform requirement token.
- The executor capability manifest is constructed only by trusted Golam code and binds an exact executor implementation identity.
- The production profile token MUST be distinct from the T003 test-only `platform:linux-x86_64-bwrap-seccomp-test-v1` token.
- Runtime executable identity and cwd identity are verified before dispatch and are part of the prepared launch/effect binding.
- Missing, stale, mismatched or unsupported profile/executor identity fails closed before process creation.

### Ambient authority removal

- The child begins from a cleared ambient environment.
- Only explicitly bound non-secret environment variables may be introduced.
- Secret plaintext cannot be placed in argv or ambient/explicit environment.
- Inherited handles/descriptors are deny-all unless explicitly admitted and verifiably enforced.
- Device and IPC access are deny-all unless explicitly admitted and verifiably enforced.

### Filesystem containment

- Read and write roots are explicit, identity-bound and independently enforced.
- Empty read/write root sets still require enforcement and mean deny-all.
- Host paths outside admitted roots must not become visible through aliases, mount propagation, cwd rebinding or descendant behavior.
- Writable state must be limited to exact admitted roots; a temporary directory is not implicitly general filesystem authority.

### Network containment

- `DenyAll` must prevent external egress for the root process and every managed descendant.
- Strict-local denial dominates any process/tool/provider request.
- An external observation independent of the child process's own claims is mandatory before admission.
- No claim of network-namespace isolation may be made unless that exact primitive is independently proven on the admitted production profile.

### Privilege and kernel boundary

- Payload privilege elevation is forbidden.
- The profile must enforce no-new-privilege semantics or an equivalently strong proven platform primitive.
- Linux capabilities/privileges available to the payload must be empty or strictly bounded by explicit reviewed necessity.
- Platform-specific syscall/OS policy claims are limited to exactly observed enforcement; no cross-platform equivalence is inferred.

### Resource bounds

- Requested CPU, memory, wall-time and output limits require explicit executor enforcement support when present.
- A missing enforcement primitive denies rather than silently dropping a limit.
- Timeout handling must supervise the full owned process tree, not only the root PID.

### Descendant supervision and terminal reconciliation

- Process-tree ownership/discovery is explicit and deterministic.
- Descendants inherit no broader filesystem/network/device/IPC/handle/secret authority than the root launch contract.
- Cancellation must initiate bounded process-tree termination but cannot by itself mark the Effect terminal.
- Terminal success/failure requires evidence that the root and all owned descendants reached an acceptable terminal state.
- An unresolved descendant, lost ownership proof, interrupted kill boundary or restart ambiguity remains reconciling/`UNKNOWN_OUTCOME` and blocks dependent consequential work as required by the existing Effect semantics.

### Secret brokerage and evidence

- Secret handles remain opaque outside the trusted broker boundary.
- An unbrokerable fallback may use only an already admitted exact process profile and the previously qualified stdin-only secret path.
- Logs/evidence must redact secret values before leaving the trusted boundary and must not persist plaintext argv/environment secrets.

### Effect and authority binding

- Tool/model/protocol content is never launch authority.
- Every launch is prepared through Kernel authorization and the existing Effect Gate.
- The prepared Effect binds exact profile, executable/cwd identity, argv, explicit environment, secret-handle references, filesystem/network/device/IPC/handle rights, resource bounds, timeout/cancellation policy and reconciliation policy.
- A profile, executable, cwd, capability, policy, lease, approval or binding change after preparation invalidates dispatch.

## Platform freeze

### Linux x86_64

Linux x86_64 is eligible for T005-071 research because canonical Spec 003 contains concrete descendant-aware OS evidence there. T005-070 does not select Bubblewrap, setpriv, seccomp, namespaces or any other primitive for production. T005-071 must independently Source-Foundry-qualify the exact production closure before use.

The prior T003 harness proves neither user-namespace nor network-namespace isolation. A production Linux profile must therefore either independently prove those primitives or omit those claims and rely only on separately qualified controls.

### macOS

No production native-containment profile is selected or admitted. Phase G must return an explicit unsupported/denial state until separate exact macOS evidence exists.

### Windows

No production native-containment profile is selected or admitted. Phase G must return an explicit unsupported/denial state until separate exact Windows evidence exists.

### Other platforms/architectures

Unsupported and denied. No inference from Linux x86_64 evidence is permitted.

## Admission gate carried into T005-071..T005-077

No native process-backed tool, local executable MCP server, executable skill helper, external search binary or shell command may launch until:

1. T005-071 closes exact Source Foundry for the selected first-platform primitive/dependency closure;
2. T005-072 implements the profile against these frozen requirements;
3. T005-073 integrates the qualified secret boundary;
4. T005-074 obtains external descendant-aware strict-local network evidence;
5. T005-075 limits claims to the exact proven platform boundary;
6. T005-076 passes hostile process-tree/environment/filesystem/device/timeout/cancel qualification;
7. T005-077 obtains focused + repository CI evidence and substantive independent semantic/security review on the exact unchanged profile head.

```text
T005_070=PASS
PRODUCTION_REQUIREMENTS_FROZEN=YES
IMPLEMENTATION_SELECTED=NO
PRODUCTION_NATIVE_EXECUTOR_ADMITTED=NO
PROCESS_LAUNCH_ENABLED=NO
CROSS_PLATFORM_EQUIVALENCE=NO
LINUX_X86_64=ELIGIBLE_FOR_T005_071_RESEARCH_ONLY
MACOS=UNSUPPORTED_PENDING_SEPARATE_QUALIFICATION
WINDOWS=UNSUPPORTED_PENDING_SEPARATE_QUALIFICATION
WAIVER_TAKEN=NO
NEXT_TASK=T005-071
```
