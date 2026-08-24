# Contract: Computer Control

## Control hierarchy

Use the highest available deterministic semantic layer:

1. domain/application API;
2. native OS automation API;
3. accessibility/semantic tree;
4. browser DOM/protocol;
5. deterministic keyboard/mouse/input injection;
6. vision/coordinate fallback.

## Observe-act-verify loop

Each action uses:

`BeforeState -> ActionIntent -> Authorization -> Act -> ObservedAfterState -> Verification`

## Element references

Semantic observations may assign stable refs scoped to a snapshot. A ref MUST include or imply a snapshot/staleness token. If the UI changes such that the ref cannot be proven to identify the same element, the action fails `STALE_REF` and requires re-observation.

## Platform contract

Each platform adapter reports capabilities and limitations. Input-injecting operations fail explicitly when the workstation is locked, secure desktop/UAC is active, permissions are missing, or the OS prevents injection.

## Human control

`TAKEOVER` immediately blocks conflicting autonomous input. Background non-control reasoning may continue only if policy permits. `STOP` cancels current control action and revokes temporary control leases.

## Privacy

User-owned blocked apps/windows/resources must be redacted from agent observations and refused for control. The agent cannot modify the blocklist without a separately authorized user action.
