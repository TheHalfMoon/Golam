# T006-012 — Desktop Shell Source Foundry Admission Candidate

**Status**: `READY_FOR_FRESH_EXACT_HEAD_QUALIFICATION_AND_INDEPENDENT_REVIEW_NOT_ADMITTED`

**Task**: T006-012 Tauri 2 + React + TypeScript desktop-shell dependency admission

**Canonical implementation base**: `main@c85b4b8f0d6ffccb039645803542d75b3bd47f29`

**Current implementation PR**: #24 (`impl/006-desktop-computer-control`)

**Product manifest mutation before admission**: `NO`

## Exact direct dependency candidate

Only the following direct versions are proposed:

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

This record is still a candidate. Product `Cargo.toml`, `Cargo.lock`, `package.json` and npm lock mutation remains forbidden until a fresh substantive independent reviewer returns an explicit `ADMIT_DESKTOP_SHELL_DEPENDENCIES` disposition on the unchanged exact head after both exact-head CI and `spec006-source-foundry` succeed.

No native automation, capture, raw-input, clipboard, accessibility, portal/EIS or other platform adapter dependency is admitted by this record.

## Review history and forward-only repairs

Historical review remains evidence of what was insufficient, not current admission authority:

1. Source Foundry run `34064679063` resolved the exact direct set but did not inventory full transitive license/notice/lifecycle surfaces. Independent review rejected admission.
2. Repaired run `34067945715` inventoried the full transitive closure. Independent review comment `5571682202` still returned `DO_NOT_ADMIT_DESKTOP_SHELL_DEPENDENCIES` because license rows lacked explicit dispositions, optional npm platform archive notices were inferred from Linux installation state, and `fsevents@2.3.3` native/lifecycle behavior was unbounded.
3. Run `34131196026` intentionally failed closed while tightening those rules. Diagnostic comment `5571853686` identified three exact issues: two permissive Cargo boolean expressions were not classified, `selectors@0.36.1` had MPL metadata but no top-level upstream license/notice file, and macOS proved the `fsevents` archive itself contains a native `.node` binary even when lifecycle scripts are disabled.
4. Forward-only repair commit `783b46f33af9435e55c15358f3406122102619da` made the policy exact. Source Foundry run `34131722942` then completed `SUCCESS` for both `qualify-exact-desktop-dependency-closure` and `prove-script-disabled-install-macos`.

This file mutation creates a newer exact head, so run `34131722942` is now pre-final evidence only. The fresh run triggered for the current exact head must succeed before independent review.

## Machine-checkable qualification contract

`.github/workflows/spec006-source-foundry.yml` is the machine-checkable closure qualification for this candidate. On the exact reviewed head it must:

- resolve and hash the exact Cargo and npm locks generated from the direct versions above;
- emit the complete transitive package identity set;
- inventory every external Cargo package's license expression, license-file metadata, observed top-level license/notice files, custom-build target and native `links` value;
- assign every Cargo license expression an explicit `ACCEPT_*` or `REJECT_*` disposition and fail on every `REJECT_*` row;
- download every exact npm package archive using `npm pack ... --ignore-scripts`, including target-specific optional packages that are not installed on Ubuntu;
- verify archive package identity, hash each archive, inspect actual archive license/notice files and enumerate relevant lifecycle scripts without executing them;
- assign every npm license expression an explicit `ACCEPT_*` or `REJECT_*` disposition and fail on every rejected/missing/unknown license or archive identity;
- require actual archive license/notice evidence for accepted npm MPL-2.0 packages;
- enumerate the exact `fsevents@2.3.3` archive native source/binary surface while denying its execution;
- prove on macOS that `fsevents` is removed from installed `node_modules` before the bounded product frontend build and that the build succeeds without it;
- prove the script-disabled frontend install/build posture on Linux as well.

Workflow success is necessary but never sufficient for admission. The independent reviewer must inspect the exact run evidence and agree that the encoded dispositions are correct and bounded.

## License disposition policy

### Permissive expressions

Cargo and npm license metadata can express permissive choices or conjunctions as SPDX `OR`, SPDX `AND`, or historical slash forms such as `MIT/Apache-2.0`.

The Source Foundry classifier examines all license leaves. If every leaf is in the explicitly accepted permissive set, the expression is accepted and the disposition records the selected OR branch plus whether conjunction terms also apply. This covers exact closure expressions such as:

```text
MIT/Apache-2.0
Apache-2.0 / MIT
Apache-2.0 AND MIT
(MIT OR Apache-2.0) AND Unicode-3.0
Unlicense/MIT
```

Unknown or missing license expressions remain rejected fail closed.

`r-efi 5.3.0` and `r-efi 6.0.0` declare `MIT OR Apache-2.0 OR LGPL-2.1-or-later`. For both packages this candidate selects **Apache-2.0**. The LGPL alternative is not selected.

```text
R_EFI_5_3_0_SELECTED_LICENSE=Apache-2.0
R_EFI_5_3_0_LGPL_SELECTED=NO
R_EFI_6_0_0_SELECTED_LICENSE=Apache-2.0
R_EFI_6_0_0_LGPL_SELECTED=NO
```

