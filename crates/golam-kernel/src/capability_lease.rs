#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use golam_ledger::capability_leases::{
    CapabilityLeaseBinding, CapabilityLeaseMutationError, CapabilityLeaseRuntimeError,
    CapabilityLeaseRuntimeState, CapabilityLeaseStore, load_capability_lease_runtime_chain,
    prepare_capability_lease_issue, prepare_capability_lease_revocation,
};

use crate::KernelApi;
use crate::authorization::{AuthorizationPolicy, DecisionId};

const LEASE_SCOPE_DOMAIN: &[u8] = b"golam:capability-lease-scope:v1";
const MAX_SCOPE_ITEMS: usize = 32;
const MAX_ACTION_BYTES: usize = 128;
const MAX_RESOURCE_BYTES: usize = 2048;
const MAX_CONTEXT_CONSTRAINT_BYTES: usize = 256;
const MAX_CANONICAL_SCOPE_BYTES: usize = 131_072;
const MAX_PRINCIPAL_ID_BYTES: usize = 512;

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
            digest: golam_ledger::payload_hash(&canonical),
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
    pub fn derive_child(&self, requested: &Self) -> Result<Self, CapabilityLeaseScopeError> {
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

impl<P: AuthorizationPolicy> KernelApi<P> {
    /// Returns the exact resource and payload hash that a typed elevated
    /// effect and its authorization decision must bind before lease issuance.
    /// This prepares data only; it does not mint authority.
    pub fn capability_lease_issue_effect_binding(
        &self,
        principal_id: &str,
        parent: Option<&CapabilityLease>,
        scope: &CapabilityLeaseScope,
        not_before: Option<&str>,
        expires_at: Option<&str>,
    ) -> Result<(String, [u8; 32]), CapabilityLeaseMutationError> {
        let prepared = prepare_issue(principal_id, parent, scope, not_before, expires_at)?;
        Ok((prepared.resource().to_owned(), prepared.intent_digest()))
    }

    /// Commits a protected capability lease only after the ledger proves that
    /// `authority_decision_id` is the latest exact allow, the supplied effect
    /// is exact authorized at-most-once elevated work, the ONCE approval is
    /// unused and exact, and any parent lease is current and non-widening.
    /// Only a successful atomic commit produces this sealed authority handle.
    pub fn issue_capability_lease(
        &mut self,
        principal_id: &str,
        parent: Option<&CapabilityLease>,
        scope: CapabilityLeaseScope,
        not_before: Option<&str>,
        expires_at: Option<&str>,
        authority_decision_id: DecisionId,
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<CapabilityLease, CapabilityLeaseMutationError> {
        let prepared = prepare_issue(principal_id, parent, &scope, not_before, expires_at)?;
        if prepared.scope_digest() != scope.digest() {
            return Err(CapabilityLeaseMutationError::InvalidStoredRecord(
                "kernel and ledger scope digests differ",
            ));
        }
        let expected_parent = parent.map(lease_binding);
        let mut store = CapabilityLeaseStore::open(&self.authority)?;
        let record = store.issue(prepared, authority_decision_id.0, approval_id, effect_id)?;
        let expected_parent_id = expected_parent.map(CapabilityLeaseBinding::lease_id);
        if record.principal_id != principal_id || record.parent_lease_id != expected_parent_id {
            return Err(CapabilityLeaseMutationError::InvalidStoredRecord(
                "committed lease identity differs from kernel request",
            ));
        }
        Ok(CapabilityLease {
            lease_id: CapabilityLeaseId(record.lease_id),
            principal_id: record.principal_id,
            parent_lease_id: record.parent_lease_id.map(CapabilityLeaseId),
            scope,
            generation: record.generation,
            issued_global_seq: record.issued_global_seq,
            authority_digest: record.authority_digest,
            _sealed: LeaseSeal,
        })
    }

    /// Returns the exact resource and payload hash that a typed elevated
    /// effect and authorization decision must bind before monotonic revocation.
    pub fn capability_lease_revoke_effect_binding(
        &self,
        lease: &CapabilityLease,
        reason_code: &str,
        revoked_at: &str,
    ) -> Result<(String, [u8; 32]), CapabilityLeaseMutationError> {
        let prepared =
            prepare_capability_lease_revocation(lease_binding(lease), reason_code, revoked_at)?;
        Ok((prepared.resource().to_owned(), prepared.intent_digest()))
    }

    /// Commits a monotonic protected revocation under the exact current lease
    /// binding. A stale generation/digest, reused approval, wrong effect or
    /// mismatched authorization decision fails closed before commit.
    pub fn revoke_capability_lease(
        &mut self,
        lease: &CapabilityLease,
        reason_code: &str,
        revoked_at: &str,
        authority_decision_id: DecisionId,
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<(), CapabilityLeaseMutationError> {
        let prepared =
            prepare_capability_lease_revocation(lease_binding(lease), reason_code, revoked_at)?;
        let mut store = CapabilityLeaseStore::open(&self.authority)?;
        store.revoke(prepared, authority_decision_id.0, approval_id, effect_id)?;
        Ok(())
    }
}

fn prepare_issue(
    principal_id: &str,
    parent: Option<&CapabilityLease>,
    scope: &CapabilityLeaseScope,
    not_before: Option<&str>,
    expires_at: Option<&str>,
) -> Result<
    golam_ledger::capability_leases::PreparedCapabilityLeaseIssue,
    CapabilityLeaseMutationError,
> {
    prepare_capability_lease_issue(
        principal_id,
        parent.map(lease_binding),
        scope.actions(),
        scope.resources(),
        scope.context_constraints(),
        not_before,
        expires_at,
    )
}

fn lease_binding(lease: &CapabilityLease) -> CapabilityLeaseBinding {
    CapabilityLeaseBinding::new(
        lease.lease_id().to_bytes(),
        lease.generation(),
        lease.authority_digest(),
    )
}

#[derive(Debug, Eq, PartialEq)]
struct LeaseSeal;

/// Non-authority evidence describing the exact lease state accepted at the
/// protected use-time boundary. This value is suitable for audit/explanation;
/// it does not itself grant or mint capability authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityLeaseUseEvidence {
    lease_id: CapabilityLeaseId,
    generation: u64,
    authority_digest: [u8; 32],
    scope_digest: [u8; 32],
}

impl CapabilityLeaseUseEvidence {
    pub const fn lease_id(self) -> CapabilityLeaseId {
        self.lease_id
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn authority_digest(self) -> [u8; 32] {
        self.authority_digest
    }

    pub const fn scope_digest(self) -> [u8; 32] {
        self.scope_digest
    }
}

#[derive(Debug)]
pub enum CapabilityLeaseUseError {
    Runtime(CapabilityLeaseRuntimeError),
    Scope(CapabilityLeaseScopeError),
    LeaseNotFound,
    HandleStateMismatch,
    StaleGeneration,
    InvalidPrincipal,
    PrincipalMismatch,
    Inactive,
    Revoked,
    InvalidObservedTime,
    InvalidStoredTime,
    NotYetValid,
    Expired,
    ActionOutOfScope,
    ResourceOutOfScope,
    ContextOutOfScope,
}

impl fmt::Display for CapabilityLeaseUseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime(error) => write!(f, "capability lease runtime state error: {error}"),
            Self::Scope(error) => write!(f, "capability lease request scope error: {error}"),
            Self::LeaseNotFound => f.write_str("capability lease does not exist"),
            Self::HandleStateMismatch => f.write_str(
                "capability lease handle does not match current protected authority state",
            ),
            Self::StaleGeneration => f.write_str("capability lease handle generation is stale"),
            Self::InvalidPrincipal => {
                f.write_str("capability lease use principal is not canonical")
            }
            Self::PrincipalMismatch => {
                f.write_str("capability lease is not bound to the requesting principal")
            }
            Self::Inactive => f.write_str("capability lease or parent lease is not active"),
            Self::Revoked => f.write_str("capability lease or parent lease is revoked"),
            Self::InvalidObservedTime => {
                f.write_str("capability lease observed use time is not canonical UTC-second time")
            }
            Self::InvalidStoredTime => {
                f.write_str("capability lease stored validity time is malformed or inconsistent")
            }
            Self::NotYetValid => f.write_str("capability lease is not yet valid"),
            Self::Expired => f.write_str("capability lease is expired"),
            Self::ActionOutOfScope => f.write_str("capability lease action is out of scope"),
            Self::ResourceOutOfScope => f.write_str("capability lease resource is out of scope"),
            Self::ContextOutOfScope => {
                f.write_str("capability lease context requirements are not satisfied")
            }
        }
    }
}

