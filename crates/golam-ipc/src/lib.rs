#![forbid(unsafe_code)]

use golam_core::{CoreError, PROTOCOL_VERSION, ResourceLimits};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameKind {
    Hello,
    Challenge,
    Authenticate,
    Ready,
    Request,
    Cancel,
    Reply,
    Event,
    Shutdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameHeader {
    pub protocol_version: u16,
    pub kind: FrameKind,
    pub request_id: Option<u64>,
    pub payload_len: u32,
}

impl FrameHeader {
    pub fn validate(self, limits: ResourceLimits) -> Result<Self, CoreError> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(CoreError::InvalidProtocolVersion);
        }
        if self.payload_len > limits.max_frame_bytes {
            return Err(CoreError::ResourceLimitExceeded);
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_frames_fail_closed() {
        let frame = FrameHeader {
            protocol_version: PROTOCOL_VERSION,
            kind: FrameKind::Hello,
            request_id: None,
            payload_len: ResourceLimits::default().max_frame_bytes + 1,
        };
        assert_eq!(
            frame.validate(ResourceLimits::default()),
            Err(CoreError::ResourceLimitExceeded)
        );
    }
}
