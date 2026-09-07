# T006-012 — Desktop Shell Source Foundry Admission Candidate

**Status**: `REPAIRED_PENDING_EXACT_HEAD_QUALIFICATION_AND_INDEPENDENT_REVIEW_NOT_ADMITTED`

**Task**: T006-012 Tauri 2 + React + TypeScript desktop-shell dependency admission

**Canonical implementation base**: `main@c85b4b8f0d6ffccb039645803542d75b3bd47f29`

**Current implementation PR**: #24 (`impl/006-desktop-computer-control`)

**Product manifest mutation before admission**: `NO`

## Decision candidate

The first desktop shell proposes exactly:

```text
TAURI=2.11.1
TAURI_DEFAULT_FEATURES=FALSE
TAURI_FEATURES=wry
TAURI_BUILD=2.6.1
TAURI_BUILD_DEFAULT_FEATURES=FALSE
TAURI_JS_API=2.11.1
TAURI_CLI=2.11.4
REACT=19.2.8
REACT_DOM=19.2.8
VITE=8.2.2
VITE_REACT_PLUGIN=6.1.1
TYPESCRIPT=6.0.3
```

This record remains an admission **candidate** until a fresh substantive independent reviewer returns an explicit `ADMIT_DESKTOP_SHELL_DEPENDENCIES` disposition on the unchanged exact head after both exact-head CI and the Source Foundry workflow succeed. It does not authorize product manifest mutation yet. It never authorizes native automation, capture, raw-input, clipboard, portal/EIS, accessibility or other platform adapter dependencies.

## Review history and forward-only repair

Historical evidence is retained but is not current qualification authority:

- isolation run `34064679063` at `6ded8116b086eb4822091912637232bbe1a14584` resolved the exact direct set but did not inventory the full transitive license/lifecycle surface;
- independent review comment `5563121428` returned `DO_NOT_ADMIT_DESKTOP_SHELL_DEPENDENCIES` because transitive license/notice and npm lifecycle evidence was incomplete;
- repaired Source Foundry run `34067945715` at `a040325555f94e80a7cc34568c0989aabd7f43b8` inventoried the full closure;
- independent review comment `5571682202` still returned `DO_NOT_ADMIT_DESKTOP_SHELL_DEPENDENCIES` because review-required license rows lacked explicit dispositions, optional npm platform archive notices were inferred from a Linux install instead of inspected from their archives, and `fsevents@2.3.3` lifecycle/native-build behavior was not bounded by an allowed product installation policy.

The current repair is forward-only. Any CI or review bound to a prior head is historical after this mutation.

## Exact closure qualification contract

`.github/workflows/spec006-source-foundry.yml` is the machine-checkable closure authority for this candidate. On the exact candidate head it MUST:

1. resolve the exact Cargo closure for `tauri = 2.11.1` with `default-features = false`, `features = ["wry"]`, plus `tauri-build = 2.6.1` with default features disabled;
2. resolve the exact npm closure for the seven exact direct JavaScript packages above;
3. emit exact lock hashes, package identities and package counts;
4. inventory every external Cargo package's declared license/license-file/top-level license or notice files, custom build target and native `links` value;
5. assign every Cargo license expression an explicit `ACCEPT_*` or `REJECT_*` disposition and fail the job on any `REJECT_*` row;
6. download every exact npm package archive with lifecycle execution disabled, including target-specific optional packages that are not installed on the Linux qualification runner;
7. verify archive package identity, hash each archive, inspect actual top-level license/notice files and package scripts, and assign every npm license expression an explicit `ACCEPT_*` or `REJECT_*` disposition;
8. fail the job on an unknown/missing/unaccepted license, an MPL package without an observed archive/source license-notice file, an archive identity mismatch, or a missing expected `fsevents@2.3.3` optional closure row;
9. enumerate every relevant npm lifecycle script without executing dependency lifecycle scripts;
10. prove on macOS that the exact frontend dependency set installs and performs the bounded production frontend build while lifecycle scripts are disabled and without producing an `fsevents.node` native addon.

A workflow success is still not admission. The independent reviewer must inspect the exact run evidence and agree with the dispositions.

## License disposition policy

