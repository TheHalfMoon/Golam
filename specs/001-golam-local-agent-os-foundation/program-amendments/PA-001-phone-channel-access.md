# PA-001 — Phone, Mobile, Voice, and Messaging-Channel Access

**Status**: PROPOSED_FOR_REVIEW  
**Date**: 2026-08-28  
**Founder direction**: Add first-class phone access so the owner can talk to and control Golam from a phone, including a native Golam mobile client and supported messaging channels such as WhatsApp and WeChat.  
**Canonical base inspected**: `main@82de7084384009ff3a00522f4e0aef09bf549529`  
**Implementation authorization**: NONE. This is a program-scope/planning amendment only. It MUST NOT enter the active Spec 003 implementation PR or authorize GolamConnect/mobile/channel product code before the owning future Spec Kit package completes its own lifecycle.

## 1. Binding scope change

This amendment changes one prior program decision in Spec 001:

- **Native mobile application is no longer deferred through Spec 010.**
- **Voice interaction is no longer wholly deferred through Spec 010.** Push-to-talk / voice-note interaction is required in the mobile/channel program; real-time full-duplex voice is a later lane of the same program and remains release-gated by platform/security evidence.
- Phone access is owned primarily by **Spec 007 GolamConnect**, with worker/scheduler integration in Spec 008 and release qualification in Spec 010.
- Telegram, standards-compliant WhatsApp Business Platform, WeChat/WeCom official integration paths, Slack, Discord, Matrix, and later channel adapters use one governed channel boundary. A messaging provider never becomes Golam's trust root.

This amendment does **not** weaken any existing constitutional or Spec 001 requirement. In particular:

- `CHANNEL != AUTHORITY` remains binding.
- Strict-local mode remains incompatible with third-party messaging providers and cloud push delivery; the UI MUST state this accurately.
- Native phone pairing is a cryptographic GolamConnect device-binding flow and MUST NOT be bootstrapped by proving control of a Telegram/WhatsApp/WeChat account.
- A model, worker, channel, mobile UI, relay, notification service, or messaging provider cannot mint or expand authority.
- Human takeover, emergency stop, effect gating, taint, secrets, replay resistance, approval freshness, protected-resource mutation rules, and audit integrity remain unchanged.

## 2. Product outcome

The owner should be able to leave the computer running at home or work and use Golam naturally from a phone:

1. Open **Golam Mobile** and continue the same sessions, goals, workers, tasks, files, approvals, receipts, and notifications visible on Desktop/CLI.
2. Type or speak: “Golam, check the project, fix the tests, and tell me when it is ready.”
3. Receive progress notifications without leaking task contents to Apple/Google push infrastructure.
4. Review a consequential action in the native mobile app using a cryptographically bound approval object rather than a loose chat reply.
5. Inspect what Golam is doing and, when an explicit control lease exists, view or control the host screen from the phone.
6. Send a photo, document, voice note, or file to Golam; the input is quarantined, tainted, hashed, and routed through the same canonical runtime as Desktop/CLI.
7. Reach Golam from **Telegram / WhatsApp / WeChat-WeCom / Slack / Discord / Matrix** where supported, while seeing an honest trust/privacy label for that surface.
8. Let Golam send notifications or task results to a selected channel only through the normal Effect Gate.

The product must feel like **one Golam, many surfaces**. There is no separate “mobile agent”, “WhatsApp agent”, or “Telegram memory”.

## 3. Two-plane architecture

Phone access is intentionally split into two planes with different assurance levels.

```text
                         PHONE ACCESS
                              |
              +---------------+----------------+
              |                                |
     NATIVE GOLAM MOBILE                CHANNEL BRIDGES
       high-assurance plane             lower-assurance plane
              |                                |
    GolamConnect protocol              Telegram Bot API
    device key + pairing               WhatsApp Business API
    signed E2E envelopes               WeChat/WeCom official API
    capability leases                  Slack / Discord / Matrix
    approval step-up                    later official adapters
              |                                |
              +---------------+----------------+
                              |
                      Channel/Connect Ingress
                              |
                 normalize + authenticate transport
                              |
                  identity/binding + taint compile
                              |
                       canonical SessionEvent
                              |
                         Golam runtime
                              |
                       privileged kernel
              identity / policy / leases / effects
              secrets / egress / audit / pairing
```

### Native Golam Mobile