impl Error for CapabilityLeaseUseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Scope(error) => Some(error),
            Self::LeaseNotFound
            | Self::HandleStateMismatch
            | Self::StaleGeneration
            | Self::InvalidPrincipal
            | Self::PrincipalMismatch
            | Self::Inactive
            | Self::Revoked
            | Self::InvalidObservedTime
            | Self::InvalidStoredTime
            | Self::NotYetValid
            | Self::Expired
            | Self::ActionOutOfScope
            | Self::ResourceOutOfScope
            | Self::ContextOutOfScope => None,
        }
    }
}

impl From<CapabilityLeaseRuntimeError> for CapabilityLeaseUseError {
    fn from(value: CapabilityLeaseRuntimeError) -> Self {
        Self::Runtime(value)
    }
}

impl From<CapabilityLeaseScopeError> for CapabilityLeaseUseError {
    fn from(value: CapabilityLeaseScopeError) -> Self {
        Self::Scope(value)
    }
}

pub(crate) fn validate_capability_lease_use(
    layout: &AuthorityLayout,
    lease: &CapabilityLease,
    principal_id: &str,
    action: &str,
    resource: &str,
    context_constraints: &[&str],
    observed_at: &str,
) -> Result<CapabilityLeaseUseEvidence, CapabilityLeaseUseError> {
    validate_principal(principal_id)?;
    if principal_id != lease.principal_id() {
        return Err(CapabilityLeaseUseError::PrincipalMismatch);
    }

    let request_scope =
        CapabilityLeaseScope::normalize(&[action], &[resource], context_constraints)?;
    validate_scope_use(lease.scope(), &request_scope)?;

    let chain = load_capability_lease_runtime_chain(layout, lease.lease_id().to_bytes())?;
    validate_loaded_chain(lease, &chain, principal_id, observed_at)
}

