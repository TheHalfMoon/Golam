#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::authority_security_write::{
    append_approval_consumption_snapshot, append_capability_lease_snapshot,
    append_capability_revocation_snapshot,
};
use crate::storage::{AuthorityStore, StorageError};

const MAX_SCOPE_ITEMS: usize = 32;
const MAX_ACTION_BYTES: usize = 128;
const MAX_RESOURCE_BYTES: usize = 2048;
const MAX_CONTEXT_BYTES: usize = 256;
const MAX_CANONICAL_SCOPE_BYTES: usize = 131_072;
const MAX_PRINCIPAL_ID_BYTES: usize = 512;
const MAX_REASON_CODE_BYTES: usize = 128;
const LEASE_SCOPE_DOMAIN: &[u8] = b"golam:capability-lease-scope:v1";
const ISSUE_INTENT_DOMAIN: &[u8] = b"golam:capability-lease-issue-intent:v1";
const ISSUE_ID_DOMAIN: &[u8] = b"golam:capability-lease-id:v1";
const AUTHORITY_DOMAIN: &[u8] = b"golam:capability-lease-authority:v1";
const REVOKE_INTENT_DOMAIN: &[u8] = b"golam:capability-lease-revocation-intent:v1";
const REVOKE_ID_DOMAIN: &[u8] = b"golam:capability-lease-revocation-id:v1";
const APPROVAL_CONSUMPTION_DOMAIN: &[u8] = b"golam:capability-lease-approval-consumption:v1";

pub const CAPABILITY_LEASE_ISSUE_ACTION: &str = "lease.issue";
pub const CAPABILITY_LEASE_REVOKE_ACTION: &str = "lease.revoke";
pub const CAPABILITY_LEASE_MUTATION_RISK_CLASS: &str = "capability_lease_mutation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityLeaseBinding {
    lease_id: [u8; 16],
    generation: u64,
    authority_digest: [u8; 32],
}

impl CapabilityLeaseBinding {
    pub const fn new(lease_id: [u8; 16], generation: u64, authority_digest: [u8; 32]) -> Self {
        Self {
            lease_id,
            generation,
            authority_digest,
        }
    }