Native Mobile is an authenticated **GolamConnect device**. It may be granted explicit, bounded trust tiers and capabilities after local pairing.

### Messaging channels

Messaging channels are third-party transports. Even a correctly bound sender remains `CHANNEL_UNTRUSTED` for content provenance. A channel can carry requests and low-risk interaction, but the provider does not authenticate a Golam effect by itself.

## 4. Assurance and trust tiers

The future Spec 007 MUST define a monotonic trust/capability model at least equivalent to:

| Surface | Identity assurance | Default authority posture |
|---|---|---|
| Authenticated local CLI/Desktop | strong local client identity | governed local client capabilities |
| Native Golam Mobile | cryptographic paired device | bounded device lease; step-up possible |
| Bound one-to-one messaging account | provider identity + local binding | request/notification; no protected mutation by default |
| Bound group/channel context | provider identity + conversation scope | stricter than one-to-one; explicit mention/command gating |
| Unbound sender | transport identity only | zero machine authority; optional low-trust input event |
| Relay / push provider | transport infrastructure only | zero authority |

Trust tier names are not frozen by this amendment. Semantics are: **native device identity may hold explicit Golam capabilities; third-party messaging identity never silently upgrades into native-device trust.**

## 5. Native mobile pairing and device security

### 5.1 Pairing

Pairing MUST be an elevated, local-user-authorized effect:

1. Desktop/CLI displays a short-lived QR/local challenge containing only bootstrap material required to establish the cryptographic ceremony.
2. The phone generates or provisions a device key pair locally.
3. Phone and host perform a mutually authenticated pairing exchange with replay protection.
4. Both surfaces display a human-verifiable confirmation/fingerprint.
5. The kernel records `Device`, owner principal, public key, trust tier, initial capabilities, generation, and revocation state.
6. The phone receives no wider capability than the locally approved grant.

Pairing by receiving a code in WhatsApp/Telegram/WeChat is explicitly forbidden as the native trust bootstrap.

### 5.2 Device key handling

The mobile client MUST use platform-protected key storage and user-presence protection where available. The exact Android/iOS APIs and libraries are Spec 007 qualification decisions, not frozen here. Exportable plaintext long-lived device private keys are not an acceptable default.

### 5.3 Lost/stolen phone

The host MUST support immediate device revocation. Another already-authorized owner/admin surface MAY revoke a lost device only if that surface already has the required protected-resource capability. Revocation is rechecked before every protected request and reconnect.

### 5.4 Mobile trust does not self-expand

A paired phone may request a broader trust tier, but the request is data. Protected expansion follows the existing Effect Gate and approval rules. A compromised mobile renderer cannot mint grants.

## 6. Native mobile product surface

The first-class mobile client MUST eventually support, under capability truth:

- session list, search, resume, new session, fork, and goal display;
- streaming assistant output and tool/activity cards;
- worker/task status, progress, blockers, completion and failure notifications;
- file/photo/document/voice-note upload;
- artifact viewing/download under policy;
- model/profile/status visibility without exposing secret material;
- approval cards with exact effect/risk/scope/expiry information;
- execution/privacy receipts;
- pause/stop/cancel and emergency-stop safety controls;
- Connect health, host online/offline state, current controller, active control lease and lease expiry;
- remote screen view and remote input only when the corresponding Spec 006/007 capabilities are implemented and granted;
- optional clipboard/file transfer as separately scoped capabilities;
- privacy mode for lock-screen/app-switcher previews;
- local encrypted cache bounded by retention policy;
- explicit sign-out/device-revoke controls.

The mobile UI MUST be a client of the canonical daemon/runtime. It MUST NOT own a private session history that becomes authoritative when it reconnects.

## 7. Mobile implementation posture

Spec 007 MUST qualify the exact mobile shell rather than choosing by fashion. The preferred architecture is to preserve a **Rust-owned protocol/crypto/state core** and a small platform UI/native-capability layer.

Tauri 2 is a serious candidate because it supports Android/iOS mobile projects and native Kotlin/Swift plugins while allowing shared Rust logic, which aligns with Golam's existing desktop direction. This amendment does not pre-admit Tauri mobile dependencies or force a WebView UI if platform evidence favors a small native Swift/Kotlin shell around a shared Rust core.

Required boundary regardless of UI framework:

```text
mobile UI / native OS integrations
          |
    narrow typed bridge
          |
Rust GolamConnect client core
identity / crypto / envelopes / replay / sync
          |
native transport
          |
golamd / privileged host enforcement
```

## 8. Push notifications: wake/sync only

APNs and FCM are convenience infrastructure, not the Golam message bus.

### Binding rule

Push payloads MUST be **content-minimized opaque wake/sync hints**. They MUST NOT carry:

- raw prompts or assistant answers;
- approval details or approval decisions;
- filenames or document contents;
- secret values/handles;
- screenshots;
- memory contents;
- effect parameters.

A push should mean approximately: `host/device X has new state; open the authenticated E2E GolamConnect channel and sync.`

Push delivery is best-effort and may be delayed, collapsed, reordered, or absent. Therefore canonical session/event correctness MUST never depend on push order or delivery.

Strict-local mode disables APNs/FCM and other external push services. On strict-local/LAN-only operation, the app may receive state while directly connected; otherwise it MUST show that background remote notification is unavailable rather than silently enabling cloud push.

Lock-screen notification content defaults to non-sensitive wording such as “Golam has an update” unless the user explicitly opts into a more revealing preview policy.

## 9. Voice and “talk to Golam”

### 9.1 Required early voice mode

Spec 007 mobile scope now includes:

- push-to-talk audio capture;
- sending/receiving voice notes;
- transcription through an allowed local or explicit remote ExecutionProfile;
- optional text-to-speech playback using an allowed local or explicit remote provider;
- interruption/cancel while audio is being recorded or generated.

### 9.2 Later real-time voice lane

Full-duplex “call Golam” mode may be delivered later in Spec 007 after the basic phone channel is reliable. It must reuse the same session, identity, policy and audit semantics; it is not a separate voice agent.

### 9.3 Voice is not authentication

Voice content MUST NOT satisfy a protected approval merely because the speaker says “approve”. Voice recognition is input, not owner authentication. Consequential approval requires the normal signed, exact-object approval flow and any required user-presence/step-up signal in the native app.

### 9.4 Microphone/camera boundary

User-initiated capture (“send this voice note/photo”) is distinct from autonomous sensor access. Agent-initiated microphone/camera remains deny-by-default under the constitution. No always-on background microphone or wake-word monitoring is a P0 requirement.

Audio and transcript retention is user-configurable and provenance/taint-aware. `SECRET_DERIVED` material remains ineligible for canonical long-term memory.

## 10. Messaging Channel Bridge contract

Every messaging provider MUST implement one common `ChannelAdapter` boundary and declare a capability descriptor. Provider-specific logic never bypasses the channel-binding or Effect Gate.

A future typed boundary MUST include equivalent responsibilities:

```text
ChannelAdapter
  describe_capabilities()
  start_ingress()
  stop_ingress()
  normalize_inbound(raw) -> ChannelEnvelope
  verify_ingress(raw, transport_context)
  send_message(effect_context, outbound)
  send_media(effect_context, outbound)
  acknowledge_if_required(...)
  reconcile_delivery(...)
```

### 10.1 Ingress modes

Adapters declare one or more exact ingress modes:

- `LOCAL_POLL` — host initiates outbound polling; no public callback required;
- `OUTBOUND_STREAM` — persistent outbound WebSocket/stream to provider;
- `PUBLIC_WEBHOOK` — provider calls a verified HTTPS endpoint;
- `SELF_HOSTED_RELAY` — user-operated internet-reachable relay terminates provider callback and forwards a signed/bounded envelope;
- `HOSTED_RELAY` — optional Golam-hosted relay in a later commercial/deployment mode, never required for local core.

Public webhook exposure MUST be a narrow dedicated ingress surface, not the authenticated local daemon control API.

### 10.2 Capability truth

Each adapter MUST declare, and the UI must honor:

- official API/product name and current qualified version/date;
- inbound/outbound availability;
- supported message/media types;
- stable sender/account/conversation identifiers;
- polling/stream/webhook modes;
- webhook verification/authentication method;
- delivery IDs, ordering, retry and dedupe semantics;
- edit/delete semantics;
- group semantics;
- interaction/button capability;
- reply-window/template restrictions if any;
- attachment limits;
- rate-limit behavior;
- public callback/relay requirement;
- metadata/privacy exposure;
- business/organization-account requirements;
- features that are unavailable or unverified.