fn validate_scope_use(
    lease_scope: &CapabilityLeaseScope,
    request_scope: &CapabilityLeaseScope,
) -> Result<(), CapabilityLeaseUseError> {
    let action = request_scope
        .actions()
        .first()
        .ok_or(CapabilityLeaseUseError::ActionOutOfScope)?;
    if lease_scope.actions().binary_search(action).is_err() {
        return Err(CapabilityLeaseUseError::ActionOutOfScope);
    }

    let resource = request_scope
        .resources()
        .first()
        .ok_or(CapabilityLeaseUseError::ResourceOutOfScope)?;
    if lease_scope.resources().binary_search(resource).is_err() {
        return Err(CapabilityLeaseUseError::ResourceOutOfScope);
    }

    if !lease_scope.context_constraints().iter().all(|constraint| {
        request_scope
            .context_constraints()
            .binary_search(constraint)
            .is_ok()
    }) {
        return Err(CapabilityLeaseUseError::ContextOutOfScope);
    }
    Ok(())
}

fn validate_loaded_chain(
    lease: &CapabilityLease,
    chain: &[CapabilityLeaseRuntimeState],
    principal_id: &str,
    observed_at: &str,
) -> Result<CapabilityLeaseUseEvidence, CapabilityLeaseUseError> {
    if !valid_utc_second(observed_at) {
        return Err(CapabilityLeaseUseError::InvalidObservedTime);
    }
    let current = chain
        .first()
        .ok_or(CapabilityLeaseUseError::LeaseNotFound)?;

    if current.lease_id != lease.lease_id().to_bytes()
        || current.parent_lease_id != lease.parent_lease_id().map(CapabilityLeaseId::to_bytes)
        || current.principal_id != lease.principal_id()
        || current.authority_digest != lease.authority_digest()
    {
        return Err(CapabilityLeaseUseError::HandleStateMismatch);
    }
    if current.generation != lease.generation() {
        return Err(CapabilityLeaseUseError::StaleGeneration);
    }
    if current.principal_id != principal_id {
        return Err(CapabilityLeaseUseError::PrincipalMismatch);
    }

    for state in chain {
        validate_runtime_state(state, observed_at)?;
    }

    Ok(CapabilityLeaseUseEvidence {
        lease_id: lease.lease_id(),
        generation: lease.generation(),
        authority_digest: lease.authority_digest(),
        scope_digest: lease.scope().digest(),
    })
}

