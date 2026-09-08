#![forbid(unsafe_code)]

use core::fmt;

use golam_core::desktop_backend::{
    CaptureBackendReceipt, DesktopBackendTerminalStatus, ValidatedCaptureIntent,
};
use golam_core::desktop_intent::{CaptureIntent, CaptureLimits};
use golam_core::digest::sha256;
use golam_core::tool_request::BindingDigest;

const NATIVE_CAPTURE_PAYLOAD_DOMAIN: &[u8] = b"golam:native-capture-payload:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EphemeralCaptureMetadata {
    pub source_identity_digest: BindingDigest,
    pub payload_digest: BindingDigest,
    pub width: u32,
    pub height: u32,
    pub payload_bytes: u32,
}

/// Raw capture bytes live only inside `golamd` until an explicitly authorized
/// local consumer takes the one-shot payload. The type intentionally does not
/// implement `Clone` or `Debug`, and `Drop` overwrites the backing allocation.
pub(crate) struct EphemeralCapturePayload {
    metadata: EphemeralCaptureMetadata,
    bytes: Vec<u8>,
}

impl EphemeralCapturePayload {
    fn new(
        intent: &CaptureIntent,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    ) -> Result<Self, NativeCaptureError> {
        intent
            .validate()
            .map_err(|_| NativeCaptureError::InvalidIntent)?;
        validate_frame_bounds(intent.limits, width, height, bytes.len())?;

        let payload_bytes = u32::try_from(bytes.len())
            .map_err(|_| NativeCaptureError::PayloadLimitExceeded)?;
        let payload_digest = capture_payload_digest(
            intent.selected_source_identity_digest,
            width,
            height,
            &bytes,
        );
        Ok(Self {
            metadata: EphemeralCaptureMetadata {
                source_identity_digest: intent.selected_source_identity_digest,
                payload_digest,
                width,
                height,
                payload_bytes,
            },
            bytes,
        })
    }

    fn receipt(&self) -> CaptureBackendReceipt {
        CaptureBackendReceipt {
            status: DesktopBackendTerminalStatus::Committed,
            source_identity_digest: self.metadata.source_identity_digest,
            payload_digest: self.metadata.payload_digest,
            payload_bytes: self.metadata.payload_bytes,
        }
    }
}

impl Drop for EphemeralCapturePayload {
    fn drop(&mut self) {
        self.bytes.fill(0);
    }
}

#[derive(Default)]
pub(crate) struct EphemeralCaptureSlot {
    payload: Option<EphemeralCapturePayload>,
}

impl EphemeralCaptureSlot {
    pub(crate) fn stage_validated(
        &mut self,
        intent: ValidatedCaptureIntent<'_>,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    ) -> Result<CaptureBackendReceipt, NativeCaptureError> {
        self.stage(intent.intent(), width, height, bytes)
    }

    fn stage(
        &mut self,
        intent: &CaptureIntent,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    ) -> Result<CaptureBackendReceipt, NativeCaptureError> {
        // Replacing an unconsumed frame deterministically drops and scrubs it.
        self.payload = None;
        let payload = EphemeralCapturePayload::new(intent, width, height, bytes)?;
        let receipt = payload.receipt();
        self.payload = Some(payload);
        Ok(receipt)
    }

    pub(crate) fn consume_exact<R>(
        &mut self,
        expected_source_identity_digest: BindingDigest,
        expected_payload_digest: BindingDigest,
        consumer: impl FnOnce(&[u8], EphemeralCaptureMetadata) -> R,
    ) -> Result<R, NativeCaptureError> {
        let payload = self
            .payload
            .take()
            .ok_or(NativeCaptureError::NoEphemeralPayload)?;
        if payload.metadata.source_identity_digest != expected_source_identity_digest {
            return Err(NativeCaptureError::SourceBindingMismatch);
        }
        if payload.metadata.payload_digest != expected_payload_digest {
            return Err(NativeCaptureError::PayloadBindingMismatch);
        }
        let metadata = payload.metadata;
        Ok(consumer(&payload.bytes, metadata))
    }

    pub(crate) fn clear(&mut self) {
        self.payload = None;
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.payload.is_none()
    }
}

fn validate_frame_bounds(
    limits: CaptureLimits,
    width: u32,
    height: u32,
    payload_len: usize,
) -> Result<(), NativeCaptureError> {
    limits
        .validate()
        .map_err(|_| NativeCaptureError::InvalidIntent)?;
    if width == 0 || height == 0 {
        return Err(NativeCaptureError::InvalidFrameDimensions);
    }
    if width > limits.max_width || height > limits.max_height {
        return Err(NativeCaptureError::DimensionLimitExceeded);
    }
    if payload_len == 0
        || payload_len > usize::try_from(limits.max_frame_bytes).unwrap_or(usize::MAX)
    {
        return Err(NativeCaptureError::PayloadLimitExceeded);
    }
    Ok(())
}

