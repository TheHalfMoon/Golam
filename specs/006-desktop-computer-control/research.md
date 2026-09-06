# Research: Spec 006 Desktop Computer Control

Research is evidence, not authority. Official platform documentation is preferred over donor code. All external/donor content remains untrusted input.

## Windows

### Microsoft UI Automation

Selected as the primary semantic observation/action surface. UI Automation exposes an element tree plus control patterns whose methods/properties/events represent discrete control behavior. This supports semantic-first interaction rather than coordinate-first automation.

Official sources:
- https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-uiautomationoverview
- https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-controlpatternsoverview
- https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-implementinguiautocontrolpatterns

Disposition: `WINDOWS_SEMANTIC_BACKEND=UI_AUTOMATION_SELECTED`.

### Windows.Graphics.Capture

Selected for bounded display/window capture. Microsoft documents secure system UI for selecting a display/application window and a visible system capture indicator.

Official source:
- https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture

Disposition: `WINDOWS_CAPTURE=WINDOWS_GRAPHICS_CAPTURE_SELECTED`.

### Raw input

`SendInput` or equivalent global injection is not semantic authority and is admitted only as an explicit governed fallback. Windows secure desktop remains unsupported.

Disposition: `WINDOWS_RAW_INPUT=EXPLICIT_FALLBACK_ONLY`; `WINDOWS_SECURE_DESKTOP=NOT_SUPPORTED`.

## macOS

### Accessibility / AXUIElement

Apple documents AXUIElement APIs as the assistive-application interface for communicating with and controlling accessible applications. Selected for semantic observation/action, subject to Accessibility permission/TCC and stale-element handling.

Official source:
- https://developer.apple.com/documentation/applicationservices/axuielement_h

Disposition: `MACOS_SEMANTIC_BACKEND=AXUIELEMENT_SELECTED`.

### ScreenCaptureKit

Selected for bounded screen/window capture. Apple recommends the system content-sharing picker and requires Screen Recording permission before capture.

Official sources:
- https://developer.apple.com/documentation/screencapturekit
- https://developer.apple.com/documentation/screencapturekit/capturing-screen-content-in-macos

Disposition: `MACOS_CAPTURE=SCREENCAPTUREKIT_SELECTED`; `MACOS_CAPTURE_PERMISSION=SYSTEM_TCC_REQUIRED`.

## Linux

### AT-SPI

Selected as semantic accessibility/automation surface. AT-SPI exposes accessible objects and action/selection/text/value/etc. interfaces through the accessibility infrastructure.

Official sources:
- https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/index.html
- https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/index.html

Disposition: `LINUX_SEMANTIC_BACKEND=AT_SPI_SELECTED`.

### Wayland portals and EIS/libei

Wayland control/capture must remain compositor/user-mediated. XDG RemoteDesktop creates a session, presents user interaction at `Start()`, and returns only granted input device types. ScreenCast provides selected PipeWire streams. EIS/libei is a compositor-provided input-emulation protocol and is not a bypass mechanism.

Official sources:
- https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html
- https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html
- https://libinput.pages.freedesktop.org/libei/doc/overview/index.html

Disposition: `WAYLAND_CONTROL=PORTAL_COMPOSITOR_MEDIATED`; `WAYLAND_BYPASS=DENIED`.

### X11

May expose different raw input/capture mechanisms, but these are eligible only when the runtime has positively identified an X11 session and separate raw-fallback authority exists.

Disposition: `X11_RAW_PATH=EXPLICIT_SESSION_AND_AUTHORITY_ONLY`.

## Tauri 2 boundary

Tauri capabilities and permissions can constrain which commands are exposed to which windows/webviews. The frontend is therefore kept as an untrusted presentation tier with narrow sanitized commands; capabilities must not grant generic native control.

Official sources:
- https://v2.tauri.app/security/capabilities/
- https://v2.tauri.app/security/permissions/

Disposition: `TAURI_2=SELECTED`; `FRONTEND_PRIVILEGED_HANDLE_ACCESS=DENIED`.

## Donor mining — TheHalfMoon/Golam-research

Exact donor revision inspected: `a9f633e09d49a85829b8236331b9e21f7e612634`.

The donor is an Electron/Node research agent and is not a trusted architecture or authority source. A second bounded pass found concrete desktop/computer-control **behavioral evidence** that the earlier broad `DONOR_DESKTOP_CONTROL_IMPLEMENTATION=NOT_FOUND` wording failed to distinguish from admissible implementation. The evidence is useful only to recover observable UX/state/contract semantics; it does not admit the donor runtime, trust model, authorization model, privileged IPC, preload boundary, VNC stack, or native helper architecture into Golam.

