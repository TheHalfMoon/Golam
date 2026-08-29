# PA-003A — Always-On User-Owned Host and Execution-Node Topology

**Status**: PROPOSED_FOR_REVIEW  
**Date**: 2026-08-28  
**Parent**: `PA-003-product-spine-golden-loop.md`  
**Reason**: Phone access, schedules, proactive workers, and long-running tasks are not truly persistent if the only Golam daemon lives on a sleeping laptop. Golam needs an optional always-on topology without making a vendor cloud the trust root.

## 1. Decision

Golam SHALL support an optional **user-chosen Authority Host** that can stay online independently of any particular work computer.

The Authority Host may be:

- the user's primary desktop/laptop;
- a home server;
- a NAS-class host where platform support is qualified;
- a small always-on private machine;
- a user-controlled private VPS where the user explicitly accepts that locality/privacy posture;
- another future qualified host profile.

Golam does **not** require a vendor-operated cloud control plane for persistence.

`ALWAYS_ON != CLOUD_REQUIRED`

The default single-computer installation may continue to run the Authority Host locally on that computer.

## 2. Topology roles

### Authority Host

Owns the active privileged Golam kernel and protected authority state for one user-owned Golam domain.

It owns or authoritatively coordinates:

- principal/device enrollment;
- policy and capability leases;
- approvals/preauthorizations;
- secret-vault authority/secret references according to configured placement;
- event/effect/audit authority;
- canonical Task/Run/Goal state;
- scheduler authority;
- canonical operational memory state;
- GolamConnect device relationships;
- worker causality.

### Execution Node

A paired computer/device that exposes bounded capabilities to the Authority Host, such as:

- filesystem/workspace access;
- shell/process execution;
- local applications;
- browser/computer control;
- local model/GPU resources;
- device-specific files or credentials through governed brokers;
- sensors where explicitly authorized.

An Execution Node does not become an authority root by advertising capabilities.

`EXECUTION_NODE != AUTHORITY_HOST`

### Mobile Client

An authenticated GolamConnect client used for conversation, task continuity, approvals, inspection, notifications, voice/file/photo input, and protected remote-control UX as specified by PA-001.

### Channel Adapter

A lower-assurance messaging transport. It remains outside native device trust regardless of which host receives its message.

### Sandbox Provider

A replaceable execution substrate behind Golam authority contracts as specified by PA-002A.

## 3. Single active authority by default

Golam v1 MUST avoid implicit multi-master protected-state authority.

For one Golam domain, exactly one Authority Host is active for protected mutations unless a later separately reviewed design proves a safe replicated authority protocol.

Security-relevant authority identity is the tuple **`(authority_domain_id, authority_generation)`**, not a generation counter by itself.

- `authority_domain_id` is a collision-resistant protected domain identity. Planned migration or lost-host recovery MUST rotate it whenever the prior Authority Host/domain can no longer be proven exclusively current.
- `authority_generation` is monotonically increasing only within the currently established domain identity and may be used for ordinary revocation/rekey/rebinding generations inside that domain.
- Cross-host protected requests, paired-device authority state, execution-node authority envelopes, approvals/queued signed intents, and migration/recovery evidence MUST bind the complete current domain/generation tuple wherever that identity is security-relevant.
- An older domain or generation cannot become current merely because an old host comes back online.
- A stale backup's stored generation MUST NOT be treated as proof of the maximum generation ever used by an unreachable host. Lost-host recovery therefore MUST NOT derive current authority identity solely as `stored_generation + 1`.

A fresh recovery domain identity MUST be generated through an implementation-qualified collision-resistant mechanism such as OS cryptographic randomness and MUST NOT be caller-selected, copied from the stale backup as current identity, or reused from a prior authority domain. This requirement is semantic; PA-003A does not admit any new runtime dependency.

