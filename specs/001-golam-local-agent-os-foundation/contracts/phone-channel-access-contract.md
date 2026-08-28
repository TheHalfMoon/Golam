# Contract: Phone, Mobile, Voice, and Messaging-Channel Access

**Authority note**: This contract is an additive Spec 001 program contract introduced by `program-amendments/PA-001-phone-channel-access.md`. It defines future requirements; it does not authorize implementation inside the active Spec 003 scope.

## 1. One Golam, multiple surfaces

Native mobile, Desktop, CLI/TUI, IDE, workers, and messaging channels MUST project from the same canonical local daemon/session/effect state. A surface MUST NOT fork its own authoritative agent runtime, memory, approvals, or effect history.

## 2. Native mobile is a GolamConnect device

A native iOS/Android Golam client is an authenticated GolamConnect `Device`, not a messaging channel.

- Pairing requires a local user-authorized elevated effect.
- Pairing establishes a device public key, owner principal, trust tier, bounded capabilities, generation and revocation state.
- Private device key material remains on the phone and uses platform-protected storage/user-presence controls where available.
- Possession of a Telegram/WhatsApp/WeChat/Slack/Discord account MUST NOT bootstrap native device trust.
- Reconnect reauthenticates and revalidates pairing, revocation, lease expiry and generation.
- Device grants can only narrow or be explicitly elevated through protected authority mutation; the phone cannot self-expand authority.

## 3. Native mobile approval object

A mobile approval MUST be cryptographically bound to the exact pending operation. At minimum the signed response binds:

- `approval_id`;
- `effect_id` and canonical effect digest;
- allowed action/resource scope and quantitative bounds;
- risk class;
- expiry/freshness;
- nonce/use counter;
- device identity;
- lease identity/generation;
- decision;
- user-presence/step-up evidence when required by policy.

Host-side authorization is re-evaluated at execution. A valid mobile signature does not override monotonic safety denial, strict-local denial, revoked/expired state, stale generation or changed effect parameters.

## 4. Messaging channels are untrusted transports

Telegram, WhatsApp, WeChat/WeCom, Slack, Discord, Matrix and later adapters are channel transports.

- Inbound content remains channel-tainted even from a bound owner account.
- A channel account maps to a Golam principal only via explicit local binding using provider-stable identifiers.
- Usernames, display names, aliases, phonebook labels, avatars and similar human-readable identity are never authority keys.
- Group/unbound participants hold zero machine authority by default.
- Cross-channel identity equivalence is never inferred.
- Binding creation, privilege change, unbinding and revocation are protected audited effects.
- Binding generation/revocation is rechecked before consequential work.

## 5. Channel approval prohibition by default

Free-form channel content MUST NOT satisfy a consequential approval. This includes:

- `yes`, `ok`, `approve`, or equivalent text;
- emoji/reactions;
- voice/audio content;
- quoted/replied messages;
- display-name changes;
- provider-side message edits.

Provider interactive buttons MAY reference a pending approval object, but high-risk classes—including protected authority mutation, secret release, destructive/irreversible effects, financial/external-account consequences, and local-computer-control escalation—MUST step up to Native Mobile or an authenticated local client unless a later reviewed policy contract explicitly proves a narrower safe class.

## 6. Safety-reducing actions

`pause`, `stop`, `cancel`, `release_control`, and emergency-stop actions MAY be assigned a lower-friction policy path because they reduce active authority/risk. The owning spec must still define which authenticated/bound surfaces may issue them and must prevent a denial-of-service-prone unauthenticated path from becoming the default.

## 7. Versioned ChannelEnvelope

Every inbound provider event is normalized before entering the canonical runtime. The envelope MUST carry at least:

- provider and adapter version;
- provider account identity;
- stable sender identity;
- stable conversation identity;
- provider message/event ID;
- provider sequence/update ID when available;
- provider timestamp when available and host receipt time;
- event/message kind;
- content/media artifact refs;
- raw payload digest;
- ingress authentication evidence;
- binding ID and generation when bound;
- dedupe key;
- causality/origin reference when available;
- taint/provenance labels.

Normalization never grants authority. Provider payloads remain hostile input.

## 8. Provider capability truth

Every admitted `ChannelAdapter` MUST publish a current capability descriptor covering:

- official API/product and qualification date;
- ingress mode(s);
- stable identity fields;
- authentication/signature/token method;
- inbound/outbound message/media types;
- groups/threads/replies/edits/deletes;
- interaction/buttons;
- ordering/retry/dedupe semantics;
- delivery/status receipts;
- rate limits;
- reply/template/time-window restrictions where applicable;
- public webhook or relay requirement;
- privacy/metadata exposure;
- business/organization-account requirements;
- unsupported/unverified capabilities.

UI and runtime MUST NOT advertise unsupported provider capabilities.

## 9. Supported ingress classes

An adapter may implement one or more declared ingress classes:

- `LOCAL_POLL`;
- `OUTBOUND_STREAM`;
- `PUBLIC_WEBHOOK`;
- `SELF_HOSTED_RELAY`;
- `HOSTED_RELAY`.

