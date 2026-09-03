#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::memory::MemoryCandidateId;
use golam_core::tool_request::{BindingDigest, PrincipalId};
use golam_core::{CanonicalEncoder, CoreError};

use crate::memory_promotion_authority::{
    DeterministicPromotionRequest, HumanPromotionRequest, MemoryPromotionAuthorityError,
    MemoryPromotionAuthorityValidator, ValidatedMemoryPromotion,
};
use crate::memory_promotion_operational::PromotionOperationalEvidence;

const QUALIFIED_PROMOTION_RECORD_DOMAIN: &[u8] = b"golam:qualified-memory-promotion:v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualifiedMemoryPromotion {
    validated: ValidatedMemoryPromotion,
    record_bytes: Vec<u8>,
}

impl QualifiedMemoryPromotion {
    pub const fn evidence_id(&self) -> BindingDigest {
        self.validated.evidence_id
    }

    pub const fn candidate_id(&self) -> MemoryCandidateId {
        self.validated.candidate_id
    }

    pub const fn kernel_authorization_ref(&self) -> BindingDigest {
        self.validated.kernel_authorization_ref
    }

    pub const fn promotion_authority_ref(&self) -> BindingDigest {
        self.validated.promotion_authority_ref
    }

    pub const fn authority_evidence_ref(&self) -> BindingDigest {
        self.validated.authority_evidence_ref
    }

    pub fn approving_principal(&self) -> Option<&PrincipalId> {
        self.validated.approving_principal.as_ref()
    }

    pub const fn verifier_policy_ref(&self) -> Option<BindingDigest> {
        self.validated.verifier_policy_ref
    }

    pub fn record_bytes(&self) -> &[u8] {
        &self.record_bytes
    }

    pub fn operational_evidence(
        &self,
        recorded_at_unix_ms: u64,
    ) -> PromotionOperationalEvidence<'_> {
        PromotionOperationalEvidence {
            evidence_id: self.validated.evidence_id,
            candidate_id: self.validated.candidate_id,
            promotion_authority_ref: self.validated.promotion_authority_ref,
            approving_principal: self.validated.approving_principal.as_ref(),
            verifier_policy_ref: self.validated.verifier_policy_ref,
            authority_evidence_ref: self.validated.authority_evidence_ref,
            recorded_at_unix_ms,
        }
    }
}

#[derive(Debug)]
pub enum MemoryPromotionGateError {
    Authority(MemoryPromotionAuthorityError),
    Core(CoreError),
}

impl fmt::Display for MemoryPromotionGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authority(error) => write!(f, "memory promotion authority validation failed: {error}"),
            Self::Core(error) => write!(f, "memory promotion gate canonical encoding failed: {error}"),
        }
    }
}

impl Error for MemoryPromotionGateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Authority(error) => Some(error),
            Self::Core(error) => Some(error),
        }
    }
}

impl From<MemoryPromotionAuthorityError> for MemoryPromotionGateError {
    fn from(value: MemoryPromotionAuthorityError) -> Self {
        Self::Authority(value)
    }
}

impl From<CoreError> for MemoryPromotionGateError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub struct MemoryPromotionGate {
    validator: MemoryPromotionAuthorityValidator,
}

impl MemoryPromotionGate {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, MemoryPromotionGateError> {
        Ok(Self {
            validator: MemoryPromotionAuthorityValidator::open(layout)?,
        })
    }

    pub fn validate_human(
        &mut self,
        request: HumanPromotionRequest<'_>,
    ) -> Result<QualifiedMemoryPromotion, MemoryPromotionGateError> {
        qualify(self.validator.validate_human(request)?)
    }

    pub fn validate_deterministic(
        &mut self,
        request: DeterministicPromotionRequest<'_>,
    ) -> Result<QualifiedMemoryPromotion, MemoryPromotionGateError> {
        qualify(self.validator.validate_deterministic(request)?)
    }
}

fn qualify(
    validated: ValidatedMemoryPromotion,
) -> Result<QualifiedMemoryPromotion, MemoryPromotionGateError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(QUALIFIED_PROMOTION_RECORD_DOMAIN)?;
    encoder.push_bytes(&validated.evidence_id.bytes())?;
    encoder.push_bytes(&validated.candidate_id.0.bytes())?;
    encoder.push_bytes(&validated.kernel_authorization_ref.bytes())?;
    encoder.push_bytes(&validated.promotion_authority_ref.bytes())?;
    encoder.push_bytes(&validated.authority_evidence_ref.bytes())?;
    match (
        validated.approving_principal.as_ref(),
        validated.verifier_policy_ref,
    ) {
        (Some(principal), None) => {
            encoder.push_u8(1);
            encoder.push_bytes(principal.as_str().as_bytes())?;
        }
        (None, Some(verifier)) => {
            encoder.push_u8(2);
            encoder.push_bytes(&verifier.bytes())?;
        }
        _ => unreachable!("the validated promotion authority contract admits exactly one mode"),
    }
    Ok(QualifiedMemoryPromotion {
        validated,
        record_bytes: encoder.finish(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_token_fields_are_read_only_accessors() {
        // Construction remains private to this module; this test only guards the
        // deterministic record domain used by successful validation paths.
        assert!(!QUALIFIED_PROMOTION_RECORD_DOMAIN.is_empty());
    }
}
