# Golam Major Program Review and Lifecycle Closure — 2026-09-23

**Status:** PROGRAM REVIEW / PLANNING ONLY
**Target:** Golam Local/Private Verified Agent OS
**Authority:** This review does not authorize implementation, dependency admission, model admission, source admission, Constitution changes or Spec 006 scope expansion.

## 1. Review objective

This pass reviews Golam as a complete product lifecycle rather than as a collection of agent features.

The review asks, for every important capability:

1. who owns canonical state;
2. what opens/enables it;
3. what revokes/disables it;
4. what happens when it fails halfway;
5. what survives restart;
6. what is backed up;
7. what is intentionally not backed up;
8. how it is migrated;
9. how it is updated;
10. how it is rolled back or removed;
11. how the user can inspect/repair it;
12. how the product is decommissioned without leaving privileged residue.

The guiding completeness rule is:

```text
CREATE
-> USE
-> OBSERVE
-> PAUSE / REVOKE
-> FAIL
-> RECOVER
-> MIGRATE / UPDATE
-> EXPORT / RESTORE
-> REMOVE / DECOMMISSION
```

A feature with only the left half of this lifecycle is not complete.

## 2. Current architecture assessment

Golam's existing planning is already strong in:

- one canonical Effect path;
- capability/authority separation;
- strict-local and egress policy;
- verification independent from model confidence;
- task/session/run/worker identity;
- worker recovery and effect fencing;
- model/source/extension admission;
- memory provenance and safe learning;
- browser/computer/desktop route hierarchy;
- long-horizon work and checkpoints;
- voice/audio safety;
- multi-agent isolation;
- external channels and interoperability;
- benchmark/release evidence;
- supply-chain governance;
- data lifecycle and backup goals;
- chaos/resilience test planning.

The major residual gaps are operational lifecycle gaps where earlier tasks state goals but do not yet assign one complete product contract.

## 3. Gap A — first-run bootstrap, owner root, replacement and decommission

Existing T168/T174/T138 cover important recovery primitives, but the product still needs one explicit lifecycle from an uninitialized installation through owner bootstrap to final decommission.

Required states:

```text
UNINITIALIZED
INITIALIZING
ACTIVE
LOCKED
RECOVERY_PENDING
MIGRATION_PENDING
DECOMMISSION_PENDING
DECOMMISSIONED
```

The product must define:

- first-run creation of the protected owner/AuthorityHost root;
- recovery material creation and explicit user verification;
- what recovery material can and cannot authorize;
- lost/stolen device flow;
- AuthorityHost replacement;
- old-device revocation and stale-copy quarantine;
- reinstall-on-same-device behavior;
- uninstall with preserve-data vs erase-local-data choices;
- autostart/service/daemon cleanup;
- local browser/session/cache/model/extension residue cleanup;
- truthful statement that already-emitted external effects cannot be erased by uninstall.

This is T237.

## 4. Gap B — secrets and credentials are a lifecycle, not a value

T119/T174/T179/T207 correctly prevent ordinary model ownership of credentials, but a full secret lifecycle is not yet explicit.

Required `SecretBinding` semantics include:

```text
secret_id
kind
provider
account_binding
scope
storage_backend
hardware/user-presence constraints
created_at
last_verified_at
expires_at
rotation_state
revocation_state
exportability
origin
current_generation
last_use_receipt
```

Critical rules:

- plaintext is not canonical configuration;
- environment variables are delivery mechanisms, not secret authority;
- model/tool prompts never receive a secret merely because a provider needs one;
- account change invalidates account-bound pending work;
- rotation/revocation invalidates stale secret generations;
- device loss can revoke or quarantine local secret bindings;
- backup/export distinguishes exportable encrypted material from OS-bound non-exportable secrets;
- reauthentication and credential replacement do not silently inherit old approval.

This is T238.

## 5. Gap C — canonical-state corruption and repair

T138/T147 require backup/restore drills and T157 covers schema migration, but corruption handling needs a first-class contract.

Golam must distinguish:

```text
CANONICAL_STATE
REBUILDABLE_DERIVATIVE
EXTERNAL_EFFECT_EVIDENCE
SECRET_BINDING
CACHE
TEMPORARY_RUNTIME_STATE
```

Required behavior:

- startup integrity checks appropriate to each store;
- detection of event-ledger/hash-chain discontinuity where used;
- database corruption detection;
- safe read-only/recovery mode;
- no automatic "repair" that discards uncertain Effects;
- derivative indexes/cache rebuild from canonical inputs;
- backup manifest with exact schema/app version;
- encrypted backup;
- restore dry-run and conflict report;
- anti-fork AuthorityHost checks after restore;
- explicit `UNKNOWN_OUTCOME` carry-forward for Effects around the lost/corrupt interval;
- verified restore drill on a clean environment;
- retention/pruning without deleting required verification or revocation history.

Restic is a useful backup-design reference because it emphasizes confidentiality, integrity and verifiable restore rather than backup creation alone. It is not automatically Golam's canonical backup format.

This is T239.

## 6. Gap D — secure update, offline update and rollback

T113/T147 state supply-chain and updater requirements, but product update semantics require one end-to-end update contract.

The trust model should borrow from The Update Framework:

- signed root trust metadata;
- separate root / targets / snapshot / timestamp responsibilities;
- metadata expiry;
- rollback/freeze protection;
- threshold/delegated trust where justified.

Sigstore/Cosign is useful for build/release signature, identity and attestation verification. Tauri updater is useful for desktop transport/install mechanics only.

The updater must never become the root of trust by itself.

Required update classes are separate:

```text
APP_BINARY_UPDATE
AUTHORITY_SCHEMA_MIGRATION
MODEL_ARTIFACT_UPDATE
EXTENSION_UPDATE
SKILL_UPDATE
CONNECTOR_ADAPTER_UPDATE
POLICY_DATA_UPDATE
```

Required lifecycle:

```text
discover
-> verify trusted metadata
-> resolve exact target
-> compatibility preflight
-> active-work safety check
-> backup/checkpoint where required
-> download/import
-> artifact verification
-> stage
-> install
-> restart/migrate
-> post-update health
-> accept OR rollback software
```

Rollback of software never claims to reverse already-emitted external Effects.

For strict-local/air-gapped deployments, support a signed offline update bundle containing exact metadata and artifacts. Importing a bundle must not silently admit a model/extension that has not separately passed its own Foundry.

This is T240.

## 7. Gap E — connector authentication, remote events and sync lifecycle

T119/T188/T207 define provider/account routing and relay safety, but connectors need a full reliability/auth lifecycle.

Required cases:

- OAuth authorization;
- refresh-token rotation;
- refresh failure;
- scope increase/decrease;
- provider-side revocation;
- account disconnect/reconnect;
- user switches provider account;
- provider account deletion;
- rate limit;
- provider outage;
- webhook signature verification;
- webhook replay;
- duplicate delivery;
- out-of-order delivery;
- cursor/checkpoint corruption;
- sync resumption;
- remote object deletion;
- tombstones;
- schema/API version drift;
- at-most-once or non-idempotent write uncertainty.

Nango is useful as a product/reference source for OAuth, token refresh, proxy, sync and webhook operator lifecycle. Its reviewed repository is Elastic License 2.0 and its hosted/runtime assumptions are not a Golam trust root. Treat it as behavior/reference unless separate exact-component rights and architecture justification support more.

This is T241.

## 8. Gap F — operating-system permission drift

Desktop and voice planning correctly handles initial permissions, but operating-system permissions can change outside Golam at any moment.

Examples:

- macOS Accessibility revoked;
- Screen Recording permission revoked;
- microphone permission revoked;
- notification permission disabled;
- Windows secure desktop/UAC changes effective control;
- portal/session grants expire;
- mobile background permission changes;
- global shortcut permission or accessibility capability disappears;
- OS update changes permission semantics.

The product needs one `PlatformCapabilityState` projection with:

```text
capability
platform
observed_state
observed_at
source
current_generation
required_for_routes[]
remediation
```

Revocation must invalidate dependent route applicability and pending protected work before dispatch. Watchers/notifications are only an optimization: immediately before protected dispatch, Golam must freshly revalidate the platform permission/capability posture (or consume an equivalently fresh OS-issued proof bound to that dispatch generation). A delayed or missed permission-change notification must not permit stale dispatch; inability to establish current posture fails closed.

This is T242.

## 9. Gap G — deployment and tenancy posture

Golam is fundamentally a user-owned local/private system. That must remain an explicit architecture decision rather than an accidental assumption.

