use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use golam_core::ResourceLimits;

use crate::lifecycle::LifecyclePhase;
use crate::{FrameHeader, FrameKind, IpcError};

pub const REQUEST_METHOD_BYTES: usize = 2;
pub const REPLY_STATUS_BYTES: usize = 2;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RequestId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MethodId(pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplyStatus {
    Ok,
    Cancelled,
    InvalidRequest,
    Denied,
    Failed,
}

impl ReplyStatus {
    const fn code(self) -> u16 {
        match self {
            Self::Ok => 0,
            Self::Cancelled => 1,
            Self::InvalidRequest => 2,
            Self::Denied => 3,
            Self::Failed => 4,
        }
    }

    const fn from_code(code: u16) -> Option<Self> {
        match code {
            0 => Some(Self::Ok),
            1 => Some(Self::Cancelled),
            2 => Some(Self::InvalidRequest),
            3 => Some(Self::Denied),
            4 => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestMessage {
    pub method: MethodId,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplyMessage {
    pub status: ReplyStatus,
    pub body: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingState {
    Active,
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientAction {
    Begin {
        request_id: RequestId,
        method: MethodId,
    },
    Cancel {
        request_id: RequestId,
    },
    CancelAlreadyRequested {
        request_id: RequestId,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settlement {
    pub request_id: RequestId,
    pub state: PendingState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServerAction {
    Reply {
        settlement: Settlement,
        message: ReplyMessage,
    },
    Event,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestProtocolError {
    Ipc(IpcError),
    NotReady { phase: LifecyclePhase },
    Closed,
    InvalidPendingLimit,
    InvalidRequestId,
    InvalidMethodId,
    PayloadLengthMismatch { declared: u32, actual: usize },
    RequestPayloadTooShort { actual: usize },
    UnexpectedCancelPayload { actual: usize },
    ReplyPayloadTooShort { actual: usize },
    UnknownReplyStatus { code: u16 },
    UnexpectedRequestId { kind: FrameKind },
    MissingRequestId { kind: FrameKind },
    ImpossibleClientDirection { kind: FrameKind },
    ImpossibleServerDirection { kind: FrameKind },
    DuplicateRequestId { request_id: RequestId },
    UnknownRequestId { request_id: RequestId },
    PendingLimitExceeded { maximum: u32 },
}

impl fmt::Display for RequestProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipc(error) => write!(f, "IPC application frame failed wire validation: {error}"),
            Self::NotReady { phase } => {
                write!(f, "IPC application request arrived before READY: {phase:?}")
            }
            Self::Closed => f.write_str("IPC application protocol is closed"),
            Self::InvalidPendingLimit => f.write_str("IPC pending-request limit must be non-zero"),
            Self::InvalidRequestId => f.write_str("IPC request id zero is reserved and invalid"),
            Self::InvalidMethodId => {
                f.write_str("IPC request method id zero is reserved and invalid")
            }
            Self::PayloadLengthMismatch { declared, actual } => write!(
                f,
                "IPC frame declared {declared} payload bytes but received {actual}"
            ),
            Self::RequestPayloadTooShort { actual } => write!(
                f,
                "IPC request payload has {actual} bytes; expected at least {REQUEST_METHOD_BYTES}"
            ),
            Self::UnexpectedCancelPayload { actual } => {
                write!(f, "IPC cancel payload must be empty; got {actual} bytes")
            }
            Self::ReplyPayloadTooShort { actual } => write!(
                f,
                "IPC reply payload has {actual} bytes; expected at least {REPLY_STATUS_BYTES}"
            ),
            Self::UnknownReplyStatus { code } => {
                write!(f, "IPC reply status {code} is unsupported")
            }
            Self::UnexpectedRequestId { kind } => {
                write!(f, "IPC {kind:?} frame must not carry a request id")
            }
            Self::MissingRequestId { kind } => {
                write!(f, "IPC {kind:?} frame requires a request id")
            }
            Self::ImpossibleClientDirection { kind } => {
                write!(f, "IPC client cannot send {kind:?} after READY")
            }
            Self::ImpossibleServerDirection { kind } => {
                write!(f, "IPC server cannot send {kind:?} after READY")
            }
            Self::DuplicateRequestId { request_id } => {
                write!(f, "IPC request id {} is already pending", request_id.0)
            }
            Self::UnknownRequestId { request_id } => {
                write!(f, "IPC request id {} is not pending", request_id.0)
            }
            Self::PendingLimitExceeded { maximum } => {
                write!(f, "IPC pending-request limit {maximum} exceeded")
            }
        }
    }
}

impl Error for RequestProtocolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Ipc(error) => Some(error),
            _ => None,
        }
    }
}

pub fn encode_request(message: &RequestMessage) -> Result<Vec<u8>, RequestProtocolError> {
    if message.method.0 == 0 {
        return Err(RequestProtocolError::InvalidMethodId);
    }
    let mut payload = Vec::with_capacity(REQUEST_METHOD_BYTES + message.body.len());
    payload.extend_from_slice(&message.method.0.to_be_bytes());
    payload.extend_from_slice(&message.body);
    Ok(payload)
}

pub fn decode_request(payload: &[u8]) -> Result<RequestMessage, RequestProtocolError> {
    if payload.len() < REQUEST_METHOD_BYTES {
        return Err(RequestProtocolError::RequestPayloadTooShort {
            actual: payload.len(),
        });
    }
    let method = MethodId(u16::from_be_bytes([payload[0], payload[1]]));
    if method.0 == 0 {
        return Err(RequestProtocolError::InvalidMethodId);
    }
    Ok(RequestMessage {
        method,
        body: payload[REQUEST_METHOD_BYTES..].to_vec(),
    })
}

pub fn encode_reply(message: &ReplyMessage) -> Vec<u8> {
    let mut payload = Vec::with_capacity(REPLY_STATUS_BYTES + message.body.len());
    payload.extend_from_slice(&message.status.code().to_be_bytes());
    payload.extend_from_slice(&message.body);
    payload
}

pub fn decode_reply(payload: &[u8]) -> Result<ReplyMessage, RequestProtocolError> {
    if payload.len() < REPLY_STATUS_BYTES {
        return Err(RequestProtocolError::ReplyPayloadTooShort {
            actual: payload.len(),
        });
    }
    let code = u16::from_be_bytes([payload[0], payload[1]]);
    let status =
        ReplyStatus::from_code(code).ok_or(RequestProtocolError::UnknownReplyStatus { code })?;
    Ok(ReplyMessage {
        status,
        body: payload[REPLY_STATUS_BYTES..].to_vec(),
    })
}

pub struct ServerRequestTracker {
    pending: BTreeMap<RequestId, PendingState>,
    limits: ResourceLimits,
    closed: bool,
}

impl ServerRequestTracker {
    pub fn new(
        phase: LifecyclePhase,
        limits: ResourceLimits,
    ) -> Result<Self, RequestProtocolError> {
        if phase != LifecyclePhase::Ready {
            return Err(RequestProtocolError::NotReady { phase });
        }
        if limits.max_pending_requests == 0 {
            return Err(RequestProtocolError::InvalidPendingLimit);
        }
        Ok(Self {
            pending: BTreeMap::new(),
            limits,
            closed: false,
        })
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn pending_state(&self, request_id: RequestId) -> Option<PendingState> {
        self.pending.get(&request_id).copied()
    }

    pub fn receive_client_frame(
        &mut self,
        header: FrameHeader,
        payload: &[u8],
    ) -> Result<ClientAction, RequestProtocolError> {
        if self.closed {
            return Err(RequestProtocolError::Closed);
        }
        self.validate_frame_payload(header, payload)?;
        match header.kind {
            FrameKind::Request => self.receive_request(header, payload),
            FrameKind::Cancel => self.receive_cancel(header, payload),
            kind => self.breach(RequestProtocolError::ImpossibleClientDirection { kind }),
        }
    }

    pub fn settle_server_frame(
        &mut self,
        header: FrameHeader,
        payload: &[u8],
    ) -> Result<ServerAction, RequestProtocolError> {
        if self.closed {
            return Err(RequestProtocolError::Closed);
        }
        self.validate_frame_payload(header, payload)?;
        match header.kind {
            FrameKind::Reply => {
                let request_id = request_id_from_header(header)?;
                if let Err(error) = validate_request_id(request_id) {
                    return self.breach(error);
                }
                let message = match decode_reply(payload) {
                    Ok(message) => message,
                    Err(error) => return self.breach(error),
                };
                let state = match self.pending.remove(&request_id) {
                    Some(state) => state,
                    None => {
                        return self.breach(RequestProtocolError::UnknownRequestId { request_id });
                    }
                };
                Ok(ServerAction::Reply {
                    settlement: Settlement { request_id, state },
                    message,
                })
            }
            FrameKind::Event => Ok(ServerAction::Event),
            kind => self.breach(RequestProtocolError::ImpossibleServerDirection { kind }),
        }
    }

    pub fn close(&mut self) {
        self.closed = true;
        self.pending.clear();
    }

    fn receive_request(
        &mut self,
        header: FrameHeader,
        payload: &[u8],
    ) -> Result<ClientAction, RequestProtocolError> {
        let request_id = request_id_from_header(header)?;
        if let Err(error) = validate_request_id(request_id) {
            return self.breach(error);
        }
        let request = match decode_request(payload) {
            Ok(request) => request,
            Err(error) => return self.breach(error),
        };
        if self.pending.contains_key(&request_id) {
            return self.breach(RequestProtocolError::DuplicateRequestId { request_id });
        }
        if self.pending.len() >= self.limits.max_pending_requests as usize {
            return self.breach(RequestProtocolError::PendingLimitExceeded {
                maximum: self.limits.max_pending_requests,
            });
        }
        self.pending.insert(request_id, PendingState::Active);
        Ok(ClientAction::Begin {
            request_id,
            method: request.method,
        })
    }

    fn receive_cancel(
        &mut self,
        header: FrameHeader,
        payload: &[u8],
    ) -> Result<ClientAction, RequestProtocolError> {
        if !payload.is_empty() {
            return self.breach(RequestProtocolError::UnexpectedCancelPayload {
                actual: payload.len(),
            });
        }
        let request_id = request_id_from_header(header)?;
        if let Err(error) = validate_request_id(request_id) {
            return self.breach(error);
        }
        let state = match self.pending.get_mut(&request_id) {
            Some(state) => state,
            None => return self.breach(RequestProtocolError::UnknownRequestId { request_id }),
        };
        match state {
            PendingState::Active => {
                *state = PendingState::Cancelled;
                Ok(ClientAction::Cancel { request_id })
            }
            PendingState::Cancelled => Ok(ClientAction::CancelAlreadyRequested { request_id }),
        }
    }

    fn validate_frame_payload(
        &mut self,
        header: FrameHeader,
        payload: &[u8],
    ) -> Result<(), RequestProtocolError> {
        if let Err(error) = header.validate(self.limits) {
            return self.breach(RequestProtocolError::Ipc(error));
        }
        let actual = payload.len();
        if usize::try_from(header.payload_len)
            .expect("u32 payload length fits usize on supported platforms")
            != actual
        {
            return self.breach(RequestProtocolError::PayloadLengthMismatch {
                declared: header.payload_len,
                actual,
            });
        }
        Ok(())
    }

    fn breach<T>(&mut self, error: RequestProtocolError) -> Result<T, RequestProtocolError> {
        self.closed = true;
        self.pending.clear();
        Err(error)
    }
}

fn request_id_from_header(header: FrameHeader) -> Result<RequestId, RequestProtocolError> {
    header
        .request_id
        .map(RequestId)
        .ok_or(RequestProtocolError::MissingRequestId { kind: header.kind })
}

fn validate_request_id(request_id: RequestId) -> Result<(), RequestProtocolError> {
    if request_id.0 == 0 {
        Err(RequestProtocolError::InvalidRequestId)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golam_core::PROTOCOL_VERSION;

    fn header(kind: FrameKind, id: Option<u64>, payload_len: usize) -> FrameHeader {
        FrameHeader {
            protocol_version: PROTOCOL_VERSION,
            kind,
            request_id: id,
            payload_len: u32::try_from(payload_len).unwrap(),
        }
    }

    fn request(method: u16, body: &[u8]) -> Vec<u8> {
        encode_request(&RequestMessage {
            method: MethodId(method),
            body: body.to_vec(),
        })
        .unwrap()
    }

    #[test]
    fn request_and_reply_payloads_round_trip() {
        let request = RequestMessage {
            method: MethodId(7),
            body: b"abc".to_vec(),
        };
        assert_eq!(
            decode_request(&encode_request(&request).unwrap()).unwrap(),
            request
        );
        let reply = ReplyMessage {
            status: ReplyStatus::Denied,
            body: b"why".to_vec(),
        };
        assert_eq!(decode_reply(&encode_reply(&reply)).unwrap(), reply);
    }

    #[test]
    fn tracker_requires_ready() {
        assert!(matches!(
            ServerRequestTracker::new(LifecyclePhase::ChallengeSent, ResourceLimits::default()),
            Err(RequestProtocolError::NotReady {
                phase: LifecyclePhase::ChallengeSent
            })
        ));
    }

    #[test]
    fn cancel_remains_pending_until_executor_settles() {
        let limits = ResourceLimits {
            max_pending_requests: 2,
            ..ResourceLimits::default()
        };
        let mut tracker = ServerRequestTracker::new(LifecyclePhase::Ready, limits).unwrap();
        let payload = request(1, b"work");
        assert_eq!(
            tracker
                .receive_client_frame(header(FrameKind::Request, Some(9), payload.len()), &payload)
                .unwrap(),
            ClientAction::Begin {
                request_id: RequestId(9),
                method: MethodId(1)
            }
        );
        assert_eq!(
            tracker
                .receive_client_frame(header(FrameKind::Cancel, Some(9), 0), b"")
                .unwrap(),
            ClientAction::Cancel {
                request_id: RequestId(9)
            }
        );
        assert_eq!(
            tracker.pending_state(RequestId(9)),
            Some(PendingState::Cancelled)
        );
        let reply = encode_reply(&ReplyMessage {
            status: ReplyStatus::Cancelled,
            body: Vec::new(),
        });
        assert_eq!(
            tracker
                .settle_server_frame(header(FrameKind::Reply, Some(9), reply.len()), &reply)
                .unwrap(),
            ServerAction::Reply {
                settlement: Settlement {
                    request_id: RequestId(9),
                    state: PendingState::Cancelled,
                },
                message: ReplyMessage {
                    status: ReplyStatus::Cancelled,
                    body: Vec::new(),
                },
            }
        );
        assert_eq!(tracker.pending_len(), 0);
    }

    #[test]
    fn duplicate_ids_and_limit_breaches_close_connection_state() {
        let limits = ResourceLimits {
            max_pending_requests: 1,
            ..ResourceLimits::default()
        };
        let mut duplicate = ServerRequestTracker::new(LifecyclePhase::Ready, limits).unwrap();
        let payload = request(1, b"");
        duplicate
            .receive_client_frame(header(FrameKind::Request, Some(1), payload.len()), &payload)
            .unwrap();
        assert!(matches!(
            duplicate
                .receive_client_frame(header(FrameKind::Request, Some(1), payload.len()), &payload),
            Err(RequestProtocolError::DuplicateRequestId { .. })
        ));
        assert!(duplicate.is_closed());

        let mut limited = ServerRequestTracker::new(LifecyclePhase::Ready, limits).unwrap();
        limited
            .receive_client_frame(header(FrameKind::Request, Some(1), payload.len()), &payload)
            .unwrap();
        assert!(matches!(
            limited
                .receive_client_frame(header(FrameKind::Request, Some(2), payload.len()), &payload),
            Err(RequestProtocolError::PendingLimitExceeded { maximum: 1 })
        ));
        assert!(limited.is_closed());
    }

    #[test]
    fn unknown_cancel_and_impossible_direction_close_connection_state() {
        let mut tracker =
            ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
        assert!(matches!(
            tracker.receive_client_frame(header(FrameKind::Cancel, Some(44), 0), b""),
            Err(RequestProtocolError::UnknownRequestId { .. })
        ));
        assert!(tracker.is_closed());

        let mut tracker =
            ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
        assert!(matches!(
            tracker.receive_client_frame(header(FrameKind::Reply, Some(1), 0), b""),
            Err(RequestProtocolError::ImpossibleClientDirection {
                kind: FrameKind::Reply
            })
        ));
        assert!(tracker.is_closed());
    }

    #[test]
    fn zero_request_id_and_malformed_request_close_connection_state() {
        let mut tracker =
            ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
        let payload = request(1, b"");
        assert!(matches!(
            tracker
                .receive_client_frame(header(FrameKind::Request, Some(0), payload.len()), &payload),
            Err(RequestProtocolError::InvalidRequestId)
        ));
        assert!(tracker.is_closed());

        let mut tracker =
            ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
        assert!(matches!(
            tracker.receive_client_frame(header(FrameKind::Request, Some(1), 1), &[0]),
            Err(RequestProtocolError::RequestPayloadTooShort { actual: 1 })
        ));
        assert!(tracker.is_closed());
    }

    #[test]
    fn frame_payload_mismatch_and_nonempty_cancel_are_protocol_breaches() {
        let mut tracker =
            ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
        let payload = request(1, b"");
        assert!(matches!(
            tracker.receive_client_frame(
                header(FrameKind::Request, Some(1), payload.len() + 1),
                &payload,
            ),
            Err(RequestProtocolError::PayloadLengthMismatch { .. })
        ));
        assert!(tracker.is_closed());

        let mut tracker =
            ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
        let payload = request(1, b"");
        tracker
            .receive_client_frame(header(FrameKind::Request, Some(1), payload.len()), &payload)
            .unwrap();
        assert!(matches!(
            tracker.receive_client_frame(header(FrameKind::Cancel, Some(1), 1), &[1]),
            Err(RequestProtocolError::UnexpectedCancelPayload { actual: 1 })
        ));
        assert!(tracker.is_closed());
    }

    #[test]
    fn zero_pending_limit_is_rejected() {
        let limits = ResourceLimits {
            max_pending_requests: 0,
            ..ResourceLimits::default()
        };
        assert!(matches!(
            ServerRequestTracker::new(LifecyclePhase::Ready, limits),
            Err(RequestProtocolError::InvalidPendingLimit)
        ));
    }

    #[test]
    fn server_reply_must_reference_pending_request() {
        let mut tracker =
            ServerRequestTracker::new(LifecyclePhase::Ready, ResourceLimits::default()).unwrap();
        assert!(matches!(
            tracker.settle_server_frame(header(FrameKind::Reply, Some(9), 2), &[0, 0]),
            Err(RequestProtocolError::UnknownRequestId { .. })
        ));
        assert!(tracker.is_closed());
    }
}
