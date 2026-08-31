# T003-071 Sandbox Plan Qualification

## Result

`T003-071` is qualified as `PASS` at exact implementation head `5f6ea41fc8372273065376f0a5ab546d18100e43`, tree `986ad7b04264339f79ea7399d839bfd4904d5ecb`, by CI #558 / run `33237313243`, which completed successfully on Windows, macOS, and Ubuntu.

## Qualified boundary

The qualified boundary compiles a protected `SandboxProfile` into a bounded, non-authority-bearing launch/admission plan description while intersecting the requested launch with current protected authority state:

- exact immutable profile id/version and declared profile rights;
- the latest exact `sandbox.launch` authorization decision;
- the bound active capability lease id and generation;
- the exact active policy bundle id/hash;
- optional protected egress permit identity and scope when the profile requires external egress;
- trusted locality selected by the compiler boundary, not by the plan requester.

The compiler is read-only with respect to sandbox admission authority. It does not insert a `sandbox_admissions` row, does not consume egress-permit use accounting, does not select or claim a platform executor, and does not launch a process.

## Monotonic locality and egress

`SandboxLocality` is owned by `SandboxPlanCompiler`; `SandboxPlanRequest` has no locality switch. A requester therefore cannot downgrade strict-local mode. In strict-local mode, any profile requiring external permit-backed egress is denied before a usable launch plan can be returned. In non-strict mode, a permit-required profile requires an exact active, unexpired, unexhausted permit whose principal, parent lease, action class, and protected scope remain valid. A deny-all profile rejects ambient permit attachment.

Compilation does not consume `uses_consumed`; the actual egress-use boundary remains responsible for use-time effective-destination authorization and accounting.

## Portability repair during qualification

The first full exact-head qualification exposed a Windows-only test-fixture cleanup defect: a SQLite `Connection` used solely to verify `uses_consumed` remained open while the test removed its temporary authority directory. Unix allowed the unlink pattern, while Windows correctly rejected deletion of the open database. The fixture now explicitly drops that connection before cleanup. A focused Windows gate passed before the final exact-head multi-platform qualification. This repair changes no authority semantics.

## Evidence

```text
T003_071=PASS
T003_071_QUALIFIED_HEAD=5f6ea41fc8372273065376f0a5ab546d18100e43
T003_071_QUALIFIED_TREE=986ad7b04264339f79ea7399d839bfd4904d5ecb
T003_071_CI_RUN=33237313243
TRUSTED_LOCALITY_CALLER_DOWNGRADE=DENIED
PROFILE_AUTHORITY_INTERSECTION=YES
EGRESS_PERMIT_COMPILE_CONSUMPTION=NO
SANDBOX_ADMISSION_WRITTEN=NO
PLATFORM_EXECUTOR_CLAIMED=NO
PROCESS_LAUNCHED=NO
STRICT_LOCAL_DOMINANCE_PRESERVED=YES
REAL_SECRETS_USED=NO
DONOR_CODE_ADMITTED=NO
NEXT_TASK=T003-072
```
