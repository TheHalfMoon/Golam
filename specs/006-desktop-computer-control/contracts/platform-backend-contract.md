# Platform Backend Contract

Every backend implements the common desktop capability/observation/action/capture interface and declares actual supported operations.

## Windows
- Semantic: Microsoft UI Automation.
- Capture: Windows.Graphics.Capture.
- Raw fallback: explicitly governed `SendInput`-class path only.
- Secure desktop: not supported.

## macOS
- Semantic: Accessibility / AXUIElement.
- Capture: ScreenCaptureKit under Screen Recording permission.
- Raw fallback: only under appropriate OS permission plus explicit Golam fallback authority.

## Linux
- Semantic: AT-SPI.
- X11: raw mechanisms only in positively identified X11 session with explicit authority.
- Wayland: XDG RemoteDesktop/ScreenCast and compositor-provided EIS/libei where available and granted.
- No compositor/portal bypass.

## Common invariants
- Native permission/session handles remain private.
- `NotSupported` and `PermissionDenied` are valid expected results.
- Permission/session state is mutable external state and must be revalidated.
- Fake backend must implement the same contract and is the first admission target.
