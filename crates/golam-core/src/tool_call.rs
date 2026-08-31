#![forbid(unsafe_code)]

use std::collections::HashSet;

use crate::CanonicalEncoder;
use crate::harness::{RequestAttemptId, ToolCallCandidateId};
use crate::harness_state::{ToolCallCandidate, ToolCallParseStatus, ToolCallSourceMode};
use crate::taint::{TaintLabel, TaintSet};

const MAX_TOOL_NAME_BYTES: usize = 256;
const MAX_ARGUMENTS: usize = 64;
const MAX_ARGUMENT_NAME_BYTES: usize = 128;
const MAX_ARGUMENT_VALUE_BYTES: usize = 16 * 1024;
const MAX_CANONICAL_ARGUMENT_BYTES: usize = 64 * 1024;
const TEXT_OPEN: &str = "<GOLAM_TOOL_CALL_V1>";
const TEXT_CLOSE: &str = "</GOLAM_TOOL_CALL_V1>";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub schema_digest: [u8; 32],
    pub argument_names: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuredToolCall {
    pub tool_name: String,
    pub arguments: Vec<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedToolCall {
    pub candidate: ToolCallCandidate,
    pub arguments_canonical: Vec<u8>,
    pub taint: TaintSet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolCallNormalizationError {
    pub parse_status: ToolCallParseStatus,
}

pub fn normalize_native(
    request_attempt_id: RequestAttemptId,
    source_event_refs: Vec<String>,
    call: StructuredToolCall,
    registry: &[ToolDefinition],
) -> Result<NormalizedToolCall, ToolCallNormalizationError> {
    normalize_structured(
        request_attempt_id,
        source_event_refs,
        ToolCallSourceMode::NativeTools,
        call,
        registry,
    )
}

pub fn normalize_grammar(
    request_attempt_id: RequestAttemptId,
    source_event_refs: Vec<String>,
    call: StructuredToolCall,
    registry: &[ToolDefinition],
) -> Result<NormalizedToolCall, ToolCallNormalizationError> {
    normalize_structured(
        request_attempt_id,
        source_event_refs,
        ToolCallSourceMode::GrammarConstrained,
        call,
        registry,
    )
}

pub fn normalize_text(
    request_attempt_id: RequestAttemptId,
    source_event_refs: Vec<String>,
    framed: &str,
    registry: &[ToolDefinition],
) -> Result<NormalizedToolCall, ToolCallNormalizationError> {
    let call = parse_text_protocol(framed)?;
    normalize_structured(
        request_attempt_id,
        source_event_refs,
        ToolCallSourceMode::TextProtocolFallback,
        call,
        registry,
    )
}

pub fn normalize_native_batch(
    request_attempt_id: RequestAttemptId,
    calls: Vec<(Vec<String>, StructuredToolCall)>,
    registry: &[ToolDefinition],
) -> Result<Vec<NormalizedToolCall>, ToolCallNormalizationError> {
    normalize_batch(
        request_attempt_id,
        calls,
        registry,
        ToolCallSourceMode::NativeTools,
    )
}

fn normalize_batch(
    request_attempt_id: RequestAttemptId,
    calls: Vec<(Vec<String>, StructuredToolCall)>,
    registry: &[ToolDefinition],
    source_mode: ToolCallSourceMode,
) -> Result<Vec<NormalizedToolCall>, ToolCallNormalizationError> {
    if calls.len() > MAX_ARGUMENTS {
        return Err(rejected(ToolCallParseStatus::RejectedOversized));
    }
    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(calls.len());
    for (source_refs, call) in calls {
        let candidate =
            normalize_structured(request_attempt_id, source_refs, source_mode, call, registry)?;
        if !seen.insert(candidate.candidate.candidate_digest) {
            return Err(rejected(ToolCallParseStatus::RejectedDuplicate));
        }
        normalized.push(candidate);
    }
    Ok(normalized)
}

fn normalize_structured(
    request_attempt_id: RequestAttemptId,
    source_event_refs: Vec<String>,
    source_mode: ToolCallSourceMode,
    call: StructuredToolCall,
    registry: &[ToolDefinition],
) -> Result<NormalizedToolCall, ToolCallNormalizationError> {
    validate_registry(registry)?;
    validate_token(&call.tool_name, MAX_TOOL_NAME_BYTES)
        .map_err(|_| rejected(ToolCallParseStatus::RejectedMalformed))?;

    let definition = registry
        .iter()
        .find(|definition| definition.name == call.tool_name)
        .ok_or_else(|| rejected(ToolCallParseStatus::RejectedUnknownTool))?;

    if call.arguments.len() > MAX_ARGUMENTS {
        return Err(rejected(ToolCallParseStatus::RejectedOversized));
    }

    let mut arguments = call.arguments;
    for (name, value) in &arguments {
        validate_argument_name(name)?;
        validate_argument_value(value)?;
    }
    arguments.sort_by(|left, right| left.0.cmp(&right.0));
    if arguments.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(rejected(ToolCallParseStatus::RejectedDuplicate));
    }

    let supplied_names = arguments
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    let expected_names = definition
        .argument_names
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if supplied_names != expected_names {
        return Err(rejected(ToolCallParseStatus::RejectedSchema));
    }

    let arguments_canonical = canonical_arguments(&arguments)?;
    let arguments_digest = sha256(&arguments_canonical);
    let candidate_digest =
        semantic_candidate_digest(&definition.name, definition.schema_digest, arguments_digest);
    let candidate_id = candidate_id_from_digest(candidate_digest);

    let candidate = ToolCallCandidate {
        candidate_id,
        request_attempt_id,
        source_mode,
        source_event_refs,
        requested_tool_name: Some(definition.name.clone()),
        schema_digest: Some(definition.schema_digest),
        arguments_digest: Some(arguments_digest),
        parse_status: ToolCallParseStatus::ValidatedCandidate,
        candidate_digest,
    };
    candidate
        .validate()
        .map_err(|_| rejected(ToolCallParseStatus::RejectedMalformed))?;

    Ok(NormalizedToolCall {
        candidate,
        arguments_canonical,
        taint: TaintSet::from_labels([TaintLabel::ModelGenerated]),
    })
}

fn parse_text_protocol(framed: &str) -> Result<StructuredToolCall, ToolCallNormalizationError> {
    if framed.len() > MAX_CANONICAL_ARGUMENT_BYTES {
        return Err(rejected(ToolCallParseStatus::RejectedOversized));
    }
    if framed.matches(TEXT_OPEN).count() != 1 || framed.matches(TEXT_CLOSE).count() != 1 {
        return Err(rejected(ToolCallParseStatus::RejectedAmbiguous));
    }
    let prefix = format!("{TEXT_OPEN}\n");
    let suffix = format!("\n{TEXT_CLOSE}");
    let body = framed
        .strip_prefix(&prefix)
        .and_then(|value| value.strip_suffix(&suffix))
        .ok_or_else(|| rejected(ToolCallParseStatus::RejectedMalformed))?;
    if body.contains(TEXT_OPEN) || body.contains(TEXT_CLOSE) || body.contains('\r') {
        return Err(rejected(ToolCallParseStatus::RejectedAmbiguous));
    }

    let mut lines = body.lines();
    let tool_line = lines
        .next()
        .ok_or_else(|| rejected(ToolCallParseStatus::RejectedMalformed))?;
    let tool_name = tool_line
        .strip_prefix("tool=")
        .ok_or_else(|| rejected(ToolCallParseStatus::RejectedMalformed))?;
    validate_token(tool_name, MAX_TOOL_NAME_BYTES)
        .map_err(|_| rejected(ToolCallParseStatus::RejectedMalformed))?;

    let mut arguments = Vec::new();
    for line in lines {
        let raw = line
            .strip_prefix("arg:")
            .ok_or_else(|| rejected(ToolCallParseStatus::RejectedMalformed))?;
        let (name, value) = raw
            .split_once('=')
            .ok_or_else(|| rejected(ToolCallParseStatus::RejectedMalformed))?;
        if value.contains('=') {
            return Err(rejected(ToolCallParseStatus::RejectedAmbiguous));
        }
        arguments.push((name.to_owned(), value.to_owned()));
        if arguments.len() > MAX_ARGUMENTS {
            return Err(rejected(ToolCallParseStatus::RejectedOversized));
        }
    }

    Ok(StructuredToolCall {
        tool_name: tool_name.to_owned(),
        arguments,
    })
}

fn validate_registry(registry: &[ToolDefinition]) -> Result<(), ToolCallNormalizationError> {
    if registry.is_empty() || registry.len() > MAX_ARGUMENTS {
        return Err(rejected(ToolCallParseStatus::RejectedSchema));
    }
    let mut previous_name: Option<&str> = None;
    for definition in registry {
        validate_token(&definition.name, MAX_TOOL_NAME_BYTES)
            .map_err(|_| rejected(ToolCallParseStatus::RejectedSchema))?;
        if previous_name.is_some_and(|previous| previous >= definition.name.as_str()) {
            return Err(rejected(ToolCallParseStatus::RejectedSchema));
        }
        previous_name = Some(&definition.name);
        if definition.argument_names.len() > MAX_ARGUMENTS {
            return Err(rejected(ToolCallParseStatus::RejectedSchema));
        }
        let mut previous_argument: Option<&str> = None;
        for argument in &definition.argument_names {
            validate_argument_name(argument)?;
            if previous_argument.is_some_and(|previous| previous >= argument.as_str()) {
                return Err(rejected(ToolCallParseStatus::RejectedSchema));
            }
            previous_argument = Some(argument);
        }
    }
    Ok(())
}

fn validate_argument_name(value: &str) -> Result<(), ToolCallNormalizationError> {
    validate_token(value, MAX_ARGUMENT_NAME_BYTES)
        .map_err(|_| rejected(ToolCallParseStatus::RejectedMalformed))
}

fn validate_argument_value(value: &str) -> Result<(), ToolCallNormalizationError> {
    if value.len() > MAX_ARGUMENT_VALUE_BYTES {
        return Err(rejected(ToolCallParseStatus::RejectedOversized));
    }
    if value.chars().any(char::is_control) {
        return Err(rejected(ToolCallParseStatus::RejectedMalformed));
    }
    Ok(())
}

fn validate_token(value: &str, max_bytes: usize) -> Result<(), ()> {
    if value.is_empty() || value.len() > max_bytes {
        return Err(());
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(());
    }
    Ok(())
}

fn canonical_arguments(
    arguments: &[(String, String)],
) -> Result<Vec<u8>, ToolCallNormalizationError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_u64(arguments.len() as u64);
    for (name, value) in arguments {
        encoder
            .push_bytes(name.as_bytes())
            .map_err(|_| rejected(ToolCallParseStatus::RejectedOversized))?;
        encoder
            .push_bytes(value.as_bytes())
            .map_err(|_| rejected(ToolCallParseStatus::RejectedOversized))?;
    }
    let bytes = encoder.finish();
    if bytes.len() > MAX_CANONICAL_ARGUMENT_BYTES {
        return Err(rejected(ToolCallParseStatus::RejectedOversized));
    }
    Ok(bytes)
}