Golam MUST NOT show a channel button, voice/file action, approval control, or delivery guarantee that the qualified provider path cannot actually honor.

## 11. Normalized ChannelEnvelope

Spec 007 MUST define a versioned channel envelope at least equivalent to:

```text
ChannelEnvelope
  channel_event_id
  provider
  provider_account_id
  sender_stable_id
  conversation_stable_id
  thread_or_reply_id?
  provider_message_id
  provider_update_or_sequence_id?
  provider_timestamp?
  host_received_at
  event_kind
  content_refs[]
  media_refs[]
  raw_payload_digest
  adapter_version
  binding_id?
  binding_generation?
  ingress_auth_evidence
  dedupe_key
  causality_ref?
  taint_labels[]
```

Raw provider payload retention MUST be minimized by policy. Large/media contents use governed artifact references rather than unbounded ledger blobs.

## 12. Identity and binding

The existing `channel-binding-contract.md` remains authoritative and is strengthened by these requirements:

- account binding requires explicit local user authorization;
- stable provider IDs are the authority key, never display names, aliases, usernames, phonebook labels, QR-profile names or avatars;
- each provider binding has an independently revocable generation;
- WhatsApp + Telegram + WeChat identities are NEVER inferred to be the same human;
- the owner MAY explicitly bind multiple channel accounts to one Golam principal, but each binding keeps independent provenance/trust/revocation state;
- group participants have no owner authority merely because the owner added the bot to the group;
- a group/conversation may be explicitly scoped to allowed commands/resources without widening participant identity;
- channel account compromise is handled by revoking that binding, not by rotating the native mobile device identity.

## 13. Approval semantics on phone and channels

### Native Mobile

Native Mobile MAY satisfy an approval only when the phone currently holds the required approval capability and returns a signed approval response bound to the exact object:

- `approval_id`;
- `effect_id` and effect digest;
- risk class;
- allowed scope/limits;
- expiry/freshness;
- one-time nonce/use counter;
- mobile device ID and current lease generation;
- user-presence/step-up evidence where policy requires it.

The kernel revalidates everything at execution time.

### Messaging bridges

A free-form message such as `yes`, `ok`, `approve`, an emoji, a voice note, or a reaction MUST NOT authorize a consequential effect.

Interactive provider buttons MAY carry a reference to an approval request, but bridge approval is disabled for protected-resource, secret-release, local-computer-control escalation, irreversible, financial, destructive, or similarly high-risk classes by default. Those requests step up to Native Mobile or an authenticated local client.

Any future channel-approval class must be explicitly added by Spec 007 policy design, scoped to low/medium-risk reversible work, and tested for replay, delayed delivery, stale binding generation and identity spoofing.

Safety-reducing commands such as **pause**, **stop**, **cancel pending work**, or **release remote control** may receive a deliberately easier policy path than authority-expanding actions, but still require an authenticated/bound source as defined by the owning spec.

## 14. Offline host, delayed delivery and queued intent

The phone and messaging provider may be online while the Golam host is offline. This MUST NOT create a cloud copy of Golam authority.

Native Mobile MAY queue a locally signed **request intent** with:

- intent ID;
- target host;
- creation time;
- TTL/expiry;
- expected binding/device generation;
- requested action description;
- no assumption that it was authorized or executed.

When the host returns, it authenticates, checks freshness/revocation/current state, and processes the request as a new canonical input. The UI must distinguish `QUEUED_ON_PHONE`, `DELIVERED_TO_HOST`, `AUTHORIZED`, `EXECUTING`, and `COMPLETED` rather than implying execution while offline.

Provider-delayed messages behave similarly. A stale/delayed “yes” never revives an expired approval.

## 15. Message edits, deletion and replay

- Provider edits create a new revision event referencing the original message; they do not rewrite canonical history already used to make a decision.
- Provider deletion may trigger privacy minimization/tombstoning for retained message content, but cannot erase mandatory effect/audit evidence already required for accountability.
- Duplicated or retried provider events are deduplicated by provider event/message/sequence semantics plus Golam's own idempotency key.
- Cross-channel replay never preserves authority.
- Outbound messages carry internal causality metadata where possible so bot/provider echoes and bridge loops can be detected.
- A bridge hop limit and origin/causality checks prevent Telegram -> Slack -> Telegram or bot-to-bot infinite loops.

