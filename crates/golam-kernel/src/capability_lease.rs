#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{CanonicalEncoder, CoreError};

const LEASE_SCOPE_DOMAIN: &[u8] = b"golam:capability-lease-scope:v1";
const MAX_SCOPE_ITEMS: usize = 32;
const MAX_ACTION_BYTES: usize = 128;
const MAX_RESOURCE_BYTES: usize = 2048;
const MAX_CONTEXT_CONSTRAINT_BYTES: usize = 256;
const MAX_CANONICAL_SCOPE_BYTES: usize = 131_072;

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

/// Canonical data-only scope requested for, or recorded on, a capability
/// lease. Constructing a scope does not mint authority.
///
/// Scope entries use exact-match semantics in Spec 003. Canonicalization sorts
/// and deduplicates entries, and child derivation rejects any requested entry
/// that is absent from the parent. No wildcard or implicit prefix semantics
/// are admitted by this type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityLeaseScope {
    actions: Vec<String>,
    resources: Vec<String>,
    context_constraints: Vec<String>,
    digest: [u8; 32],
}

impl CapabilityLeaseScope {
    pub fn normalize(
        actions: &[&str],
        resources: &[&str],
        context_constraints: &[&str],
    ) -> Result<Self, CapabilityLeaseScopeError> {
        let actions = normalize_entries(actions, ScopeEntryKind::Action)?;
        let resources = normalize_entries(resources, ScopeEntryKind::Resource)?;
        let context_constraints =
            normalize_entries(context_constraints, ScopeEntryKind::ContextConstraint)?;
        let canonical = encode_scope(&actions, &resources, &context_constraints)?;
        if canonical.len() > MAX_CANONICAL_SCOPE_BYTES {
            return Err(CapabilityLeaseScopeError::ScopeTooLarge);
        }
        Ok(Self {
            actions,
            resources,
            context_constraints,
            digest: *blake3::hash(&canonical).as_bytes(),
        })
    }

    pub fn actions(&self) -> &[String] {
        &self.actions
    }

    pub fn resources(&self) -> &[String] {
        &self.resources
    }

    pub fn context_constraints(&self) -> &[String] {
        &self.context_constraints
    }

    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    /// Returns the requested child scope only when every requested exact entry
    /// is already present in the parent. A widening request fails instead of
    /// being silently intersected into a different authority request.
    pub fn derive_child(
        &self,
        requested: &Self,
    ) -> Result<Self, CapabilityLeaseScopeError> {
        if is_subset(&requested.actions, &self.actions)
            && is_subset(&requested.resources, &self.resources)
            && is_subset(&requested.context_constraints, &self.context_constraints)
        {
            Ok(requested.clone())
        } else {
            Err(CapabilityLeaseScopeError::RequestedWidening)
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum CapabilityLeaseScopeError {
    TooManyActions,
    TooManyResources,
    TooManyContextConstraints,
    InvalidAction,
    InvalidResource,
    InvalidContextConstraint,
    ScopeTooLarge,
    RequestedWidening,
    CanonicalEncoding(CoreError),
}

impl fmt::Display for CapabilityLeaseScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyActions => f.write_str("capability lease action scope exceeds item bound"),
            Self::TooManyResources => {
                f.write_str("capability lease resource scope exceeds item bound")
            }
            Self::TooManyContextConstraints => {
                f.write_str("capability lease context scope exceeds item bound")
            }
            Self::InvalidAction => f.write_str("capability lease action scope is not canonical"),
            Self::InvalidResource => {
                f.write_str("capability lease resource scope is not canonical")
            }
            Self::InvalidContextConstraint => {
                f.write_str("capability lease context constraint is not canonical")
            }
            Self::ScopeTooLarge => {
                f.write_str("capability lease canonical scope exceeds byte bound")
            }
            Self::RequestedWidening => {
                f.write_str("child capability lease scope would widen parent authority")
            }
            Self::CanonicalEncoding(error) => {
                write!(f, "capability lease scope encoding failed: {error}")
            }
        }
    }
}

