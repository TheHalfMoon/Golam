# Contract: Kernel Boundary and Privilege Separation

## Purpose

This contract turns the phrase "trusted Rust kernel" into an enforceable authority boundary.

## Two distinct boundaries

- **Rust trusted path**: reviewable Rust components that participate in local runtime operation.
- **Privileged kernel**: the strictly smaller authority-bearing subset that alone may read or mutate authority state, mint capability material, authorize effects, broker secrets, sign receipts, or commit security-critical ledger state.

Rust implementation alone does not make a component privileged.

## Kernel-owned protected state

The following are protected resources and MUST NOT be writable through generic filesystem, shell, plugin, skill, worker, browser, MCP, ACP, or computer-control capabilities:

- principal registry and identity roots;
- capability/lease signing material and revocation state;
- Cedar policy store and Golam policy schema state;
- approval records used for authorization;
- secret vault, redaction keys, and secret-handle registry;
- effect journal, idempotency registry, and reconciliation state;
- security-critical session/goal journal records;
- audit hash chain and receipt-signing keys;
- GolamConnect pairing/device-revocation registry;
- strict-local egress policy state;
- skill lockfile/admission state;
- schedule/automation authority definitions.

Changes to protected state are themselves typed elevated effects and require policy evaluation plus the appropriate user approval.

## Authority construction

- Capability/lease tokens MUST have no public constructor outside privileged kernel modules.
- Authority-bearing types MUST be sealed or otherwise unforgeable by ordinary crates.
- Child leases MUST only narrow parent authority.
- Privileged kernel crates MUST use `#![forbid(unsafe_code)]` unless a narrowly reviewed platform/crypto boundary is separately isolated and justified.
- External data, model output, parser output, and adapter results are data only; none may instantiate authority.

## Parser and adapter isolation

Network/protocol-facing and supply-chain parsing surfaces SHOULD execute outside the privileged kernel and MUST communicate through typed, bounded interfaces. This includes MCP, ACP, GolamConnect transport parsing, skill package parsing, browser/CDP protocol handling, and optional Python/Node adapters.

Where practical these surfaces run in child processes or Wasmtime/WASI sandboxes. Their outputs are treated as untrusted input regardless of implementation language.

## Kernel API

The privileged kernel API MUST be explicit, typed, versioned, and process-splittable. A single-process v1 is permitted only if the semantic boundary would survive a later kernel-process split without changing authorization behavior.

Minimum authority API families:

- `AuthenticateClient`
- `Authorize(principal, action, resource, context)`
- `AcquireCapability`
- `ApproveOrDeny`
- `BeginEffect`
- `RecordEffectOutcome`
- `ReconcileEffect`
- `BrokerSecretUse`
- `AuthorizeEgress`
- `AppendSecurityEvent`
- `SignReceipt`
- `Pair/RevokeDevice`

## Verification gate

Spec 002 MUST include a compromised-adapter fault test proving an unprivileged harness/adapter cannot mint capabilities, read vault secrets, modify policy/principal/lease state, forge audit entries, or bypass effect/egress gates.