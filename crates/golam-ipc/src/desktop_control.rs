#![forbid(unsafe_code)]

use core::fmt;

pub const DESKTOP_CONTROL_REPLY_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopControlSurfaceState {
    NoLease,
    AgentAllowed,
    Paused,
    HumanExclusive,
    Revoked,
}

impl DesktopControlSurfaceState {
    const fn code(self) -> u8 {
        match self {
            Self::NoLease => 0,
            Self::AgentAllowed => 1,
            Self::Paused => 2,
            Self::HumanExclusive => 3,
            Self::Revoked => 4,
        }
    }

    const fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::NoLease),
            1 => Some(Self::AgentAllowed),
            2 => Some(Self::Paused),
            3 => Some(Self::HumanExclusive),
            4 => Some(Self::Revoked),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoLease => "no-lease",
            Self::AgentAllowed => "agent-allowed",
            Self::Paused => "paused",
            Self::HumanExclusive => "human-exclusive",
            Self::Revoked => "revoked",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopControlSurfaceReply {
    pub state: DesktopControlSurfaceState,
    pub visible_channel_qualified: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopControlReplyCodecError {
    InvalidLength { actual: usize },
    UnsupportedVersion(u8),
    UnknownState(u8),
    InvalidBoolean(u8),
}

impl fmt::Display for DesktopControlReplyCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { actual } => {
                write!(f, "desktop control reply has invalid length {actual}")
            }
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported desktop control reply version {version}")
            }
            Self::UnknownState(state) => write!(f, "unknown desktop control state {state}"),
            Self::InvalidBoolean(value) => {
                write!(f, "desktop control reply has invalid boolean {value}")
            }
        }
    }
}

impl std::error::Error for DesktopControlReplyCodecError {}

pub fn encode_desktop_control_reply(reply: DesktopControlSurfaceReply) -> [u8; 3] {
    [
        DESKTOP_CONTROL_REPLY_VERSION,
        reply.state.code(),
        u8::from(reply.visible_channel_qualified),
    ]
}

pub fn decode_desktop_control_reply(
    bytes: &[u8],
) -> Result<DesktopControlSurfaceReply, DesktopControlReplyCodecError> {
    if bytes.len() != 3 {
        return Err(DesktopControlReplyCodecError::InvalidLength {
            actual: bytes.len(),
        });
    }
    if bytes[0] != DESKTOP_CONTROL_REPLY_VERSION {
        return Err(DesktopControlReplyCodecError::UnsupportedVersion(bytes[0]));
    }
    let state = DesktopControlSurfaceState::from_code(bytes[1])
        .ok_or(DesktopControlReplyCodecError::UnknownState(bytes[1]))?;
    let visible_channel_qualified = match bytes[2] {
        0 => false,
        1 => true,
        value => return Err(DesktopControlReplyCodecError::InvalidBoolean(value)),
    };
    Ok(DesktopControlSurfaceReply {
        state,
        visible_channel_qualified,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitized_desktop_control_reply_round_trips_every_state() {
        for state in [
            DesktopControlSurfaceState::NoLease,
            DesktopControlSurfaceState::AgentAllowed,
            DesktopControlSurfaceState::Paused,
            DesktopControlSurfaceState::HumanExclusive,
            DesktopControlSurfaceState::Revoked,
        ] {
            for visible_channel_qualified in [false, true] {
                let reply = DesktopControlSurfaceReply {
                    state,
                    visible_channel_qualified,
                };
                assert_eq!(
                    decode_desktop_control_reply(&encode_desktop_control_reply(reply)).unwrap(),
                    reply
                );
            }
        }
    }

    #[test]
    fn malformed_or_extended_replies_fail_closed() {
        assert!(matches!(
            decode_desktop_control_reply(&[1, 0]),
            Err(DesktopControlReplyCodecError::InvalidLength { .. })
        ));
        assert_eq!(
            decode_desktop_control_reply(&[2, 0, 0]),
            Err(DesktopControlReplyCodecError::UnsupportedVersion(2))
        );
        assert_eq!(
            decode_desktop_control_reply(&[1, 9, 0]),
            Err(DesktopControlReplyCodecError::UnknownState(9))
        );
        assert_eq!(
            decode_desktop_control_reply(&[1, 0, 2]),
            Err(DesktopControlReplyCodecError::InvalidBoolean(2))
        );
    }
}
