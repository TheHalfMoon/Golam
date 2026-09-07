# T006-012 — Desktop Shell Admission and Product-Lock Evidence

**Canonical implementation base**: `main@c85b4b8f0d6ffccb039645803542d75b3bd47f29`

**Implementation PR**: #24 (`impl/006-desktop-computer-control`)

## Independent Source Foundry admission

The bounded desktop-shell dependency candidate received the required independent disposition in PR #24 comment `5572375757` against exact reviewed head:

```text
SOURCE_FOUNDRY_REVIEW_HEAD=491be6e220437d6ca79b8349e9e1640e76cc0173
EXACT_HEAD_CI_34134058456=SUCCESS_UBUNTU_MACOS_WINDOWS
SOURCE_FOUNDRY_RUN_34134054131=SUCCESS
NPM_ARCHIVE_BYTES_BOUND_TO_PACKAGE_LOCK_SRI=YES
PRODUCTION_PRUNE_CONTROL_QUALIFIED_ON_LINUX_AND_MACOS=YES
ADMIT_DESKTOP_SHELL_DEPENDENCIES=YES
T006_012_PRODUCT_MANIFEST_LOCK_MUTATION=AUTHORIZED_ONLY
```

The disposition is intentionally narrow. It admits only the exact reviewed Tauri 2 / React / TypeScript desktop-shell dependency set and authorizes the T006-012 product manifest/lock mutation. It does not admit native automation, capture, raw-input, clipboard, accessibility, portal/EIS, renderer authority, product network authority, platform permissions, or any other native adapter.

## Product lock reconciliation

After admission, the product manifests were introduced with the exact admitted direct versions. A temporary, branch-scoped lock generator was used only to produce deterministic product locks from those manifests with dependency lifecycle scripts disabled.

GitHub Actions run `34142590635` (`spec006-product-locks`) completed `SUCCESS`. Its generated commit was:

```text
PRODUCT_LOCK_COMMIT=f4ec4434a1cfd6d0876c20aeb9114aa992cb9479
```

The exact compare from generator-trigger commit `b88aa2dee1f443960cc3fd495db4fc094b96d4ea` to `f4ec4434a1cfd6d0876c20aeb9114aa992cb9479` changed only:

```text
Cargo.lock
apps/golam-desktop/package-lock.json
```

The temporary lock-generator workflow was then removed in forward-only commit:

```text
LOCK_GENERATOR_REMOVAL=5b028fb32ac3cd4fc121f76fd73d03b00c40fb5d
```

No persistent write-capable lock-generation workflow remains part of the intended product surface.

## Pinned formatter repair evidence

The normal PR CI exposed Rust formatting drift before Clippy. A one-shot workflow used exactly the repository-pinned `rustfmt 1.98.0`, then removed itself in the same generated repair commit.

```text
RUSTFMT_ONE_SHOT_RUN=34143992161
RUSTFMT_ONE_SHOT_RESULT=SUCCESS
RUSTFMT_REPAIR_COMMIT=f4526b141633a7aef2972db28bf3bce726d49703
```

The exact compare from one-shot trigger commit `ff7daedd1a21b4ed625cb7764f9c7017ac27cf13` to `f4526b141633a7aef2972db28bf3bce726d49703` changed only:

```text
.github/workflows/spec006-rustfmt-once.yml  (removed)
crates/golam-kernel/src/client_auth.rs     (rustfmt-only changes)
```

The `ci` run created directly from the bot-authored formatter commit ended as `action_required` with zero jobs, so it is not code/test qualification evidence. This evidence commit intentionally creates a normal repository-authored head so the ordinary PR CI can execute again.

## Current authority boundary

```text
DESKTOP_SHELL_DEPENDENCIES=ADMITTED_FOR_EXACT_REVIEWED_SET
T006_012_PRODUCT_MANIFEST_LOCK_MUTATION=AUTHORIZED_AND_PERFORMED
NATIVE_ADAPTER_ADMISSION=NO
RENDERER_AUTHORITY_ADMISSION=NO
PRODUCT_RUNTIME_NETWORK_AUTHORITY=NO
PLATFORM_PERMISSION_ADMISSION=NO
FINAL_EXACT_HEAD_CI=PENDING
FINAL_INDEPENDENT_IMPLEMENTATION_REVIEW=PENDING
SPEC_006_IMPLEMENTATION_COMPLETE=NO
SPEC_006_CLOSED_CANONICAL=NO
WAIVER_TAKEN=NO
```

This record is evidence only. Live GitHub state, canonical governance, exact-head CI, and the final post-CI independent implementation review remain authoritative for later lifecycle decisions.
