#![forbid(unsafe_code)]

//! Secret-safe process launch evidence for Spec 005 T005-073.
//!
//! This module is an evidence boundary, not an authority source. Broker/lease/decision/approval
//! validation remains owned by the existing protected Kernel/Ledger paths. No plaintext secret is
//! retained in a returned structure, argv, environment evidence, or captured output.

use std::error::Error;
use std::fmt;

use golam_core::CanonicalEncoder;
use golam_core::digest::sha256;

const EVIDENCE_DOMAIN: &[u8] = b"golam:process-secret-evidence:v2";
const REDACTION: &[u8] = b"[REDACTED_SECRET]";
const MAX_SECRET_BYTES: usize = 1024 * 1024;
const MAX_ARGV_ITEMS: usize = 4096;
const MAX_ENV_ITEMS: usize = 4096;
const MAX_CAPTURE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessSecretDeliveryMode {
    BrokeredHandle,
    StdinFallback,
}

impl ProcessSecretDeliveryMode {
    const fn code(self) -> u8 {
        match self {
            Self::BrokeredHandle => 1,
            Self::StdinFallback => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessSecretAuthorityRefs {
    pub use_id: [u8; 16],
    pub handle_id: [u8; 16],
    pub secret_id: [u8; 16],
    pub version: u64,
    pub lease_id: [u8; 16],
    pub lease_generation: u64,
    pub decision_id: [u8; 16],
    pub approval_id: Option<[u8; 16]>,
    pub taint_digest: [u8; 32],
    pub launch_plan_hash: [u8; 32],
    pub executable_identity_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FallbackEffectBinding {
    pub effect_id: [u8; 16],
    pub approval_id: [u8; 16],
    pub at_most_once: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FallbackInjectorAttestation {
    pub cleared_environment: bool,
    pub stdin_only_delivery: bool,
    pub stdin_closed_after_delivery: bool,
    pub no_secret_argv: bool,
    pub no_secret_environment: bool,
    pub no_ambient_descendant_inheritance: bool,
    pub stdout_stderr_captured: bool,
}

pub struct FallbackProcessSecretInput<'a> {
    pub authority: ProcessSecretAuthorityRefs,
    pub fallback_effect: FallbackEffectBinding,
    pub injector: FallbackInjectorAttestation,
    pub secret_value: &'a [u8],
    pub argv: &'a [Vec<u8>],
    pub explicit_environment_values: &'a [Vec<u8>],
    pub captured_stdout: &'a [u8],
    pub captured_stderr: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessSecretEvidence {
    pub mode: ProcessSecretDeliveryMode,
    pub authority: ProcessSecretAuthorityRefs,
    pub fallback_effect: Option<FallbackEffectBinding>,
    pub evidence_hash: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedactedProcessCapture {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug)]
pub enum ProcessSecretEvidenceError {
    Canonical(golam_core::CoreError),
    InvalidAuthorityBinding(&'static str),
    InvalidSecretLength,
    TooManyArguments,
    TooManyEnvironmentValues,
    CaptureTooLarge,
    SecretInArgv,
    SecretInEnvironment,
    InvalidFallbackBinding(&'static str),
}

impl fmt::Display for ProcessSecretEvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Canonical(error) => write!(f, "process secret evidence encoding failed: {error}"),
            Self::InvalidAuthorityBinding(reason) => {
                write!(f, "process secret authority reference is invalid: {reason}")
            }
            Self::InvalidSecretLength => f.write_str("process secret plaintext length is invalid"),
            Self::TooManyArguments => f.write_str("process argv exceeds the bounded item count"),
            Self::TooManyEnvironmentValues => {
                f.write_str("process environment exceeds the bounded item count")
            }
            Self::CaptureTooLarge => {
                f.write_str("captured process output exceeds the evidence bound")
            }
            Self::SecretInArgv => f.write_str("plaintext secret is forbidden in process argv"),
            Self::SecretInEnvironment => {
                f.write_str("plaintext secret is forbidden in process environment")
            }
            Self::InvalidFallbackBinding(reason) => {
                write!(f, "process secret fallback binding is invalid: {reason}")
            }
        }
    }
}

impl Error for ProcessSecretEvidenceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Canonical(error) => Some(error),
            _ => None,
        }
    }
}

impl From<golam_core::CoreError> for ProcessSecretEvidenceError {
    fn from(value: golam_core::CoreError) -> Self {
        Self::Canonical(value)
    }
}

/// Bind an already-authorized brokered secret use into exact process-launch evidence.
///
/// The opaque references are evidence only. Construction cannot authorize a secret use.
pub fn bind_brokered_process_secret(
    authority: ProcessSecretAuthorityRefs,
) -> Result<ProcessSecretEvidence, ProcessSecretEvidenceError> {
    validate_authority(authority)?;
    let evidence_hash = evidence_hash(ProcessSecretDeliveryMode::BrokeredHandle, authority, None)?;
    Ok(ProcessSecretEvidence {
        mode: ProcessSecretDeliveryMode::BrokeredHandle,
        authority,
        fallback_effect: None,
        evidence_hash,
    })
}

/// Validate and bind the already-authorized stdin fallback path.
///
/// `secret_value` is borrowed transiently only for exact leak detection/redaction. It is never
/// copied into returned evidence. The caller remains responsible for zeroizing its own plaintext
/// buffer after the existing crate-internal vault callback completes.
pub fn bind_stdin_fallback_process_secret(
    input: FallbackProcessSecretInput<'_>,
) -> Result<(ProcessSecretEvidence, RedactedProcessCapture), ProcessSecretEvidenceError> {
    validate_authority(input.authority)?;
    validate_fallback(input.authority, input.fallback_effect, input.injector)?;
    validate_secret_value(input.secret_value)?;
    validate_no_argv_or_environment_leak(
        input.secret_value,
        input.argv,
        input.explicit_environment_values,
    )?;
    if input.captured_stdout.len() > MAX_CAPTURE_BYTES
        || input.captured_stderr.len() > MAX_CAPTURE_BYTES
    {
        return Err(ProcessSecretEvidenceError::CaptureTooLarge);
    }

    let capture = RedactedProcessCapture {
        stdout: redact_exact_value(input.captured_stdout, input.secret_value),
        stderr: redact_exact_value(input.captured_stderr, input.secret_value),
    };
    let evidence_hash = evidence_hash(
        ProcessSecretDeliveryMode::StdinFallback,
        input.authority,
        Some(input.fallback_effect),
    )?;
    Ok((
        ProcessSecretEvidence {
            mode: ProcessSecretDeliveryMode::StdinFallback,
            authority: input.authority,
            fallback_effect: Some(input.fallback_effect),
            evidence_hash,
        },
        capture,
    ))
}

fn validate_authority(
    authority: ProcessSecretAuthorityRefs,
) -> Result<(), ProcessSecretEvidenceError> {
    if authority.version == 0 || authority.lease_generation == 0 {
        return Err(ProcessSecretEvidenceError::InvalidAuthorityBinding(
            "secret version and lease generation must be nonzero",
        ));
    }
    if authority.use_id == [0; 16]
        || authority.handle_id == [0; 16]
        || authority.secret_id == [0; 16]
        || authority.lease_id == [0; 16]
        || authority.decision_id == [0; 16]
        || authority.taint_digest == [0; 32]
        || authority.launch_plan_hash == [0; 32]
        || authority.executable_identity_hash == [0; 32]
    {
        return Err(ProcessSecretEvidenceError::InvalidAuthorityBinding(
            "opaque authority and launch bindings must be nonzero",
        ));
    }
    Ok(())
}

fn validate_fallback(
    authority: ProcessSecretAuthorityRefs,
    fallback_effect: FallbackEffectBinding,
    injector: FallbackInjectorAttestation,
) -> Result<(), ProcessSecretEvidenceError> {
    if fallback_effect.effect_id == [0; 16]
        || fallback_effect.approval_id == [0; 16]
        || !fallback_effect.at_most_once
    {
        return Err(ProcessSecretEvidenceError::InvalidFallbackBinding(
            "fallback requires exact at-most-once effect and approval references",
        ));
    }
    if authority.approval_id != Some(fallback_effect.approval_id) {
        return Err(ProcessSecretEvidenceError::InvalidFallbackBinding(
            "fallback approval does not match the authorized secret-use approval",
        ));
    }
    if !injector.cleared_environment
        || !injector.stdin_only_delivery
        || !injector.stdin_closed_after_delivery
        || !injector.no_secret_argv
        || !injector.no_secret_environment
        || !injector.no_ambient_descendant_inheritance
        || !injector.stdout_stderr_captured
    {
        return Err(ProcessSecretEvidenceError::InvalidFallbackBinding(
            "fallback injector attestation is incomplete",
        ));
    }
    Ok(())
}

fn validate_secret_value(secret_value: &[u8]) -> Result<(), ProcessSecretEvidenceError> {
    if secret_value.is_empty() || secret_value.len() > MAX_SECRET_BYTES {
        return Err(ProcessSecretEvidenceError::InvalidSecretLength);
    }
    Ok(())
}

fn validate_no_argv_or_environment_leak(
    secret_value: &[u8],
    argv: &[Vec<u8>],
    environment: &[Vec<u8>],
) -> Result<(), ProcessSecretEvidenceError> {
    if argv.len() > MAX_ARGV_ITEMS {
        return Err(ProcessSecretEvidenceError::TooManyArguments);
    }
    if environment.len() > MAX_ENV_ITEMS {
        return Err(ProcessSecretEvidenceError::TooManyEnvironmentValues);
    }
    if argv
        .iter()
        .any(|item| contains_exact_value(item, secret_value))
    {
        return Err(ProcessSecretEvidenceError::SecretInArgv);
    }
    if environment
        .iter()
        .any(|item| contains_exact_value(item, secret_value))
    {
        return Err(ProcessSecretEvidenceError::SecretInEnvironment);
    }
    Ok(())
}

fn contains_exact_value(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn redact_exact_value(input: &[u8], secret_value: &[u8]) -> Vec<u8> {
    if secret_value.is_empty() || input.len() < secret_value.len() {
        return input.to_vec();
    }
    let mut output = Vec::with_capacity(input.len());
    let mut cursor = 0_usize;
    while cursor < input.len() {
        if cursor + secret_value.len() <= input.len()
            && &input[cursor..cursor + secret_value.len()] == secret_value
        {
            output.extend_from_slice(REDACTION);
            cursor += secret_value.len();
        } else {
            output.push(input[cursor]);
            cursor += 1;
        }
    }
    output
}

fn evidence_hash(
    mode: ProcessSecretDeliveryMode,
    authority: ProcessSecretAuthorityRefs,
    fallback_effect: Option<FallbackEffectBinding>,
) -> Result<[u8; 32], ProcessSecretEvidenceError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(EVIDENCE_DOMAIN)?;
    encoder.push_u8(mode.code());
    encoder.push_bytes(&authority.use_id)?;
    encoder.push_bytes(&authority.handle_id)?;
    encoder.push_bytes(&authority.secret_id)?;
    encoder.push_u64(authority.version);
    encoder.push_bytes(&authority.lease_id)?;
    encoder.push_u64(authority.lease_generation);
    encoder.push_bytes(&authority.decision_id)?;
    encode_optional_id(&mut encoder, authority.approval_id)?;
    encoder.push_bytes(&authority.taint_digest)?;
    encoder.push_bytes(&authority.launch_plan_hash)?;
    encoder.push_bytes(&authority.executable_identity_hash)?;
    match fallback_effect {
        Some(binding) => {
            encoder.push_u8(1);
            encoder.push_bytes(&binding.effect_id)?;
            encoder.push_bytes(&binding.approval_id)?;
            encoder.push_u8(u8::from(binding.at_most_once));
        }
        None => encoder.push_u8(0),
    }
    Ok(sha256(&encoder.finish()))
}

fn encode_optional_id(
    encoder: &mut CanonicalEncoder,
    id: Option<[u8; 16]>,
) -> Result<(), golam_core::CoreError> {
    match id {
        Some(id) => {
            encoder.push_u8(1);
            encoder.push_bytes(&id)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authority() -> ProcessSecretAuthorityRefs {
        ProcessSecretAuthorityRefs {
            use_id: [1; 16],
            handle_id: [2; 16],
            secret_id: [3; 16],
            version: 7,
            lease_id: [4; 16],
            lease_generation: 2,
            decision_id: [5; 16],
            approval_id: Some([6; 16]),
            taint_digest: [7; 32],
            launch_plan_hash: [8; 32],
            executable_identity_hash: [9; 32],
        }
    }

    fn fallback() -> FallbackEffectBinding {
        FallbackEffectBinding {
            effect_id: [10; 16],
            approval_id: [6; 16],
            at_most_once: true,
        }
    }

    fn injector() -> FallbackInjectorAttestation {
        FallbackInjectorAttestation {
            cleared_environment: true,
            stdin_only_delivery: true,
            stdin_closed_after_delivery: true,
            no_secret_argv: true,
            no_secret_environment: true,
            no_ambient_descendant_inheritance: true,
            stdout_stderr_captured: true,
        }
    }

    fn fallback_input<'a>(
        secret: &'a [u8],
        argv: &'a [Vec<u8>],
        env: &'a [Vec<u8>],
        stdout: &'a [u8],
        stderr: &'a [u8],
    ) -> FallbackProcessSecretInput<'a> {
        FallbackProcessSecretInput {
            authority: authority(),
            fallback_effect: fallback(),
            injector: injector(),
            secret_value: secret,
            argv,
            explicit_environment_values: env,
            captured_stdout: stdout,
            captured_stderr: stderr,
        }
    }

    #[test]
    fn brokered_evidence_contains_only_opaque_authority_and_launch_references() {
        let evidence = bind_brokered_process_secret(authority()).unwrap();
        assert_eq!(evidence.mode, ProcessSecretDeliveryMode::BrokeredHandle);
        assert_eq!(evidence.fallback_effect, None);
        assert_ne!(evidence.evidence_hash, [0; 32]);
    }

    #[test]
    fn exact_launch_binding_changes_evidence_hash() {
        let first = bind_brokered_process_secret(authority()).unwrap();
        let mut changed = authority();
        changed.launch_plan_hash = [0x55; 32];
        let second = bind_brokered_process_secret(changed).unwrap();
        assert_ne!(first.evidence_hash, second.evidence_hash);
    }

    #[test]
    fn fallback_rejects_plaintext_in_argv_or_environment() {
        let secret = b"spec005-canary-secret";
        let argv = [b"--token=spec005-canary-secret".to_vec()];
        assert!(matches!(
            bind_stdin_fallback_process_secret(fallback_input(secret, &argv, &[], &[], &[])),
            Err(ProcessSecretEvidenceError::SecretInArgv)
        ));
        let env = [b"TOKEN=spec005-canary-secret".to_vec()];
        assert!(matches!(
            bind_stdin_fallback_process_secret(fallback_input(secret, &[], &env, &[], &[])),
            Err(ProcessSecretEvidenceError::SecretInEnvironment)
        ));
    }

    #[test]
    fn fallback_redacts_exact_secret_from_captured_output() {
        let secret = b"spec005-canary-secret";
        let argv = [b"tool".to_vec()];
        let env = [b"MODE=test".to_vec()];
        let (_, capture) = bind_stdin_fallback_process_secret(fallback_input(
            secret,
            &argv,
            &env,
            b"before spec005-canary-secret after",
            b"spec005-canary-secretspec005-canary-secret",
        ))
        .unwrap();
        assert_eq!(capture.stdout, b"before [REDACTED_SECRET] after");
        assert_eq!(capture.stderr, b"[REDACTED_SECRET][REDACTED_SECRET]");
    }

    #[test]
    fn incomplete_fallback_attestation_fails_closed() {
        let mut input = fallback_input(b"secret", &[], &[], &[], &[]);
        input.injector.no_ambient_descendant_inheritance = false;
        assert!(matches!(
            bind_stdin_fallback_process_secret(input),
            Err(ProcessSecretEvidenceError::InvalidFallbackBinding(_))
        ));
    }

    #[test]
    fn mismatched_fallback_approval_fails_closed() {
        let mut input = fallback_input(b"secret", &[], &[], &[], &[]);
        input.fallback_effect.approval_id = [0x44; 16];
        assert!(matches!(
            bind_stdin_fallback_process_secret(input),
            Err(ProcessSecretEvidenceError::InvalidFallbackBinding(_))
        ));
    }

    #[test]
    fn plaintext_changes_do_not_change_persistable_binding_hash() {
        let first =
            bind_stdin_fallback_process_secret(fallback_input(b"secret-one", &[], &[], &[], &[]))
                .unwrap()
                .0;
        let second =
            bind_stdin_fallback_process_secret(fallback_input(b"secret-two", &[], &[], &[], &[]))
                .unwrap()
                .0;
        assert_eq!(first.evidence_hash, second.evidence_hash);
    }
}
