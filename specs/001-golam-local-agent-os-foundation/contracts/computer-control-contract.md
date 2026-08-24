# Contract: Computer Control

## Action hierarchy

Golam MUST prefer:

1. domain/application API;
2. native OS automation API;
3. accessibility/semantic tree;
4. browser DOM/protocol;
5. deterministic keyboard/mouse input injection;
6. vision/pixel fallback.

Vision is a fallback, not the default authority surface.

## Closed-loop action

Every protected UI action follows:

`Observe -> ActionIntent -> Authorize -> Act -> ObserveAfter -> Verify`

Semantic element refs are snapshot-bound and carry staleness tokens. `STALE_REF` MUST fail and trigger re-observation; it must never silently retarget a different element.

## Platform capability matrix

### Windows
- UI Automation patterns first.
- Input injection requires an unlocked interactive user desktop and fails clearly otherwise.
- UAC/secure desktop is not bypassed.
- Admin-context/credential-prompt bypass is out of scope.
- The interactive control executor runs in the user's interactive session rather than pretending a service session can safely drive UI.

### macOS
- Accessibility/AX requires explicit TCC Accessibility permission.
- Screen/vision capture requires Screen Recording permission.
- Permission state is observable and failures are explicit/fail-closed.
- Golam does not silently bypass TCC or secure input surfaces.

### Linux
- AT-SPI is the semantic path where available.
- X11/XWayland input may use supported XTEST-class mechanisms.
- Pure Wayland control/capture is limited to supported RemoteDesktop/screencast portals/compositor capabilities.
- Unsupported compositor/portal combinations fail closed rather than claiming parity.

## Sensitive capabilities

- Clipboard read and clipboard write are separate capabilities; clipboard read may expose secrets and is policy/taint aware.
- Camera and microphone are distinct deny-by-default capabilities.
- Browser use of real user profiles is explicit policy; uploads/downloads/form submissions are effects.
- Protected/sensitive app observations may be redacted before screenshot/remote streaming where feasible.

## Human takeover

Human takeover revokes/suspends conflicting agent input authority at the lease layer. The agent may continue non-input reasoning only if policy permits. Returning input control to the agent requires explicit reauthorization.

## Verification

Platform release claims require on-device tests for supported semantic operations, locked/protected surfaces, stale refs, takeover latency, permission failures and fallback behavior.
