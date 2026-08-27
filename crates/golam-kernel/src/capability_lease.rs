#![forbid(unsafe_code)]

/// Opaque identifier for a kernel-owned capability lease.
///
/// The identifier is inspectable and persistable as a reference, but its
/// tuple field is private so an identifier cannot be constructed externally
/// and confused with a kernel-issued authority handle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CapabilityLeaseId([u8; 16]);

impl CapabilityLeaseId {
    pub const fn to_bytes(self) -> [u8; 16] {
        self.0
    }
}

/// Sealed proof of capability-lease authority owned by the privileged kernel.
///
/// The handle deliberately does not implement `Clone` or `Copy`. External
/// callers may inspect durable identity/evidence fields after a protected
/// kernel API returns a lease, but cannot construct or duplicate the handle.
/// The production mint path is intentionally introduced only with T003-023,
/// where issuance is a protected authorized mutation rather than a free
/// constructor.
///
/// ```compile_fail
/// use golam_kernel::CapabilityLeaseId;
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

    pub const fn authority_digest(&self) -> [u8; 32] {
        self.authority_digest
    }
}

#[derive(Debug, Eq, PartialEq)]
struct LeaseSeal;

#[cfg(test)]
mod tests {
    use super::*;

    fn kernel_fixture_lease() -> CapabilityLease {
        CapabilityLease {
            lease_id: CapabilityLeaseId([1_u8; 16]),
            principal_id: "owner:local".to_owned(),
            parent_lease_id: Some(CapabilityLeaseId([2_u8; 16])),
            generation: 3,
            issued_global_seq: 4,
            authority_digest: [5_u8; 32],
            _sealed: LeaseSeal,
        }
    }

    #[test]
    fn sealed_lease_exposes_identity_without_public_construction() {
        let lease = kernel_fixture_lease();
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
}