A fresh identifier is not sufficient authentication. Every current authority domain MUST also be cryptographically bound to an implementation-qualified **authority-domain authentication root** (for example, a host/domain signing or authenticated-transport credential whose private material is protected by the qualified platform boundary). Lost-host recovery MUST generate fresh private authentication/root material that is unavailable to the unreachable old host and is not restored from the stale backup as current authority. The current `authority_domain_id`, generation, and public credential/key identifier MUST be bound together in protected state and in the authentication evidence used by paired devices/nodes. Merely presenting a fresh random domain label while authenticating with an old backup-carried private key MUST NOT establish current authority.

`FRESH_DOMAIN_ID != FRESH_AUTHENTICATION_ROOT`
`STALE_BACKUP_KEY != CURRENT_AUTHORITY_CREDENTIAL`

Backups/standby copies are not active authority.

`BACKUP_STATE != ACTIVE_AUTHORITY`
`STALE_AUTHORITY_EPOCH != ACTIVE_AUTHORITY`
`STALE_BACKUP_COUNTER != RECOVERY_DOMAIN_IDENTITY`

This avoids split-brain approvals, leases, effects, scheduler decisions, and audit ordering even when the newest reachable backup predates authority activity on a lost host.

## 4. Host migration and lost-host recovery

Moving authority to a new host is a protected migration operation, not ordinary file copy.

A future migration protocol MUST cover:

- explicit user intent;
- target-host enrollment and attestation as applicable;
- protected-state integrity verification;
- current effect/UNKNOWN outcome reconciliation;
- approval/lease expiry or re-mint rules;
- secret re-sealing/re-brokering;
- device/channel rebinding generation changes where required;
- **authority-domain identity rotation plus fresh authority-domain authentication-root/key rotation and pairing-domain/generation rotation at cutover** so credentials and signed objects cannot remain valid merely because protected bytes were copied to a new Authority Host;
- **invalidation of every pre-cutover mobile approval response and queued signed request intent whose domain/generation/authentication-root binding names the old Authority Host**, followed by fresh authorization or re-signing only after the new host relationship is current;
- old-host revocation/fencing before the new host accepts protected mutations when the old host is reachable;
- monotonic audit evidence binding the old domain/generation/authentication-root identity, migration operation, new domain/generation/authentication-root identity and cutover point;
- rollback before cutover where safe;
- explicit lost-host recovery when the old host is unavailable.

### Planned migration

When the old Authority Host is reachable, it MUST durably enter a fenced/decommissioned state for the old domain/generation before the new host accepts protected mutations under the replacement domain/generation. The old host cannot later resume the prior authority identity through ordinary restart/reconnect.

A planned migration MAY transfer enough verified predecessor state to establish the replacement domain deterministically as part of the protected migration transaction, but the replacement domain identity still MUST be distinct from the old domain and bound into all post-cutover authority-bearing objects. The target host MUST establish fresh current authority-domain authentication private material (or an equivalently strong protected key-rotation result) before accepting post-cutover protected requests; copying the old host's private authority credential to the target as the current credential is not a valid migration cutover.

### Lost-host recovery

When the old host cannot be contacted, Golam cannot truthfully claim to have changed unreachable bytes on that machine or know that a reachable backup contains the last generation ever used there. Recovery therefore establishes a **fresh collision-resistant authority domain identity and fresh cryptographic authority-domain authentication root** and logically fences the complete old domain across every re-established trusted relationship rather than pretending physical remote revocation occurred.

The recovery protocol MUST ensure:

- the new Authority Host starts protected mutations only under a newly generated authority-domain identity, its current generation, and freshly generated authority-domain authentication private material unavailable to the old host;
- the new domain identity is not computed only by incrementing a generation/counter read from a stale backup;
- old/stale authority-host private signing/authentication keys restored from backup are never promoted as the current domain credential; they may be retained only where strictly necessary for bounded historical verification/recovery and MUST NOT authenticate current protected traffic;
- re-enrolled/reconnected devices and Execution Nodes pin the complete fresh domain/generation plus current authority authentication public-key/key-id binding and reject protected requests, approvals, leases, queued intents, nonces, or control envelopes from stale domains, generations, or authority credentials;
- knowledge of the fresh `authority_domain_id` or generation alone is insufficient to authenticate the new Authority Host; a returning old host that still controls prior private credentials cannot impersonate the recovered domain merely by relabeling requests with the fresh identifier;
- cached or backup-carried authority-bearing objects retain their original domain/generation/authentication-root binding and cannot be relabeled into the new domain by restoration code;
- secrets/approvals/leases that cannot be proven safely transferable are re-sealed/re-minted/re-authorized rather than inherited by byte-copy alone;
- an old host that later returns is treated as **stale/recovered-old-host**, not as an equal active peer;
- a returning old host may enter bounded recovery/export/reconciliation tooling but MUST NOT resume protected mutations or rejoin the active domain until an explicit protected reprovisioning process enrolls it into the current domain/generation under the current authentication root;
- protected mutations made by a stale old host after the recovery cutover are divergent non-canonical evidence and MUST NOT auto-merge into current authority/effect/audit state;
- any user-data artifacts recovered from the old host enter explicit provenance/conflict handling rather than silently rewriting current canonical state;
- the migration/recovery record distinguishes physical old-host decommissioning from logical stale-domain fencing and records the predecessor backup identity/hash, predecessor authority credential identity, newly generated domain identity, and newly generated authority credential identity without pretending the backup proves unreachable-host recency.

#### Effect and scheduler uncertainty across a stale-backup gap

A fresh authority domain fences stale authority; it does **not** prove that external effects after the backup horizon did not occur.

Lost-host recovery MUST record the exact restored backup/canonical cut point (for example, the last authenticated global/effect/audit position available in that backup) and conservatively handle work whose real-world outcome may have changed after that point.

- Any protected external effect that was in-flight/UNKNOWN at the restored cut point remains UNKNOWN until the normal Effect Gate reconciler obtains sufficient current evidence.
- Any Task/Run/effect whose continuation could duplicate a consequence that the lost host may have dispatched after the restored cut point MUST enter an explicit recovery-blocked/UNKNOWN state until provider/environment reconciliation or a separate attributable user decision makes a safe continuation possible.
- The new host MUST NOT infer `NOT_EXECUTED` merely because an effect transition, receipt, scheduler record, or task event is absent from a stale backup.
- Consequence-bearing scheduled work whose trigger window falls inside the unobserved lost-host gap MUST NOT be automatically “caught up” or replayed. It is treated as outcome-unknown until reconciled or deliberately rescheduled under current authority.
- At-most-once/irreversible operations MUST NOT be blind-retried across the recovery gap. Existing provider idempotency/reconciliation identities are reused where they are known and still valid; absence of such identity does not authorize guessing.
- Recovered canonical state MUST preserve an explicit uncertainty boundary so later Trust Receipts, Task satisfaction, learning, and memory do not describe post-backup real-world outcomes as known facts without evidence.
- Recovery-domain rotation, authentication-root rotation, effect reconciliation, and user-visible recovery decisions are separately attributable. Creating a new domain or key MUST NOT reset Effect Gate history or erase UNKNOWN outcomes.

`LOST_HOST_RECOVERY != EFFECT_OUTCOME_RESET`
`ABSENT_FROM_STALE_BACKUP != NOT_EXECUTED`
`MISSED_SCHEDULE_WINDOW != SAFE_REPLAY`

A migration or recovery MUST NOT create a canonical window where both old and new domains/generations/authentication roots are accepted as current authority. Host migration continuity applies to user-visible Task/session state, not to automatic portability of stale authority material.

`OLD_HOST_RETURNS != AUTHORITY_RESTORED`
`USER_VISIBLE_CONTINUITY != AUTHORITY_OBJECT_CONTINUITY`
`BACKUP_GENERATION_PLUS_ONE != LOST_HOST_FENCING`

No vendor service may silently become authority during migration or recovery.