A public webhook receiver MUST be a narrow dedicated provider ingress surface. It MUST NOT expose local IPC/control routes or become an alternate `golamd` control API.

Provider authentication is verified before normalization. Dedupe/replay control happens before task/effect creation.

## 10. Official-provider-only baseline

- Telegram core support uses the official Telegram Bot API.
- WhatsApp core support uses the official WhatsApp Business Platform/Cloud API or another later officially sanctioned path. Unofficial WhatsApp Web session scraping/personal-account automation is not a core dependency.
- WeChat-family core support uses current officially sanctioned WeChat/WeCom integration APIs. Personal consumer-account automation is not silently substituted when no compliant API exists.
- Slack and Discord use official application/bot APIs.
- Matrix may use its standardized Application Service/client APIs as an open/self-hostable bridge.
- Additional channels require current provider/developer-path qualification before support is claimed.

## 11. Push notification boundary

APNs/FCM or later push services are wake/sync hints only.

Push payloads MUST NOT include sensitive Golam content such as prompts, assistant text, filenames, screenshots, memory, secret data, or approval/effect details.

Push delivery MUST NOT be treated as ordered, durable or authoritative. Mobile opens an authenticated end-to-end GolamConnect channel and reconciles from canonical state.

Strict-local mode disables external push providers. Lock-screen preview defaults to content-minimized wording.

## 12. Voice boundary

- Push-to-talk and voice-note input are required phone-access capabilities once implemented.
- Voice recognition is content input, not identity proof.
- Voice MUST NOT bypass approval/step-up requirements.
- User-initiated media capture is distinct from autonomous sensor access.
- Agent-initiated microphone/camera access remains deny-by-default.
- No always-on background microphone is required by the initial phone scope.
- Audio/transcript retention obeys user policy, taint and secret/memory rules.

## 13. Attachments and media

Inbound attachments MUST enter a quarantine pipeline before trusted use:

`transport verification -> limits -> quarantine -> hash -> metadata/inspection -> taint -> sandboxed decode/transcribe -> governed copy/use`

Received executable content, archives, macros, scripts or documents MUST NOT auto-execute. Attachment contents do not become durable truth merely because the sender is bound.

## 14. Offline/delayed input

A mobile client may queue a signed **request intent**, not an authorized effect, while the host is offline. The intent carries target, creation time, TTL, device/binding generation and requested operation. When the host reconnects it performs fresh authentication, policy, approval and state checks.

Channel-provider queueing/delivery delay follows the same principle. Delayed content never revives an expired approval or stale lease.

## 15. Edit/delete/replay semantics

- Message edit creates a revision event; it never rewrites history already consumed by a task/effect.
- Provider deletion may minimize/tombstone retained content according to privacy policy but cannot erase mandatory effect/audit evidence.
- Retried/duplicated deliveries are deduplicated with provider IDs/sequences plus Golam idempotency state.
- Cross-channel replay preserves provenance, not authority.
- Outbound and inbound bridge events carry causality/origin metadata where feasible and enforce a hop limit to stop loops.

## 16. Outbound messages are effects

Sending a text, file, image, audio item, reaction, card, notification or other externally visible channel action is a normal `EffectIntent`.

The effect records destination, account/adapter, sensitivity/taint, authorization, approval if required, idempotency semantics, provider message ID/status where available, delivery reconciliation and receipt evidence.

A worker, model, skill or channel adapter never owns provider credentials directly when a brokered boundary is possible.

## 17. Worker/scheduler integration

Inbound channel/phone events may become tasks/triggers only through typed policy-bound rules. Scheduled or worker-generated outbound notifications use the same Effect Gate. Unattended work does not inherit broader authority merely because its trigger originated from a bound owner channel.

## 18. Remote-control separation

Third-party messaging channels are not the native remote-control transport.

Screen/media, keyboard/mouse/touch input, clipboard, files, multi-monitor and control arbitration run through authenticated GolamConnect and the computer-control capability/lease model. Human takeover suspends conflicting controller/agent input at the lease layer.

## 19. Required adversarial verification

The owning specs MUST cover at minimum:

- lost/stolen/revoked phone;
- mobile replay/stale generation;
- malicious mobile renderer;
- forged/duplicate/reordered push;
- lock-screen leakage;
- forged provider webhook;
- provider replay/duplicate/out-of-order events;
- spoofed/recycled usernames and display names;
- group impersonation;
- cross-channel replay;
- delayed/stale approval attempts;
- message edit/delete after dispatch;
- malicious attachment/parser/archive input;
- provider rate-limit/outage;
- channel echo/recursive bot loop;
- strict-local unexpected channel/push egress;
- voice replay/spoof attempting approval;
- camera/microphone capability confusion;
- reconnect racing remote-control takeover/emergency stop.

## 20. Release claim rule

Golam may claim a phone/platform/channel capability only when the exact adapter/mobile build and its identity, privacy, delivery, failure/recovery and security scenarios have reproducible exact-head evidence. “Configured” or “connected” is not equivalent to “safe”, “delivered”, “approved”, or “executed”.