    pub const fn lease_id(self) -> [u8; 16] {
        self.lease_id
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn authority_digest(self) -> [u8; 32] {
        self.authority_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityLeaseRecord {
    pub lease_id: [u8; 16],
    pub principal_id: String,
    pub parent_lease_id: Option<[u8; 16]>,
    pub generation: u64,
    pub issued_by: String,
    pub issued_global_seq: u64,
    pub authority_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityLeaseRevocationRecord {
    pub revocation_id: [u8; 16],
    pub lease_id: [u8; 16],
    pub revoked_by: String,
    pub revoked_global_seq: u64,
    pub revoked_at: String,
    pub reason_code: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedCapabilityLeaseIssue {
    principal_id: String,
    parent: Option<CapabilityLeaseBinding>,
    actions: Vec<String>,
    resources: Vec<String>,
    context_constraints: Vec<String>,
    actions_blob: Vec<u8>,
    resources_blob: Vec<u8>,
    context_blob: Vec<u8>,
    scope_digest: [u8; 32],
    not_before: Option<String>,
    expires_at: Option<String>,
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedCapabilityLeaseIssue {
    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }

    pub const fn scope_digest(&self) -> [u8; 32] {
        self.scope_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedCapabilityLeaseRevocation {
    lease: CapabilityLeaseBinding,
    reason_code: String,
    revoked_at: String,
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedCapabilityLeaseRevocation {
    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }
}

#[derive(Debug)]
pub enum CapabilityLeaseMutationError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidPrincipal,
    InvalidScope(&'static str),
    InvalidTime,
    InvalidReasonCode,
    IntegerOverflow,
    MissingAuthorityDecision,
    AuthorityDecisionMismatch,
    StaleAuthorityDecision,
    SelfGrantForbidden,
    ParentNotFound,
    ParentPrincipalMismatch,
    ParentInactive,
    ParentRevoked,
    ParentEvidenceMismatch,
    ParentScopeWidening,
    ParentTemporalWidening,
    EffectNotFound,
    EffectMismatch,
    ApprovalNotFound,
    ApprovalMismatch,
    ApprovalAlreadyUsed,
    DuplicateLease,
    LeaseNotFound,
    LeaseInactive,
    LeaseAlreadyRevoked,
    LeaseEvidenceMismatch,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for CapabilityLeaseMutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "capability lease authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "capability lease sqlite error: {error}"),
            Self::Core(error) => write!(f, "capability lease canonical encoding error: {error}"),
            Self::Integrity(error) => write!(f, "capability lease integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "capability lease authority-security error: {error}")
            }
            Self::InvalidPrincipal => f.write_str("capability lease principal id is not canonical"),
            Self::InvalidScope(reason) => write!(f, "capability lease scope is invalid: {reason}"),
            Self::InvalidTime => f.write_str("capability lease validity time is invalid"),
            Self::InvalidReasonCode => {
                f.write_str("capability lease revocation reason code is not canonical")
            }
            Self::IntegerOverflow => f.write_str("capability lease integer conversion overflow"),
            Self::MissingAuthorityDecision => {
                f.write_str("capability lease mutation has no durable authorization decision")
            }
            Self::AuthorityDecisionMismatch => f.write_str(
                "capability lease mutation authorization decision does not match exact action/resource",
            ),
            Self::StaleAuthorityDecision => {
                f.write_str("capability lease mutation authorization decision is stale")
            }
            Self::SelfGrantForbidden => {
                f.write_str("a principal cannot issue capability authority to itself")
            }
            Self::ParentNotFound => f.write_str("parent capability lease does not exist"),
            Self::ParentPrincipalMismatch => {
                f.write_str("parent capability lease belongs to a different principal")
            }
            Self::ParentInactive => f.write_str("parent capability lease is not active"),
            Self::ParentRevoked => f.write_str("parent capability lease is revoked"),
            Self::ParentEvidenceMismatch => {
                f.write_str("parent capability lease evidence is stale or mismatched")
            }
            Self::ParentScopeWidening => {
                f.write_str("child capability lease scope would widen parent authority")
            }
            Self::ParentTemporalWidening => {
                f.write_str("child capability lease validity would widen parent lifetime")
            }
            Self::EffectNotFound => f.write_str("capability lease mutation effect does not exist"),
            Self::EffectMismatch => f.write_str(
                "capability lease mutation effect is not exact authorized at-most-once elevated work",
            ),
            Self::ApprovalNotFound => {
                f.write_str("capability lease mutation approval does not exist")
            }
            Self::ApprovalMismatch => f.write_str(
                "capability lease mutation approval does not match exact effect/action/resource",
            ),
            Self::ApprovalAlreadyUsed => {
                f.write_str("capability lease mutation one-shot approval was already consumed")
            }
            Self::DuplicateLease => f.write_str("capability lease already exists"),
            Self::LeaseNotFound => f.write_str("capability lease does not exist"),
            Self::LeaseInactive => f.write_str("capability lease is not active"),
            Self::LeaseAlreadyRevoked => f.write_str("capability lease is already revoked"),
            Self::LeaseEvidenceMismatch => {
                f.write_str("capability lease evidence is stale or mismatched")
            }
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored capability lease record is invalid: {reason}")
            }
        }
    }
}

impl Error for CapabilityLeaseMutationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for CapabilityLeaseMutationError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for CapabilityLeaseMutationError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for CapabilityLeaseMutationError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub fn prepare_capability_lease_issue(
    principal_id: &str,
    parent: Option<CapabilityLeaseBinding>,
    actions: &[String],
    resources: &[String],
    context_constraints: &[String],
    not_before: Option<&str>,
    expires_at: Option<&str>,
) -> Result<PreparedCapabilityLeaseIssue, CapabilityLeaseMutationError> {
    validate_principal_id(principal_id)?;
    let actions = normalize_entries(actions, ScopeEntryKind::Action)?;
    let resources = normalize_entries(resources, ScopeEntryKind::Resource)?;
    let context_constraints = normalize_entries(context_constraints, ScopeEntryKind::Context)?;
    let scope_digest = scope_digest(&actions, &resources, &context_constraints)?;
    let actions_blob = encode_stored_entries(&actions);
    let resources_blob = encode_stored_entries(&resources);
    let context_blob = encode_stored_entries(&context_constraints);
    let not_before = validate_optional_time(not_before)?;
    let expires_at = validate_optional_time(expires_at)?;
    if let (Some(start), Some(end)) = (not_before.as_deref(), expires_at.as_deref())
        && start >= end
    {
        return Err(CapabilityLeaseMutationError::InvalidTime);
    }

    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(ISSUE_INTENT_DOMAIN)?;
    encoder.push_bytes(principal_id.as_bytes())?;
    encode_optional_binding(&mut encoder, parent)?;
    encoder.push_bytes(&actions_blob)?;
    encoder.push_bytes(&resources_blob)?;
    encoder.push_bytes(&context_blob)?;
    encode_optional_text(&mut encoder, not_before.as_deref())?;
    encode_optional_text(&mut encoder, expires_at.as_deref())?;
    let intent_digest = crate::payload_hash(&encoder.finish());
    let resource = format!("capability-lease-issue:{}", hex_bytes(&intent_digest));

    Ok(PreparedCapabilityLeaseIssue {
        principal_id: principal_id.to_owned(),
        parent,
        actions,
        resources,
        context_constraints,
        actions_blob,
        resources_blob,
        context_blob,
        scope_digest,
        not_before,
        expires_at,
        intent_digest,
        resource,
    })
}

pub fn prepare_capability_lease_revocation(
    lease: CapabilityLeaseBinding,
    reason_code: &str,
    revoked_at: &str,
) -> Result<PreparedCapabilityLeaseRevocation, CapabilityLeaseMutationError> {
    if lease.generation == 0 {
        return Err(CapabilityLeaseMutationError::LeaseEvidenceMismatch);
    }
    validate_reason_code(reason_code)?;
    if !valid_utc_second(revoked_at) {
        return Err(CapabilityLeaseMutationError::InvalidTime);
    }
    let resource = capability_lease_resource(lease.lease_id);
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(REVOKE_INTENT_DOMAIN)?;
    encode_binding(&mut encoder, lease)?;
    encoder.push_bytes(reason_code.as_bytes())?;
    encoder.push_bytes(revoked_at.as_bytes())?;
    let intent_digest = crate::payload_hash(&encoder.finish());
    Ok(PreparedCapabilityLeaseRevocation {
        lease,
        reason_code: reason_code.to_owned(),
        revoked_at: revoked_at.to_owned(),
        intent_digest,
        resource,
    })
}

pub fn capability_lease_resource(lease_id: [u8; 16]) -> String {
    format!("capability-lease:{}", hex_bytes(&lease_id))
}

pub struct CapabilityLeaseStore {
    connection: Connection,
}

impl CapabilityLeaseStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, CapabilityLeaseMutationError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn issue(
        &mut self,
        prepared: PreparedCapabilityLeaseIssue,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<CapabilityLeaseRecord, CapabilityLeaseMutationError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let authority = verify_current_authority(
            &transaction,
            authority_decision_id,
            CAPABILITY_LEASE_ISSUE_ACTION,
            &prepared.resource,
        )?;
        if authority.principal == prepared.principal_id {
            return Err(CapabilityLeaseMutationError::SelfGrantForbidden);
        }
        verify_mutation_effect(
            &transaction,
            effect_id,
            CAPABILITY_LEASE_ISSUE_ACTION,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        verify_once_approval(
            &transaction,
            approval_id,
            effect_id,
            CAPABILITY_LEASE_ISSUE_ACTION,
            &prepared.resource,
        )?;
        verify_parent_lease(&transaction, &prepared)?;

        let lease_id = derived_id(
            ISSUE_ID_DOMAIN,
            prepared.intent_digest,
            effect_id,
            authority_decision_id,
            approval_id,
        );
        let duplicate = transaction
            .query_row(
                "SELECT 1 FROM capability_leases WHERE lease_id = ?1 LIMIT 1",
                params![&lease_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if duplicate {
            return Err(CapabilityLeaseMutationError::DuplicateLease);
        }

        let generation = 1_u64;
        let parent_lease_id = prepared.parent.map(CapabilityLeaseBinding::lease_id);
        let authority_digest = issue_authority_digest(
            lease_id,
            &prepared,
            &authority.principal,
            authority.global_seq,
            generation,
            authority_decision_id,
            approval_id,
            effect_id,
        )?;
        transaction.execute(
            "INSERT INTO capability_leases (lease_id, principal_id, parent_lease_id, actions_scope, resources_scope, context_constraints, issued_by, issued_global_seq, not_before, expires_at, generation, status, authority_digest) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'active', ?12)",
            params![
                &lease_id[..],
                &prepared.principal_id,
                parent_lease_id.map(|id| id.to_vec()),
                &prepared.actions_blob,
                &prepared.resources_blob,
                &prepared.context_blob,
                &authority.principal,
                to_i64(authority.global_seq)?,
                prepared.not_before.as_deref(),
                prepared.expires_at.as_deref(),
                to_i64(generation)?,
                &authority_digest[..],
            ],
        )?;
        append_capability_lease_snapshot(&transaction, &lease_id)
            .map_err(|error| CapabilityLeaseMutationError::AuthoritySecurity(error.to_string()))?;
        consume_once_approval(&transaction, approval_id, effect_id, authority.global_seq)?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| CapabilityLeaseMutationError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(CapabilityLeaseRecord {
            lease_id,
            principal_id: prepared.principal_id,
            parent_lease_id,
            generation,
            issued_by: authority.principal,
            issued_global_seq: authority.global_seq,
            authority_digest,
        })
    }

    pub fn revoke(
        &mut self,
        prepared: PreparedCapabilityLeaseRevocation,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<CapabilityLeaseRevocationRecord, CapabilityLeaseMutationError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let authority = verify_current_authority(
            &transaction,
            authority_decision_id,
            CAPABILITY_LEASE_REVOKE_ACTION,
            &prepared.resource,
        )?;
        verify_mutation_effect(
            &transaction,
            effect_id,
            CAPABILITY_LEASE_REVOKE_ACTION,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        verify_once_approval(
            &transaction,
            approval_id,
            effect_id,
            CAPABILITY_LEASE_REVOKE_ACTION,
            &prepared.resource,
        )?;
        verify_revocation_target(&transaction, prepared.lease)?;

        let already_revoked = transaction
            .query_row(
                "SELECT 1 FROM capability_revocations WHERE lease_id = ?1 LIMIT 1",
                params![&prepared.lease.lease_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if already_revoked {
            return Err(CapabilityLeaseMutationError::LeaseAlreadyRevoked);
        }

        let revocation_id = derived_id(
            REVOKE_ID_DOMAIN,
            prepared.intent_digest,
            effect_id,
            authority_decision_id,
            approval_id,
        );
        transaction.execute(
            "INSERT INTO capability_revocations (revocation_id, lease_id, revoked_by, reason_code, revoked_global_seq, revoked_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &revocation_id[..],
                &prepared.lease.lease_id[..],
                &authority.principal,
                &prepared.reason_code,
                to_i64(authority.global_seq)?,
                &prepared.revoked_at,
            ],
        )?;
        append_capability_revocation_snapshot(&transaction, &revocation_id)
            .map_err(|error| CapabilityLeaseMutationError::AuthoritySecurity(error.to_string()))?;
        consume_once_approval(&transaction, approval_id, effect_id, authority.global_seq)?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| CapabilityLeaseMutationError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(CapabilityLeaseRevocationRecord {
            revocation_id,
            lease_id: prepared.lease.lease_id,
            revoked_by: authority.principal,
            revoked_global_seq: authority.global_seq,
            revoked_at: prepared.revoked_at,
            reason_code: prepared.reason_code,
        })
    }
}

struct AuthorityEvidence {
    principal: String,
    global_seq: u64,
}

fn verify_transaction_integrity(
    transaction: &Transaction<'_>,
) -> Result<(), CapabilityLeaseMutationError> {
    crate::integrity::verify(transaction)
        .map_err(|error| CapabilityLeaseMutationError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| CapabilityLeaseMutationError::AuthoritySecurity(error.to_string()))
}

fn verify_current_authority(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    expected_action: &str,
    expected_resource: &str,
) -> Result<AuthorityEvidence, CapabilityLeaseMutationError> {
    let row = transaction
        .query_row(
            "SELECT principal, action, resource, decision, global_seq FROM authorization_decisions WHERE decision_id = ?1",
            params![&decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()?
        .ok_or(CapabilityLeaseMutationError::MissingAuthorityDecision)?;
    if row.1 != expected_action || row.2 != expected_resource || row.3 != "allow" {
        return Err(CapabilityLeaseMutationError::AuthorityDecisionMismatch);
    }
    let global_seq = seq_from_i64(row.4)?;
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (SELECT global_seq FROM session_events UNION ALL SELECT global_seq FROM effect_transitions UNION ALL SELECT global_seq FROM authorization_decisions)",
        [],
        |row| row.get(0),
    )?;
    if global_seq != seq_from_i64(latest)? {
        return Err(CapabilityLeaseMutationError::StaleAuthorityDecision);
    }
    Ok(AuthorityEvidence {
        principal: row.0,
        global_seq,
    })
}

fn verify_mutation_effect(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    expected_action: &str,
    expected_resource: &str,
    expected_payload_hash: [u8; 32],
) -> Result<(), CapabilityLeaseMutationError> {
    let row = transaction
        .query_row(
            "SELECT i.action, i.resource, i.risk_class, i.execution_semantics, i.payload_hash, t.to_state FROM effect_intents i JOIN effect_transitions t ON t.effect_id = i.effect_id WHERE i.effect_id = ?1 AND t.global_seq = (SELECT MAX(t2.global_seq) FROM effect_transitions t2 WHERE t2.effect_id = i.effect_id)",
            params![&effect_id.0.to_be_bytes()[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or(CapabilityLeaseMutationError::EffectNotFound)?;
    if row.0 != expected_action
        || row.1 != expected_resource
        || row.2 != CAPABILITY_LEASE_MUTATION_RISK_CLASS
        || row.3 != "at_most_once"
        || row.4.as_slice() != expected_payload_hash
        || row.5 != "authorized"
    {
        return Err(CapabilityLeaseMutationError::EffectMismatch);
    }
    Ok(())
}

fn verify_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    expected_action: &str,
    expected_resource: &str,
) -> Result<(), CapabilityLeaseMutationError> {
    let row = transaction
        .query_row(
            "SELECT class, action_scope, resource_scope, effect_id, session_id, risk_class, expires_at, max_uses, revoked_at FROM approvals WHERE approval_id = ?1",
            params![&approval_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            },
        )
        .optional()?
        .ok_or(CapabilityLeaseMutationError::ApprovalNotFound)?;
    if row.0 != "ONCE"
        || row.1.as_slice() != expected_action.as_bytes()
        || row.2.as_slice() != expected_resource.as_bytes()
        || row.3.as_deref() != Some(effect_id.0.to_be_bytes().as_slice())
        || row.4.is_some()
        || row.5 != CAPABILITY_LEASE_MUTATION_RISK_CLASS
        || row.6.is_some()
        || row.7 != Some(1)
        || row.8.is_some()
    {
        return Err(CapabilityLeaseMutationError::ApprovalMismatch);
    }
    let already_used = transaction
        .query_row(
            "SELECT 1 FROM approval_consumptions WHERE approval_id = ?1 LIMIT 1",
            params![&approval_id[..]],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if already_used {
        return Err(CapabilityLeaseMutationError::ApprovalAlreadyUsed);
    }
    Ok(())
}

fn consume_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    global_seq: u64,
) -> Result<(), CapabilityLeaseMutationError> {
    let consumption_id = approval_consumption_id(approval_id, effect_id);
    transaction.execute(
        "INSERT INTO approval_consumptions (consumption_id, approval_id, effect_or_operation_id, reserved_global_seq, consumed_global_seq, state) VALUES (?1, ?2, ?3, ?4, ?5, 'consumed')",
        params![
            &consumption_id[..],
            &approval_id[..],
            &effect_id.0.to_be_bytes()[..],
            to_i64(global_seq)?,
            to_i64(global_seq)?,
        ],
    )?;
    append_approval_consumption_snapshot(transaction, &consumption_id)
        .map_err(|error| CapabilityLeaseMutationError::AuthoritySecurity(error.to_string()))
}

fn verify_parent_lease(
    transaction: &Transaction<'_>,
    prepared: &PreparedCapabilityLeaseIssue,
) -> Result<(), CapabilityLeaseMutationError> {
    let Some(parent) = prepared.parent else {
        return Ok(());
    };
    let row = transaction
        .query_row(
            "SELECT principal_id, actions_scope, resources_scope, context_constraints, not_before, expires_at, generation, status, authority_digest, EXISTS(SELECT 1 FROM capability_revocations r WHERE r.lease_id = capability_leases.lease_id) FROM capability_leases WHERE lease_id = ?1",
            params![&parent.lease_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Vec<u8>>(8)?,
                    row.get::<_, i64>(9)?,
                ))
            },
        )
        .optional()?
        .ok_or(CapabilityLeaseMutationError::ParentNotFound)?;
    if row.0 != prepared.principal_id {
        return Err(CapabilityLeaseMutationError::ParentPrincipalMismatch);
    }
    let generation = seq_from_i64(row.6)?;
    let authority_digest: [u8; 32] = row.8.try_into().map_err(|_| {
        CapabilityLeaseMutationError::InvalidStoredRecord("parent authority digest is not 32 bytes")
    })?;
    if generation != parent.generation || authority_digest != parent.authority_digest {
        return Err(CapabilityLeaseMutationError::ParentEvidenceMismatch);
    }
    if row.7 != "active" {
        return Err(CapabilityLeaseMutationError::ParentInactive);
    }
    if row.9 != 0 {
        return Err(CapabilityLeaseMutationError::ParentRevoked);
    }

    let parent_actions = decode_stored_entries(&row.1, ScopeEntryKind::Action)?;
    let parent_resources = decode_stored_entries(&row.2, ScopeEntryKind::Resource)?;
    let parent_context = decode_stored_entries(&row.3, ScopeEntryKind::Context)?;
    if !is_subset(&prepared.actions, &parent_actions)
        || !is_subset(&prepared.resources, &parent_resources)
        || !is_subset(&prepared.context_constraints, &parent_context)
    {
        return Err(CapabilityLeaseMutationError::ParentScopeWidening);
    }
    if let Some(parent_start) = row.4.as_deref() {
        if !valid_utc_second(parent_start) {
            return Err(CapabilityLeaseMutationError::InvalidStoredRecord(
                "parent not_before is malformed",
            ));
        }
        if prepared
            .not_before
            .as_deref()
            .is_none_or(|child_start| child_start < parent_start)
        {
            return Err(CapabilityLeaseMutationError::ParentTemporalWidening);
        }
    }
    if let Some(parent_end) = row.5.as_deref() {
        if !valid_utc_second(parent_end) {
            return Err(CapabilityLeaseMutationError::InvalidStoredRecord(
                "parent expires_at is malformed",
            ));
        }
        if prepared
            .expires_at
            .as_deref()
            .is_none_or(|child_end| child_end > parent_end)
        {
            return Err(CapabilityLeaseMutationError::ParentTemporalWidening);
        }
    }
    Ok(())
}

fn verify_revocation_target(
    transaction: &Transaction<'_>,
    lease: CapabilityLeaseBinding,
) -> Result<(), CapabilityLeaseMutationError> {
    let row = transaction
        .query_row(
            "SELECT generation, status, authority_digest FROM capability_leases WHERE lease_id = ?1",
            params![&lease.lease_id[..]],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or(CapabilityLeaseMutationError::LeaseNotFound)?;
    let generation = seq_from_i64(row.0)?;
    let authority_digest: [u8; 32] = row.2.try_into().map_err(|_| {
        CapabilityLeaseMutationError::InvalidStoredRecord("lease authority digest is not 32 bytes")
    })?;
    if generation != lease.generation || authority_digest != lease.authority_digest {
        return Err(CapabilityLeaseMutationError::LeaseEvidenceMismatch);
    }
    if row.1 != "active" {
        return Err(CapabilityLeaseMutationError::LeaseInactive);
    }
    Ok(())
}

fn issue_authority_digest(
    lease_id: [u8; 16],
    prepared: &PreparedCapabilityLeaseIssue,
    issued_by: &str,
    issued_global_seq: u64,
    generation: u64,
    decision_id: [u8; 16],
    approval_id: [u8; 16],
    effect_id: EffectId,
) -> Result<[u8; 32], CapabilityLeaseMutationError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(AUTHORITY_DOMAIN)?;
    encoder.push_bytes(&lease_id)?;
    encoder.push_bytes(prepared.principal_id.as_bytes())?;
    encode_optional_binding(&mut encoder, prepared.parent)?;
    encoder.push_bytes(&prepared.actions_blob)?;
    encoder.push_bytes(&prepared.resources_blob)?;
    encoder.push_bytes(&prepared.context_blob)?;
    encoder.push_bytes(issued_by.as_bytes())?;
    encoder.push_u64(issued_global_seq);
    encode_optional_text(&mut encoder, prepared.not_before.as_deref())?;
    encode_optional_text(&mut encoder, prepared.expires_at.as_deref())?;
    encoder.push_u64(generation);
    encoder.push_bytes(b"active")?;
    encoder.push_bytes(&decision_id)?;
    encoder.push_bytes(&approval_id)?;
    encoder.push_u128(effect_id.0);
    Ok(crate::payload_hash(&encoder.finish()))
}

fn derived_id(
    domain: &[u8],
    intent_digest: [u8; 32],
    effect_id: EffectId,
    decision_id: [u8; 16],
    approval_id: [u8; 16],
) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&intent_digest);
    hasher.update(&effect_id.0.to_be_bytes());
    hasher.update(&decision_id);
    hasher.update(&approval_id);
    let digest = hasher.finalize();
    let mut id = [0_u8; 16];
    id.copy_from_slice(&digest.as_bytes()[..16]);
    id
}

fn approval_consumption_id(approval_id: [u8; 16], effect_id: EffectId) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(APPROVAL_CONSUMPTION_DOMAIN);
    hasher.update(&approval_id);
    hasher.update(&effect_id.0.to_be_bytes());
    let digest = hasher.finalize();
    let mut id = [0_u8; 16];
    id.copy_from_slice(&digest.as_bytes()[..16]);
    id
}

fn scope_digest(
    actions: &[String],
    resources: &[String],
    context_constraints: &[String],
) -> Result<[u8; 32], CapabilityLeaseMutationError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(LEASE_SCOPE_DOMAIN)?;
    encode_digest_entries(&mut encoder, actions)?;
    encode_digest_entries(&mut encoder, resources)?;
    encode_digest_entries(&mut encoder, context_constraints)?;
    let canonical = encoder.finish();
    if canonical.len() > MAX_CANONICAL_SCOPE_BYTES {
        return Err(CapabilityLeaseMutationError::InvalidScope(
            "canonical scope exceeds byte bound",
        ));
    }
    Ok(crate::payload_hash(&canonical))
}

fn encode_digest_entries(
    encoder: &mut CanonicalEncoder,
    entries: &[String],
) -> Result<(), CapabilityLeaseMutationError> {
    encoder.push_u64(
        u64::try_from(entries.len()).map_err(|_| CapabilityLeaseMutationError::IntegerOverflow)?,
    );
    for entry in entries {
        encoder.push_bytes(entry.as_bytes())?;
    }
    Ok(())
}

fn encode_stored_entries(entries: &[String]) -> Vec<u8> {
    entries.join("\n").into_bytes()
}

fn decode_stored_entries(
    bytes: &[u8],
    kind: ScopeEntryKind,
) -> Result<Vec<String>, CapabilityLeaseMutationError> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let text = std::str::from_utf8(bytes).map_err(|_| {
        CapabilityLeaseMutationError::InvalidStoredRecord("scope bytes are not UTF-8")
    })?;
    let entries = text.split('\n').map(str::to_owned).collect::<Vec<_>>();
    if entries.len() > MAX_SCOPE_ITEMS {
        return Err(CapabilityLeaseMutationError::InvalidStoredRecord(
            "scope contains too many entries",
        ));
    }
    for entry in &entries {
        validate_scope_entry(entry, kind).map_err(|_| {
            CapabilityLeaseMutationError::InvalidStoredRecord("scope entry is not canonical")
        })?;
    }
    if entries.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(CapabilityLeaseMutationError::InvalidStoredRecord(
            "scope entries are not strictly sorted and unique",
        ));
    }
    Ok(entries)
}

#[derive(Clone, Copy)]
enum ScopeEntryKind {
    Action,
    Resource,
    Context,
}

fn normalize_entries(
    entries: &[String],
    kind: ScopeEntryKind,
) -> Result<Vec<String>, CapabilityLeaseMutationError> {
    if entries.len() > MAX_SCOPE_ITEMS {
        return Err(CapabilityLeaseMutationError::InvalidScope(
            "too many entries",
        ));
    }
    let mut normalized = Vec::with_capacity(entries.len());
    for entry in entries {
        validate_scope_entry(entry, kind)?;
        normalized.push(entry.clone());
    }
    normalized.sort_unstable();
    normalized.dedup();
    Ok(normalized)
}

fn validate_scope_entry(
    value: &str,
    kind: ScopeEntryKind,
) -> Result<(), CapabilityLeaseMutationError> {
    match kind {
        ScopeEntryKind::Action => validate_action(value),
        ScopeEntryKind::Resource => validate_resource(value),
        ScopeEntryKind::Context => validate_context(value),
    }
}

fn validate_action(value: &str) -> Result<(), CapabilityLeaseMutationError> {
    if value.is_empty() || value.len() > MAX_ACTION_BYTES {
        return Err(CapabilityLeaseMutationError::InvalidScope(
            "action is empty or oversized",
        ));
    }
    let bytes = value.as_bytes();
    if !bytes.first().is_some_and(u8::is_ascii_lowercase)
        || !bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        || bytes.windows(2).any(|pair| pair == b"..")
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-'))
        })
    {
        return Err(CapabilityLeaseMutationError::InvalidScope(
            "action is not canonical",
        ));
    }
    Ok(())
}

fn validate_resource(value: &str) -> Result<(), CapabilityLeaseMutationError> {
    if value.is_empty()
        || value.len() > MAX_RESOURCE_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(CapabilityLeaseMutationError::InvalidScope(
            "resource is not canonical",
        ));
    }
    Ok(())
}