fn validate_runtime_state(
    state: &CapabilityLeaseRuntimeState,
    observed_at: &str,
) -> Result<(), CapabilityLeaseUseError> {
    if state.status != "active" {
        return Err(CapabilityLeaseUseError::Inactive);
    }
    if state.revoked {
        return Err(CapabilityLeaseUseError::Revoked);
    }

    if let Some(not_before) = state.not_before.as_deref() {
        if !valid_utc_second(not_before) {
            return Err(CapabilityLeaseUseError::InvalidStoredTime);
        }
        if observed_at < not_before {
            return Err(CapabilityLeaseUseError::NotYetValid);
        }
    }
    if let Some(expires_at) = state.expires_at.as_deref() {
        if !valid_utc_second(expires_at) {
            return Err(CapabilityLeaseUseError::InvalidStoredTime);
        }
        if observed_at >= expires_at {
            return Err(CapabilityLeaseUseError::Expired);
        }
    }
    if let (Some(not_before), Some(expires_at)) =
        (state.not_before.as_deref(), state.expires_at.as_deref())
        && not_before >= expires_at
    {
        return Err(CapabilityLeaseUseError::InvalidStoredTime);
    }
    Ok(())
}

fn validate_principal(value: &str) -> Result<(), CapabilityLeaseUseError> {
    if value.is_empty()
        || value.len() > MAX_PRINCIPAL_ID_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(CapabilityLeaseUseError::InvalidPrincipal);
    }
    Ok(())
}

fn valid_utc_second(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for index in [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !bytes[index].is_ascii_digit() {
            return false;
        }
    }

    let year = decimal(bytes, 0, 4);
    let month = decimal(bytes, 5, 7);
    let day = decimal(bytes, 8, 10);
    let hour = decimal(bytes, 11, 13);
    let minute = decimal(bytes, 14, 16);
    let second = decimal(bytes, 17, 19);
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let max_day = match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=max_day).contains(&day)
}

