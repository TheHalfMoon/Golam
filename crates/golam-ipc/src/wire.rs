#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};

use golam_core::ResourceLimits;

use crate::{FRAME_HEADER_LEN, FrameHeader, IpcError, decode_header, encode_frame};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnedFrame {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub enum WireError {
    Io(io::Error),
    Ipc(IpcError),
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "IPC stream I/O error: {error}"),
            Self::Ipc(error) => write!(f, "IPC stream frame error: {error}"),
        }
    }
}

impl Error for WireError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Ipc(error) => Some(error),
        }
    }
}

impl From<io::Error> for WireError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<IpcError> for WireError {
    fn from(value: IpcError) -> Self {
        Self::Ipc(value)
    }
}

pub fn read_frame<R: Read>(
    reader: &mut R,
    limits: ResourceLimits,
) -> Result<OwnedFrame, WireError> {
    let mut header_bytes = [0_u8; FRAME_HEADER_LEN];
    reader.read_exact(&mut header_bytes)?;
    let header = decode_header(&header_bytes, limits)?;
    let payload_len = usize::try_from(header.payload_len).expect("u32 payload length fits usize");
    let mut payload = vec![0_u8; payload_len];
    reader.read_exact(&mut payload)?;
    Ok(OwnedFrame { header, payload })
}

pub fn write_frame<W: Write>(
    writer: &mut W,
    header: FrameHeader,
    payload: &[u8],
    limits: ResourceLimits,
) -> Result<(), WireError> {
    if usize::try_from(header.payload_len).expect("u32 payload length fits usize") != payload.len() {
        return Err(WireError::Ipc(IpcError::LengthMismatch {
            expected: header.frame_len(),
            actual: FRAME_HEADER_LEN + payload.len(),
        }));
    }
    let encoded = encode_frame(header.kind, header.request_id, payload, limits)?;
    writer.write_all(&encoded)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FrameKind;
    use golam_core::PROTOCOL_VERSION;
    use std::io::Cursor;

    fn header(kind: FrameKind, request_id: Option<u64>, payload: &[u8]) -> FrameHeader {
        FrameHeader {
            protocol_version: PROTOCOL_VERSION,
            kind,
            request_id,
            payload_len: u32::try_from(payload.len()).unwrap(),
        }
    }

    #[test]
    fn stream_round_trip_reads_exactly_one_bounded_frame() {
        let limits = ResourceLimits::default();
        let payload = b"bounded-request";
        let mut bytes = Cursor::new(Vec::new());
        write_frame(
            &mut bytes,
            header(FrameKind::Request, Some(7), payload),
            payload,
            limits,
        )
        .unwrap();
        bytes.set_position(0);
        let decoded = read_frame(&mut bytes, limits).unwrap();
        assert_eq!(decoded.header.kind, FrameKind::Request);
        assert_eq!(decoded.header.request_id, Some(7));
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn oversized_header_is_rejected_before_payload_allocation() {
        let limits = ResourceLimits {
            max_frame_bytes: 64,
            ..ResourceLimits::default()
        };
        let mut bytes = encode_frame(FrameKind::Hello, None, b"", ResourceLimits::default())
            .unwrap()[..FRAME_HEADER_LEN]
            .to_vec();
        bytes[16..20].copy_from_slice(&100_u32.to_be_bytes());
        let error = read_frame(&mut Cursor::new(bytes), limits).unwrap_err();
        assert!(matches!(
            error,
            WireError::Ipc(IpcError::FrameTooLarge { .. })
        ));
    }

    #[test]
    fn truncated_payload_fails_closed() {
        let limits = ResourceLimits::default();
        let payload = b"complete";
        let mut bytes = encode_frame(FrameKind::Reply, Some(8), payload, limits).unwrap();
        bytes.pop();
        let error = read_frame(&mut Cursor::new(bytes), limits).unwrap_err();
        assert!(matches!(
            error,
            WireError::Io(ref io_error) if io_error.kind() == io::ErrorKind::UnexpectedEof
        ));
    }

    #[test]
    fn write_rejects_header_payload_mismatch() {
        let limits = ResourceLimits::default();
        let mut bytes = Cursor::new(Vec::new());
        let error = write_frame(
            &mut bytes,
            header(FrameKind::Event, None, b"abc"),
            b"ab",
            limits,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            WireError::Ipc(IpcError::LengthMismatch { .. })
        ));
        assert!(bytes.get_ref().is_empty());
    }
}
