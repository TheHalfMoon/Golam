# Platform Backend Contract

Every backend implements the common desktop capability/observation/action/capture interface and declares actual supported operations. Platform support never implies Golam authority, and any new implementation dependency must be Source Foundry admitted before manifest/code admission.

## Windows
- Semantic: Microsoft UI Automation.
- Capture: Windows.Graphics.Capture.
- Raw fallback: explicitly governed `SendInput`-class path only.
- Locked desktop: fail closed for actuation that requires an interactive user desktop.
- UAC/secure desktop: not supported; no bypass or elevation path.
- Interactive-session transition: invalidate prepared target/focus/permission assumptions and require fresh observation/authority validation.

## macOS
- Semantic: Accessibility / AXUIElement.
- Capture: ScreenCaptureKit under Screen Recording permission.
- Raw fallback: only under appropriate OS permission plus explicit Golam fallback authority.
- TCC/Accessibility/Screen Recording changes invalidate stale prepared state.

## Linux
- Semantic: AT-SPI.
- X11: raw mechanisms only in positively identified X11 session with explicit authority.
- Wayland: XDG RemoteDesktop/ScreenCast and compositor-provided EIS/libei where available and granted.
- Portal/compositor/session termination invalidates prepared external permission/session evidence.
- No compositor/portal bypass.

## Common invariants
- Native permission/session handles remain private.
- `NotSupported`, `PermissionDenied`, `Interrupted` and `UnknownOutcome` are valid expected results.
- Permission/session/interactive-desktop state is mutable external state and must be revalidated.
- Human pause/stop/takeover is enforced above adapters at protected control-lease/input authority; adapters cannot ignore a superseded generation.
- A pixel-derived region/coordinate is untrusted evidence and never sufficient target identity or authority.
- Fake backend must implement the same contract and is the first admission target.
