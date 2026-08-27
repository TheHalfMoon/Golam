#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

const MAX_PRINCIPAL_ID_BYTES: usize = 512;

/// Opaque identifier for a kernel-minted capability lease.
///
/// The identifier is safe to inspect and persist as a reference, but its
/// constructor remains private so an identifier cannot be confused with a
/// kernel-issued authority handle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CapabilityLeaseId([u8; 16]);

impl CapabilityLeaseId {
    pub const fn to_bytes(self) -> [u8; 16] {
        self.0
    }

    pub(crate) const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

/// Sealed proof that a capability lease was minted inside the privileged
/// kernel boundary.
///
/// Callers may inspect the lease identity and principal binding, but safe
/// external Rust code cannot construct this value and the handle deliberately
/// does not implement `Clone` or `Copy`.
///
/// ```compile_fail
/// use golam_kernel::{CapabilityLease, CapabilityLeaseId};
/// let _ = CapabilityLeaseId([0_u8; 16]);
/// ```
///
/// ```compile_fail
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<golam_kernel::CapabilityLease>();
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct CapabilityLease {
    lease_id: CapabilityLeaseId,
    principal_id: String,
    parent_lease_id: Option<CapabilityLeaseId>,
    generation: u64,
    issued_global_seq: u64,
    authority_digest: [u8; 32],
    _sealed: LeaseSeal,
}

impl CapabilityLease {
    pub const fn lease_id(&self) -> CapabilityLeaseId {
        self.lease_id
    }

    pub fn principal_id(&self) -> &str {
        &self.principal_id
    }

    pub const fn parent_lease_id(&self) -> Option<CapabilityLeaseId> {
        self.parent_lease_id
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn issued_global_seq(&self) -> u64 {
        self.issued_global_seq
    }

    pub(crate) const fn authority_digest(&self) -> [u8; 32] {
        self.authority_digest
    }
}

#[derive(Debug, Eq, PartialEq)]
struct LeaseSeal;

pub(crate) struct MintCapabilityLease<'a> {
    pub lease_id: [u8; 16],
    pub principal_id: &'a str,
    pub parent_lease_id: Option<[u8; 16]>,
    pub generation: u64,
    pub issued_global_seq: u64,
    pub authority_digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CapabilityLeaseMintError {
    InvalidPrincipal,
    InvalidGeneration,
    InvalidIssuedSequence,
}

impl fmt::Display for CapabilityLeaseMintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrincipal => {
                f.write_str("capability lease principal is empty, non-canonical or too large")
            }
            Self::InvalidGeneration => f.write_str("capability lease generation must be non-zero"),
            Self::InvalidIssuedSequence => {
                f.write_str("capability lease issued sequence must be non-zero")
            }
        }
    }
}

impl Error for CapabilityLeaseMintError {}

pub(crate) fn mint_capability_lease(
    input: MintCapabilityLease<'_>,
) -> Result<CapabilityLease, CapabilityLeaseMintError> {
    if input.principal_id.is_empty()
        || input.principal_id.len() > MAX_PRINCIPAL_ID_BYTES
        || input.principal_id.trim() != input.principal_id
        || input.principal_id.chars().any(char::is_control)
    {
        return Err(CapabilityLeaseMintError::InvalidPrincipal);
    }
    if input.generation == 0 {
        return Err(CapabilityLeaseMintError::InvalidGeneration);
    }
    if input.issued_global_seq == 0 {
        return Err(CapabilityLeaseMintError::InvalidIssuedSequence);
    }

    Ok(CapabilityLease {
        lease_id: CapabilityLeaseId::from_bytes(input.lease_id),
        principal_id: input.principal_id.to_owned(),
        parent_lease_id: input.parent_lease_id.map(CapabilityLeaseId::from_bytes),
        generation: input.generation,
        issued_global_seq: input.issued_global_seq,
        authority_digest: input.authority_digest,
        _sealed: LeaseSeal,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mint() -> MintCapabilityLease<'static> {
        MintCapabilityLease {
            lease_id: [1_u8; 16],
            principal_id: "owner:local",
            parent_lease_id: Some([2_u8; 16]),
            generation: 3,
            issued_global_seq: 4,
            authority_digest: [5_u8; 32],
        }
    }

    #[test]
    fn privileged_mint_seals_exact_lease_identity() {
        let lease = mint_capability_lease(mint()).unwrap();
        assert_eq!(lease.lease_id().to_bytes(), [1_u8; 16]);
        assert_eq!(lease.principal_id(), "owner:local");
        assert_eq!(
            lease.parent_lease_id().map(CapabilityLeaseId::to_bytes),
            Some([2_u8; 16])
        );
        assert_eq!(lease.generation(), 3);
        assert_eq!(lease.issued_global_seq(), 4);
        assert_eq!(lease.authority_digest(), [5_u8; 32]);
    }

    #[test]
    fn privileged_mint_rejects_malformed_identity_metadata() {
        let mut input = mint();
        input.principal_id = " owner:local";
        assert_eq!(
            mint_capability_lease(input).unwrap_err(),
            CapabilityLeaseMintError::InvalidPrincipal
        );

        let mut input = mint();
        input.generation = 0;
        assert_eq!(
            mint_capability_lease(input).unwrap_err(),
            CapabilityLeaseMintError::InvalidGeneration
        );

        let mut input = mint();
        input.issued_global_seq = 0;
        assert_eq!(
            mint_capability_lease(input).unwrap_err(),
            CapabilityLeaseMintError::InvalidIssuedSequence
        );
    }
}