## 16. Attachments, photos, files and media

Inbound media is a supply-chain/input boundary:

1. verify transport/provider event;
2. enforce configured size/type limits before full acquisition where possible;
3. stream into quarantine rather than a trusted project path;
4. compute content hash and media metadata;
5. malware/file-structure inspection where applicable;
6. assign channel/user-input provenance and taint;
7. decode/transcribe only in the allowed sandbox/profile;
8. require normal capability/effect checks before copying into a project or sending onward;
9. never auto-execute a received script/archive/document macro;
10. never promote attachment claims to durable memory merely because they came from the owner channel.

## 17. Provider baseline and implementation posture

### Telegram — first reference adapter

Use the official Bot API. Spec 007 should prefer `LOCAL_POLL` during local-first bootstrap because `getUpdates` works without a public inbound endpoint; webhook mode is optional and must verify Telegram's secret-token header. `update_id` is used in dedupe/order recovery. Telegram currently exposes rich messages, media and voice-note capabilities; exact Bot API behavior is requalified at implementation time.

### WhatsApp — official business platform only

Core support means the official **WhatsApp Business Platform / Cloud API**, not scraping WhatsApp Web, browser session automation, reverse-engineered personal-account protocols, or an unofficial consumer-account dependency.

The official Cloud API currently requires Meta business assets including a WhatsApp Business Account/business phone number and uses webhooks for inbound events. Therefore Spec 007 must explicitly solve public callback/relay deployment, webhook verification, WABA subscription, stable business/sender identifiers, message ID/status reconciliation, media handling and current conversation/template restrictions.

If a user only has a personal WhatsApp account and Meta provides no sanctioned bot path for that use case, Golam MUST say it is unsupported rather than silently using an unofficial automation stack.

### WeChat ecosystem — official paths only

Baseline support targets officially sanctioned **WeCom intelligent robot/application** and, if qualified, WeChat Official Account/business integration APIs. Current research evidence shows WeCom intelligent bots support encrypted callback-based interaction including text, mixed rich content, image, voice, file, video and quoted messages, plus bounded response URLs for some delayed replies. These details MUST be reverified against the current official developer source during Spec 007 because official-site accessibility and regional/product variations can change.

Personal consumer WeChat account automation is NOT a core dependency and remains unsupported until a current official compliant API explicitly permits the intended mode.

### Slack

Support official Slack app APIs. Prefer Socket Mode where its product/distribution constraints are acceptable because it allows Events API/interactivity over an outbound WebSocket without a public Request URL. HTTP Events API remains another mode and must implement event dedupe/retry/auth verification. Exact OAuth scopes are least-privilege and workspace-specific.

### Discord

Support an official bot/application path. The Gateway provides a persistent WebSocket event stream with sequence/resume semantics; intents and privileged message-content access must be capability-truthed and least-privilege. Webhooks are outbound-notification tools, not a substitute for a fully authenticated inbound bot identity.

### Matrix

Matrix is a valuable open/self-hostable adapter candidate. Application Services provide a standard bridge/gateway interface with explicit tokens, namespaces and transaction IDs. It is not P0 ahead of native mobile + Telegram + WhatsApp/WeCom, but it is strategically important for users who want an open channel they can host themselves.

### Signal / iMessage / SMS / RCS / other channels

No channel is added through unofficial account automation merely for parity. Spec 007 must verify a sanctioned provider/developer path before claiming support. If only a business messaging service exists, Golam labels that scope accurately. Generic SMS/RCS providers, enterprise chat and additional regional channels can be added through the same adapter contract once Source Foundry/security/product qualification closes.

## 18. Third-party relay and webhook ingress

The native GolamConnect relay and third-party messaging callback relay are separate concerns.

For provider webhooks:

- the exposed receiver must contain only health/challenge/provider-hook endpoints required for configured adapters;
- it must not expose the local Golam control API;
- provider authentication/signature/token verification happens before event normalization;
- replay/dedupe occurs before task creation;
- raw content is treated as hostile input;
- relay infrastructure cannot mint bindings or approvals;
- hosted relay is optional and must not be required for local core;
- self-hosted relay and tunnel choices are explicit deployment configuration, not hidden dependencies.

## 19. Integration with workers and automations

Phone/channel input enters the **same runtime**:

