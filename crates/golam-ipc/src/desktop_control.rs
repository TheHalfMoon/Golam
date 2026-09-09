#![forbid(unsafe_code)]

use core::fmt;

use crate::request::{MethodId, RequestMessage};

pub const METHOD_DESKTOP_CONTROL_STATUS: MethodId = MethodId(600);
pub const METHOD_DESKTOP_CONTROL_HEARTBEAT: MethodId = MethodId(601);
pub const METHOD_DESKTOP_CONTROL_PAUSE: MethodId = MethodId(602);
pub const METHOD_DESKTOP_CONTROL_STOP: MethodId = MethodId(603);
pub const METHOD_DESKTOP_CONTROL_TAKEOVER: MethodId = MethodId(604);
pub const DESKTOP_CONTROL_REPLY_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopControlRequest {
    Status,
    Heartbeat,
    Pause,
    Stop,
    Takeover,
}

impl DesktopControlRequest {
    pub const fn method(self) -> MethodId {
        match self {
            Self::Status => METHOD_DESKTOP_CONTROL_STATUS,
            Self::Heartbeat => METHOD_DESKTOP_CONTROL_HEARTBEAT,
            Self::Pause => METHOD_DESKTOP_CONTROL_PAUSE,
            Self::Stop => METHOD_DESKTOP_CONTROL_STOP,
            Self::Takeover => METHOD_DESKTOP_CONTROL_TAKEOVER,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopControlRequestCodecError {
    UnknownMethod(u16),
    UnexpectedBody { actual: usize },
}

impl fmt::Display for DesktopControlRequestCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownMethod(method) => {
                write!(f, "unknown desktop control request method {method}")
            }
            Self::UnexpectedBody { actual } => {
                write!(
                    f,
                    "desktop control request body must be empty; got {actual} bytes"
                )
            }
        }
    }
}

impl std::error::Error for DesktopControlRequestCodecError {}

pub const fn is_desktop_control_method(method: MethodId) -> bool {
    matches!(
        method,
        METHOD_DESKTOP_CONTROL_STATUS
            | METHOD_DESKTOP_CONTROL_HEARTBEAT
            | METHOD_DESKTOP_CONTROL_PAUSE
            | METHOD_DESKTOP_CONTROL_STOP
            | METHOD_DESKTOP_CONTROL_TAKEOVER
    )
}

pub fn encode_desktop_control_request(request: DesktopControlRequest) -> RequestMessage {
    RequestMessage {
        method: request.method(),
        body: Vec::new(),
    }
}

pub fn decode_desktop_control_request(
    message: &RequestMessage,
) -> Result<DesktopControlRequest, DesktopControlRequestCodecError> {
    if !message.body.is_empty() {
        return Err(DesktopControlRequestCodecError::UnexpectedBody {
            actual: message.body.len(),
        });
    }
    match message.method {
        METHOD_DESKTOP_CONTROL_STATUS => Ok(DesktopControlRequest::Status),
        METHOD_DESKTOP_CONTROL_HEARTBEAT => Ok(DesktopControlRequest::Heartbeat),
        METHOD_DESKTOP_CONTROL_PAUSE => Ok(DesktopControlRequest::Pause),
        METHOD_DESKTOP_CONTROL_STOP => Ok(DesktopControlRequest::Stop),
        METHOD_DESKTOP_CONTROL_TAKEOVER => Ok(DesktopControlRequest::Takeover),
        MethodId(method) => Err(DesktopControlRequestCodecError::UnknownMethod(method)),
    }
}

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
    fn desktop_control_requests_round_trip_without_renderer_authority_fields() {
        for request in [
            DesktopControlRequest::Status,
            DesktopControlRequest::Heartbeat,
            DesktopControlRequest::Pause,
            DesktopControlRequest::Stop,
            DesktopControlRequest::Takeover,
        ] {
            let encoded = encode_desktop_control_request(request);
            assert!(encoded.body.is_empty());
            assert_eq!(decode_desktop_control_request(&encoded).unwrap(), request);
        }
    }

    #[test]
    fn desktop_control_request_body_and_unknown_methods_fail_closed() {
        let mut request = encode_desktop_control_request(DesktopControlRequest::Pause);
        request.body.push(1);
        assert_eq!(
            decode_desktop_control_request(&request),
            Err(DesktopControlRequestCodecError::UnexpectedBody { actual: 1 })
        );
        assert_eq!(
            decode_desktop_control_request(&RequestMessage {
                method: MethodId(999),
                body: Vec::new(),
            }),
            Err(DesktopControlRequestCodecError::UnknownMethod(999))
        );
    }

    #[test]
    fn desktop_control_method_classifier_is_exact() {
        for request in [
            DesktopControlRequest::Status,
            DesktopControlRequest::Heartbeat,
            DesktopControlRequest::Pause,
            DesktopControlRequest::Stop,
            DesktopControlRequest::Takeover,
        ] {
            assert!(is_desktop_control_method(request.method()));
        }
        assert!(!is_desktop_control_method(MethodId(0)));
        assert!(!is_desktop_control_method(MethodId(100)));
        assert!(!is_desktop_control_method(MethodId(605)));
    }

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
