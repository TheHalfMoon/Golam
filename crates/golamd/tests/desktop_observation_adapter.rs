#![forbid(unsafe_code)]

#[path = "../src/desktop_observation.rs"]
pub mod desktop_observation;

use desktop_observation::{NativeObservationRequest, observe_native_desktop};
use golam_core::desktop_control::{
    DesktopLimits, DesktopPlatform, DesktopSessionKind,
};
use golam_core::tool_request::BindingDigest;

#[test]
fn native_observation_adapter_is_linked_and_request_validation_is_fail_closed() {
    let observe = observe_native_desktop
        as fn(
            NativeObservationRequest,
        ) -> Result<
            desktop_observation::NativeObservationReport,
            desktop_observation::NativeObservationError,
        >;
    let _ = observe;

    let valid = NativeObservationRequest {
        platform: DesktopPlatform::Linux,
        session_kind: DesktopSessionKind::LinuxX11,
        capability_session_evidence: BindingDigest::new([7; 32]),
        observed_at_unix_ms: 1_900_000_000_000,
        observation_generation: 1,
        limits: DesktopLimits::default(),
    };
    assert!(valid.validate().is_ok());

    let substituted = NativeObservationRequest {
        session_kind: DesktopSessionKind::WindowsInteractive,
        ..valid
    };
    assert!(substituted.validate().is_err());
}