fn validate_context(value: &str) -> Result<(), CapabilityLeaseMutationError> {
    if value.is_empty() || value.len() > MAX_CONTEXT_BYTES {
        return Err(CapabilityLeaseMutationError::InvalidScope(
            "context constraint is empty or oversized",
        ));
    }
    let bytes = value.as_bytes();
    if !bytes.first().is_some_and(u8::is_ascii_lowercase)
        || !bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-' | b':'))
        })
    {
        return Err(CapabilityLeaseMutationError::InvalidScope(
            "context constraint is not canonical",
        ));
    }
    Ok(())
}

fn validate_principal_id(value: &str) -> Result<(), CapabilityLeaseMutationError> {
    let known_prefix = ["owner:", "client:", "kernel:", "test:"]
        .iter()
        .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len());
    if !known_prefix
        || value.len() > MAX_PRINCIPAL_ID_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(CapabilityLeaseMutationError::InvalidPrincipal);
    }
    Ok(())
}

fn validate_reason_code(value: &str) -> Result<(), CapabilityLeaseMutationError> {
    if value.is_empty() || value.len() > MAX_REASON_CODE_BYTES {
        return Err(CapabilityLeaseMutationError::InvalidReasonCode);
    }
    let bytes = value.as_bytes();
    if !bytes.first().is_some_and(u8::is_ascii_lowercase)
        || !bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-'))
        })
    {
        return Err(CapabilityLeaseMutationError::InvalidReasonCode);
    }
    Ok(())
}