Baseline:

```text
PERSONAL_SINGLE_OWNER
one protected AuthorityHost
multiple bounded devices/execution nodes allowed
multiple agents/workers allowed
no implicit multi-tenant trust
```

If organization/team/enterprise mode is ever introduced, it requires separate qualification for:

- tenant identity;
- user/principal separation;
- organization policy;
- delegated administration;
- role/group mapping;
- SSO/SCIM or equivalent lifecycle;
- managed-device posture if used;
- data residency;
- audit access;
- offboarding;
- legal hold/retention where applicable;
- tenant-specific keys/secrets;
- cross-tenant isolation;
- admin override semantics;
- recovery ownership;
- billing/entitlement only if such a service exists.

A team workspace is not permitted to reuse the personal-owner authority model by simply adding more users.

This is T243.

## 10. Gap H — operator health and safe repair

T139/T178/T159/T182 provide diagnostics, observability and chaos evidence. They still need a single product-facing repair contract.

`golam doctor` / Health UI should project:

- canonical store integrity;
- pending/unknown Effects;
- AuthorityHost state;
- paired device state;
- model/provider readiness;
- extension/skill admission and revocation state;
- connector account health;
- browser/computer/runtime health;
- OS permission health;
- disk/resource pressure;
- backup freshness and last restore drill;
- update channel/metadata freshness;
- clock/time anomalies;
- local network/listener exposure;
- crash-loop/restart state.

A `RepairPlan` is a proposal, not authority.

Repairs that mutate state must go through the normal Effect/authority path and produce receipts. Diagnostic/support bundles are local/redacted by default and must never silently upload prompts, secrets or private artifacts.

This is T244.



## 10. Gap I — canonical configuration revision and policy precedence

Golam has many protected configuration surfaces: privacy profile, model routing, connectors/accounts, agent/workspace settings, skills/extensions, notification/attention behavior, device settings and future managed policy. These cannot rely on ambient "latest settings" or last-write-wins behavior.

One canonical `ConfigurationRevision` contract must define:

- configuration scope and owner;
- immutable revision/generation;
- expected prior revision for mutation;
- deterministic precedence;
- provenance/source surface;
- effective policy derivation;
- diff;
- migration version;
- stale-write rejection;
- rollback-as-new-effect;
- unsupported-future-version behavior;
- import/export semantics.

Protected configuration changes that alter privacy, egress, authority, secret release, model/provider routing or execution posture are governed Effects. Runtime environment variables, feature flags, cached settings and provider responses are inputs/derivatives, not canonical policy truth.

Cross-device and cross-surface clients must use optimistic concurrency or an equivalent revision check. A stale mobile/CLI/UI client cannot overwrite a newer policy silently.

This is T245.

## 12. External source additions

These sources are added only because they close measured lifecycle gaps.

| Source | Reviewed pin/state | License/posture | Use in Golam |
| --- | --- | --- | --- |
| `theupdateframework/specification` | `7dd5faca4251995063b851c060a12ac915b17ae3` | Community Specification License 1.0; implementation/reference | secure update metadata/trust model, expiry, rollback/freeze protection |
| `theupdateframework/python-tuf` | `aeb6ca76b42c11991c28ee7f0af57106cd9f2b4a` | Apache-2.0 / MIT files observed | implementation/test reference; no Python runtime requirement |
| `sigstore/cosign` | `0c66ecdff337f647bbcb0259efe61a81e33a76e8` | Apache-2.0 | build/release signing, verification, attestations/transparency reference/tool candidate |
| `tauri-apps/tauri-plugin-updater` | `ca61ba54fa1a806c527b539467f12f57918b16dd` | MIT or MIT/Apache-2.0 where applicable | desktop update transport/install donor candidate; not update trust authority |
| `restic/restic` | `6adedec6b48ae9ff0ffbc37bd675ccebd02c728f` | BSD-2-Clause | encrypted/verifiable backup/restore design and test reference |
| `open-source-cooperative/keyring-rs` | `430b34b83cb15e97d608aed494b49537e35a20b8` | MIT / Apache-2.0 | OS-native credential-store adapter candidate; platform behavior independently qualified |
| `NangoHQ/nango` | `f176680bb0a6df4ae8380fa9973a8540f346a56d` | Elastic License 2.0 | behavior/reference only for OAuth/token refresh/webhook/sync lifecycle by default |