fn capture_payload_digest(
    source_identity_digest: BindingDigest,
    width: u32,
    height: u32,
    bytes: &[u8],
) -> BindingDigest {
    let mut input = Vec::with_capacity(
        NATIVE_CAPTURE_PAYLOAD_DOMAIN.len() + 32 + 8 + bytes.len(),
    );
    input.extend_from_slice(NATIVE_CAPTURE_PAYLOAD_DOMAIN);
    input.extend_from_slice(&source_identity_digest.bytes());
    input.extend_from_slice(&width.to_le_bytes());
    input.extend_from_slice(&height.to_le_bytes());
    input.extend_from_slice(bytes);
    BindingDigest::new(sha256(&input))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeCaptureError {
    InvalidIntent,
    InvalidFrameDimensions,
    DimensionLimitExceeded,
    PayloadLimitExceeded,
    NoEphemeralPayload,
    SourceBindingMismatch,
    PayloadBindingMismatch,
}

impl fmt::Display for NativeCaptureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIntent => formatter.write_str("invalid native capture intent"),
            Self::InvalidFrameDimensions => {
                formatter.write_str("native capture frame dimensions are invalid")
            }
            Self::DimensionLimitExceeded => {
                formatter.write_str("native capture frame dimensions exceed the bound")
            }
            Self::PayloadLimitExceeded => {
                formatter.write_str("native capture payload exceeds the bound")
            }
            Self::NoEphemeralPayload => formatter.write_str("no ephemeral capture payload"),
            Self::SourceBindingMismatch => {
                formatter.write_str("ephemeral capture source binding mismatch")
            }
            Self::PayloadBindingMismatch => {
                formatter.write_str("ephemeral capture payload binding mismatch")
            }
        }
    }
}

impl std::error::Error for NativeCaptureError {}

#[cfg(test)]
mod tests {
    use golam_core::desktop_control::DESKTOP_CONTROL_SCHEMA_VERSION;
    use golam_core::desktop_intent::{
        AuthorityBindings, CaptureRetentionPolicy, EffectBinding, RequestBinding,
    };
    use golam_core::tool_request::ToolRequestId;
    use golam_core::EffectId;

    use super::*;

    fn digest(byte: u8) -> BindingDigest {
        BindingDigest::new([byte; 32])
    }

    fn intent() -> CaptureIntent {
        CaptureIntent {
            schema_version: DESKTOP_CONTROL_SCHEMA_VERSION,
            request: RequestBinding {
                request_id: ToolRequestId::from_u128(1),
                canonical_request_digest: digest(1),
            },
            effect: EffectBinding {
                effect_id: EffectId(2),
                immutable_effect_digest: digest(2),
                gate_authorization_digest: digest(3),
            },
            selected_source_identity_digest: digest(4),
            authority: AuthorityBindings {
                capability_ref: digest(5),
                policy_ref: digest(6),
                approval_ref: digest(7),
            },
            limits: CaptureLimits {
                max_width: 16,
                max_height: 16,
                max_frame_bytes: 1_024,
                max_duration_ms: 1_000,
            },
            include_cursor: false,
            audio_enabled: false,
            prepared_permission_session_evidence_ref: digest(8),
            retention_policy: CaptureRetentionPolicy::EphemeralOnly,
            expires_at_unix_ms: 10_000,
        }
    }

    #[test]
    fn staged_capture_returns_metadata_only_receipt_and_consumes_once() {
        let intent = intent();
        let mut slot = EphemeralCaptureSlot::default();
        let receipt = slot.stage(&intent, 2, 2, vec![11; 16]).unwrap();

        assert_eq!(receipt.status, DesktopBackendTerminalStatus::Committed);
        assert_eq!(receipt.source_identity_digest, intent.selected_source_identity_digest);
        assert_eq!(receipt.payload_bytes, 16);
        assert_ne!(receipt.payload_digest.bytes(), [0; 32]);

        let observed = slot
            .consume_exact(
                receipt.source_identity_digest,
                receipt.payload_digest,
                |bytes, metadata| (bytes.to_vec(), metadata),
            )
            .unwrap();
        assert_eq!(observed.0, vec![11; 16]);
        assert_eq!(observed.1.payload_bytes, 16);
        assert!(slot.is_empty());
        assert!(matches!(
            slot.consume_exact(receipt.source_identity_digest, receipt.payload_digest, |_, _| ()),
            Err(NativeCaptureError::NoEphemeralPayload)
        ));
    }

    #[test]
    fn frame_bounds_fail_closed_before_payload_is_staged() {
        let intent = intent();
        let mut slot = EphemeralCaptureSlot::default();
        assert!(matches!(
            slot.stage(&intent, 17, 2, vec![1; 16]),
            Err(NativeCaptureError::DimensionLimitExceeded)
        ));
        assert!(slot.is_empty());
        assert!(matches!(
            slot.stage(&intent, 2, 2, vec![1; 1_025]),
            Err(NativeCaptureError::PayloadLimitExceeded)
        ));
        assert!(slot.is_empty());
        assert!(matches!(
            slot.stage(&intent, 0, 2, vec![1; 16]),
            Err(NativeCaptureError::InvalidFrameDimensions)
        ));
        assert!(slot.is_empty());
    }

    #[test]
    fn substituted_consumer_binding_drops_payload_without_exposure() {
        let intent = intent();
        let mut slot = EphemeralCaptureSlot::default();
        let receipt = slot.stage(&intent, 2, 2, vec![22; 16]).unwrap();

        assert!(matches!(
            slot.consume_exact(digest(99), receipt.payload_digest, |_, _| ()),
            Err(NativeCaptureError::SourceBindingMismatch)
        ));
        assert!(slot.is_empty());

        let receipt = slot.stage(&intent, 2, 2, vec![33; 16]).unwrap();
        assert!(matches!(
            slot.consume_exact(receipt.source_identity_digest, digest(98), |_, _| ()),
            Err(NativeCaptureError::PayloadBindingMismatch)
        ));
        assert!(slot.is_empty());
    }

    #[test]
    fn clearing_an_unconsumed_payload_removes_it() {
        let intent = intent();
        let mut slot = EphemeralCaptureSlot::default();
        let _ = slot.stage(&intent, 2, 2, vec![44; 16]).unwrap();
        assert!(!slot.is_empty());
        slot.clear();
        assert!(slot.is_empty());
    }
}