### Permissive alternatives

Historical Cargo metadata commonly expresses dual permissive licenses as `MIT/Apache-2.0`, `Apache-2.0 / MIT`, `Unlicense/MIT` or equivalent slash alternatives instead of canonical SPDX `OR`. The Source Foundry classifier normalizes those forms as alternative choices and records an explicit selected permissive alternative. Where Apache-2.0 is offered, this candidate selects Apache-2.0 for the admission record; otherwise it selects the first exact permissive alternative emitted by the package.

`r-efi 5.3.0` and `r-efi 6.0.0` declare `MIT OR Apache-2.0 OR LGPL-2.1-or-later`. This candidate selects **Apache-2.0**. The LGPL alternative is **not selected** and does not define Golam's applicable distribution posture for these packages.

```text
R_EFI_5_3_0_SELECTED_LICENSE=Apache-2.0
R_EFI_5_3_0_LGPL_ALTERNATIVE_SELECTED=NO
R_EFI_6_0_0_SELECTED_LICENSE=Apache-2.0
R_EFI_6_0_0_LGPL_ALTERNATIVE_SELECTED=NO
```

Any license expression that cannot be reduced to an explicitly accepted permissive alternative or the bounded MPL-2.0 policy below is rejected fail closed.

### MPL-2.0 packages

The exact Tauri/Wry and Vite closure includes MPL-2.0 packages, including the Cargo CSS-selector/parser family and the npm `lightningcss` family/platform packages. They are accepted **only** under MPL-2.0's file-level conditions; this is not a blanket acceptance of reciprocal licenses.

For every exact MPL-2.0 package emitted by the Source Foundry run, Golam records the same mandatory distribution posture:

1. preserve the substance of license, copyright, patent, warranty-disclaimer and liability notices in MPL-covered source;
2. if Golam distributes MPL-covered source, distribute that Covered Software under MPL-2.0 and inform recipients how to obtain a copy of the license;
3. if Golam distributes MPL-covered executable form, make the corresponding MPL-covered source available under the MPL conditions by reasonable timely means and inform recipients how to obtain it;
4. modifications to MPL-covered files remain subject to MPL-2.0; this does not relicense unrelated Golam files in a Larger Work;
5. do not remove upstream MPL license/notice material from distributed package/source artifacts.

Normative reference: Mozilla Public License 2.0 sections 3.1–3.4 (`https://www.mozilla.org/MPL/2.0/`). The Source Foundry workflow MUST observe an actual license/notice file for each accepted MPL package source/archive or fail closed.

```text
MPL_2_0_POLICY=ACCEPT_FILE_LEVEL_WITH_DISTRIBUTION_OBLIGATIONS
UNKNOWN_RECIPROCAL_LICENSE_POLICY=REJECT
MISSING_LICENSE_METADATA_POLICY=REJECT
MISSING_MPL_LICENSE_NOTICE_FILE_POLICY=REJECT
```

## npm archive and lifecycle policy

The normal Golam product dependency installation command for this exact frontend closure is constrained to:

```text
npm ci --ignore-scripts --no-audit --no-fund
```

Dependency lifecycle execution is denied. The bounded product frontend build command is:

```text
npm run build --ignore-scripts
```

The application-owned `build` script may invoke the exact-pinned Vite binary. Dependency `preinstall`, `install`, `postinstall`, `prepare`, `rebuild`, `prepack`, `postpack` and other package lifecycle hooks are not authorized to run as part of Golam installation/build merely because they exist in a package archive.

The Source Foundry workflow uses `npm pack <exact-name>@<exact-version> --ignore-scripts` to inspect each resolved archive directly. Therefore optional Windows/macOS/Linux platform packages receive actual archive-level license/notice and script evidence even when the Ubuntu runner does not install them into `node_modules`.

Any future change to the allowed install/build commands, lifecycle-script policy, direct version, resolved lock, package, feature, native helper or platform artifact invalidates this candidate and requires new Source Foundry qualification and review.

## `fsevents@2.3.3` disposition

`fsevents@2.3.3` is retained only as the optional macOS package selected by the frontend tooling closure. Its package archive declares an `install` script using `node-gyp rebuild` and build scripts that can compile a native addon. Those script surfaces are **not admitted for execution**.