No source above becomes an admitted runtime dependency from this review.

## 13. Completeness matrix

| Product lifecycle | Existing owner(s) | Closure added by this review |
| --- | --- | --- |
| repository governance | T110–T113 | none |
| release signing/provenance | T113/T146/T147 | T240 makes update client lifecycle explicit |
| first install / bootstrap | partial T168/T174 | T237 |
| owner recovery / replacement | T168/T174/T138 | T237 |
| decommission / uninstall | partial T160 | T237 |
| secrets / credentials | T119/T179/T207 | T238 |
| backup/export/restore | T138/T147/T160 | T239 |
| corruption / repair | partial T157/T182 | T239/T244 |
| schema migration | T157 | T239/T240 |
| application update | T113/T147 | T240 |
| offline/air-gapped update | absent as product contract | T240 |
| model updates | T175 | T240 coordinates; T175 still owns artifact admission |
| extension/skill updates | T180/T233 | T240 coordinates; existing owners still admit |
| connectors/accounts | T119/T188/T207 | T241 |
| webhook/sync integrity | partial scheduler/event semantics | T241 |
| OS permissions | Spec 006/T196 | T242 |
| personal deployment topology | T168 | T243 makes posture explicit |
| enterprise/multi-tenant claim | not currently admitted | T243 defines separate future gate |
| diagnostics/observability | T139/T178 | T244 |
| safe repair UX | partial T139/T182 | T244 |
| configuration revision / policy precedence / stale cross-surface writes | implicit only | T245 |
| data retention/export/delete | T160/T201 | T237/T239/T241 consume it |
| resource pressure | T132/T159/T182 | existing coverage sufficient |
| time/DST/scheduler | T129/T130/T182 | existing coverage sufficient |
| memory/context | T120–T126/T161/T164/T191/T192 | existing coverage sufficient |
| skills/learning | T122/T123/T162/T181/T215/T232/T233 | existing coverage sufficient |
| execution isolation | T151/T168/T173/T205 | existing coverage sufficient |
| browser/computer | Spec 006/T115/T117/T173/T199/T236 | existing coverage sufficient |
| voice/audio | T196/T217–T228 | existing coverage sufficient |
| multi-agent/workers | T127/T134/T163/T167/T184/T230/T231 | existing coverage sufficient |
| external channels | T169/T183/T231 | existing coverage sufficient |
| model decision fabric | T175/T203/T204/T229/T235 | existing coverage sufficient |
| verification | T149/T150/T213/T216 | existing coverage sufficient |
| resilience/chaos | T182 | T237–T245 add operator-specific expected outcomes |

## 14. Canonical ownership extensions required in T198

Before any implementation lifecycle consumes T237–T245, T198 must assign exactly one owner/version/migration authority for:

- `BootstrapState` / owner-root state;
- `RecoveryMaterialRef` and device-replacement state;
- `SecretBinding` / credential generation/revocation state;
- `BackupManifest` / `RestorePlan` / corruption status;
- `UpdateTrustRoot` / `UpdateManifest` / staged update state;
- `ConnectorBinding` / auth generation / sync cursor / webhook receipt state;
- `PlatformCapabilityState`;
- deployment/tenancy profile;
- `HealthSnapshot` / `RepairPlan`;
- `ConfigurationRevision` / policy-precedence / stale-write state.

No package-local substitute is permitted.

## 15. New hard invariants

