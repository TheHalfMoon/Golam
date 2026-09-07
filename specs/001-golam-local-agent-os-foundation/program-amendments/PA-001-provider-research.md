# PA-001 Provider and Platform Research — Phone / Mobile / Channels

**Research date**: 2026-08-28  
**Purpose**: evidence inputs for PA-001 and future Spec 007.  
**Admission status**: REFERENCE / PLANNING EVIDENCE ONLY. No SDK, crate, service, or source code is admitted by this file. All exact dependencies and APIs must be requalified at Spec 007 implementation time.

## Research conclusions

1. **Native Golam Mobile should be the high-assurance phone surface.** It can use GolamConnect cryptographic device identity, signed envelopes and host-side capability leases instead of outsourcing authority to a chat provider.
2. **Third-party channels should share one adapter contract but retain provider-specific truth.** Telegram can work via outbound long polling; WhatsApp Cloud API needs Business Platform assets and webhook infrastructure; Slack offers outbound Socket Mode; Discord offers a resumable Gateway; Matrix offers an open application-service bridge; WeCom currently exposes rich robot/application paths.
3. **Cloud push cannot be the mobile state transport.** APNs and FCM are best-effort, can delay/reorder/collapse notifications, and FCM explicitly states its data transport is not end-to-end encrypted by default. Golam should push only opaque wake/sync hints and fetch canonical state over E2E GolamConnect.
4. **Official APIs only.** Personal WhatsApp Web scraping and unsupported personal WeChat account automation would create fragile security, account-policy, provenance and maintenance dependencies and are excluded from the core design.
5. **Tauri 2 mobile is architecturally plausible but not pre-admitted.** Current Tauri documentation supports Android/iOS projects and Kotlin/Swift native mobile plugins while sharing Rust code. Spec 007 should benchmark it against a smaller native shell around a shared Rust core before freezing the mobile UI stack.

---

## Telegram Bot API

**Source**: `https://core.telegram.org/bots/api`  
**Source status**: OFFICIAL  
**Observed current version**: Bot API 10.2, 2026-07-14.

Relevant current behavior:

- Telegram defines the Bot API as an HTTPS HTTP interface for bots.
- Two mutually exclusive inbound modes are documented: `getUpdates` long polling or webhooks.
- Incoming updates are retained by Telegram for no longer than 24 hours.
- `update_id` is a unique update identifier intended for duplicate suppression and sequence recovery.
- `setWebhook` supports a `secret_token`; Telegram sends it in `X-Telegram-Bot-Api-Secret-Token`.
- Current API supports text/media and voice-note/rich-message functions.
- Telegram also publishes a local Bot API server implementation, but using it does not make Telegram itself a strict-local transport.

### Golam implications

- **Recommended first bridge** because `LOCAL_POLL` avoids a public callback surface during early local-first development.
- Store the highest safely processed `update_id`/dedupe state durably before side effects.
- Webhook mode must authenticate the secret header and separately dedupe/replay-check the update.
- Provider user/chat IDs are identity inputs; usernames/display names are not authority.
- Telegram-hosted content and metadata mean this bridge is never marketed as strict-local.

---

## WhatsApp Business Platform / Cloud API

**Sources**:

- `https://www.postman.com/meta/whatsapp-business-platform/documentation/wlk6lh4/whatsapp-cloud-api`
- official Meta WhatsApp Business Platform Postman `Webhooks`, `Webhook Subscriptions`, and `Messages` collections.

**Source status**: OFFICIAL META WORKSPACE / current planning evidence.

Relevant current behavior:

- Cloud API is Meta-hosted and is described as the official WhatsApp Business Platform API.
- Getting started requires a Meta business portfolio, a WhatsApp Business Account (WABA), and a business phone number.
- Inbound events are delivered through webhooks; applications subscribe to the WABA to receive webhook events for phone numbers under the account.
- `/PHONE_NUMBER_ID/messages` sends text and media including audio/documents/images/video/templates.
- Messages have unique provider IDs and delivery/status changes can be observed via webhooks.

### Golam implications

