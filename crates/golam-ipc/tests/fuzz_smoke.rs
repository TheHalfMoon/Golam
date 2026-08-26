#![forbid(unsafe_code)]

use golam_core::ResourceLimits;
use golam_ipc::{FrameKind, decode_exact, decode_header, encode_frame};

fn corpus_bytes(len: usize, seed: u8) -> Vec<u8> {
    let mut state = u32::from(seed).wrapping_add(1);
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        out.push(state.to_le_bytes()[0]);
    }
    out
}

#[test]
fn ipc_decoder_bounded_fuzz_smoke_never_panics() {
    let limits = ResourceLimits::default();
    for len in 0..=128 {
        for seed in [0_u8, 1, 7, 31, 127, 255] {
            let bytes = corpus_bytes(len, seed);
            let _ = decode_header(&bytes, limits);
            let _ = decode_exact(&bytes, limits);
        }
    }
}

#[test]
fn valid_frames_survive_single_byte_mutation_corpus_without_panics() {
    let limits = ResourceLimits::default();
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

    for (ordinal, kind) in kinds.into_iter().enumerate() {
        let request_id = kind.requires_request_id().then_some(ordinal as u64 + 1);
        let payload = corpus_bytes(32 + ordinal, ordinal as u8 + 3);
        let encoded = encode_frame(kind, request_id, &payload, limits).unwrap();
        let decoded = decode_exact(&encoded, limits).unwrap();
        assert_eq!(decoded.header.kind, kind);
        assert_eq!(decoded.payload, payload);

        for index in 0..encoded.len() {
            let mut mutated = encoded.clone();
            mutated[index] ^= 0x5a;
            let _ = decode_header(&mutated, limits);
            let _ = decode_exact(&mutated, limits);
        }
    }
}