### MPL-2.0

The exact closure contains MPL-2.0 packages, including Cargo CSS/parser/selector packages and npm `lightningcss` packages. MPL-2.0 is accepted only under its file-level obligations; this is not blanket reciprocal-license admission.

For every accepted MPL-2.0 package Golam requires:

1. preservation of the substance of license, copyright, patent, warranty-disclaimer and liability notices in MPL-covered source;
2. MPL-2.0 terms for distributed Covered Software in Source Code Form and recipient notice of how to obtain the license;
3. when MPL-covered executable form is distributed, availability of the corresponding MPL-covered Source Code Form by reasonable timely means and recipient notice of how to obtain it;
4. MPL-2.0 treatment for modifications to MPL-covered files while unrelated Golam files in a Larger Work remain under their own terms;
5. preservation of upstream license/notice material that is present.

Normative reference: Mozilla Public License 2.0 sections 3.1–3.4 (`https://www.mozilla.org/MPL/2.0/`).

Cargo registry packages may truthfully declare `license = "MPL-2.0"` without shipping a separate top-level license file. `selectors@0.36.1` is such an observed case. That absence is recorded rather than hidden. Acceptance therefore additionally requires Golam's distribution bundle/third-party notices to carry the MPL-2.0 license reference/text and the corresponding source-availability notice for any distributed Covered Software. It is not legitimate to misclassify the package as unlicensed merely because an extra top-level file is absent.

For npm platform archives, the previous review specifically required direct archive inspection rather than Linux `node_modules` inference. Therefore accepted npm MPL packages must have their actual archive license/notice files enumerated by the exact qualification run; absence is fail closed.

```text
MPL_2_0_POLICY=ACCEPT_FILE_LEVEL_WITH_DISTRIBUTION_OBLIGATIONS
CARGO_MPL_WITHOUT_SEPARATE_TOP_LEVEL_FILE=RECORD_AND_REQUIRE_GOLAM_DISTRIBUTION_LICENSE_SOURCE_NOTICE
NPM_MPL_ARCHIVE_WITHOUT_LICENSE_NOTICE_FILE=REJECT
UNKNOWN_RECIPROCAL_LICENSE=REJECT
MISSING_LICENSE_METADATA=REJECT
```

## npm install/build and lifecycle policy

The only admitted product dependency-install sequence for this candidate is:

```text
npm ci --ignore-scripts --no-audit --no-fund
node scripts/prune-denied-native-optionals.mjs
```

The prune step is Golam-owned product code that must remove `node_modules/fsevents` and fail closed if removal cannot be proved. No product tool/build command may execute between `npm ci` and the successful prune check.

The bounded frontend build command is:

```text
npm run build --ignore-scripts
```

Dependency lifecycle execution is denied. Package `preinstall`, `install`, `postinstall`, `prepare`, `rebuild`, `prepack`, `postpack` and related hooks are not authorized merely because they exist in a resolved package archive. Any future need to execute a dependency lifecycle script is a new Source Foundry event.

## `fsevents@2.3.3` exact disposition

The macOS qualification evidence proved that `fsevents@2.3.3` can contain a prebuilt `.node` native binary even when npm lifecycle scripts are disabled. Therefore disabling scripts alone is not the boundary.

`fsevents@2.3.3` is accepted only as an exact optional **lock/archive qualification row** and is explicitly excluded from the installed product tool/runtime closure before any product build/tool execution:

```text
FSEVENTS_2_3_3_LOCK_ENTRY=EXPECTED_OPTIONAL_MACOS
FSEVENTS_2_3_3_ARCHIVE=QUALIFIED_FOR_IDENTITY_LICENSE_AND_SURFACE_ONLY
FSEVENTS_2_3_3_PRODUCT_NODE_MODULES=PRUNED_BEFORE_BUILD_OR_TOOL_EXECUTION
FSEVENTS_2_3_3_INSTALL_SCRIPT_EXECUTION=DENIED
FSEVENTS_2_3_3_NODE_GYP_EXECUTION=DENIED
FSEVENTS_2_3_3_NODE_HEADER_NETWORK_ACQUISITION=DENIED
FSEVENTS_2_3_3_NATIVE_ADDON_EXECUTION=DENIED
FSEVENTS_2_3_3_AUTHORITY=NONE
```

The exact Source Foundry workflow must prove on macOS that the native package is present after script-disabled npm installation, is removed before build, remains absent, and the bounded Vite production frontend build succeeds. A future desire to use native `fsevents`, its prebuilt addon, `node-gyp`, watcher behavior, Node-header acquisition or any helper requires separate Source Foundry admission.

## Native and build-surface disposition

Tauri/Wry is intentionally a native desktop closure. Cargo custom-build targets and native `links` rows are expected and must remain visible to review.