fn validate_optional_time(
    value: Option<&str>,
) -> Result<Option<String>, CapabilityLeaseMutationError> {
    match value {
        Some(value) if valid_utc_second(value) => Ok(Some(value.to_owned())),
        Some(_) => Err(CapabilityLeaseMutationError::InvalidTime),
        None => Ok(None),
    }
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

fn is_subset(child: &[String], parent: &[String]) -> bool {
    child
        .iter()
        .all(|entry| parent.binary_search(entry).is_ok())
}

fn encode_binding(
    encoder: &mut CanonicalEncoder,
    binding: CapabilityLeaseBinding,
) -> Result<(), CapabilityLeaseMutationError> {
    encoder.push_bytes(&binding.lease_id)?;
    encoder.push_u64(binding.generation);
    encoder.push_bytes(&binding.authority_digest)?;
    Ok(())
}

fn encode_optional_binding(
    encoder: &mut CanonicalEncoder,
    binding: Option<CapabilityLeaseBinding>,
) -> Result<(), CapabilityLeaseMutationError> {
    match binding {
        Some(binding) => {
            encoder.push_u8(1);
            encode_binding(encoder, binding)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn encode_optional_text(
    encoder: &mut CanonicalEncoder,
    value: Option<&str>,
) -> Result<(), CapabilityLeaseMutationError> {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value.as_bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[(byte >> 4) as usize]));
        value.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    value
}

fn seq_from_i64(value: i64) -> Result<u64, CapabilityLeaseMutationError> {
    u64::try_from(value)
        .map_err(|_| CapabilityLeaseMutationError::InvalidStoredRecord("negative sequence"))
}

fn to_i64(value: u64) -> Result<i64, CapabilityLeaseMutationError> {
    i64::try_from(value).map_err(|_| CapabilityLeaseMutationError::IntegerOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_preparation_is_deterministic_and_order_independent() {
        let first = prepare_capability_lease_issue(
            "client:9:alice",
            None,
            &["session.read".to_owned(), "session.create".to_owned()],
            &["session:2".to_owned(), "session:1".to_owned()],
            &["local-owner".to_owned()],
            Some("2026-08-27T00:00:00Z"),
            Some("2026-08-28T00:00:00Z"),
        )
        .unwrap();
        let reordered = prepare_capability_lease_issue(
            "client:9:alice",
            None,
            &["session.create".to_owned(), "session.read".to_owned()],
            &["session:1".to_owned(), "session:2".to_owned()],
            &["local-owner".to_owned()],
            Some("2026-08-27T00:00:00Z"),
            Some("2026-08-28T00:00:00Z"),
        )
        .unwrap();
        assert_eq!(first.scope_digest(), reordered.scope_digest());
        assert_eq!(first.intent_digest(), reordered.intent_digest());
        assert_eq!(first.resource(), reordered.resource());
    }

    #[test]
    fn preparation_rejects_invalid_lifetime_and_principal() {
        assert!(matches!(
            prepare_capability_lease_issue(
                "alice",
                None,
                &["session.read".to_owned()],
                &["session:1".to_owned()],
                &[],
                None,
                None,
            ),
            Err(CapabilityLeaseMutationError::InvalidPrincipal)
        ));
        assert!(matches!(
            prepare_capability_lease_issue(
                "client:9:alice",
                None,
                &["session.read".to_owned()],
                &["session:1".to_owned()],
                &[],
                Some("2026-08-28T00:00:00Z"),
                Some("2026-08-27T00:00:00Z"),
            ),
            Err(CapabilityLeaseMutationError::InvalidTime)
        ));
    }
}