```text
FSEVENTS_2_3_3_LOCK_POSTURE=OPTIONAL_MACOS_CLOSURE
FSEVENTS_2_3_3_PACKAGE_PRESENCE=ACCEPTED_AS_LOCKED_OPTIONAL_DEPENDENCY
FSEVENTS_2_3_3_INSTALL_SCRIPT_EXECUTION=DENIED
FSEVENTS_2_3_3_NODE_GYP_EXECUTION=DENIED
FSEVENTS_2_3_3_NATIVE_ADDON_BUILD=DENIED
FSEVENTS_2_3_3_NODE_HEADER_NETWORK_ACQUISITION=DENIED
FSEVENTS_2_3_3_AUTHORITY=NONE
```

The exact Source Foundry workflow must prove on macOS that `npm ci --ignore-scripts --no-audit --no-fund` followed by the bounded Vite production build succeeds without an `fsevents.node` native addon. If the product later needs native `fsevents` execution, watcher behavior, `node-gyp`, Node-header acquisition, or another native helper, that is a **new Source Foundry event** and is not authorized by this candidate.

## Native/build surface disposition

Tauri is intentionally not treated as a pure library. The selected closure includes Wry/webview/native target surfaces, Cargo custom-build targets and target-specific native links. Those are dependency/runtime implementation surfaces, not Golam authority.

```text
NATIVE_WEBVIEW_RUNTIME=PRESENT
TARGET_SPECIFIC_NATIVE_CLOSURE=PRESENT
CARGO_CUSTOM_BUILD_SURFACES=PRESENT_AND_REVIEW_REQUIRED
CARGO_NATIVE_LINK_SURFACES=PRESENT_AND_REVIEW_REQUIRED
NPM_DEPENDENCY_LIFECYCLE_EXECUTION=DENIED
ARBITRARY_DONOR_CODE_REUSE=NO
PLATFORM_AUTOMATION_AUTHORITY=NO
PLATFORM_CAPTURE_AUTHORITY=NO
RAW_INPUT_AUTHORITY=NO
CLIPBOARD_AUTHORITY=NO
```

The independent review must explicitly confirm that observed custom-build/native-link rows are expected for the exact Tauri/Wry closure and do not smuggle platform automation/capture/raw-input/clipboard authority or unreviewed native helpers into later tasks.

## Runtime authority and network disposition

The shell is a local UI/client boundary, not an authority source.

```text
RENDERER_AUTHENTICATION_MATERIAL=DENIED
RENDERER_AUTHORIZATION_STATE=DENIED
RENDERER_CAPABILITY_MINTING=DENIED
RENDERER_DIRECT_ADAPTER_ACCESS=DENIED
RENDERER_RAW_PLATFORM_HANDLES=DENIED
RENDERER_PROTECTED_CONTROL_LEASE_MUTATION=DENIED
TAURI_HOST_PRINCIPAL=AUTHENTICATED_LOCAL_GOLAMD_CLIENT_ONLY
PRODUCT_RUNTIME_NETWORK_AUTHORITY_FROM_TAURI=DENIED
REMOTE_NAVIGATION=DENIED_BY_PRODUCT_POLICY
REMOTE_FALLBACK=DENIED
TELEMETRY=DENIED_UNLESS_SEPARATELY_AUTHORIZED
CREDENTIAL_ACCESS_BY_RENDERER=DENIED
```

Package-manager network access during Source Foundry qualification is qualification-only. It grants no product runtime network authority. The existing Golam strict-local network guard remains authoritative.

## Required product posture after admission

If and only if this exact dependency set is admitted, Phase C must preserve all of the following:

1. the native Rust host authenticates through the existing local `golamd` IPC/client-enrollment boundary;
2. the renderer receives sanitized DTOs only;
3. credentials, capabilities, policy/approval evidence, Gate authorization, raw platform handles and protected control-lease mutation authority never enter the renderer;
4. renderer code cannot call native desktop adapters directly;
5. a persistent qualified visible autonomous-control surface exposes immediate pause/stop/takeover;
6. renderer crash/reload or loss of every qualified visible channel suspends new autonomous actuation fail closed;
7. Tauri capabilities/permissions are least privilege and deny unneeded shell/filesystem/network/window surfaces;
8. CSP/navigation configuration is local-only and fail closed;
9. exact product Cargo/npm locks are reconciled against the reviewed Source Foundry closure before product qualification;
10. product install/build keeps dependency lifecycle scripts disabled unless a later Source Foundry record separately admits a specific script/native helper.

## Evidence semantics

A successful isolation/qualification run proves only that the selected exact dependencies resolve and that the workflow's bounded supply-chain checks passed under the recorded qualification environments. It does **not** prove final product behavior, OS permission correctness, native platform adapter correctness, semantic safety, final product-lock equivalence or runtime authority.

```text
SOURCE_FOUNDRY_ADMISSION=NO_PENDING_FRESH_INDEPENDENT_REVIEW
PRODUCT_MANIFEST_MUTATION=NO
TAURI_PRODUCT_RUNTIME=NOT_ADMITTED
NATIVE_PLATFORM_ADAPTERS=NOT_ADMITTED
WAIVER_TAKEN=NO
```

## Independent review gate

Before any Tauri/React/TypeScript package is written into a Golam product manifest or lock, a fresh substantive independent reviewer must inspect the exact unchanged candidate head after both exact-head CI and `spec006-source-foundry` succeed. The review must verify at minimum:

1. exact direct versions and exact run/head identity;
2. exact Cargo/npm lock hashes, package counts and transitive closure;
3. every Cargo and npm license disposition, including permissive slash/OR alternatives, the selected Apache-2.0 `r-efi` alternative, and every MPL-2.0 package/notice file;
4. MPL-2.0 notice/source-availability obligations are correctly bounded and recorded;
5. every optional npm platform package archive was directly inspected rather than inferred from Linux `node_modules`;
6. all relevant npm lifecycle scripts are enumerated and dependency lifecycle execution remains denied by the product install/build policy;
7. `fsevents@2.3.3` remains optional and its `node-gyp`/native-addon/header-download execution is denied, with macOS proof that the bounded production frontend build succeeds without the addon;
8. Cargo custom-build/native-link surfaces are expected for the exact Tauri/Wry closure and do not confer authority;
9. renderer authentication/authorization/capability-minting/raw-handle/direct-adapter/control-lease authority remains denied;
10. strict-local network, local-only navigation/CSP, no hidden remote fallback and separate later Source Foundry admission for native desktop adapters remain mandatory;
11. no donor source is copied/vendored by this candidate;
12. qualification evidence is not misrepresented as product/runtime/platform-permission qualification.

If any material blocker remains, disposition must be `DO_NOT_ADMIT_DESKTOP_SHELL_DEPENDENCIES`. Only an explicit clean `ADMIT_DESKTOP_SHELL_DEPENDENCIES` disposition on the exact unchanged head can unblock product manifest mutation.

## Current disposition

```text
T006_012=BLOCKED_PENDING_EXACT_HEAD_CI_SOURCE_FOUNDRY_AND_INDEPENDENT_REVIEW
DESKTOP_SHELL_SOURCE_FOUNDRY_CANDIDATE=REPAIRED_NOT_ADMITTED
TAURI_2_11_1_ADMITTED=NO
TAURI_BUILD_2_6_1_ADMITTED=NO
TAURI_JS_API_2_11_1_ADMITTED=NO
TAURI_CLI_2_11_4_ADMITTED=NO
REACT_19_2_8_ADMITTED=NO
REACT_DOM_19_2_8_ADMITTED=NO
VITE_8_2_2_ADMITTED=NO
VITE_REACT_PLUGIN_6_1_1_ADMITTED=NO
TYPESCRIPT_6_0_3_ADMITTED=NO
PRODUCT_MANIFEST_MUTATION=NO
NATIVE_ADAPTER_ADMISSION=NO
WAIVER_TAKEN=NO
NEXT_GATE=SUCCESSFUL_EXACT_HEAD_CI_AND_SOURCE_FOUNDRY_THEN_FRESH_INDEPENDENT_REVIEW
```