- Core support is **WhatsApp Business**, not consumer-account automation.
- A local-only daemon cannot receive Meta webhooks from the public internet without an explicit callback/relay/tunnel deployment choice.
- The callback receiver should be a narrow channel-ingress service, never the local control API.
- Webhook request authenticity/signature details, current Graph API version, WABA identity fields, sender identity fields, message-window/template policy, media limits and rate limits MUST be reverified against current Meta docs when Spec 007 is implemented.
- Provider message IDs/statuses belong in delivery reconciliation; `send` success is not equivalent to recipient delivery.
- If a user has only a personal WhatsApp account and Meta provides no sanctioned bot path for Golam's use case, Golam reports that limitation instead of installing an unofficial WhatsApp Web/session scraper.

---

## WeChat / WeCom ecosystem

Official developer source URLs identified during research:

- WeCom intelligent bot receive-message source: `https://developer.work.weixin.qq.com/document/path/100719`
- WeCom intelligent bot active-reply source: `https://developer.work.weixin.qq.com/document/path/101138`
- WeCom application-message source: `https://developer.work.weixin.qq.com/document/path/90236`

**Source status**: PARTIALLY_VERIFIED_CURRENT_REFERENCE. The official developer host was not reliably retrievable in this research environment; current mirrors that preserve the official `source` URL and update date were cross-checked. Future Spec 007 MUST revisit the current official source directly before claiming behavior.

Current mirrored official-source evidence indicates:

- WeCom intelligent robot interactions can be delivered as encrypted callbacks.
- Documented callback interaction types include text, rich mixed content, image, voice, local file, video and quoted messages in supported one-to-one/group scenarios.
- Some callbacks include a `response_url` for active replies; the mirrored 2026-03-24 documentation states a response URL is single-use and valid for one hour.
- WeCom application messaging supports text, image, video, file and rich/news-style messages through the official server API.

### Golam implications

- Target **WeCom intelligent robot/application** first and evaluate WeChat Official Account/business APIs separately.
- Callback encryption/authentication must terminate in a narrow untrusted adapter boundary before normalization.
- Temporary/single-use reply URLs are capabilities and must be handled as secrets/ephemeral delivery handles, never stored in model-visible memory.
- Personal consumer WeChat account automation is not a core path unless a current official compliant API is verified.

---

## Slack Events API / Socket Mode

**Sources**:

- `https://docs.slack.dev/apis/events-api/`
- `https://docs.slack.dev/apis/events-api/using-socket-mode/`

**Source status**: OFFICIAL.

Relevant current behavior:

- Events API apps choose Socket Mode or an HTTP Request URL.
- Socket Mode provides Events API/interactivity over a WebSocket without exposing a public HTTP Request URL.
- Socket Mode uses an app-level token to obtain a temporary WebSocket URL; connections refresh and clients must handle reconnects.
- Socket Mode payloads include an `envelope_id` and must be acknowledged.
- HTTP Events API events include an `event_id`; Slack recommends acknowledging HTTP events quickly and queueing actual processing because failed delivery is retried.
- OAuth scopes bound what events the app can observe.

### Golam implications

- Socket Mode is attractive for local installations because it is outbound-initiated, but its distribution/Marketplace constraints and token model must be evaluated for Golam's product mode.
- Event ID/envelope ID are dedupe/ack inputs, not authorization.
- OAuth scopes should be generated from least-privilege requested Golam capabilities.

---

## Discord Gateway

**Source**: `https://docs.discord.com/developers/events/gateway`  
**Source status**: OFFICIAL.

Relevant current behavior:

- Discord Gateway is a secure WebSocket event stream for apps/bots.
- Clients select intents that bound event classes.
- Dispatch events carry sequence numbers.
- Gateway connections require heartbeats and can reconnect/resume using session state; after successful resume, missed events can be replayed from the last sequence.
- Some intents are privileged, including message-content-related access in applicable cases.

### Golam implications

- Treat Gateway sequence/resume as provider delivery semantics and still maintain Golam canonical dedupe/idempotency.
- Request the minimum intents needed for configured channel behaviors.
- Discord webhooks are useful for outbound notifications but are not a complete authenticated inbound-bot replacement.

---

## Matrix Application Service API

**Source**: `https://spec.matrix.org/` Application Service API  
**Source status**: OFFICIAL OPEN SPEC.

Relevant current behavior:

- Matrix Application Services provide a standard bridge/extensibility boundary.
- Registration uses explicit namespaces plus `as_token` and `hs_token` credentials.
- Homeserver -> Application Service requests are authenticated with a Bearer token.
- Event delivery uses transaction IDs, which are useful for idempotent delivery handling.
- Application Services can observe configured namespaces and inject events using the Matrix APIs.

### Golam implications

- Strategically useful for an open/self-hostable messaging option.
- Application-service registration can confer broad visibility; namespace configuration is a security-sensitive deployment artifact and must be least privilege.
- Matrix bridge tokens are brokered secret handles, not model context.

---

## Apple Push Notification service (APNs)

**Sources**:

- `https://developer.apple.com/documentation/usernotifications/registering-your-app-with-apns`
- `https://developer.apple.com/documentation/usernotifications/sending-notification-requests-to-apns`

**Source status**: OFFICIAL.

Relevant current behavior:

- An app receives an APNs device token specific to the app/device registration and sends that token to its provider server.
- APNs is documented as best-effort; notifications can be reordered, delayed/stored or throttled and stored delivery may collapse to one notification per bundle/device context.

### Golam implication

Never encode canonical ordering or approval semantics in push delivery. Use APNs only to wake the app to perform an authenticated GolamConnect sync.

---

## Firebase Cloud Messaging (FCM)

**Sources**:

- `https://firebase.google.com/docs/cloud-messaging/fcm-architecture`
- `https://firebase.google.com/docs/cloud-messaging/customize-messages/set-message-type`
- `https://firebase.google.com/docs/cloud-messaging/customize-messages/collapsible-message-types`

**Source status**: OFFICIAL; docs refreshed in August 2026.

Relevant current behavior:

- FCM routes server-sent messages to an app instance through Google/platform transport infrastructure.
- It supports notification messages and data messages.
- Current docs explicitly state FCM transport is encrypted but not end-to-end encrypted and recommend application-level E2E protection for sensitive data.
- FCM does not guarantee message order.
- Collapsible messages are explicitly suitable for content-free “ping/sync” behavior.

### Golam implication

Use a collapsible/opaque wake hint such as a sync generation, never the session content itself. APNs/FCM provider credentials/tokens are secrets and egress is disabled in strict-local mode.

---

## Tauri 2 mobile

**Sources**:

- `https://v2.tauri.app/develop/plugins/develop-mobile/`
- `https://v2.tauri.app/develop/plugins/`
- `https://v2.tauri.app/distribute/`

**Source status**: OFFICIAL.

Current architecture capabilities:

- Tauri supports mobile plugin implementations with native Android Kotlin/Java and iOS Swift code.
- Plugins can share Rust logic and expose narrow commands/events to native/mobile layers.
- Tauri's current distribution docs include Android/Google Play and iOS/App Store paths.
- Official plugin support has platform capability metadata and mobile features such as barcode scanning/biometric access, but exact plugins must still be individually qualified.

### Golam implication

Tauri mobile can preserve a shared Rust client/protocol core and align with the Desktop stack, but mobile background execution, notification/push, secure key storage, networking, camera/mic, remote screen/input and store-distribution behavior must be benchmarked on real iOS/Android before selection. The future spec should compare:

1. Tauri 2 mobile + native plugins;
2. native Swift/Kotlin UI around a shared Rust library/FFI boundary;
3. any other candidate only if it preserves Golam's Rust-owned protocol/authority semantics.

---

## Provider qualification checklist for Spec 007

Every provider gets an exact record containing:

- official source/documentation URL and retrieval date;
- API/version/release identity where available;
- account/product prerequisites;
- authentication credentials and secret-storage boundary;
- stable identity identifiers;
- ingress mode and public endpoint requirement;
- request authentication/verification;
- retry/order/dedupe semantics;
- supported media/interactions;
- group/thread behavior;
- edit/delete/reaction behavior;
- outbound idempotency/delivery-status model;
- rate limits and backpressure;
- current policy/template/reply-window constraints;
- regional/product availability;
- telemetry/provider metadata exposure;
- SDK/source candidate license/provenance if code reuse is proposed;
- sandbox/egress profile;
- adversarial and live acceptance-test plan.

A provider does not become `SUPPORTED` merely because one demo message can be sent.