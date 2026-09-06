# T006-012 — Desktop Shell Source Foundry Admission Candidate

**Status**: `READY_FOR_EXACT_HEAD_CI_AND_INDEPENDENT_REVIEW_NOT_ADMITTED`

**Task**: T006-012 Tauri 2 + React + TypeScript desktop-shell dependency admission

**Canonical implementation base**: `main@c85b4b8f0d6ffccb039645803542d75b3bd47f29`

**Isolation candidate commit**: `6ded8116b086eb4822091912637232bbe1a14584`

**Isolation workflow**: `spec006-source-foundry` run `34064679063`, job `101571252005` — `SUCCESS`

**Current implementation PR**: #24 (`impl/006-desktop-computer-control`)

**Product manifest mutation before admission**: `NO`

## Decision candidate

The first desktop shell is proposed to use the following exact direct versions only after this record receives fresh exact-head CI and a substantive independent Source Foundry review:

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

This record is an admission candidate only. It does not admit any dependency, does not authorize product manifest mutation, and does not authorize any native desktop adapter dependency. T006-013 through T006-015 remain blocked until explicit admission is recorded after review.

## Exact isolated closure binding

The dedicated Source Foundry workflow built throwaway Rust and npm manifests outside Golam product manifests and resolved the exact selected closures.

Rust isolation evidence:

```text
CARGO_LOCK_SHA256=833cc5ab8c5bf574d633516b79ae0f528fa45199d5058671587c75bc3141fa48
RESOLVED_CARGO_PACKAGE_COUNT=418
DIRECT_TAURI=2.11.1
DIRECT_TAURI_BUILD=2.6.1
TAURI_CODEGEN=2.6.3
TAURI_MACROS=2.6.3
TAURI_RUNTIME=2.11.3
TAURI_RUNTIME_WRY=2.11.4
TAURI_UTILS=2.9.3
WRY=0.55.1
```

The resolved Rust closure is intentionally native and platform-sensitive. It includes the Wry/webview stack and target-specific Windows, macOS/Objective-C, GTK/WebKit and related platform crates. The full resolved graph is emitted by the successful isolation job. The package count above includes the temporary Source Foundry workspace package; the eventual product lock MUST be compared against the reviewed dependency graph rather than assuming that package count alone proves equivalence.

Frontend isolation evidence:

```text
PACKAGE_LOCK_SHA256=2617928c0bb2818bdeb4a688109910d1b67e68687b950c01897ca5d58957eb75
RESOLVED_NPM_PACKAGE_COUNT=58
TAURI_JS_API=2.11.1
TAURI_CLI=2.11.4
REACT=19.2.8
REACT_DOM=19.2.8
VITE=8.2.2
VITE_REACT_PLUGIN=6.1.1
TYPESCRIPT=6.0.3
```

The npm isolation install used `--ignore-scripts`; package script metadata was inspected separately by the Source Foundry workflow. Eventual product installation/build behavior MUST remain bounded to the reviewed scripts and exact lock. A new package, version, lifecycle script, native helper, or feature invalidates this candidate and requires requalification.

## Direct package provenance and licenses

The isolation metadata records the following direct package posture:

```text
@tauri-apps/api 2.11.1
LICENSE=Apache-2.0 OR MIT
UPSTREAM=tauri-apps/tauri

@tauri-apps/cli 2.11.4
LICENSE=Apache-2.0 OR MIT
UPSTREAM=tauri-apps/tauri
OPTIONAL_PLATFORM_CLI_PACKAGES=PRESENT

react 19.2.8
LICENSE=MIT
UPSTREAM=facebook/react

react-dom 19.2.8
LICENSE=MIT
UPSTREAM=facebook/react
DEPENDENCY=scheduler ^0.27.0
PEER_REACT=^19.2.8

typescript 6.0.3
LICENSE=Apache-2.0
UPSTREAM=microsoft/TypeScript
NODE_ENGINE=>=14.17

vite 8.2.2
LICENSE=MIT
DEPENDENCIES=postcss,rolldown,picomatch,tinyglobby,lightningcss
NODE_ENGINE=^20.19.0 OR >=22.12.0

@vitejs/plugin-react 6.1.1
LICENSE=MIT
PEER_VITE=^8
NODE_ENGINE=^20.19.0 OR >=22.12.0
```

No upstream source is vendored by this candidate. Distribution notice/license obligations for the final locked closure remain mandatory. Any copied upstream implementation is a separate Source Foundry event and is not admitted by package dependency admission.

## Build and native risk disposition

Tauri is not a pure data/library dependency. The selected closure intentionally introduces native desktop build/runtime surfaces. Admission therefore MUST NOT be interpreted as authority admission.

