# Contract — Skills, MCP & ACP

**Spec**: 005 Local Tools, Context & Memory

## 1. Boundary

Skills and protocols are interoperability/configuration mechanisms, never authority roots.

```text
SKILL != AUTHORITY
MCP_CAPABILITY_ADVERTISEMENT != GOLAM_CAPABILITY
MCP_RESULT != VERIFIED_TRUTH
ACP_CONNECTION != AUTHENTICATED_AUTHORITY
PROTOCOL_TRANSPORT != POLICY_BYPASS
```

## 2. Skill packages

Instruction-first skill packaging is compatible with Agent Skills `SKILL.md` concepts while Golam adds explicit provenance/version/capability lifecycle metadata.

A `SkillDescriptor` binds:

- package identity and immutable version/content digest;
- provenance/source/permission record;
- human-readable name/description;
- instruction content reference;
- requested capability classes;
- network posture;
- optional script references;
- admission/lifecycle state.

Instruction content is untrusted model-visible context. It may request actions but cannot authorize them.

## 3. Skill lifecycle and dispatch gate

Minimum lifecycle:

```text
DISCOVERED
-> PROVENANCE_RECORDED
-> REVIEWED
-> INSTRUCTION_ADMITTED
-> EXECUTABLE_ADMITTED (optional, sandbox-gated)
-> LOCKED_VERSION
-> DEPRECATED/REVOKED
```

Discovery or package presence is not admission. Updates require a new immutable version and renewed review when execution/capability semantics change.

Immediately before every instruction activation or executable-skill dispatch, Golam MUST re-read the current lifecycle state and verify the exact reviewed package/version/content digest. Dispatch requires a currently active admitted state appropriate to the requested operation; `DEPRECATED`, `REVOKED`, replaced, unknown or version-mismatched packages fail closed.

Capabilities, approvals, prepared-but-not-dispatched calls, queued calls and cached dispatch decisions are scoped to the exact active package/binding version. A lifecycle transition to `DEPRECATED`/`REVOKED`, version replacement or inability to validate the current state invalidates those cached decisions and queued calls; they MUST NOT be replayed under prior authority. A replacement version requires fresh review and fresh authority evaluation.

Already-running protected effects remain governed by their exact prepared effect identity and normal cancellation/reconciliation semantics. Revocation does not retroactively convert an ambiguous or still-running effect into success.

## 4. Executable skills

Executable scripts remain disabled unless:

- exact source/permission/dependency closure is admitted;
- the requested platform has an exact admitted production containment profile;
- environment/FS/network/process/resource limits are explicit;
- the active exact package version is revalidated at dispatch time;
- launch occurs as a normal governed tool/effect under current capability/approval state;
- output is treated as untrusted/tainted;
- cancellation and descendant supervision are qualified.

No script metadata may create implicit shell/network authority.

## 5. MCP server binding

An `McpServerBinding` records:

```text
binding id
server identity/version lock
transport
local process profile or remote endpoint
allowed protocol feature set
Golam-local tool/resource mapping
network policy
secret policy
taint class
lifecycle state
```

A server-advertised tool/resource/prompt is descriptive input. Golam maps it into an `ExternalToolDescriptor` whose maximum authority cannot exceed explicit local configuration and current policy/capability state.

Every MCP dispatch MUST revalidate immediately before dispatch that the exact `McpServerBinding` identity/version lock remains reviewed and active and that its Golam-local mapping has not been replaced or revoked. `DEPRECATED`, `REVOKED`, replaced, unreviewed, unknown or version-mismatched bindings reject cached capabilities, cached approvals, queued calls and stale mapped descriptors. No queued/cached request may inherit authority from a superseded binding.

## 6. MCP local process

Local MCP child-process launch requires the same exact production containment gate as shell/process tools. `native:unqualified` denies launch.

The child receives a cleared ambient environment, only explicitly brokered secrets, bounded FS/network/resource access and supervised cancellation/descendants. MCP stdout/stderr/protocol results are untrusted input.

The exact active binding/version and current local mapping are revalidated after queueing and immediately before process dispatch. A revocation or replacement before launch denies the dispatch rather than relying on stale capability/approval state.

## 7. MCP remote transport

Remote MCP requires explicit network/egress authority, endpoint identity policy, TLS/transport qualification as applicable, redirect/proxy policy, secret brokerage and strict-local compatibility.

Strict-local denies external remote MCP. Loopback/local IPC remains separately governed and still requires authenticated binding where relevant.

The exact active binding/version and mapping are revalidated immediately before any remote request is sent. Revocation/replacement invalidates queued remote calls and cached authorization decisions.

## 8. MCP protocol errors and unsupported states

Unsupported/unqualified protocol features fail explicitly; Golam does not silently downgrade to a broader transport, uncontained child process, cloud endpoint or alternate plugin path.

Protocol/schema violations, oversized messages, unknown content types and malformed tool descriptors fail closed under bounded parsing limits.

Lifecycle/version validation failure is an unsupported dispatch state and fails closed; it never falls back to a stale cached binding or broader transport.

## 9. MCP authority mapping

A server cannot:

- mint Golam capabilities or leases;
- set approval state;
- mutate policy/protected resources;
- clear taint;
- declare itself authoritative merely through metadata;
- bypass the Effect Gate by naming a tool `read_only` or equivalent;
- widen network/filesystem authority through nested tool calls.

Every consequential mapped operation is independently authorized at the Golam action boundary after the active exact MCP binding/version and Golam-local mapping are revalidated. Cached capability/approval material never outlives the binding/version for which it was evaluated.

## 10. ACP boundary

ACP is a client/IDE interoperability adapter. ACP sessions use existing authenticated local-client enrollment semantics. Loopback, process ancestry, transport connection, editor name or workspace presence is not authentication.

An ACP client receives only explicitly scoped operations/capabilities and cannot call privileged kernel state directly. All protected mutations remain normal Golam effects.

## 11. Protocol provenance and taint

Inbound protocol content retains server/client identity, version, transport, observation time and taint. Protocol resource metadata and model-facing content never erase existing taint or upgrade source authority.

## 12. Dependency admission

The official MCP specification is a protocol reference. The official Rust SDK is a dependency candidate, not automatically admitted. Implementation must select minimal exact crates/features and record dependency/network/process closure before direct use.

ACP and Agent Skills references likewise define compatibility targets, not blanket dependency/code admission.

## 13. Required adversarial corpus

Qualification includes:

- malicious tool names/descriptors/schema bombs;
- oversized/deeply nested protocol payloads;
- capability-spoofing metadata;
- nested calls attempting authority widening;
- malicious prompt/resource content trying to clear taint or approve actions;
- child-process environment/secret leakage;
- descendant/network escape;
- remote endpoint/redirect confusion;
- stale/replaced server version lock;
- cached capability/approval reuse after skill/MCP replacement or revocation;
- queued instruction/executable/MCP calls surviving `DEPRECATED`/`REVOKED` transitions;
- dispatch-time lifecycle/version validation unavailable or mismatched;
- protocol disconnect/restart during an ambiguous consequential effect;
- ACP unauthenticated/local-spoof attempts;
- skill package update replacing reviewed executable content without version change.
