#![forbid(unsafe_code)]

const MAX_FREE_TEXT_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecognizedSecretKind {
    GitHubToken,
    OpenAiKey,
    AwsAccessKeyId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SecretDetectionError {
    InputTooLarge,
}

/// Bounded defense-in-depth recognition for common textual credential shapes.
///
/// This utility deliberately returns only a kind, never the matched value or
/// its offset. It is not an authority source and is not the guarantee for the
/// explicit user-designated secret-entry path.
pub(crate) fn recognized_secret_kind(
    input: &[u8],
) -> Result<Option<RecognizedSecretKind>, SecretDetectionError> {
    if input.len() > MAX_FREE_TEXT_BYTES {
        return Err(SecretDetectionError::InputTooLarge);
    }

    if contains_prefixed_token(input, b"github_pat_", 20)
        || contains_prefixed_token(input, b"ghp_", 20)
    {
        return Ok(Some(RecognizedSecretKind::GitHubToken));
    }
    if contains_prefixed_token(input, b"sk-", 20) {
        return Ok(Some(RecognizedSecretKind::OpenAiKey));
    }
    if contains_aws_access_key_id(input) {
        return Ok(Some(RecognizedSecretKind::AwsAccessKeyId));
    }
    Ok(None)
}

fn contains_prefixed_token(input: &[u8], prefix: &[u8], minimum_tail: usize) -> bool {
    if input.len() < prefix.len() + minimum_tail {
        return false;
    }
    input
        .windows(prefix.len())
        .enumerate()
        .any(|(start, window)| {
            if window != prefix {
                return false;
            }
            input[start + prefix.len()..]
                .iter()
                .take_while(|byte| is_token_byte(**byte))
                .count()
                >= minimum_tail
        })
}

fn contains_aws_access_key_id(input: &[u8]) -> bool {
    const PREFIX: &[u8] = b"AKIA";
    const TAIL_BYTES: usize = 16;
    if input.len() < PREFIX.len() + TAIL_BYTES {
        return false;
    }
    input
        .windows(PREFIX.len())
        .enumerate()
        .any(|(start, window)| {
            window == PREFIX
                && input
                    .get(start + PREFIX.len()..start + PREFIX.len() + TAIL_BYTES)
                    .is_some_and(|tail| tail.iter().all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit()))
        })
}

const fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognized_shapes_return_kind_without_exposing_match_material() {
        assert_eq!(
            recognized_secret_kind(b"token=ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890").unwrap(),
            Some(RecognizedSecretKind::GitHubToken)
        );
        assert_eq!(
            recognized_secret_kind(b"Authorization: Bearer sk-abcdefghijklmnopqrstuvwxyz0123456789")
                .unwrap(),
            Some(RecognizedSecretKind::OpenAiKey)
        );
        assert_eq!(
            recognized_secret_kind(b"aws=AKIAABCDEFGHIJKLMNOP").unwrap(),
            Some(RecognizedSecretKind::AwsAccessKeyId)
        );
    }

    #[test]
    fn unknown_format_is_not_misrepresented_as_detectable() {
        assert_eq!(
            recognized_secret_kind(b"orchid::seven-moons::unknown-secret-shape::T003-056")
                .unwrap(),
            None
        );
    }

    #[test]
    fn free_text_scan_is_bounded() {
        let oversized = vec![b'x'; MAX_FREE_TEXT_BYTES + 1];
        assert_eq!(
            recognized_secret_kind(&oversized),
            Err(SecretDetectionError::InputTooLarge)
        );
    }
}