```text
ChannelEnvelope / ConnectEvent
        -> canonical SessionEvent
        -> goal/task/run
        -> harness/worker
        -> EffectIntent
        -> kernel authorization
        -> result/receipt
        -> optional outbound message effect
```

A channel adapter never runs an alternate agent loop. Workers do not own provider tokens. Scheduled messages and worker notifications are normal external effects with destination, content sensitivity, taint, idempotency and reconciliation semantics.

Incoming webhook/channel events may become Spec 008 triggers only through typed, policy-bound trigger rules. Receiving an arbitrary message is not permission to execute a standing high-risk automation.

## 20. Proposed logical data-model additions

These are future Spec 007 logical entities, not an implementation schema migration now.

### MobileDeviceProfile

- `device_id`
- `platform`: ios | android
- `app_instance_id`
- `device_key_id`
- `trust_tier`
- `capabilities[]`
- `push_provider?`
- `push_token_handle?`
- `notification_privacy_policy`
- `paired_at`
- `last_authenticated_at?`
- `generation`
- `revoked_at?`

### ChannelAdapterDescriptor

- `provider`
- `adapter_version`
- `official_api_identity`
- `qualified_at`
- `ingress_modes[]`
- `supports_inbound`
- `supports_outbound`
- `message_types[]`
- `media_types[]`
- `stable_identity_fields[]`
- `verification_method`
- `delivery_semantics`
- `edit_delete_semantics`
- `group_semantics`
- `rate_limit_profile`
- `public_callback_required`
- `privacy_metadata_exposure`
- `unsupported_features[]`

### ChannelEventRecord

Fields correspond to the normalized `ChannelEnvelope` plus canonical event/ref and dedupe/reconciliation state.

### MobileApprovalResponse

- `approval_id`
- `effect_digest`
- `device_id`
- `lease_id`
- `lease_generation`
- `decision`
- `scope_digest`
- `nonce`
- `issued_at`
- `signature`
- `user_presence_evidence?`

### PushWakeRecord

- `wake_id`
- `device_id`
- `reason_class`
- `collapse_key?`
- `requested_at`
- `provider`
- `provider_delivery_id?`
- `status`
- `contains_sensitive_content`: MUST be false

## 21. New binding functional requirements

- **FR-PHONE-001**: Golam MUST provide a first-class iOS/Android mobile client under Spec 007 planning and qualification.
- **FR-PHONE-002**: Mobile MUST use native GolamConnect cryptographic device pairing; messaging account possession is insufficient for native trust.
- **FR-PHONE-003**: Mobile session/task/worker state MUST project from the same canonical local daemon state used by CLI/Desktop.
- **FR-PHONE-004**: Push infrastructure MUST be optional, content-minimized and non-authoritative; canonical correctness cannot depend on push delivery/order.
- **FR-PHONE-005**: Strict-local mode MUST disable third-party channels and cloud push and present the limitation clearly.
- **FR-PHONE-006**: Mobile approvals MUST bind to an exact approval/effect/scope/expiry/nonce/device generation and be revalidated host-side.
- **FR-PHONE-007**: Free-form messaging/voice/reaction content MUST NOT satisfy consequential approval.
- **FR-PHONE-008**: Early mobile voice MUST support push-to-talk/voice-note interaction; voice is input, not identity.
- **FR-PHONE-009**: Agent-initiated camera/microphone access remains deny-by-default and separate from user-initiated attachment capture.
- **FR-CHANNEL-001**: All messaging adapters MUST implement a common normalized envelope, capability descriptor, stable identity binding, taint and Effect Gate path.
- **FR-CHANNEL-002**: Core WhatsApp support MUST use a current official WhatsApp Business Platform path; unofficial WhatsApp Web/personal-account automation is not a core dependency.
- **FR-CHANNEL-003**: Core WeChat-family support MUST use qualified official WeChat/WeCom integration APIs; unsupported consumer-account automation is not silently substituted.
- **FR-CHANNEL-004**: Provider event duplicates, retries, edits, deletion and out-of-order delivery MUST be explicitly modeled and tested.
- **FR-CHANNEL-005**: Public webhook receivers MUST be narrow dedicated ingress surfaces and MUST NOT expose local Golam control authority.
- **FR-CHANNEL-006**: Cross-channel identity equivalence MUST require explicit local binding; it is never inferred.
- **FR-CHANNEL-007**: Inbound attachments MUST enter quarantine with hash/provenance/taint and MUST NOT auto-execute.
- **FR-CHANNEL-008**: Outbound channel messages, media and notifications MUST be normal auditable EffectIntents with delivery reconciliation where the provider supports receipts/status.
- **FR-CHANNEL-009**: Channel bridge loops MUST be bounded by causality/origin/dedupe/hop-limit rules.
- **FR-CHANNEL-010**: Each adapter MUST publish an honest capability/privacy matrix and fail explicitly for unsupported provider features.

