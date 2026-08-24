#![forbid(unsafe_code)]

use core::fmt;

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SessionId(pub u128);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ClientId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
    pub max_frame_bytes: u32,
    pub max_pending_requests: u32,
    pub max_concurrent_clients: u16,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_frame_bytes: 1024 * 1024,
            max_pending_requests: 128,
            max_concurrent_clients: 16,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreError {
    InvalidProtocolVersion,
    ResourceLimitExceeded,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProtocolVersion => f.write_str("invalid protocol version"),
            Self::ResourceLimitExceeded => f.write_str("resource limit exceeded"),
        }
    }
}

impl std::error::Error for CoreError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_bounded() {
        let limits = ResourceLimits::default();
        assert!(limits.max_frame_bytes <= 1024 * 1024);
        assert!(limits.max_pending_requests > 0);
        assert!(limits.max_concurrent_clients > 0);
    }
}