## 5. Persistence when a work computer sleeps

When the Authority Host is always-on:

- phone/mobile conversations can continue;
- routines and timers can fire;
- workers that only need host-available resources can continue;
- memory/index maintenance can continue under policy;
- notifications can be emitted under Initiative/Attention rules;
- remote sandbox/provider work can continue where authorized.

If a Task needs a sleeping/offline Execution Node, Golam MUST represent `WAITING_NODE` / equivalent blocked state rather than pretending the resource is available or silently moving sensitive work elsewhere.

`OFFLINE_NODE != PERMISSION_TO_FALLBACK`

## 6. Data placement

Authority Host ownership does not imply every user file is copied to that host.

Data may remain node-local. Task evidence can reference node-scoped resources and request them only when the node is online and the active lease permits access.

Future data-placement policy SHOULD distinguish:

- authority-host canonical state;
- portable user-owned memory/artifacts;
- node-local workspaces/files;
- node-local secrets/credentials;
- remote-provider ephemeral data;
- explicitly synchronized material.

A node-local resource MUST NOT be silently replicated merely to make the assistant appear always-on.

## 7. Secret placement

Some secrets may be Authority-Host-brokered; others may intentionally remain device/node local.

The architecture MUST allow a task to block until the correct secret-owning node is available rather than copying all credentials into a central vault.

Secret movement between hosts/nodes is a protected operation with explicit provenance and policy.

## 8. Capability and locality truth

Every Execution Node publishes a capability/availability descriptor that is validated by conformance evidence where applicable.

The scheduler/planner must know whether a node is:

- online/offline;
- locked/unlocked where relevant;
- interactive/non-interactive;
- allowed for sensitive workloads;
- local LAN / remote user-owned / third-party hosted;
- capable of GPU/browser/desktop/CLI/file operations;
- currently holding a valid lease for the task.

Planning against stale node availability MUST fail/replan honestly.

## 9. Worker placement

A Worker may execute on the Authority Host, a paired Execution Node, or an authorized sandbox/remote provider.

Placement does not transfer kernel authority.

The Worker receives only task-scoped delegated capabilities and must return evidence/effect requests through Golam-owned semantics.

`WORKER_PLACEMENT != AUTHORITY_TRANSFER`

## 10. Phone and channel implications

PA-001 remains authoritative for native mobile/channel trust.

The always-on Authority Host solves availability, not authentication:

- possession of WhatsApp/Telegram/WeChat account access cannot enroll a native device;
- phone pairing remains cryptographic and protected;
- mobile approval and queued-intent signatures remain bound to the exact current Authority Host/pairing domain identity and generation defined by the PA-001 contract and, once PA-003A topology is implemented, to the current authority-domain authentication credential/key identity as part of the protected current-domain binding;
- Authority Host migration/recovery invalidates pre-cutover signed mobile authority/request objects rather than treating user-visible continuity as authority continuity;
- a returning stale old host/domain cannot authenticate as current authority merely because a device remembers it or because it retains an old private authority credential;
- channel messages remain channel-tainted;
- high-risk approval still steps up to an authenticated trusted surface;
- push remains a wake/sync convenience rather than canonical order or authority.

## 11. Strict-local implications

`STRICT_LOCAL` may support an always-on Authority Host only when that host and transport path satisfy the user's configured locality boundary.

Examples:

- same local machine: eligible;
- qualified LAN/home-server path: may be eligible under explicit policy;
- public VPS: not strict-local unless the definition is explicitly changed by user policy, which must never happen silently;
- third-party relay may forward encrypted native transport only under its declared metadata/privacy posture and does not become execution authority.

## 12. Product modes

Golam SHOULD eventually make topology understandable through simple setup modes, for example:

### This Computer

Everything runs on the current computer. Simplest Core Alpha/Desktop path.

### Home Golam

A user-owned always-on machine is the Authority Host; laptops/desktops become execution nodes.

### Private Remote Golam

