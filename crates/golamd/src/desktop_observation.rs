#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::fmt;
use std::sync::mpsc;
use std::time::Duration;

use golam_core::desktop_control::{
    DESKTOP_CONTROL_SCHEMA_VERSION, DesktopLimits, DesktopObservation, DesktopObservationId,
    DesktopPlatform, DesktopSessionKind, SemanticElementId, SemanticElementIdentity, WorkSurfaceId,
    WorkSurfaceIdentity,
};
use golam_core::digest::sha256;
use golam_core::tool_request::BindingDigest;

const OBSERVATION_ADAPTER_DOMAIN: &[u8] = b"golam:desktop-native-observation-adapter:v1";
const SURFACE_ID_DOMAIN: &[u8] = b"golam:desktop-native-surface-id:v1";
const SEMANTIC_ID_DOMAIN: &[u8] = b"golam:desktop-native-semantic-id:v1";
const SUMMARY_DOMAIN: &[u8] = b"golam:desktop-native-semantic-summary:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeObservationDisposition {
    Complete,
    Partial,
    PermissionDenied,
    NotSupported,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeMonitorDisposition {
    Enumerated,
    NotSupported,
    PermissionDenied,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeObservationRequest {
    pub platform: DesktopPlatform,
    pub session_kind: DesktopSessionKind,
    pub capability_session_evidence: BindingDigest,
    pub observed_at_unix_ms: u64,
    pub observation_generation: u64,
    pub limits: DesktopLimits,
}

impl NativeObservationRequest {
    pub fn validate(&self) -> Result<(), NativeObservationError> {
        self.limits
            .validate()
            .map_err(|_| NativeObservationError::InvalidRequest("invalid desktop limits"))?;
        if self.observed_at_unix_ms == 0 || self.observation_generation == 0 {
            return Err(NativeObservationError::InvalidRequest(
                "observation time and generation must be nonzero",
            ));
        }
        if self
            .capability_session_evidence
            .bytes()
            .iter()
            .all(|byte| *byte == 0)
        {
            return Err(NativeObservationError::InvalidRequest(
                "capability/session evidence must be nonzero",
            ));
        }
        let compatible = matches!(
            (self.platform, self.session_kind),
            (
                DesktopPlatform::Windows,
                DesktopSessionKind::WindowsInteractive
            ) | (DesktopPlatform::Macos, DesktopSessionKind::MacosLogin)
                | (DesktopPlatform::Linux, DesktopSessionKind::LinuxX11)
                | (DesktopPlatform::Linux, DesktopSessionKind::LinuxWayland)
        );
        if !compatible {
            return Err(NativeObservationError::InvalidRequest(
                "desktop platform/session kind mismatch",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedWorkSurface {
    pub identity: WorkSurfaceIdentity,
    pub semantic_elements: Vec<SemanticElementIdentity>,
    pub focused: bool,
    pub semantic_truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeObservationReport {
    pub disposition: NativeObservationDisposition,
    pub monitor_disposition: NativeMonitorDisposition,
    pub monitor_count: u16,
    pub surfaces: Vec<ObservedWorkSurface>,
    pub semantic_node_count: u32,
    pub strings_truncated: bool,
    pub surfaces_truncated: bool,
    pub observation: DesktopObservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeObservationError {
    InvalidRequest(&'static str),
    TimedOut,
    WorkerUnavailable,
    InternalInvariant,
}

impl fmt::Display for NativeObservationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(formatter, "invalid request: {message}"),
            Self::TimedOut => formatter.write_str("native desktop observation timed out"),
            Self::WorkerUnavailable => {
                formatter.write_str("native desktop observation worker unavailable")
            }
            Self::InternalInvariant => {
                formatter.write_str("native desktop observation invariant failed")
            }
        }
    }
}

impl std::error::Error for NativeObservationError {}

#[derive(Clone, Debug)]
struct PlatformSnapshot {
    disposition: NativeObservationDisposition,
    monitor_disposition: NativeMonitorDisposition,
    monitor_count: u16,
    surfaces: Vec<ObservedWorkSurface>,
    summary: SummaryAccumulator,
    surfaces_truncated: bool,
}

#[derive(Clone, Debug)]
struct SummaryAccumulator {
    bytes: Vec<u8>,
    string_bytes: u32,
    node_count: u32,
    strings_truncated: bool,
    nodes_truncated: bool,
    max_string_bytes: u32,
    max_nodes: u32,
}

impl SummaryAccumulator {
    fn new(limits: DesktopLimits) -> Self {
        let mut bytes = Vec::with_capacity(512);
        bytes.extend_from_slice(SUMMARY_DOMAIN);
        Self {
            bytes,
            string_bytes: 0,
            node_count: 0,
            strings_truncated: false,
            nodes_truncated: false,
            max_string_bytes: limits.max_string_bytes,
            max_nodes: limits.max_semantic_nodes,
        }
    }

    fn can_accept_node(&self) -> bool {
        self.node_count < self.max_nodes
    }

    fn record_node(
        &mut self,
        platform_reference: &str,
        role: &str,
        name: Option<&str>,
        value: Option<&str>,
        actions: &[String],
        state_geometry: &str,
    ) -> bool {
        if !self.can_accept_node() {
            self.nodes_truncated = true;
            return false;
        }
        self.node_count += 1;
        self.push_bounded_text(platform_reference);
        self.push_bounded_text(role);
        if let Some(name) = name {
            self.push_bounded_text(name);
        }
        if let Some(value) = value {
            self.push_bounded_text(value);
        }
        for action in actions {
            self.push_bounded_text(action);
        }
        self.push_bounded_text(state_geometry);
        true
    }

    fn push_bounded_text(&mut self, value: &str) {
        self.bytes.push(0xff);
        for character in value.chars() {
            if should_drop_character(character) {
                continue;
            }
            let mut encoded = [0_u8; 4];
            let encoded = character.encode_utf8(&mut encoded).as_bytes();
            let encoded_len = u32::try_from(encoded.len()).unwrap_or(u32::MAX);
            if self.string_bytes.saturating_add(encoded_len) > self.max_string_bytes {
                self.strings_truncated = true;
                break;
            }
            self.bytes.extend_from_slice(encoded);
            self.string_bytes += encoded_len;
        }
    }

    fn binding_digest(&self) -> BindingDigest {
        digest(&self.bytes)
    }
}

pub fn observe_native_desktop(
    request: NativeObservationRequest,
) -> Result<NativeObservationReport, NativeObservationError> {
    request.validate()?;
    let timeout = Duration::from_millis(request.limits.max_observation_duration_ms);
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("golam-desktop-observation".to_owned())
        .spawn(move || {
            let result = observe_platform(request).and_then(|snapshot| finalize(request, snapshot));
            let _ = sender.send(result);
        })
        .map_err(|_| NativeObservationError::WorkerUnavailable)?;

    match receiver.recv_timeout(timeout) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(NativeObservationError::TimedOut),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(NativeObservationError::WorkerUnavailable),
    }
}

fn finalize(
    request: NativeObservationRequest,
    snapshot: PlatformSnapshot,
) -> Result<NativeObservationReport, NativeObservationError> {
    let work_surface_digests = snapshot
        .surfaces
        .iter()
        .map(|surface| {
            surface
                .identity
                .binding_digest()
                .map_err(|_| NativeObservationError::InternalInvariant)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let focused_surface_digest = snapshot
        .surfaces
        .iter()
        .find(|surface| surface.focused)
        .map(|surface| {
            surface
                .identity
                .binding_digest()
                .map_err(|_| NativeObservationError::InternalInvariant)
        })
        .transpose()?;
    let focused_element_digest = snapshot
        .surfaces
        .iter()
        .find(|surface| surface.focused)
        .and_then(|surface| {
            surface
                .semantic_elements
                .iter()
                .find(|element| element.state_geometry_digest == digest(b"focused"))
        })
        .map(|element| {
            element
                .binding_digest()
                .map_err(|_| NativeObservationError::InternalInvariant)
        })
        .transpose()?;

    let observation = DesktopObservation {
        schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
        observation_id: DesktopObservationId::from_u128(id_from_parts(&[
            OBSERVATION_ADAPTER_DOMAIN,
            &request.capability_session_evidence.bytes(),
            &request.observed_at_unix_ms.to_le_bytes(),
            &request.observation_generation.to_le_bytes(),
        ])),
        observed_at_unix_ms: request.observed_at_unix_ms,
        capability_session_evidence: request.capability_session_evidence,
        work_surface_digests,
        semantic_summary_digest: snapshot.summary.binding_digest(),
        focused_surface_digest,
        focused_element_digest,
        limits: request.limits,
    };
    observation
        .validate()
        .map_err(|_| NativeObservationError::InternalInvariant)?;

    Ok(NativeObservationReport {
        disposition: snapshot.disposition,
        monitor_disposition: snapshot.monitor_disposition,
        monitor_count: snapshot.monitor_count,
        surfaces: snapshot.surfaces,
        semantic_node_count: snapshot.summary.node_count,
        strings_truncated: snapshot.summary.strings_truncated,
        surfaces_truncated: snapshot.surfaces_truncated,
        observation,
    })
}

fn make_surface(
    request: NativeObservationRequest,
    surface_reference: &str,
    application_reference: Option<&str>,
    incarnation_reference: &str,
    geometry_reference: &str,
) -> Result<WorkSurfaceIdentity, NativeObservationError> {
    let identity = WorkSurfaceIdentity {
        schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
        platform: request.platform,
        session_kind: request.session_kind,
        surface_id: WorkSurfaceId::from_u128(id_from_parts(&[
            SURFACE_ID_DOMAIN,
            surface_reference.as_bytes(),
        ])),
        application_identity: application_reference.map(|value| digest(value.as_bytes())),
        incarnation_evidence: digest(incarnation_reference.as_bytes()),
        bounds_geometry_digest: digest(geometry_reference.as_bytes()),
        observation_generation: request.observation_generation,
    };
    identity
        .validate()
        .map_err(|_| NativeObservationError::InternalInvariant)?;
    Ok(identity)
}

fn make_semantic_element(
    request: NativeObservationRequest,
    parent_work_surface_digest: BindingDigest,
    platform_reference: &str,
    role: &str,
    actions: &[String],
    state_geometry: &str,
) -> Result<SemanticElementIdentity, NativeObservationError> {
    let mut action_bytes = Vec::new();
    let mut actions = actions.to_vec();
    actions.sort();
    for action in actions {
        action_bytes.extend_from_slice(action.as_bytes());
        action_bytes.push(0);
    }
    let state_geometry_digest = if state_geometry == "focused" {
        digest(b"focused")
    } else {
        digest(state_geometry.as_bytes())
    };
    let identity = SemanticElementIdentity {
        schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
        parent_work_surface_digest,
        element_id: SemanticElementId::from_u128(id_from_parts(&[
            SEMANTIC_ID_DOMAIN,
            &parent_work_surface_digest.bytes(),
            platform_reference.as_bytes(),
        ])),
        platform_reference_digest: digest(platform_reference.as_bytes()),
        role_control_type_digest: digest(role.as_bytes()),
        supported_action_set_digest: digest(&action_bytes),
        state_geometry_digest,
        observation_generation: request.observation_generation,
    };
    identity
        .validate()
        .map_err(|_| NativeObservationError::InternalInvariant)?;
    Ok(identity)
}

fn digest(value: &[u8]) -> BindingDigest {
    BindingDigest::new(sha256(value))
}

fn id_from_parts(parts: &[&[u8]]) -> u128 {
    let mut input = Vec::new();
    for part in parts {
        input.extend_from_slice(&(part.len() as u64).to_le_bytes());
        input.extend_from_slice(part);
    }
    let hash = sha256(&input);
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    let value = u128::from_le_bytes(bytes);
    if value == 0 { 1 } else { value }
}

fn should_drop_character(character: char) -> bool {
    character == '\0'
        || character == '\r'
        || (character.is_control() && character != '\n' && character != '\t')
        || matches!(
            character,
            '\u{200e}'
                | '\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2066}'..='\u{2069}'
        )
}

#[cfg(target_os = "windows")]
fn observe_platform(
    request: NativeObservationRequest,
) -> Result<PlatformSnapshot, NativeObservationError> {
    use std::sync::Arc;

    use xa11y_core::{App, Provider};
    use xa11y_windows::WindowsProvider;

    if request.platform != DesktopPlatform::Windows
        || request.session_kind != DesktopSessionKind::WindowsInteractive
    {
        return Err(NativeObservationError::InvalidRequest(
            "Windows adapter received a non-Windows session",
        ));
    }

    let mut summary = SummaryAccumulator::new(request.limits);
    let provider: Arc<dyn Provider> = match WindowsProvider::new() {
        Ok(provider) => Arc::new(provider),
        Err(_) => {
            return empty_platform_snapshot(
                request,
                NativeObservationDisposition::Unavailable,
                NativeMonitorDisposition::NotSupported,
            );
        }
    };
    let foreground_reference = App::foreground_with(Arc::clone(&provider), Duration::ZERO)
        .ok()
        .map(|app| windows_surface_reference(&app));
    let apps = match App::list_with(provider) {
        Ok(apps) => apps,
        Err(_) => {
            return empty_platform_snapshot(
                request,
                NativeObservationDisposition::Unavailable,
                NativeMonitorDisposition::NotSupported,
            );
        }
    };
    let surfaces_truncated = apps.len() > usize::from(request.limits.max_work_surfaces);
    let mut surfaces = Vec::new();
    let mut disposition = NativeObservationDisposition::Complete;

    for app in apps
        .into_iter()
        .take(usize::from(request.limits.max_work_surfaces))
    {
        let surface_reference = windows_surface_reference(&app);
        let application_reference = format!("windows-pid:{:?}", app.pid);
        let incarnation_reference =
            format!("windows-incarnation:{:?}:{:?}", app.pid, app.data.stable_id);
        let geometry_reference = format!("windows-bounds:{:?}", app.data.bounds);
        let identity = make_surface(
            request,
            &surface_reference,
            Some(&application_reference),
            &incarnation_reference,
            &geometry_reference,
        )?;
        let parent_digest = identity
            .binding_digest()
            .map_err(|_| NativeObservationError::InternalInvariant)?;
        let focused = foreground_reference.as_deref() == Some(surface_reference.as_str())
            || app.data.states.focused;
        let mut semantic_elements = Vec::new();
        let mut semantic_truncated = false;
        let mut queue = VecDeque::from([(app.as_element(), 0_u16)]);

        while let Some((element, depth)) = queue.pop_front() {
            if !summary.can_accept_node() {
                summary.nodes_truncated = true;
                semantic_truncated = true;
                break;
            }
            if depth > request.limits.max_semantic_depth {
                summary.nodes_truncated = true;
                semantic_truncated = true;
                continue;
            }
            let platform_reference =
                windows_element_reference(&element, request.observation_generation);
            let role = format!("{:?}", element.role);
            let state_geometry = if element.states.focused {
                "focused".to_owned()
            } else {
                format!("{:?}:{:?}", element.states, element.bounds)
            };
            if !summary.record_node(
                &platform_reference,
                &role,
                element.name.as_deref(),
                element.value.as_deref(),
                &element.actions,
                &state_geometry,
            ) {
                semantic_truncated = true;
                break;
            }
            semantic_elements.push(make_semantic_element(
                request,
                parent_digest,
                &platform_reference,
                &role,
                &element.actions,
                &state_geometry,
            )?);

            if depth < request.limits.max_semantic_depth {
                match element.children() {
                    Ok(children) => {
                        let remaining = request
                            .limits
                            .max_semantic_nodes
                            .saturating_sub(summary.node_count)
                            as usize;
                        if children.len() > remaining {
                            semantic_truncated = true;
                            summary.nodes_truncated = true;
                        }
                        queue.extend(
                            children
                                .into_iter()
                                .take(remaining)
                                .map(|child| (child, depth + 1)),
                        );
                    }
                    Err(_) => {
                        disposition = NativeObservationDisposition::Partial;
                        semantic_truncated = true;
                    }
                }
            }
        }

        surfaces.push(ObservedWorkSurface {
            identity,
            semantic_elements,
            focused,
            semantic_truncated,
        });
    }

    if surfaces_truncated || summary.nodes_truncated || summary.strings_truncated {
        disposition = NativeObservationDisposition::Partial;
    }
    Ok(PlatformSnapshot {
        disposition,
        monitor_disposition: NativeMonitorDisposition::NotSupported,
        monitor_count: 0,
        surfaces,
        summary,
        surfaces_truncated,
    })
}

#[cfg(target_os = "windows")]
fn windows_surface_reference(app: &xa11y_core::App) -> String {
    format!(
        "windows-surface:{:?}:{:?}:{:?}",
        app.pid, app.data.stable_id, app.data.bounds
    )
}

#[cfg(target_os = "windows")]
fn windows_element_reference(element: &xa11y_core::Element, generation: u64) -> String {
    if let Some(stable_id) = element.stable_id.as_deref() {
        format!("windows-uia:{:?}:{stable_id}", element.pid)
    } else {
        format!(
            "windows-uia-ephemeral:{:?}:{:?}:{:?}:{generation}",
            element.pid, element.role, element.bounds
        )
    }
}

#[cfg(target_os = "macos")]
fn observe_platform(
    request: NativeObservationRequest,
) -> Result<PlatformSnapshot, NativeObservationError> {
    use axuielement::prelude::*;
    use screencapturekit::prelude::*;

    if request.platform != DesktopPlatform::Macos
        || request.session_kind != DesktopSessionKind::MacosLogin
    {
        return Err(NativeObservationError::InvalidRequest(
            "macOS adapter received a non-macOS session",
        ));
    }

    let mut summary = SummaryAccumulator::new(request.limits);
    let mut surfaces = Vec::new();
    let mut monitor_count = 0_u16;
    let mut monitor_disposition = NativeMonitorDisposition::Enumerated;
    let mut disposition = NativeObservationDisposition::Complete;
    let mut surfaces_truncated = false;

    match SCShareableContent::get() {
        Ok(content) => {
            let displays = content.displays();
            monitor_count = u16::try_from(displays.len()).unwrap_or(u16::MAX);
            if displays.len() > usize::from(request.limits.max_work_surfaces) {
                surfaces_truncated = true;
            }
            for display in displays
                .into_iter()
                .take(usize::from(request.limits.max_work_surfaces))
            {
                let reference = format!("macos-display:{}", display.display_id());
                let geometry = format!("{}x{}", display.width(), display.height());
                let identity = make_surface(request, &reference, None, &reference, &geometry)?;
                surfaces.push(ObservedWorkSurface {
                    identity,
                    semantic_elements: Vec::new(),
                    focused: false,
                    semantic_truncated: false,
                });
            }

            for window in content
                .windows()
                .into_iter()
                .filter(|window| window.is_on_screen())
            {
                if surfaces.len() >= usize::from(request.limits.max_work_surfaces) {
                    surfaces_truncated = true;
                    break;
                }
                let app_name = window
                    .owning_application()
                    .map(|app| app.application_name())
                    .unwrap_or_default();
                let reference = format!("macos-window:{}", window.window_id());
                let application = format!("macos-application:{app_name}");
                let geometry = format!("{:?}", window.frame());
                let incarnation = format!("{reference}:{app_name}");
                let identity = make_surface(
                    request,
                    &reference,
                    Some(&application),
                    &incarnation,
                    &geometry,
                )?;
                surfaces.push(ObservedWorkSurface {
                    identity,
                    semantic_elements: Vec::new(),
                    focused: false,
                    semantic_truncated: false,
                });
            }
        }
        Err(_) => {
            disposition = NativeObservationDisposition::PermissionDenied;
            monitor_disposition = NativeMonitorDisposition::PermissionDenied;
        }
    }

    if let Some(system) = system_wide() {
        match system.focused_application() {
            Ok(Some(app)) => {
                if surfaces.len() < usize::from(request.limits.max_work_surfaces) {
                    let pid = app.pid().unwrap_or_default();
                    let reference = format!("macos-ax-app:{pid}");
                    let identity = make_surface(
                        request,
                        &reference,
                        Some(&format!("macos-pid:{pid}")),
                        &format!("macos-ax-incarnation:{pid}"),
                        "macos-ax-app-geometry-unavailable",
                    )?;
                    let parent_digest = identity
                        .binding_digest()
                        .map_err(|_| NativeObservationError::InternalInvariant)?;
                    let mut semantic_elements = Vec::new();
                    let mut semantic_truncated = false;
                    let mut queue = VecDeque::from([(app, 0_u16, format!("macos-ax-root:{pid}"))]);

                    while let Some((element, depth, platform_reference)) = queue.pop_front() {
                        if !summary.can_accept_node() {
                            summary.nodes_truncated = true;
                            semantic_truncated = true;
                            break;
                        }
                        if depth > request.limits.max_semantic_depth {
                            summary.nodes_truncated = true;
                            semantic_truncated = true;
                            continue;
                        }
                        let role = element
                            .string_attribute(axuielement::ax_attribute::AX_ROLE_ATTRIBUTE)
                            .ok()
                            .flatten()
                            .unwrap_or_else(|| "unknown".to_owned());
                        let title = element
                            .string_attribute(axuielement::ax_attribute::AX_TITLE_ATTRIBUTE)
                            .ok()
                            .flatten();
                        let actions = element.action_names().unwrap_or_default();
                        let state_geometry = "macos-ax-state-geometry-unavailable".to_owned();
                        if !summary.record_node(
                            &platform_reference,
                            &role,
                            title.as_deref(),
                            None,
                            &actions,
                            &state_geometry,
                        ) {
                            semantic_truncated = true;
                            break;
                        }
                        semantic_elements.push(make_semantic_element(
                            request,
                            parent_digest,
                            &platform_reference,
                            &role,
                            &actions,
                            &state_geometry,
                        )?);

                        if depth < request.limits.max_semantic_depth {
                            match element.children() {
                                Ok(children) => {
                                    let remaining = request
                                        .limits
                                        .max_semantic_nodes
                                        .saturating_sub(summary.node_count)
                                        as usize;
                                    if children.len() > remaining {
                                        semantic_truncated = true;
                                        summary.nodes_truncated = true;
                                    }
                                    queue.extend(
                                        children.into_iter().take(remaining).enumerate().map(
                                            |(index, child)| {
                                                (
                                                    child,
                                                    depth + 1,
                                                    format!(
                                                        "{platform_reference}/child/{index}/gen/{}",
                                                        request.observation_generation
                                                    ),
                                                )
                                            },
                                        ),
                                    );
                                }
                                Err(_) => {
                                    disposition = NativeObservationDisposition::Partial;
                                    semantic_truncated = true;
                                }
                            }
                        }
                    }
                    surfaces.push(ObservedWorkSurface {
                        identity,
                        semantic_elements,
                        focused: true,
                        semantic_truncated,
                    });
                } else {
                    surfaces_truncated = true;
                }
            }
            Ok(None) => {
                disposition = NativeObservationDisposition::Partial;
            }
            Err(_) => {
                if disposition == NativeObservationDisposition::Complete {
                    disposition = NativeObservationDisposition::PermissionDenied;
                }
            }
        }
    } else if disposition == NativeObservationDisposition::Complete {
        disposition = NativeObservationDisposition::Unavailable;
    }

    if surfaces_truncated || summary.nodes_truncated || summary.strings_truncated {
        disposition = NativeObservationDisposition::Partial;
    }
    Ok(PlatformSnapshot {
        disposition,
        monitor_disposition,
        monitor_count,
        surfaces,
        summary,
        surfaces_truncated,
    })
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn observe_platform(
    request: NativeObservationRequest,
) -> Result<PlatformSnapshot, NativeObservationError> {
    use golam_bounded_linux_async::{BoundedAsyncError, run_bounded};

    if request.platform != DesktopPlatform::Linux
        || !matches!(
            request.session_kind,
            DesktopSessionKind::LinuxX11 | DesktopSessionKind::LinuxWayland
        )
    {
        return Err(NativeObservationError::InvalidRequest(
            "Linux adapter received a non-Linux session",
        ));
    }

    let timeout = Duration::from_millis(request.limits.max_observation_duration_ms);
    match run_bounded(timeout, async move { observe_linux_async(request).await }) {
        Ok(result) => result,
        Err(BoundedAsyncError::TimedOut) => Err(NativeObservationError::TimedOut),
        Err(BoundedAsyncError::InvalidTimeout | BoundedAsyncError::RuntimeUnavailable) => {
            Err(NativeObservationError::WorkerUnavailable)
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
async fn observe_linux_async(
    request: NativeObservationRequest,
) -> Result<PlatformSnapshot, NativeObservationError> {
    use atspi::proxy::accessible::ObjectRefExt;

    let mut summary = SummaryAccumulator::new(request.limits);
    let connection = match atspi::AccessibilityConnection::new().await {
        Ok(connection) => connection,
        Err(_) => {
            return empty_platform_snapshot(
                request,
                NativeObservationDisposition::Unavailable,
                NativeMonitorDisposition::NotSupported,
            );
        }
    };
    let root = match connection.root_accessible_on_registry().await {
        Ok(root) => root,
        Err(_) => {
            return empty_platform_snapshot(
                request,
                NativeObservationDisposition::Unavailable,
                NativeMonitorDisposition::NotSupported,
            );
        }
    };
    let application_refs = match root.get_children().await {
        Ok(children) => children,
        Err(_) => {
            return empty_platform_snapshot(
                request,
                NativeObservationDisposition::Unavailable,
                NativeMonitorDisposition::NotSupported,
            );
        }
    };
    let surfaces_truncated = application_refs.len() > usize::from(request.limits.max_work_surfaces);
    let mut surfaces = Vec::new();
    let mut disposition = NativeObservationDisposition::Complete;

    for application_ref in application_refs
        .into_iter()
        .take(usize::from(request.limits.max_work_surfaces))
    {
        let bus = application_ref.name_as_str().unwrap_or("unknown-bus");
        let path = application_ref.path_as_str();
        let surface_reference = format!("linux-atspi:{bus}:{path}");
        let identity = make_surface(
            request,
            &surface_reference,
            Some(&format!("linux-atspi-bus:{bus}")),
            &format!("linux-atspi-incarnation:{bus}"),
            "linux-atspi-surface-geometry-unavailable",
        )?;
        let parent_digest = identity
            .binding_digest()
            .map_err(|_| NativeObservationError::InternalInvariant)?;
        let application_proxy = match application_ref
            .as_accessible_proxy(connection.connection())
            .await
        {
            Ok(proxy) => proxy,
            Err(_) => {
                disposition = NativeObservationDisposition::Partial;
                surfaces.push(ObservedWorkSurface {
                    identity,
                    semantic_elements: Vec::new(),
                    focused: false,
                    semantic_truncated: true,
                });
                continue;
            }
        };
        let root_name = application_proxy.name().await.unwrap_or_default();
        let root_role = application_proxy
            .get_role_name()
            .await
            .unwrap_or_else(|_| "unknown".to_owned());
        let root_state = application_proxy.get_state().await.ok();
        let focused = root_state
            .map(|state| state.contains(atspi::State::Focused))
            .unwrap_or(false);
        let mut semantic_elements = Vec::new();
        let root_actions = Vec::new();
        let root_state_geometry = if focused {
            "focused".to_owned()
        } else {
            format!("linux-atspi-state:{root_state:?}")
        };
        if summary.record_node(
            &surface_reference,
            &root_role,
            Some(&root_name),
            None,
            &root_actions,
            &root_state_geometry,
        ) {
            semantic_elements.push(make_semantic_element(
                request,
                parent_digest,
                &surface_reference,
                &root_role,
                &root_actions,
                &root_state_geometry,
            )?);
        }

        let children = application_proxy.get_children().await.unwrap_or_default();
        let mut queue = VecDeque::new();
        let initial_remaining = request
            .limits
            .max_semantic_nodes
            .saturating_sub(summary.node_count) as usize;
        if children.len() > initial_remaining {
            summary.nodes_truncated = true;
        }
        queue.extend(
            children
                .into_iter()
                .take(initial_remaining)
                .map(|child| (child, 1_u16)),
        );
        let mut semantic_truncated = false;

        while let Some((object_ref, depth)) = queue.pop_front() {
            if !summary.can_accept_node() {
                summary.nodes_truncated = true;
                semantic_truncated = true;
                break;
            }
            if depth > request.limits.max_semantic_depth {
                summary.nodes_truncated = true;
                semantic_truncated = true;
                continue;
            }
            let bus = object_ref.name_as_str().unwrap_or("unknown-bus");
            let path = object_ref.path_as_str();
            let platform_reference = format!("linux-atspi:{bus}:{path}");
            let proxy = match object_ref
                .as_accessible_proxy(connection.connection())
                .await
            {
                Ok(proxy) => proxy,
                Err(_) => {
                    disposition = NativeObservationDisposition::Partial;
                    semantic_truncated = true;
                    continue;
                }
            };
            let name = proxy.name().await.unwrap_or_default();
            let role = proxy
                .get_role_name()
                .await
                .unwrap_or_else(|_| "unknown".to_owned());
            let state = proxy.get_state().await.ok();
            let is_focused = state
                .map(|state| state.contains(atspi::State::Focused))
                .unwrap_or(false);
            let state_geometry = if is_focused {
                "focused".to_owned()
            } else {
                format!("linux-atspi-state:{state:?}")
            };
            let actions = Vec::new();
            if !summary.record_node(
                &platform_reference,
                &role,
                Some(&name),
                None,
                &actions,
                &state_geometry,
            ) {
                semantic_truncated = true;
                break;
            }
            semantic_elements.push(make_semantic_element(
                request,
                parent_digest,
                &platform_reference,
                &role,
                &actions,
                &state_geometry,
            )?);

            if depth < request.limits.max_semantic_depth {
                match proxy.get_children().await {
                    Ok(children) => {
                        let remaining = request
                            .limits
                            .max_semantic_nodes
                            .saturating_sub(summary.node_count)
                            as usize;
                        if children.len() > remaining {
                            semantic_truncated = true;
                            summary.nodes_truncated = true;
                        }
                        queue.extend(
                            children
                                .into_iter()
                                .take(remaining)
                                .map(|child| (child, depth + 1)),
                        );
                    }
                    Err(_) => {
                        disposition = NativeObservationDisposition::Partial;
                        semantic_truncated = true;
                    }
                }
            }
        }

        surfaces.push(ObservedWorkSurface {
            identity,
            semantic_elements,
            focused,
            semantic_truncated,
        });
    }

    if surfaces_truncated || summary.nodes_truncated || summary.strings_truncated {
        disposition = NativeObservationDisposition::Partial;
    }
    Ok(PlatformSnapshot {
        disposition,
        monitor_disposition: NativeMonitorDisposition::NotSupported,
        monitor_count: 0,
        surfaces,
        summary,
        surfaces_truncated,
    })
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    all(target_os = "linux", target_arch = "x86_64")
)))]
fn observe_platform(
    request: NativeObservationRequest,
) -> Result<PlatformSnapshot, NativeObservationError> {
    empty_platform_snapshot(
        request,
        NativeObservationDisposition::NotSupported,
        NativeMonitorDisposition::NotSupported,
    )
}

fn empty_platform_snapshot(
    request: NativeObservationRequest,
    disposition: NativeObservationDisposition,
    monitor_disposition: NativeMonitorDisposition,
) -> Result<PlatformSnapshot, NativeObservationError> {
    Ok(PlatformSnapshot {
        disposition,
        monitor_disposition,
        monitor_count: 0,
        surfaces: Vec::new(),
        summary: SummaryAccumulator::new(request.limits),
        surfaces_truncated: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest_for(byte: u8) -> BindingDigest {
        BindingDigest::new([byte; 32])
    }

    fn request() -> NativeObservationRequest {
        NativeObservationRequest {
            platform: DesktopPlatform::Linux,
            session_kind: DesktopSessionKind::LinuxX11,
            capability_session_evidence: digest_for(7),
            observed_at_unix_ms: 1_900_000_000_000,
            observation_generation: 4,
            limits: DesktopLimits::default(),
        }
    }

    #[test]
    fn request_rejects_platform_session_substitution() {
        let mut request = request();
        request.session_kind = DesktopSessionKind::WindowsInteractive;
        assert!(matches!(
            request.validate(),
            Err(NativeObservationError::InvalidRequest(_))
        ));
    }

    #[test]
    fn sanitizer_drops_controls_and_attests_truncation() {
        let limits = DesktopLimits {
            max_string_bytes: 4,
            ..DesktopLimits::default()
        };
        let mut summary = SummaryAccumulator::new(limits);
        assert!(summary.record_node("ref", "button", Some("a\u{202e}bcdef"), None, &[], "state"));
        assert!(summary.strings_truncated);
        assert!(summary.string_bytes <= limits.max_string_bytes);
    }

    #[test]
    fn node_budget_fails_closed_to_explicit_truncation() {
        let limits = DesktopLimits {
            max_semantic_nodes: 1,
            ..DesktopLimits::default()
        };
        let mut summary = SummaryAccumulator::new(limits);
        assert!(summary.record_node("one", "button", None, None, &[], "state"));
        assert!(!summary.record_node("two", "button", None, None, &[], "state"));
        assert!(summary.nodes_truncated);
        assert_eq!(summary.node_count, 1);
    }

    #[test]
    fn opaque_ids_change_with_platform_reference_not_display_text() {
        let first = id_from_parts(&[SEMANTIC_ID_DOMAIN, b"stable-ref-a"]);
        let same = id_from_parts(&[SEMANTIC_ID_DOMAIN, b"stable-ref-a"]);
        let different = id_from_parts(&[SEMANTIC_ID_DOMAIN, b"stable-ref-b"]);
        assert_eq!(first, same);
        assert_ne!(first, different);
        assert_ne!(first, 0);
    }

    #[test]
    fn empty_snapshot_still_binds_limits_and_session_evidence() {
        let request = request();
        let report = finalize(
            request,
            empty_platform_snapshot(
                request,
                NativeObservationDisposition::NotSupported,
                NativeMonitorDisposition::NotSupported,
            )
            .expect("empty snapshot"),
        )
        .expect("finalize");
        assert_eq!(report.observation.limits, request.limits);
        assert_eq!(
            report.observation.capability_session_evidence,
            request.capability_session_evidence
        );
        assert!(report.observation.work_surface_digests.is_empty());
        assert_eq!(
            report.disposition,
            NativeObservationDisposition::NotSupported
        );
    }
}