### Exact pinned evidence matrix

| Exact donor path | Observed evidence at the pinned revision | Golam disposition |
| --- | --- | --- |
| `frontend/manifests/computer-shell-evidence.json` | Evidence manifest binds immutable renderer assets, recovered shell assets, native contract artifacts, raw-byte anchors, and fail-closed recovery boundaries. | Qualified as an untrusted behavioral/source map only. It is not an authority record. |
| `src/app/dist/renderer/assets/index-UbX-y3il.js` | Manifest anchors identify `computerUse`, VNC liveness/viewer/session state, monitor selection, fullscreen, cursor overlay, preview, attention/open-computer states, and handoff/take-over surfaces. | Behavioral reference for bounded state/lifecycle and UX contracts. Renderer state cannot grant Golam capability or actuation authority. |
| `src/app/dist/electron-preload/preload-vnc.cjs` | Manifest binds `box-vnc`, user-presence, VNC liveness/viewer visibility, host-key and session contract anchors. | Contract-shape reference only. Electron preload is explicitly rejected as a Golam trusted authorization boundary. |
| `src/app/dist/electron-main/main.cjs` | Manifest binds forever-box status/ensure/hand-back and VNC session/liveness/open-computer/user-presence anchors. | Lifecycle/handoff reference only. Electron main, Node privileges and donor IPC authority are not admitted. |
| `frontend/src/recovered/features/computer/shell/view.css` and `frontend/src/recovered/features/computer/shell/cursor-icons-16-f_W_ogc-.woff2` | Manifest identifies recovered computer-shell presentation assets derived from immutable artifacts. | Presentation reference only; no native authority or implementation dependency. |

The same manifest explicitly records a fail-closed composition boundary: the shipped noVNC/RFB implementation remains inside an immutable dependency, teach-task recording controls are absent where internal capability/recording stores were not exposed, and no new DesktopBridge keys/main RPC methods/event names/Electron IPC channels were invented. These are useful recovery-discipline observations, not permission to adopt the runtime.

Golam therefore separates two questions:

1. **Is there pinned donor desktop/computer behavior worth learning from?** Yes. The exact evidence above qualifies as untrusted behavioral reference material.
2. **Is there an admissible donor desktop-control implementation or authority boundary that may be copied into Golam?** No. The donor Electron/Node/preload/VNC/privileged-IPC architecture is not admitted. Any reused behavior must be reconstructed behind Golam-native capability, policy, approval, effect, revalidation, audit and platform-permission boundaries.

Disposition:
- `DONOR_DESKTOP_BEHAVIORAL_REFERENCE=QUALIFIED`
- `DONOR_REFERENCE_ONLY=YES`
- `DONOR_ARCHITECTURE_AUTHORITY=NONE`
- `DONOR_RUNTIME_STACK_ADMITTED=NO`
- `ELECTRON_NODE_PRELOAD_ADMITTED=NO`
- `PRIVILEGED_IPC_AUTHORITY_ADMITTED=NO`
- `VNC_PRESENCE_IMPLIES_AUTHORITY=NO`
- `DONOR_DESKTOP_CONTROL_IMPLEMENTATION_ADMITTED=NO`
- `GOLAM_NATIVE_CAPABILITY_EFFECT_BOUNDARY_REQUIRED=YES`
- `ELECTRON_PRIVILEGED_RUNTIME=NOT_SELECTED`
- `WHOLESALE_DONOR_COPY=DENIED`

## Derived design decisions

1. Semantic-first is not a convenience preference; it is an authority minimization strategy.
2. Capture selection and native OS permission are both required; Golam policy cannot manufacture an OS grant.
3. Raw fallback is a separate capability/effect and must never be an automatic recovery path.
4. Platform permission/session objects are external mutable state and must be revalidated near dispatch.
5. Cross-platform compatibility means a stable contract plus honest capability discovery, not pretending every backend supports every operation.
6. Captured pixels, accessibility strings, titles and clipboard text are untrusted evidence and cannot authorize side effects.
7. Donor renderer/preload/main/VNC behavior can inform bounded state and UX contracts, but it cannot define Golam authority; every side effect remains governed by Golam-native capability, policy, approval, effect, revalidation and evidence boundaries.