fn semantic_candidate_digest(
    tool_name: &str,
    schema_digest: [u8; 32],
    arguments_digest: [u8; 32],
) -> [u8; 32] {
    let mut encoder = CanonicalEncoder::new();
    encoder
        .push_bytes(b"golam:tool-call-candidate:v1")
        .expect("fixed domain fits canonical encoder");
    encoder
        .push_bytes(tool_name.as_bytes())
        .expect("validated tool name fits canonical encoder");
    encoder
        .push_bytes(&schema_digest)
        .expect("fixed digest fits canonical encoder");
    encoder
        .push_bytes(&arguments_digest)
        .expect("fixed digest fits canonical encoder");
    sha256(&encoder.finish())
}

fn candidate_id_from_digest(digest: [u8; 32]) -> ToolCallCandidateId {
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    ToolCallCandidateId::from_u128(u128::from_be_bytes(bytes))
}

const fn rejected(parse_status: ToolCallParseStatus) -> ToolCallNormalizationError {
    ToolCallNormalizationError { parse_status }
}

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256(input: &[u8]) -> [u8; 32] {
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(input.len().saturating_add(72));
    padded.extend_from_slice(input);
    padded.push(0x80);
    while (padded.len() & 63) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = [
        0x6a09e667_u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    for chunk in padded.chunks_exact(64) {
        let mut words = [0_u32; 64];
        for (index, word) in words[..16].iter_mut().enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(SHA256_K[index])
                .wrapping_add(words[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut output = [0_u8; 32];
    for (index, word) in state.iter().enumerate() {
        output[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "lookup".into(),
            schema_digest: [7; 32],
            argument_names: vec!["query".into(), "scope".into()],
        }]
    }

    fn structured() -> StructuredToolCall {
        StructuredToolCall {
            tool_name: "lookup".into(),
            arguments: vec![
                ("scope".into(), "local".into()),
                ("query".into(), "alpha".into()),
            ],
        }
    }

    #[test]
    fn native_grammar_and_text_converge_on_semantic_digest() {
        let attempt = RequestAttemptId::from_u128(9);
        let native =
            normalize_native(attempt, vec!["event:1".into()], structured(), &registry()).unwrap();
        let grammar =
            normalize_grammar(attempt, vec!["event:2".into()], structured(), &registry()).unwrap();
        let text = normalize_text(
            attempt,
            vec!["event:3".into()],
            "<GOLAM_TOOL_CALL_V1>\ntool=lookup\narg:query=alpha\narg:scope=local\n</GOLAM_TOOL_CALL_V1>",
            &registry(),
        )
        .unwrap();

        assert_eq!(
            native.candidate.candidate_digest,
            grammar.candidate.candidate_digest
        );
        assert_eq!(
            native.candidate.candidate_digest,
            text.candidate.candidate_digest
        );
        assert_eq!(native.arguments_canonical, grammar.arguments_canonical);
        assert_eq!(native.arguments_canonical, text.arguments_canonical);
        assert_eq!(
            native.candidate.candidate_id,
            grammar.candidate.candidate_id
        );
        assert!(native.taint.contains(TaintLabel::ModelGenerated));
    }

    #[test]
    fn malformed_oversized_unknown_schema_ambiguous_and_duplicate_reject() {
        let attempt = RequestAttemptId::from_u128(9);
        let malformed = normalize_text(attempt, vec![], "tool=lookup", &registry()).unwrap_err();
        assert_eq!(
            malformed.parse_status,
            ToolCallParseStatus::RejectedAmbiguous
        );

        let oversized = normalize_native(
            attempt,
            vec![],
            StructuredToolCall {
                tool_name: "lookup".into(),
                arguments: vec![("query".into(), "x".repeat(MAX_ARGUMENT_VALUE_BYTES + 1))],
            },
            &registry(),
        )
        .unwrap_err();
        assert_eq!(
            oversized.parse_status,
            ToolCallParseStatus::RejectedOversized
        );

        let unknown = normalize_native(
            attempt,
            vec![],
            StructuredToolCall {
                tool_name: "missing".into(),
                arguments: vec![],
            },
            &registry(),
        )
        .unwrap_err();
        assert_eq!(
            unknown.parse_status,
            ToolCallParseStatus::RejectedUnknownTool
        );

        let schema = normalize_native(
            attempt,
            vec![],
            StructuredToolCall {
                tool_name: "lookup".into(),
                arguments: vec![("query".into(), "alpha".into())],
            },
            &registry(),
        )
        .unwrap_err();
        assert_eq!(schema.parse_status, ToolCallParseStatus::RejectedSchema);

        let ambiguous = normalize_text(
            attempt,
            vec![],
            "<GOLAM_TOOL_CALL_V1>\ntool=lookup\narg:query=a=b\narg:scope=local\n</GOLAM_TOOL_CALL_V1>",
            &registry(),
        )
        .unwrap_err();
        assert_eq!(
            ambiguous.parse_status,
            ToolCallParseStatus::RejectedAmbiguous
        );

        let duplicate = normalize_native_batch(
            attempt,
            vec![
                (vec!["event:1".into()], structured()),
                (vec!["event:2".into()], structured()),
            ],
            &registry(),
        )
        .unwrap_err();
        assert_eq!(
            duplicate.parse_status,
            ToolCallParseStatus::RejectedDuplicate
        );
    }

    #[test]
    fn candidate_normalization_has_no_privileged_authority_surface() {
        let source = include_str!("tool_call.rs");
        for forbidden in [
            "golam_kernel",
            "golam_ledger",
            "CapabilityLease",
            "ApprovalRecord",
            "EffectId",
            "EffectAttemptId",
        ] {
            assert!(
                !source.contains(forbidden),
                "forbidden authority symbol: {forbidden}"
            );
        }
        let candidate = normalize_native(
            RequestAttemptId::from_u128(9),
            vec!["event:1".into()],
            structured(),
            &registry(),
        )
        .unwrap();
        assert!(candidate.taint.contains(TaintLabel::ModelGenerated));
        assert_eq!(
            candidate.candidate.parse_status,
            ToolCallParseStatus::ValidatedCandidate
        );
    }

    #[test]
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256(b"abc"),
            [
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
                0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
                0xf2, 0x00, 0x15, 0xad,
            ]
        );
    }
}