## 22. Success criteria

- **SC-PHONE-001**: A paired iOS/Android client resumes a canonical session after mobile app restart and host reconnect without creating divergent history.
- **SC-PHONE-002**: Lost-device revocation blocks protected requests and reconnect on every tested race boundary.
- **SC-PHONE-003**: Push payload inspection proves no prompts, answers, filenames, secrets, approval details, screenshots or memory contents reach APNs/FCM.
- **SC-PHONE-004**: A delayed/reordered/collapsed push causes only resync and never corrupts canonical state.
- **SC-PHONE-005**: Voice can request work but cannot bypass an approval step in adversarial tests.
- **SC-PHONE-006**: Mobile pause/stop/emergency-stop acts through current lease/policy state and defeats stale controller input.
- **SC-CHANNEL-001**: Telegram duplicate/out-of-order update corpus produces one canonical input effect per intended message.
- **SC-CHANNEL-002**: WhatsApp/WeCom webhook authentication, replay, duplicated delivery and stale callback tests fail closed.
- **SC-CHANNEL-003**: Spoofed display names, recycled usernames/handles, group impersonation and cross-channel replay never inherit owner authority.
- **SC-CHANNEL-004**: Edited/deleted messages cannot rewrite already-committed effect/audit history.
- **SC-CHANNEL-005**: Media fuzz/quarantine tests prove received attachments cannot escape into trusted execution without the normal gates.
- **SC-CHANNEL-006**: Provider outage/rate-limit/retry tests do not duplicate outbound irreversible or externally visible effects.
- **SC-CHANNEL-007**: Strict-local external observation proves mobile cloud push and third-party messaging egress are disabled.

## 23. Spec 007 implementation lanes

The future Spec 007 package should be decomposed into evidence-gated lanes instead of one giant feature drop.

### 007A — Connect Core and Device Identity

- exact Iroh/transport qualification;
- native device identity/pairing/revocation;
- signed encrypted envelopes;
- replay/dedupe/generation;
- reconnect/resume;
- narrow relay metadata model;
- device capability leases.

**Exit gate**: two-device authenticated message/resume/revoke tests; no screen/control yet required.

### 007B — Golam Mobile

- mobile-shell architecture qualification;
- shared Rust protocol client core;
- iOS/Android pairing and secure key storage;
- session/task/worker projections;
- push wake/sync with privacy tests;
- mobile approvals;
- file/photo/voice-note upload;
- pause/stop safety controls.

**Exit gate**: useful daily chat/task management from both iOS and Android, with zero authority in push infrastructure.

### 007C — Channel Bridge Core + Telegram

- `ChannelAdapterDescriptor` + `ChannelEnvelope`;
- binding/revocation/generation UI;
- `LOCAL_POLL` and narrow webhook ingress;
- Telegram official Bot API adapter;
- dedupe/edit/delete/group/media tests;
- outbound Effect Gate + delivery receipt model.

**Exit gate**: Telegram can request/observe work without becoming authority; adversarial identity tests pass.

### 007D — WhatsApp Business + WeChat/WeCom + Slack/Discord/Matrix

- official WhatsApp Business Cloud API path;
- webhook/relay deployment contract;
- WABA/message-status/media semantics;
- official WeCom/WeChat sanctioned path requalification;
- Slack Socket Mode/Events and Discord Gateway candidates;
- Matrix Application Service optional open/self-hostable bridge;
- capability/privacy matrix for every admitted adapter.

**Exit gate**: each claimed adapter has current official-source qualification and independent scenario/security evidence.

### 007E — Remote Control + Rich Voice

Depends on Spec 006 computer-control readiness:

- remote screen/media;
- input authority lease;
- multi-monitor;
- clipboard/file transfer;
- visible host indicator;
- human takeover/emergency stop;
- full-duplex voice if justified;
- mobile remote-control UX and network-loss recovery.