```text
NATIVE_WEBVIEW_RUNTIME=PRESENT
TARGET_SPECIFIC_NATIVE_CLOSURE=PRESENT
CARGO_CUSTOM_BUILD_SURFACES=PRESENT_AND_MUST_BE_REVIEWED
CARGO_NATIVE_LINK_SURFACES=PRESENT_AND_MUST_BE_REVIEWED
NPM_DEPENDENCY_LIFECYCLE_EXECUTION=DENIED
ARBITRARY_DONOR_SOURCE_REUSE=NO
```

These dependency/build surfaces confer no Golam platform authority. Native automation/capture/raw-input/clipboard adapters remain separate later Source Foundry admissions.

## Runtime authority and network boundary

The Tauri shell is a local client/UI boundary, never an authority source.

```text
RENDERER_AUTHENTICATION_MATERIAL=DENIED
RENDERER_AUTHORIZATION_STATE=DENIED
RENDERER_CAPABILITY_MINTING=DENIED
RENDERER_DIRECT_ADAPTER_ACCESS=DENIED
RENDERER_RAW_PLATFORM_HANDLES=DENIED
RENDERER_PROTECTED_CONTROL_LEASE_MUTATION=DENIED
TAURI_HOST_PRINCIPAL=AUTHENTICATED_LOCAL_GOLAMD_CLIENT_ONLY
PRODUCT_RUNTIME_NETWORK_AUTHORITY_FROM_TAURI=DENIED
REMOTE_NAVIGATION=DENIED
REMOTE_FALLBACK=DENIED
TELEMETRY=DENIED_UNLESS_SEPARATELY_AUTHORIZED
CREDENTIAL_ACCESS_BY_RENDERER=DENIED
```

Package-manager network use in Source Foundry is qualification-only. It grants no product runtime network permission. Golam's existing strict-local network guard remains authoritative.

## Required Phase C posture after admission

If and only if the exact dependency set is admitted, T006-012..015 must preserve:

1. native Rust host authentication through the existing local `golamd` IPC/client-enrollment boundary using the existing desktop client kind rather than renderer-held credentials;
2. sanitized typed DTOs only across the renderer boundary;
3. no credentials, authorization state, capability tokens, Gate authorization, raw OS handles or protected control-lease mutation authority in renderer state;
4. no direct renderer-to-native-adapter calls;
5. local-only navigation/CSP and no hidden remote fallback;
6. least-privilege Tauri capabilities/permissions;
7. persistent qualified visible autonomous-control state with immediate pause/stop/takeover;
8. fail-closed suspension of new autonomous actuation when every qualified visible control channel is lost;
9. exact product Cargo/npm lock reconciliation against the reviewed Source Foundry closure;
10. the script-disabled install plus native-optional prune policy above.

## Evidence semantics

A successful Source Foundry run proves only exact isolated dependency resolution plus the bounded machine checks encoded in the workflow. It does **not** prove final product behavior, final product-lock equivalence, native OS permission correctness, semantic safety, platform-adapter correctness or runtime authority.

```text
SOURCE_FOUNDRY_ADMISSION=NO_PENDING_FRESH_EXACT_HEAD_REVIEW
PRODUCT_MANIFEST_MUTATION=NO
TAURI_PRODUCT_RUNTIME=NOT_ADMITTED
NATIVE_PLATFORM_ADAPTERS=NOT_ADMITTED
WAIVER_TAKEN=NO
```

## Independent review gate

Fresh independent review on the unchanged exact candidate head must verify at minimum:

1. exact direct versions and exact CI/Source Foundry run/head identities;
2. exact Cargo/npm lock hashes and resolved transitive closure;
3. every Cargo/npm license disposition, including permissive boolean/slash expressions, `r-efi` Apache-2.0 selection and MPL-2.0 obligations;
4. accurate treatment of the Cargo `selectors@0.36.1` MPL metadata/top-level-file absence and the required Golam distribution notice/source posture;
5. direct archive inspection for optional npm platform packages and actual npm MPL archive license/notice evidence;
6. all relevant npm lifecycle scripts and the product rule that dependency lifecycle execution is denied;
7. exact `fsevents@2.3.3` archive/native surface plus macOS proof that the package is pruned before build/tool execution and the build succeeds without it;
8. expected Cargo custom-build/native-link rows for the exact Tauri/Wry closure;
9. renderer/authentication/authorization/capability/raw-handle/direct-adapter/control-lease boundaries;
10. strict-local network, local-only navigation/CSP, no hidden remote fallback and separate later native-adapter admissions;
11. no donor source copied or vendored;
12. no claim that isolation evidence is product/runtime/platform-permission qualification.

Any material blocker requires `DO_NOT_ADMIT_DESKTOP_SHELL_DEPENDENCIES`. Only an explicit `ADMIT_DESKTOP_SHELL_DEPENDENCIES` disposition on the unchanged exact head can authorize product manifest mutation.

## Current disposition

```text
T006_012=BLOCKED_PENDING_FRESH_EXACT_HEAD_CI_SOURCE_FOUNDRY_AND_INDEPENDENT_REVIEW
DESKTOP_SHELL_SOURCE_FOUNDRY_CANDIDATE=READY_NOT_ADMITTED
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