impl Error for CapabilityLeaseScopeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalEncoding(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CoreError> for CapabilityLeaseScopeError {
    fn from(value: CoreError) -> Self {
        Self::CanonicalEncoding(value)
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
    scope: CapabilityLeaseScope,
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

    pub const fn scope(&self) -> &CapabilityLeaseScope {
        &self.scope
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

#[derive(Clone, Copy)]
enum ScopeEntryKind {
    Action,
    Resource,
    ContextConstraint,
}

fn normalize_entries(
    entries: &[&str],
    kind: ScopeEntryKind,
) -> Result<Vec<String>, CapabilityLeaseScopeError> {
    if entries.len() > MAX_SCOPE_ITEMS {
        return Err(match kind {
            ScopeEntryKind::Action => CapabilityLeaseScopeError::TooManyActions,
            ScopeEntryKind::Resource => CapabilityLeaseScopeError::TooManyResources,
            ScopeEntryKind::ContextConstraint => {
                CapabilityLeaseScopeError::TooManyContextConstraints
            }
        });
    }
    let mut canonical = Vec::with_capacity(entries.len());
    for entry in entries {
        validate_scope_entry(entry, kind)?;
        canonical.push((*entry).to_owned());
    }
    canonical.sort_unstable();
    canonical.dedup();
    Ok(canonical)
}

fn validate_scope_entry(
    value: &str,
    kind: ScopeEntryKind,
) -> Result<(), CapabilityLeaseScopeError> {
    match kind {
        ScopeEntryKind::Action => validate_action(value),
        ScopeEntryKind::Resource => validate_resource(value),
        ScopeEntryKind::ContextConstraint => validate_context_constraint(value),
    }
}

fn validate_action(value: &str) -> Result<(), CapabilityLeaseScopeError> {
    if value.is_empty() || value.len() > MAX_ACTION_BYTES {
        return Err(CapabilityLeaseScopeError::InvalidAction);
    }
    let bytes = value.as_bytes();
    let first_is_canonical = bytes.first().is_some_and(u8::is_ascii_lowercase);
    let last_is_canonical = bytes.last().is_some_and(u8::is_ascii_alphanumeric);
    if !first_is_canonical
        || !last_is_canonical
        || bytes.windows(2).any(|pair| pair == b"..")
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-'))
        })
    {
        return Err(CapabilityLeaseScopeError::InvalidAction);
    }
    Ok(())
}

fn validate_resource(value: &str) -> Result<(), CapabilityLeaseScopeError> {
    if value.is_empty()
        || value.len() > MAX_RESOURCE_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(CapabilityLeaseScopeError::InvalidResource);
    }
    Ok(())
}

fn validate_context_constraint(value: &str) -> Result<(), CapabilityLeaseScopeError> {
    if value.is_empty() || value.len() > MAX_CONTEXT_CONSTRAINT_BYTES {
        return Err(CapabilityLeaseScopeError::InvalidContextConstraint);
    }
    let bytes = value.as_bytes();
    let first_is_canonical = bytes.first().is_some_and(u8::is_ascii_lowercase);
    let last_is_canonical = bytes.last().is_some_and(u8::is_ascii_alphanumeric);
    if !first_is_canonical
        || !last_is_canonical
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-' | b':'))
        })
    {
        return Err(CapabilityLeaseScopeError::InvalidContextConstraint);
    }
    Ok(())
}

fn encode_scope(
    actions: &[String],
    resources: &[String],
    context_constraints: &[String],
) -> Result<Vec<u8>, CapabilityLeaseScopeError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(LEASE_SCOPE_DOMAIN)?;
    encode_entries(&mut encoder, actions)?;
    encode_entries(&mut encoder, resources)?;
    encode_entries(&mut encoder, context_constraints)?;
    Ok(encoder.finish())
}

fn encode_entries(
    encoder: &mut CanonicalEncoder,
    entries: &[String],
) -> Result<(), CapabilityLeaseScopeError> {
    encoder.push_u64(
        u64::try_from(entries.len()).map_err(|_| CapabilityLeaseScopeError::ScopeTooLarge)?,
    );
    for entry in entries {
        encoder.push_bytes(entry.as_bytes())?;
    }
    Ok(())
}