**Exit gate**: two-machine/phone computer-control safety matrix and GolamBench remote-control gates pass on every claimed platform.

## 24. Relationship to Spec 008 and Spec 010

Spec 008 consumes the trusted inputs established by Spec 007:

- channels may create typed trigger events;
- workers may send channel notifications through effects;
- schedules may target mobile/channel notification surfaces;
- unattended work cannot expand channel/mobile approval authority.

Spec 010 adds release gates for:

- mobile pairing/revocation/reconnect;
- push privacy and strict-local no-egress;
- channel impersonation/replay/outage/rate-limit behavior;
- voice approval bypass attempts;
- remote-control phone takeover/emergency stop;
- mobile app platform support matrix.

## 25. Threat-model minimum

The owning spec MUST test at least:

- stolen/lost paired phone;
- extracted/replayed mobile envelope;
- stale lease generation;
- malicious mobile renderer against Rust client boundary;
- forged push notification / push token theft;
- lock-screen content leakage;
- notification reorder/collapse/duplicate;
- spoofed/recycled channel display name/handle;
- compromised bound messaging account;
- forged webhook signature/token;
- duplicate/out-of-order provider delivery;
- stale button/callback/reply URL;
- delayed “approve” after approval expiry;
- malicious group participant prompt injection;
- message edit after task dispatch;
- provider deletion after effect execution;
- attachment parser bomb/malware/archive traversal;
- channel-to-channel echo loop;
- bot-to-bot recursive loop;
- provider outage/rate limiting;
- public webhook scanning/DoS;
- malicious/self-hosted relay;
- strict-local accidental adapter/push egress;
- voice spoof/replay attempting approval;
- microphone/camera permission confusion;
- remote-control reconnect racing human takeover.

## 26. Explicit non-goals / rejected shortcuts

- No unofficial WhatsApp Web browser/session scraping as core support.
- No unsupported personal WeChat automation as core support.
- No pairing a native Golam device by “message this code to the bot”.
- No cloud service holding Golam's policy/approval/private device authority.
- No plaintext sensitive push payloads.
- No free-form chat reply as high-risk approval.
- No always-on background microphone/wake-word requirement in the initial mobile scope.
- No hidden channel fallback in strict-local mode.
- No remote control through Telegram/WhatsApp/WeChat pixels or arbitrary bot commands; native GolamConnect remains the protected control plane.
- No separate mobile/channel memory or agent runtime.

## 27. Research anchors inspected for this amendment

Public provider/platform research was refreshed on 2026-08-28. These are evidence anchors, not permanently pinned implementation dependencies; Spec 007 must requalify current behavior at implementation time.

- Telegram Bot API: `https://core.telegram.org/bots/api`
- Meta WhatsApp Business Platform / Cloud API official Postman collection: `https://www.postman.com/meta/whatsapp-business-platform/documentation/wlk6lh4/whatsapp-cloud-api`
- Meta WhatsApp webhooks/messages collections under the same official workspace.
- WeCom official-source paths referenced by current mirrors: `https://developer.work.weixin.qq.com/document/path/100719`, `.../101138`, and `.../90236` — implementation MUST revisit the official current source directly.
- Slack Events API / Socket Mode: `https://docs.slack.dev/apis/events-api/` and `/using-socket-mode/`
- Discord Gateway: `https://docs.discord.com/developers/events/gateway`
- Matrix Application Service API: `https://spec.matrix.org/`
- Apple APNs: `https://developer.apple.com/documentation/usernotifications/`
- Firebase Cloud Messaging: `https://firebase.google.com/docs/cloud-messaging`
- Tauri 2 mobile/plugin documentation: `https://v2.tauri.app/develop/plugins/develop-mobile/`

## 28. Review and merge gate

This amendment becomes canonical only after review and merge. Before merge:

1. verify it does not weaken the current Spec 003 boundary or change its implementation task order;
2. verify existing Spec 001 channel/GolamConnect/authority contracts remain monotonic;
3. verify `tasks.md` no longer contradicts the new founder scope by deferring native mobile/voice;
4. verify no product code/dependency/schema changes are included;
5. record exact PR head and review evidence under normal Golam governance.

After merge, future agents MUST read this amendment before planning Spec 007. The next active implementation action remains whatever is authorized by live canonical Spec 003; PA-001 does not leapfrog the program sequence.