```text
NATIVE_WEBVIEW_RUNTIME=PRESENT
TARGET_SPECIFIC_NATIVE_CLOSURE=PRESENT
CARGO_BUILD_DEPENDENCIES=PRESENT
NPM_INSTALL_SCRIPTS_IGNORED_DURING_ISOLATION=YES
PRODUCT_BUILD_SCRIPTS=REQUIRE_EXACT_LOCK_REVALIDATION
ARBITRARY_DONOR_CODE_REUSE=NO
PLATFORM_AUTOMATION_AUTHORITY=NO
PLATFORM_CAPTURE_AUTHORITY=NO
RAW_INPUT_AUTHORITY=NO
CLIPBOARD_AUTHORITY=NO
```

The Tauri host may expose only Golam-owned, typed, sanitized commands. Native platform adapters remain separate later Source Foundry admissions under T006-022/T006-027/T006-032 and cannot be smuggled through this shell admission.

## Runtime authority and network disposition

The shell is a local UI/client boundary, not an authority source.

```text
RENDERER_AUTHENTICATION_MATERIAL=DENIED
RENDERER_AUTHORIZATION_STATE=DENIED
RENDERER_CAPABILITY_MINTING=DENIED
RENDERER_DIRECT_ADAPTER_ACCESS=DENIED
RENDERER_RAW_PLATFORM_HANDLES=DENIED
TAURI_HOST_PRINCIPAL=AUTHENTICATED_LOCAL_GOLAMD_CLIENT_ONLY
NETWORK_AUTHORITY_FROM_TAURI=DENIED
REMOTE_NAVIGATION=DENIED_BY_PRODUCT_POLICY
REMOTE_FALLBACK=DENIED
TELEMETRY=DENIED_UNLESS_SEPARATELY_AUTHORIZED
CREDENTIAL_ACCESS_BY_RENDERER=DENIED
```

The existing Golam strict-local network guard remains authoritative. Package capability must never be interpreted as network permission. The eventual Tauri configuration/CSP/capabilities must be reviewed to ensure the webview cannot silently widen local-only behavior.

## Required product posture after admission

If admitted, T006-013 through T006-015 must preserve all of the following:

1. native Rust host authenticates through the existing local `golamd` IPC/client-enrollment boundary;
2. renderer receives sanitized DTOs only;
3. credentials, capabilities, policy/approval evidence and raw platform handles never enter the renderer;
4. renderer cannot call native adapters directly;
5. persistent visible autonomous-control state exposes immediate pause/stop/takeover;
6. renderer crash/reload cannot leave autonomous actuation silently active when it removes the only qualified visible control channel;
7. Tauri capability files are least-privilege and deny unneeded shell/filesystem/network/window operations;
8. CSP/navigation configuration is local-only and fail closed;
9. dependency versions are exact and the eventual locks are reconciled against this candidate before qualification.

## Source Foundry isolation result

The exact isolation workflow at `6ded8116b086eb4822091912637232bbe1a14584` completed successfully. That result proves that the selected direct candidates resolve together in isolated package-manager projects and records their exact resolved metadata. It does **not** prove product behavior, native platform permissions, semantic safety, final lock equivalence, or authority correctness.

```text
ISOLATION_RESOLUTION=PASS
SOURCE_FOUNDRY_ADMISSION=NO
PRODUCT_MANIFEST_MUTATION=NO
TAURI_PRODUCT_RUNTIME=NOT_ADMITTED
NATIVE_PLATFORM_ADAPTERS=NOT_ADMITTED
```

## Independent review gate

Before any Tauri/React/TypeScript package is written into a Golam product `Cargo.toml`, `Cargo.lock`, `package.json`, or package lock, a fresh substantive independent reviewer must inspect the exact candidate head after successful exact-head CI and verify at minimum:

1. exact direct versions and successful isolation-run identity;
2. Rust and npm lock hashes/counts and the emitted transitive closure;
3. direct and transitive license/notice posture;
4. native Wry/webview/platform closure and build-script implications are explicitly acknowledged rather than classified as a pure library;
5. npm lifecycle-script posture and eventual exact-lock revalidation requirement;
6. no renderer authentication, authorization, capability-minting, platform-handle or direct-adapter authority is admitted;
7. strict-local network policy, local navigation/CSP and no hidden remote fallback remain constitutional requirements;
8. native automation/capture/raw-input/clipboard adapters are explicitly outside this admission and require separate Source Foundry records;
9. no donor source is copied or vendored by this candidate;
10. the candidate does not claim product/runtime qualification from isolation-only evidence.

Status-only, summary-only, owner/self-review, stale-head review, CI-only output, unavailable-provider output or a review that does not examine supply-chain/native/authority boundaries is insufficient.

## Current disposition

```text
T006_012=BLOCKED_PENDING_EXACT_HEAD_CI_AND_INDEPENDENT_SOURCE_FOUNDRY_REVIEW
DESKTOP_SHELL_SOURCE_FOUNDRY_CANDIDATE=READY_NOT_ADMITTED
ISOLATION_RUN_34064679063=SUCCESS
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
NEXT_GATE=SUCCESSFUL_EXACT_HEAD_CI_THEN_FRESH_INDEPENDENT_SOURCE_FOUNDRY_REVIEW
```
