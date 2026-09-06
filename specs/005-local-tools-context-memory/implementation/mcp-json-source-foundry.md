# T005-088 — MCP JSON Strategy Source Foundry

**Decision**: `ADMIT_MINIMAL_IN_PROCESS_PARSER_ONLY`  
**Selected crate**: `serde_json = 1.0.151`  
**Upstream**: `serde-rs/json`  
**Exact release commit**: `de8500740cdcabffb9734f503e4889def823cf10`  
**Exact upstream Cargo.toml blob**: `888735e94e38e879801710a83f1f55fa3a7933fb`  
**Exact upstream build.rs blob**: `bbf4a749bb4b8042c2ac11cb3d938b773a83b134`  
**License**: `MIT OR Apache-2.0`  
**MIT license blob**: `31aa79387f27e730e33d871925e152e35e428031`  
**Apache-2.0 license blob**: `1b5ec8b78e237b5c3b3d812a7c0a6589d0f7161d`  
**Upstream MSRV**: Rust 1.71  
**Golam Rust**: Rust 1.98  
**Golam exact lock version**: `serde_json 1.0.151`  
**Golam lock checksum**: `c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14`

This record admits only a bounded JSON parser surface needed to normalize untrusted MCP protocol advertisements. It does not admit the official MCP Rust SDK, any MCP transport library, HTTP, OAuth, child-process management, remote egress, server-side authority, or donor authority semantics.

## Exact strategy

Golam will implement the Spec 005 MCP binding/lifecycle/normalization rules itself and use `serde_json` only to parse bounded inbound JSON into a non-authoritative representation before Golam-owned validation and canonical encoding.

The direct dependency edge must be pinned as:

```toml
serde_json = { version = "=1.0.151", default-features = false, features = ["std"] }
```

The following `serde_json` features are explicitly outside the admitted surface and must not be enabled by Golam for MCP:

```text
preserve_order
raw_value
unbounded_depth
arbitrary_precision
float_roundtrip
alloc-only mode
```

The parser is never authority. Parsed field names, schemas, descriptions, annotations, tool/resource/prompt metadata and nested protocol content remain untrusted input.

## Dependency/build closure

The exact upstream manifest declares the selected `std` surface over the normal pure-Rust JSON dependencies (`itoa`, `memchr`, `serde_core`, `zmij`). `indexmap` is optional upstream and is not required by Golam's selected MCP parser feature set.

Golam's current pre-T005-088 lockfile already resolves `serde_json 1.0.151` and records `indexmap 2.14.0`, `itoa`, `memchr`, `serde`, `serde_core` and `zmij` in the existing workspace graph. Therefore the implementation gate requires that adding the direct Golam dependency does not change `Cargo.lock`; any lockfile delta reopens this admission decision.

The exact upstream `build.rs` reads Cargo target architecture/pointer-width environment supplied by Cargo and emits compile cfg values only. It contains no network access, subprocess launch, generated source download, credential handling or device/runtime initialization.

## Unsafe/native/process posture

`serde_json` contains internal Rust `unsafe` implementation sites in its source tree. This admission does not claim a safe-only dependency. The crate is already present and compiled in Golam's existing dependency graph before T005-088, so adding the direct parser edge must not introduce a new crate/native/FFI/process dependency into the resolved build.

Golam's MCP normalizer itself remains `#![forbid(unsafe_code)]`. No parser output may enter Kernel authority types without Golam-owned bounded validation and mapping.

No FFI, native library, OS process, network client, TLS stack, OAuth flow, telemetry client, updater or remote service is admitted by this record.

## Resource and parser boundary

T005-089 implementation must enforce independent Golam ceilings before accepting normalized protocol data, including:

- bounded input bytes;
- bounded JSON nesting depth below `serde_json`'s built-in recursion ceiling;
- bounded array/object member counts;
- bounded strings/names/descriptions/URIs/schema material;
- explicit accepted advertisement kinds;
- explicit rejection of unsupported fields/content shapes where their semantics are not implemented;
- canonical Golam digests over normalized values rather than treating JSON serialization order as authority.

`unbounded_depth` is prohibited. Oversized or deeply nested payloads fail closed.

## MCP SDK disposition

The planning reference for `modelcontextprotocol/rust-sdk` was an implementation candidate only. Its observed public surface includes independent server/client, HTTP, auth, request-state, child-process and other transport features. Golam does not need that runtime surface for T005-089 normalization and therefore does not admit it.

```text
MCP_RUST_SDK_ADMITTED=NO
RMCP_DEPENDENCY_ADDED=NO
MCP_HTTP_DEPENDENCY_ADDED=NO
MCP_CHILD_PROCESS_DEPENDENCY_ADDED=NO
MCP_AUTH_DEPENDENCY_ADDED=NO
```

If a later task demonstrates that the Golam-owned bounded strategy cannot satisfy the frozen contract, a new exact Source Foundry decision is required before any MCP SDK crate/feature is added.

## Authority and strict-local invariants

This parser admission does not change any authority or transport posture:

```text
MCP_ADVERTISEMENT != GOLAM_CAPABILITY
MCP_BINDING != KERNEL_AUTHORITY
REMOTE_MCP_STRICT_LOCAL=DENY
LOCAL_MCP_PROCESS_REQUIRES_ADMITTED_PROCESS_GATE=YES
MCP_NETWORK_AUTHORITY_ADDED=NO
MCP_SECRET_AUTHORITY_ADDED=NO
TAINT_CLEARING_BY_PROTOCOL=NO
```

Server-advertised capability, approval, network, filesystem, secret, sandbox or trust metadata can only be retained as untrusted evidence or rejected. It cannot widen the locally reviewed mapping.

## Implementation gate

T005-088 authorizes only the following next step:

1. add the exact direct `serde_json 1.0.151` `std` dependency with `default-features = false`;
2. require `Cargo.lock` to remain byte-identical;
3. implement Golam-owned bounded MCP advertisement normalization and binding lifecycle validation;
4. keep local/remote transport execution separately gated by T005-090/T005-091;
5. run focused fmt/Clippy/tests before any permanent implementation commit.

```text
T005_088=PASS
DECISION=ADMIT_MINIMAL_IN_PROCESS_PARSER_ONLY
SERDE_JSON_VERSION=1.0.151
SERDE_JSON_UPSTREAM_COMMIT=de8500740cdcabffb9734f503e4889def823cf10
CARGO_LOCK_CHANGE_ALLOWED=NO
MCP_RUST_SDK_ADMITTED=NO
NETWORK_WIDENING=NO
PROCESS_WIDENING=NO
WAIVER_TAKEN=NO
NEXT_TASK=T005-089
```