fn is_subset(child: &[String], parent: &[String]) -> bool {
    child
        .iter()
        .all(|entry| parent.binary_search(entry).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parent_scope() -> CapabilityLeaseScope {
        CapabilityLeaseScope::normalize(
            &["session.read", "session.create", "effect.simulate"],
            &["session:1", "session:2", "effect:fixture"],
            &["local-owner", "local-session:1"],
        )
        .unwrap()
    }

    fn kernel_fixture_lease() -> CapabilityLease {
        CapabilityLease {
            lease_id: CapabilityLeaseId([1_u8; 16]),
            principal_id: "owner:local".to_owned(),
            parent_lease_id: Some(CapabilityLeaseId([2_u8; 16])),
            scope: parent_scope(),
            generation: 3,
            issued_global_seq: 4,
            authority_digest: [5_u8; 32],
            _sealed: LeaseSeal,
        }
    }

    #[test]
    fn sealed_lease_exposes_identity_and_canonical_scope_without_public_construction() {
        let lease = kernel_fixture_lease();
        assert_eq!(lease.lease_id().to_bytes(), [1_u8; 16]);
        assert_eq!(lease.principal_id(), "owner:local");
        assert_eq!(
            lease.parent_lease_id().map(CapabilityLeaseId::to_bytes),
            Some([2_u8; 16])
        );
        assert_eq!(
            lease.scope().actions(),
            &["effect.simulate", "session.create", "session.read"]
        );
        assert_eq!(lease.generation(), 3);
        assert_eq!(lease.issued_global_seq(), 4);
        assert_eq!(lease.authority_digest(), [5_u8; 32]);
    }

    #[test]
    fn normalization_is_order_independent_deduplicated_and_field_sensitive() {
        let first = CapabilityLeaseScope::normalize(
            &["session.read", "session.create", "session.read"],
            &["session:2", "session:1"],
            &["local-session:1", "local-owner"],
        )
        .unwrap();
        let reordered = CapabilityLeaseScope::normalize(
            &["session.create", "session.read"],
            &["session:1", "session:2"],
            &["local-owner", "local-session:1"],
        )
        .unwrap();
        assert_eq!(first, reordered);
        assert_eq!(first.digest(), reordered.digest());

        let changed = CapabilityLeaseScope::normalize(
            &["session.read"],
            &["session:1", "session:2"],
            &["local-owner", "local-session:1"],
        )
        .unwrap();
        assert_ne!(first.digest(), changed.digest());
    }

    #[test]
    fn normalization_rejects_noncanonical_or_oversized_scope_entries() {
        assert_eq!(
            CapabilityLeaseScope::normalize(&["Session.read"], &[], &[]).unwrap_err(),
            CapabilityLeaseScopeError::InvalidAction
        );
        assert_eq!(
            CapabilityLeaseScope::normalize(&[], &[" session:1"], &[]).unwrap_err(),
            CapabilityLeaseScopeError::InvalidResource
        );
        assert_eq!(
            CapabilityLeaseScope::normalize(&[], &[], &["Local-owner"]).unwrap_err(),
            CapabilityLeaseScopeError::InvalidContextConstraint
        );
        let actions = vec!["session.read"; MAX_SCOPE_ITEMS + 1];
        assert_eq!(
            CapabilityLeaseScope::normalize(&actions, &[], &[]).unwrap_err(),
            CapabilityLeaseScopeError::TooManyActions
        );
    }

    #[test]
    fn child_derivation_accepts_equal_or_narrow_scope_and_empty_deny_all_scope() {
        let parent = parent_scope();
        assert_eq!(parent.derive_child(&parent).unwrap(), parent);

        let narrow = CapabilityLeaseScope::normalize(
            &["session.read"],
            &["session:1"],
            &["local-session:1"],
        )
        .unwrap();
        assert_eq!(parent.derive_child(&narrow).unwrap(), narrow);

        let empty = CapabilityLeaseScope::normalize(&[], &[], &[]).unwrap();
        assert_eq!(parent.derive_child(&empty).unwrap(), empty);
    }

    #[test]
    fn child_derivation_rejects_widening_in_every_authority_dimension() {
        let parent = parent_scope();
        for requested in [
            CapabilityLeaseScope::normalize(
                &["session.read", "client.revoke"],
                &["session:1"],
                &["local-owner"],
            )
            .unwrap(),
            CapabilityLeaseScope::normalize(
                &["session.read"],
                &["session:1", "session:3"],
                &["local-owner"],
            )
            .unwrap(),
            CapabilityLeaseScope::normalize(
                &["session.read"],
                &["session:1"],
                &["local-owner", "local-client"],
            )
            .unwrap(),
        ] {
            assert_eq!(
                parent.derive_child(&requested).unwrap_err(),
                CapabilityLeaseScopeError::RequestedWidening
            );
        }
    }
}
