#![forbid(unsafe_code)]

pub mod lifecycle;

use std::error::Error;
use std::fmt;

use golam_core::{PROTOCOL_VERSION, ResourceLimits};

pub const FRAME_MAGIC: [u8; 4] = *b"GIPC";
pub const FRAME_HEADER_LEN: usize = 20;
const FLAG_REQUEST_ID: u8 = 0x01;
const KNOWN_FLAGS: u8 = FLAG_REQUEST_ID;

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

impl FrameKind {
    pub const fn code(self) -> u8 {
        match self {
            Self::Hello => 1,
            Self::Challenge => 2,
            Self::Authenticate => 3,
            Self::Ready => 4,
            Self::Request => 5,
            Self::Cancel => 6,
            Self::Reply => 7,
            Self::Event => 8,
            Self::Shutdown => 9,
        }
    }

    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Hello),
            2 => Some(Self::Challenge),
            3 => Some(Self::Authenticate),
            4 => Some(Self::Ready),
            5 => Some(Self::Request),
            6 => Some(Self::Cancel),
            7 => Some(Self::Reply),
            8 => Some(Self::Event),
            9 => Some(Self::Shutdown),
            _ => None,
        }
    }

    pub const fn requires_request_id(self) -> bool {
        matches!(self, Self::Request | Self::Cancel | Self::Reply)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameHeader {
    pub protocol_version: u16,
    pub kind: FrameKind,
    pub request_id: Option<u64>,
    pub payload_len: u32,
}

impl FrameHeader {
    pub fn validate(self, limits: ResourceLimits) -> Result<Self, IpcError> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(IpcError::UnsupportedProtocolVersion {
                found: self.protocol_version,
            });
        }
        validate_request_id(self.kind, self.request_id)?;
        validate_frame_size(self.payload_len, limits)?;
        Ok(self)
    }

    pub fn frame_len(self) -> usize {
        FRAME_HEADER_LEN + self.payload_len as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodedFrame<'a> {
    pub header: FrameHeader,
    pub payload: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpcError {
    TruncatedHeader { actual: usize },
    InvalidMagic,
    UnsupportedProtocolVersion { found: u16 },
    UnknownFrameKind { code: u8 },
    UnknownFlags { flags: u8 },
    NonCanonicalRequestId,
    MissingRequestId { kind: FrameKind },
    UnexpectedRequestId { kind: FrameKind },
    PayloadLengthOverflow,
    FrameTooLarge { declared: u64, maximum: u32 },
    LengthMismatch { expected: usize, actual: usize },
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader { actual } => write!(
                f,
                "IPC frame header is truncated: expected {FRAME_HEADER_LEN} bytes, got {actual}"
            ),
            Self::InvalidMagic => f.write_str("IPC frame magic is invalid"),
            Self::UnsupportedProtocolVersion { found } => write!(
                f,
                "unsupported IPC protocol version {found}; expected {PROTOCOL_VERSION}"
            ),
            Self::UnknownFrameKind { code } => write!(f, "unknown IPC frame kind {code}"),
            Self::UnknownFlags { flags } => write!(f, "unknown IPC frame flags 0x{flags:02x}"),
            Self::NonCanonicalRequestId => {
                f.write_str("request-id bytes must be zero when the request-id flag is absent")
            }
            Self::MissingRequestId { kind } => {
                write!(f, "IPC frame {kind:?} requires a request id")
            }
            Self::UnexpectedRequestId { kind } => {
                write!(f, "IPC frame {kind:?} must not carry a request id")
            }
            Self::PayloadLengthOverflow => f.write_str("IPC payload length exceeds u32"),
            Self::FrameTooLarge { declared, maximum } => write!(
                f,
                "IPC frame declares {declared} bytes; maximum is {maximum}"
            ),
            Self::LengthMismatch { expected, actual } => write!(
                f,
                "IPC frame length mismatch: expected {expected} bytes, got {actual}"
            ),
        }
    }
}

impl Error for IpcError {}