```text
FIRST_RUN_COMPLETE != OWNER_AUTHORIZED
RECOVERY_MATERIAL != EFFECT_AUTHORIZATION
BACKUP_PRESENT != RESTORE_PROVEN
RESTORE_SUCCESS != EXTERNAL_EFFECT_ROLLBACK
DATABASE_REPAIR != EFFECT_OUTCOME_REWRITE
SECRET_PRESENT != SECRET_RELEASE_AUTHORITY
ENV_VAR_NAME != SECRET_CAPABILITY
TOKEN_REFRESH != APPROVAL_REFRESH
ACCOUNT_REAUTH != PENDING_EFFECT_REAUTHORIZATION
UPDATE_SIGNATURE != UPDATE_POLICY
UPDATER_AVAILABLE != TRUST_ROOT
SOFTWARE_ROLLBACK != EXTERNAL_EFFECT_ROLLBACK
OFFLINE_BUNDLE != ARTIFACT_ADMISSION
WEBHOOK_SIGNATURE != EVENT_SEMANTIC_TRUTH
SYNC_CURSOR != SOURCE_OF_TRUTH
OS_PERMISSION_GRANTED_ONCE != CURRENT_ROUTE_APPLICABILITY
CACHED_PLATFORM_PERMISSION != DISPATCH_PERMISSION_PROOF
PERMISSION_NOTIFICATION != DISPATCH_REVALIDATION
SAME_HOST != SAME_TENANT
TEAM_WORKSPACE != SHARED_OWNER_AUTHORITY
ADMIN_ROLE != OWNER_PRESENCE
TRACE != REPAIR_AUTHORITY
HEALTH_WARNING != AUTOMATIC_MUTATION_AUTHORITY
STALE_CONFIG_WRITE != VALID_CONFIGURATION_UPDATE
RUNTIME_DERIVED_CONFIG != CANONICAL_CONFIGURATION
FEATURE_FLAG != AUTHORITY_OVERRIDE
CONFIG_IMPORT != AUTHORITY_IMPORT
UNINSTALL != REMOTE_DATA_ERASURE
```

## 16. Failure journeys that must exist before stable release

### Lost primary device

```text
device lost
-> old authority copy considered suspect/stale
-> fresh recovery presence
-> new AuthorityHost bootstrap
-> old device/credential generations revoked
-> backup restore dry-run
-> unresolved external Effects preserved
-> connectors/secrets revalidated
-> verified resumed state
```

### Corrupt canonical database

```text
integrity failure
-> fail closed / read-only recovery mode
-> report affected stores/time window
-> never fabricate missing Effect outcomes
-> choose verified backup or bounded repair
-> rebuild derivatives
-> anti-fork/current-authority validation
-> operator-visible recovery receipt
```

### Bad update

```text
trusted update selected
-> artifact valid
-> install succeeds
-> health check fails
-> application rollback if schema permits
-> no replay of Effects
-> restored binary revalidates current canonical state
```

### OAuth/token expiry during an external write

```text
write prepared
-> credential generation checked
-> provider accepts or outcome becomes uncertain
-> token expires / reconnect occurs
-> no blind replay
-> reconcile remote state
-> new credential generation does not inherit old dispatch authority
```

### OS permission revoked while a task is waiting

```text
route prepared
-> user revokes Accessibility/Mic/Screen permission externally
-> OS permission-change notification delayed/lost
-> protected dispatch performs fresh platform permission revalidation
-> current permission cannot be proven / revocation observed
-> dispatch fails closed
-> pending route is invalidated
-> no weaker route silently selected
-> task reports blocker/remediation
```

### Uninstall / decommission

```text
user chooses decommission
-> show local vs remote data consequences
-> stop new work
-> settle/report running Effects
-> revoke local sessions/devices/secrets as possible
-> remove background services/autostart
-> erase selected local canonical/derived data
-> preserve/export only what user selected
-> disclose remote/external data that cannot be recalled
```

## 17. Explicit non-goals / future gates

The review does not make these current product requirements:

- mandatory cloud account;
- mandatory enterprise/multi-user deployment;
- centralized vendor control plane;
- automatic remote telemetry;
- mandatory Sigstore public-good service at runtime;
- mandatory restic executable;
- mandatory Nango runtime;
- mandatory Python TUF runtime;
- automatic repair of protected state;
- automatic recovery that bypasses owner presence;
- cloud backup by default.

The architecture must support future bounded implementations without making them implicit trust roots today.

## 18. Review disposition

```text
MAJOR_PROGRAM_REVIEW_2026_09_23_COMPLETE=YES
MEASURED_LIFECYCLE_GAPS_FOUND=9
NEW_TASK_RANGE=T237-T245
NEW_PARALLEL_AUTHORITY_SYSTEM=NO
NEW_RUNTIME_DEPENDENCY_ADMITTED=NO
NEW_MODEL_ADMITTED=NO
NEW_SOURCE_COMPONENT_ADMITTED=NO
ACTIVE_SPEC_006_PR_24_WIDENED=NO
CURRENT_POINTER_CHANGED=NO
FUTURE_IMPLEMENTATION_AUTHORITY_GRANTED=NO
WAIVER_TAKEN=NO
```