fn decimal(bytes: &[u8], start: usize, end: usize) -> u32 {
    bytes[start..end]
        .iter()
        .fold(0_u32, |value, byte| value * 10 + u32::from(*byte - b'0'))
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

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

    fn runtime_state(
        lease_id: [u8; 16],
        parent_lease_id: Option<[u8; 16]>,
    ) -> CapabilityLeaseRuntimeState {
        CapabilityLeaseRuntimeState {
            lease_id,
            principal_id: "owner:local".to_owned(),
            parent_lease_id,
            not_before: Some("2026-08-27T00:00:00Z".to_owned()),
            expires_at: Some("2026-08-28T00:00:00Z".to_owned()),
            generation: 3,
            status: "active".to_owned(),
            authority_digest: [5_u8; 32],
            revoked: false,
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
    fn mutation_preparation_keeps_kernel_and_ledger_scope_digests_identical() {
        let scope = CapabilityLeaseScope::normalize(
            &["session.read", "session.create"],
            &["session:1"],
            &["local-owner"],
        )
        .unwrap();
        let prepared = prepare_issue(
            "client:9:alice",
            None,
            &scope,
            Some("2026-08-27T00:00:00Z"),
            Some("2026-08-28T00:00:00Z"),
        )
        .unwrap();
        assert_eq!(prepared.scope_digest(), scope.digest());
        assert!(prepared.resource().starts_with("capability-lease-issue:"));
    }

    #[test]
    fn mutation_binding_uses_current_sealed_generation_and_digest() {
        let lease = kernel_fixture_lease();
        let binding = lease_binding(&lease);
        assert_eq!(binding.lease_id(), lease.lease_id().to_bytes());
        assert_eq!(binding.generation(), lease.generation());
        assert_eq!(binding.authority_digest(), lease.authority_digest());
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

    #[test]
    fn use_time_validation_accepts_current_bound_active_chain() {
        let lease = kernel_fixture_lease();
        let mut parent = runtime_state([2_u8; 16], None);
        parent.generation = 9;
        parent.authority_digest = [8_u8; 32];
        let evidence = validate_loaded_chain(
            &lease,
            &[runtime_state([1_u8; 16], Some([2_u8; 16])), parent],
            "owner:local",
            "2026-08-27T12:00:00Z",
        )
        .unwrap();
        assert_eq!(evidence.lease_id(), lease.lease_id());
        assert_eq!(evidence.generation(), 3);
    }

    #[test]
    fn use_time_validation_rejects_stale_generation_principal_and_handle_mismatch() {
        let lease = kernel_fixture_lease();
        let mut stale = runtime_state([1_u8; 16], Some([2_u8; 16]));
        stale.generation = 4;
        assert!(matches!(
            validate_loaded_chain(&lease, &[stale], "owner:local", "2026-08-27T12:00:00Z"),
            Err(CapabilityLeaseUseError::StaleGeneration)
        ));

        let current = runtime_state([1_u8; 16], Some([2_u8; 16]));
        assert!(matches!(
            validate_loaded_chain(
                &lease,
                std::slice::from_ref(&current),
                "owner:other",
                "2026-08-27T12:00:00Z"
            ),
            Err(CapabilityLeaseUseError::PrincipalMismatch)
        ));

        let mut mismatch = current;
        mismatch.authority_digest = [7_u8; 32];
        assert!(matches!(
            validate_loaded_chain(&lease, &[mismatch], "owner:local", "2026-08-27T12:00:00Z"),
            Err(CapabilityLeaseUseError::HandleStateMismatch)
        ));
    }

    #[test]
    fn use_time_validation_rejects_revoked_inactive_and_invalid_parent_state() {
        let lease = kernel_fixture_lease();
        let child = runtime_state([1_u8; 16], Some([2_u8; 16]));

        let mut revoked_parent = runtime_state([2_u8; 16], None);
        revoked_parent.generation = 9;
        revoked_parent.authority_digest = [8_u8; 32];
        revoked_parent.revoked = true;
        assert!(matches!(
            validate_loaded_chain(
                &lease,
                &[child.clone(), revoked_parent],
                "owner:local",
                "2026-08-27T12:00:00Z",
            ),
            Err(CapabilityLeaseUseError::Revoked)
        ));

        let mut inactive = child;
        inactive.status = "suspended".to_owned();
        assert!(matches!(
            validate_loaded_chain(&lease, &[inactive], "owner:local", "2026-08-27T12:00:00Z"),
            Err(CapabilityLeaseUseError::Inactive)
        ));
    }

    #[test]
    fn use_time_validation_rejects_not_yet_valid_expired_and_malformed_time() {
        let lease = kernel_fixture_lease();
        let current = runtime_state([1_u8; 16], Some([2_u8; 16]));
        assert!(matches!(
            validate_loaded_chain(
                &lease,
                std::slice::from_ref(&current),
                "owner:local",
                "2026-08-26T23:59:59Z"
            ),
            Err(CapabilityLeaseUseError::NotYetValid)
        ));
        assert!(matches!(
            validate_loaded_chain(
                &lease,
                std::slice::from_ref(&current),
                "owner:local",
                "2026-08-28T00:00:00Z"
            ),
            Err(CapabilityLeaseUseError::Expired)
        ));
        assert!(matches!(
            validate_loaded_chain(&lease, &[current], "owner:local", "2026-02-30T00:00:00Z"),
            Err(CapabilityLeaseUseError::InvalidObservedTime)
        ));
    }

    #[test]
    fn use_time_scope_requires_exact_action_resource_and_all_lease_context() {
        let lease = kernel_fixture_lease();
        let valid = CapabilityLeaseScope::normalize(
            &["session.read"],
            &["session:1"],
            &["local-owner", "local-session:1", "local-extra"],
        )
        .unwrap();
        assert!(validate_scope_use(lease.scope(), &valid).is_ok());

        let bad_action = CapabilityLeaseScope::normalize(
            &["client.revoke"],
            &["session:1"],
            &["local-owner", "local-session:1"],
        )
        .unwrap();
        assert!(matches!(
            validate_scope_use(lease.scope(), &bad_action),
            Err(CapabilityLeaseUseError::ActionOutOfScope)
        ));

        let bad_resource = CapabilityLeaseScope::normalize(
            &["session.read"],
            &["session:3"],
            &["local-owner", "local-session:1"],
        )
        .unwrap();
        assert!(matches!(
            validate_scope_use(lease.scope(), &bad_resource),
            Err(CapabilityLeaseUseError::ResourceOutOfScope)
        ));

        let bad_context =
            CapabilityLeaseScope::normalize(&["session.read"], &["session:1"], &["local-owner"])
                .unwrap();
        assert!(matches!(
            validate_scope_use(lease.scope(), &bad_context),
            Err(CapabilityLeaseUseError::ContextOutOfScope)
        ));
    }

    #[test]
    fn canonical_utc_second_validation_handles_leap_years_and_bounds() {
        assert!(valid_utc_second("2028-02-29T23:59:59Z"));
        assert!(!valid_utc_second("2027-02-29T00:00:00Z"));
        assert!(!valid_utc_second("2026-13-01T00:00:00Z"));
        assert!(!valid_utc_second("2026-01-01T24:00:00Z"));
        assert!(!valid_utc_second("2026-01-01T00:00:60Z"));
    }
}