pub fn encode_frame(
    kind: FrameKind,
    request_id: Option<u64>,
    payload: &[u8],
    limits: ResourceLimits,
) -> Result<Vec<u8>, IpcError> {
    let payload_len = u32::try_from(payload.len()).map_err(|_| IpcError::PayloadLengthOverflow)?;
    let header = FrameHeader {
        protocol_version: PROTOCOL_VERSION,
        kind,
        request_id,
        payload_len,
    }
    .validate(limits)?;

    let mut encoded = Vec::with_capacity(header.frame_len());
    encoded.extend_from_slice(&FRAME_MAGIC);
    encoded.extend_from_slice(&header.protocol_version.to_be_bytes());
    encoded.push(header.kind.code());
    encoded.push(if header.request_id.is_some() {
        FLAG_REQUEST_ID
    } else {
        0
    });
    encoded.extend_from_slice(&header.request_id.unwrap_or(0).to_be_bytes());
    encoded.extend_from_slice(&header.payload_len.to_be_bytes());
    encoded.extend_from_slice(payload);
    Ok(encoded)
}

pub fn decode_header(bytes: &[u8], limits: ResourceLimits) -> Result<FrameHeader, IpcError> {
    if bytes.len() < FRAME_HEADER_LEN {
        return Err(IpcError::TruncatedHeader {
            actual: bytes.len(),
        });
    }
    if bytes[..4] != FRAME_MAGIC {
        return Err(IpcError::InvalidMagic);
    }

    let protocol_version = u16::from_be_bytes([bytes[4], bytes[5]]);
    if protocol_version != PROTOCOL_VERSION {
        return Err(IpcError::UnsupportedProtocolVersion {
            found: protocol_version,
        });
    }

    let kind =
        FrameKind::from_code(bytes[6]).ok_or(IpcError::UnknownFrameKind { code: bytes[6] })?;
    let flags = bytes[7];
    if flags & !KNOWN_FLAGS != 0 {
        return Err(IpcError::UnknownFlags { flags });
    }

    let request_id_bytes: [u8; 8] = bytes[8..16]
        .try_into()
        .expect("fixed request-id header range");
    let encoded_request_id = u64::from_be_bytes(request_id_bytes);
    let request_id = if flags & FLAG_REQUEST_ID != 0 {
        Some(encoded_request_id)
    } else {
        if encoded_request_id != 0 {
            return Err(IpcError::NonCanonicalRequestId);
        }
        None
    };
    validate_request_id(kind, request_id)?;

    let payload_len_bytes: [u8; 4] = bytes[16..20]
        .try_into()
        .expect("fixed payload-length header range");
    let payload_len = u32::from_be_bytes(payload_len_bytes);
    validate_frame_size(payload_len, limits)?;

    Ok(FrameHeader {
        protocol_version,
        kind,
        request_id,
        payload_len,
    })
}

pub fn decode_exact(bytes: &[u8], limits: ResourceLimits) -> Result<DecodedFrame<'_>, IpcError> {
    let header = decode_header(bytes, limits)?;
    let expected = header.frame_len();
    if bytes.len() != expected {
        return Err(IpcError::LengthMismatch {
            expected,
            actual: bytes.len(),
        });
    }
    Ok(DecodedFrame {
        header,
        payload: &bytes[FRAME_HEADER_LEN..],
    })
}

fn validate_request_id(kind: FrameKind, request_id: Option<u64>) -> Result<(), IpcError> {
    if kind.requires_request_id() && request_id.is_none() {
        return Err(IpcError::MissingRequestId { kind });
    }
    if !kind.requires_request_id() && request_id.is_some() {
        return Err(IpcError::UnexpectedRequestId { kind });
    }
    Ok(())
}