A user-selected private remote host is the Authority Host, with explicit privacy/locality disclosure.

These are setup projections; the underlying security contracts remain identical.

## 13. Owning specs

### Spec 007

Define native host/node pairing, device topology, reconnect, node availability, protected host migration prerequisites, collision-resistant authority-domain identity and generation rotation, fresh authority-domain authentication-root/key rotation and pinning, stale mobile approval/queued-intent invalidation, stale-backup-safe lost-host recovery fencing, returning-old-host disposition, and phone continuity semantics.

### Spec 008

Use the topology for scheduler/worker placement and WAITING_NODE recovery. Proactive work must respect data/secret/node placement and attention policy. Scheduler recovery across a lost-host gap MUST preserve the effect/outcome uncertainty rules above rather than automatically replaying missed consequential work.

### Spec 010

Test:

- Authority Host restart;
- work-node sleep/offline/return;
- no silent execution fallback;
- node revocation;
- stale capability advertisement;
- worker placement changes;
- planned host migration/fencing where implemented;
- lost old host followed by new-domain recovery;
- recovery from a deliberately stale backup whose stored generation collides with or predates a generation used by the lost host;
- proof that lost-host recovery does not derive authority identity solely as `backup_generation + 1`;
- proof that lost-host recovery generates a fresh authority-domain authentication private credential/root rather than promoting backup-carried old private authority keys;
- a returning old host retaining the prior authority private key attempting to impersonate the new domain after learning the fresh domain ID, and proof that paired devices/nodes reject it;
- paired-device/node pinning of the fresh domain/generation plus current authority authentication public-key/key-id binding;
- old host returning after recovery and being denied current-authority status;
- stale-domain/generation/authentication-root device/node/control replay;
- no automatic merge of post-cutover protected mutations from a stale old host;
- an irreversible/at-most-once external effect executed by the old host after the restored backup cut point and proof that recovery does not blindly re-dispatch it;
- scheduled consequential work whose trigger window lies inside the lost-host/backup gap and proof that recovery marks it unknown/reconciliation-required instead of automatic catch-up;
- absence of a post-backup effect record being rejected as proof that the effect never happened;
- cross-host replay of pre-cutover mobile approvals, queued intents, leases and nonces;
- secret placement and re-brokering;
- phone continuity while a work node is offline;
- strict-local topology claims from outside the process boundary.

## 14. Deferred complexity

PA-003A does not authorize:

- multi-master authority;
- transparent cross-device CRDT protected state;
- automatic cloud failover;
- vendor-hosted mandatory control plane;
- copying every node filesystem/credential to the Authority Host;
- general peer-to-peer federation between unrelated Golam users.

Those require separate reviewed designs.

## 15. Binding invariants

```text
ALWAYS_ON != CLOUD_REQUIRED
EXECUTION_NODE != AUTHORITY_HOST
BACKUP_STATE != ACTIVE_AUTHORITY
STALE_AUTHORITY_EPOCH != ACTIVE_AUTHORITY
STALE_BACKUP_COUNTER != RECOVERY_DOMAIN_IDENTITY
BACKUP_GENERATION_PLUS_ONE != LOST_HOST_FENCING
FRESH_DOMAIN_ID != FRESH_AUTHENTICATION_ROOT
STALE_BACKUP_KEY != CURRENT_AUTHORITY_CREDENTIAL
LOST_HOST_RECOVERY != EFFECT_OUTCOME_RESET
ABSENT_FROM_STALE_BACKUP != NOT_EXECUTED
MISSED_SCHEDULE_WINDOW != SAFE_REPLAY
OFFLINE_NODE != PERMISSION_TO_FALLBACK
WORKER_PLACEMENT != AUTHORITY_TRANSFER
NODE_CAPABILITY_ADVERTISEMENT != CAPABILITY_LEASE
OLD_HOST_RETURNS != AUTHORITY_RESTORED
USER_VISIBLE_CONTINUITY != AUTHORITY_OBJECT_CONTINUITY
```