fn validate_frame_size(payload_len: u32, limits: ResourceLimits) -> Result<(), IpcError> {
    let declared = u64::try_from(FRAME_HEADER_LEN).expect("frame header length fits u64")
        + u64::from(payload_len);
    if declared > u64::from(limits.max_frame_bytes) {
        return Err(IpcError::FrameTooLarge {
            declared,
            maximum: limits.max_frame_bytes,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_id(kind: FrameKind) -> Option<u64> {
        kind.requires_request_id().then_some(41)
    }

    #[test]
    fn all_frame_kinds_round_trip_without_payload_allocation_on_decode() {
        let kinds = [
            FrameKind::Hello,
            FrameKind::Challenge,
            FrameKind::Authenticate,
            FrameKind::Ready,
            FrameKind::Request,
            FrameKind::Cancel,
            FrameKind::Reply,
            FrameKind::Event,
            FrameKind::Shutdown,
        ];
        for kind in kinds {
            let encoded = encode_frame(
                kind,
                request_id(kind),
                b"payload",
                ResourceLimits::default(),
            )
            .unwrap();
            let decoded = decode_exact(&encoded, ResourceLimits::default()).unwrap();
            assert_eq!(decoded.header.kind, kind);
            assert_eq!(decoded.header.request_id, request_id(kind));
            assert_eq!(decoded.payload, b"payload");
        }
    }

    #[test]
    fn oversized_declared_frame_is_rejected_from_header_alone() {
        let limits = ResourceLimits {
            max_frame_bytes: 64,
            ..ResourceLimits::default()
        };
        let mut header = [0_u8; FRAME_HEADER_LEN];
        header[..4].copy_from_slice(&FRAME_MAGIC);
        header[4..6].copy_from_slice(&PROTOCOL_VERSION.to_be_bytes());
        header[6] = FrameKind::Hello.code();
        header[16..20].copy_from_slice(&100_u32.to_be_bytes());
        assert!(matches!(
            decode_header(&header, limits),
            Err(IpcError::FrameTooLarge { .. })
        ));
    }

    #[test]
    fn truncated_header_and_body_fail_closed() {
        assert!(matches!(
            decode_exact(&[0_u8; FRAME_HEADER_LEN - 1], ResourceLimits::default()),
            Err(IpcError::TruncatedHeader { .. })
        ));

        let encoded =
            encode_frame(FrameKind::Hello, None, b"body", ResourceLimits::default()).unwrap();
        assert!(matches!(
            decode_exact(&encoded[..encoded.len() - 1], ResourceLimits::default()),
            Err(IpcError::LengthMismatch { .. })
        ));
    }

    #[test]
    fn unknown_kind_version_flags_and_noncanonical_request_id_are_rejected() {
        let mut encoded =
            encode_frame(FrameKind::Hello, None, b"", ResourceLimits::default()).unwrap();

        encoded[6] = 250;
        assert_eq!(
            decode_exact(&encoded, ResourceLimits::default()),
            Err(IpcError::UnknownFrameKind { code: 250 })
        );

        encoded[6] = FrameKind::Hello.code();
        encoded[4..6].copy_from_slice(&(PROTOCOL_VERSION + 1).to_be_bytes());
        assert_eq!(
            decode_exact(&encoded, ResourceLimits::default()),
            Err(IpcError::UnsupportedProtocolVersion {
                found: PROTOCOL_VERSION + 1
            })
        );

        encoded[4..6].copy_from_slice(&PROTOCOL_VERSION.to_be_bytes());
        encoded[7] = 0x80;
        assert_eq!(
            decode_exact(&encoded, ResourceLimits::default()),
            Err(IpcError::UnknownFlags { flags: 0x80 })
        );

        encoded[7] = 0;
        encoded[8..16].copy_from_slice(&1_u64.to_be_bytes());
        assert_eq!(
            decode_exact(&encoded, ResourceLimits::default()),
            Err(IpcError::NonCanonicalRequestId)
        );
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut encoded =
            encode_frame(FrameKind::Event, None, b"ok", ResourceLimits::default()).unwrap();
        encoded.push(0);
        assert!(matches!(
            decode_exact(&encoded, ResourceLimits::default()),
            Err(IpcError::LengthMismatch { .. })
        ));
    }

    #[test]
    fn request_id_presence_rules_are_enforced() {
        assert_eq!(
            encode_frame(FrameKind::Request, None, b"", ResourceLimits::default()),
            Err(IpcError::MissingRequestId {
                kind: FrameKind::Request
            })
        );
        assert_eq!(
            encode_frame(FrameKind::Hello, Some(1), b"", ResourceLimits::default()),
            Err(IpcError::UnexpectedRequestId {
                kind: FrameKind::Hello
            })
        );
    }
}
