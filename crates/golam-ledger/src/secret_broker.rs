#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId, SessionId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::approval_binding::{APPROVAL_ISSUE_ACTION, prepare_approval};
use crate::approvals::{ApprovalClass, ApprovalScope};
use crate::authority_security_write::{
    append_approval_consumption_snapshot, append_secret_use_record_snapshot,
};
use crate::storage::{AuthorityStore, StorageError};

const BROKER_ACTION: &str = "secret.use";
const BROKER_RISK_CLASS: &str = "secret_broker_use";
const USE_ID_DOMAIN: &[u8] = b"golam:secret-broker-use-id:v1";
const BROKER_RESOURCE_DOMAIN: &[u8] = b"golam:secret-broker-resource:v1";
const APPROVAL_BINDING_DOMAIN: &[u8] = b"golam:approval-binding:v1";
const MAX_PRINCIPAL_BYTES: usize = 512;
const MAX_PURPOSE_BYTES: usize = 512;
const MAX_DESTINATION_BYTES: usize = 2_048;
const MAX_SCOPE_ITEMS: usize = 32;
const MAX_PARENT_CHAIN_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BrokerLocality {
    StrictLocal,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BrokerSecretUseRequest<'a> {
    pub handle_id: [u8; 16],
    pub principal: &'a str,
    pub purpose: &'a str,
    pub destination_or_process: &'a str,
    pub locality: BrokerLocality,
    pub observed_at: &'a str,
    pub decision_id: [u8; 16],
    pub approval_effect_id: Option<EffectId>,
    pub approval_session_id: Option<SessionId>,
    pub taint_digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BrokerSecretUsePermit {
    use_id: [u8; 16],
    handle_id: [u8; 16],
    secret_id: [u8; 16],
    version: u64,
    lease_id: [u8; 16],
    lease_generation: u64,
    decision_id: [u8; 16],
    approval_id: Option<[u8; 16]>,
}

impl BrokerSecretUsePermit {
    pub(crate) const fn use_id(self) -> [u8; 16] {
        self.use_id
    }

    pub(crate) const fn handle_id(self) -> [u8; 16] {
        self.handle_id
    }

    pub(crate) const fn secret_id(self) -> [u8; 16] {
        self.secret_id
    }

    pub(crate) const fn version(self) -> u64 {
        self.version
    }

    pub(crate) const fn lease_id(self) -> [u8; 16] {
        self.lease_id
    }

    pub(crate) const fn lease_generation(self) -> u64 {
        self.lease_generation
    }

    pub(crate) const fn decision_id(self) -> [u8; 16] {
        self.decision_id
    }

    pub(crate) const fn approval_id(self) -> Option<[u8; 16]> {
        self.approval_id
    }
}

#[derive(Debug)]
pub(crate) enum SecretBrokerError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidRequest(&'static str),
    ExternalDestinationDenied,
    HandleNotFound,
    HandlePurposeMismatch,
    HandleExpired,
    SecretNotFound,
    SecretRevoked,
    SecretInactive,
    StaleHandleVersion,
    SecretVersionNotFound,
    SecretVersionRetired,
    DecisionNotFound,
    DecisionMismatch,
    StaleDecision,
    LeaseNotFound,
    LeaseMismatch,
    LeaseInactive,
    LeaseRevoked,
    LeaseNotYetValid,
    LeaseExpired,
    LeaseScopeMismatch,
    LeaseParentCycle,
    LeaseParentTooDeep,
    ActivePolicyMissing,
    PolicyMismatch,
    PolicyBundleInvalid,
    ApprovalNotFound,
    ApprovalMismatch,
    ApprovalNotYetValid,
    ApprovalExpired,
    ApprovalRevoked,
    ApprovalScopeMismatch,
    ApprovalRiskMismatch,
    ApprovalTaintMismatch,
    ApprovalUsageLimitReached,
    DuplicateUse,
    InvalidStoredRecord(&'static str),
    IntegerOverflow,
}

impl fmt::Display for SecretBrokerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "secret broker authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "secret broker sqlite error: {error}"),
            Self::Core(error) => write!(f, "secret broker canonical encoding error: {error}"),
            Self::Integrity(error) => write!(f, "secret broker integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "secret broker authority-security error: {error}")
            }
            Self::InvalidRequest(reason) => write!(f, "secret broker request is invalid: {reason}"),
            Self::ExternalDestinationDenied => {
                f.write_str("secret broker strict-local boundary denies external destination")
            }
            Self::HandleNotFound => f.write_str("secret broker handle does not exist"),
            Self::HandlePurposeMismatch => {
                f.write_str("secret broker handle does not authorize the exact purpose")
            }
            Self::HandleExpired => f.write_str("secret broker handle is expired"),
            Self::SecretNotFound => f.write_str("secret broker secret does not exist"),
            Self::SecretRevoked => f.write_str("secret broker secret is revoked"),
            Self::SecretInactive => f.write_str("secret broker secret is not active"),
            Self::StaleHandleVersion => {
                f.write_str("secret broker handle version constraint is stale")
            }
            Self::SecretVersionNotFound => {
                f.write_str("secret broker selected secret version does not exist")
            }
            Self::SecretVersionRetired => {
                f.write_str("secret broker selected secret version is retired")
            }
            Self::DecisionNotFound => {
                f.write_str("secret broker authorization decision is missing")
            }
            Self::DecisionMismatch => {
                f.write_str("secret broker authorization decision does not match exact use")
            }
            Self::StaleDecision => {
                f.write_str("secret broker authorization decision is not current")
            }
            Self::LeaseNotFound => f.write_str("secret broker lease does not exist"),
            Self::LeaseMismatch => f.write_str("secret broker lease binding is mismatched"),
            Self::LeaseInactive => f.write_str("secret broker lease is not active"),
            Self::LeaseRevoked => f.write_str("secret broker lease or ancestor is revoked"),
            Self::LeaseNotYetValid => f.write_str("secret broker lease is not yet valid"),
            Self::LeaseExpired => f.write_str("secret broker lease is expired"),
            Self::LeaseScopeMismatch => {
                f.write_str("secret broker lease does not cover exact action/resource")
            }
            Self::LeaseParentCycle => {
                f.write_str("secret broker lease parent chain contains a cycle")
            }
            Self::LeaseParentTooDeep => {
                f.write_str("secret broker lease parent chain exceeds bounded depth")
            }
            Self::ActivePolicyMissing => f.write_str("secret broker active policy is missing"),
            Self::PolicyMismatch => {
                f.write_str("secret broker decision policy evidence is not the active policy")
            }
            Self::PolicyBundleInvalid => {
                f.write_str("secret broker active policy bundle is missing or not validated")
            }
            Self::ApprovalNotFound => f.write_str("secret broker approval does not exist"),
            Self::ApprovalMismatch => f.write_str("secret broker approval binding is mismatched"),
            Self::ApprovalNotYetValid => f.write_str("secret broker approval is not yet valid"),
            Self::ApprovalExpired => f.write_str("secret broker approval is expired"),
            Self::ApprovalRevoked => f.write_str("secret broker approval is revoked"),
            Self::ApprovalScopeMismatch => {
                f.write_str("secret broker approval does not cover exact use")
            }
            Self::ApprovalRiskMismatch => {
                f.write_str("secret broker approval risk class is mismatched")
            }
            Self::ApprovalTaintMismatch => {
                f.write_str("secret broker approval taint binding is mismatched")
            }
            Self::ApprovalUsageLimitReached => {
                f.write_str("secret broker approval usage limit is exhausted")
            }
            Self::DuplicateUse => f.write_str("secret broker use has already been recorded"),
            Self::InvalidStoredRecord(reason) => {
                write!(f, "secret broker stored record is invalid: {reason}")
            }
            Self::IntegerOverflow => f.write_str("secret broker integer conversion overflow"),
        }
    }
}

impl Error for SecretBrokerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for SecretBrokerError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for SecretBrokerError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for SecretBrokerError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub(crate) struct SecretBrokerStore {
    connection: Connection,
}

impl SecretBrokerStore {
    pub(crate) fn open(layout: &AuthorityLayout) -> Result<Self, SecretBrokerError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub(crate) fn authorize_brokered_use(
        &mut self,
        request: BrokerSecretUseRequest<'_>,
    ) -> Result<BrokerSecretUsePermit, SecretBrokerError> {
        validate_request(&request)?;
        let resource = broker_resource(
            request.handle_id,
            request.purpose,
            request.destination_or_process,
            request.locality,
        )?;

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;

        let handle = load_handle(&transaction, request.handle_id)?;
        if handle.purpose_scope.as_slice() != request.purpose.as_bytes() {
            return Err(SecretBrokerError::HandlePurposeMismatch);
        }
        if let Some(expires_at) = handle.expires_at.as_deref() {
            require_stored_time(expires_at, "handle expiry is malformed")?;
            if request.observed_at >= expires_at {
                return Err(SecretBrokerError::HandleExpired);
            }
        }

        let secret = load_secret(&transaction, handle.secret_id)?;
        if secret.status == "revoked" || secret.revoked_at.is_some() {
            return Err(SecretBrokerError::SecretRevoked);
        }
        if secret.status != "active" {
            return Err(SecretBrokerError::SecretInactive);
        }
        let version = match handle.version_constraint {
            Some(version) if version != secret.current_version => {
                return Err(SecretBrokerError::StaleHandleVersion);
            }
            Some(version) => version,
            None => secret.current_version,
        };
        verify_version_active(&transaction, handle.secret_id, version)?;

        let decision = load_current_decision(&transaction, request.decision_id, &resource)?;
        if decision.principal != request.principal {
            return Err(SecretBrokerError::DecisionMismatch);
        }
        verify_active_policy(&transaction, &decision)?;
        verify_lease_chain(
            &transaction,
            decision.lease_id,
            decision.lease_generation,
            request.principal,
            &resource,
            request.observed_at,
        )?;

        let approval_id =
            verify_optional_approval(&transaction, decision.approval_id, &request, &resource)?;
        let use_id = derive_use_id(
            request.handle_id,
            version,
            request.decision_id,
            request.principal,
            request.purpose,
            request.destination_or_process,
        )?;
        let duplicate = transaction
            .query_row(
                "SELECT 1 FROM secret_use_records WHERE use_id = ?1 LIMIT 1",
                params![&use_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if duplicate {
            return Err(SecretBrokerError::DuplicateUse);
        }

        transaction.execute(
            "INSERT INTO secret_use_records (use_id, handle_id, principal, purpose, destination_or_process, mode, approval_id, decision_id, created_global_seq) VALUES (?1, ?2, ?3, ?4, ?5, 'brokered', ?6, ?7, ?8)",
            params![
                &use_id[..],
                &request.handle_id[..],
                request.principal,
                request.purpose,
                request.destination_or_process,
                approval_id.map(|value| value.to_vec()),
                &request.decision_id[..],
                to_i64(decision.global_seq)?,
            ],
        )?;
        append_secret_use_record_snapshot(&transaction, &use_id)
            .map_err(|error| SecretBrokerError::AuthoritySecurity(error.to_string()))?;
        if let Some(approval_id) = approval_id {
            consume_approval(&transaction, approval_id, use_id, decision.global_seq)?;
        }
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| SecretBrokerError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(BrokerSecretUsePermit {
            use_id,
            handle_id: request.handle_id,
            secret_id: handle.secret_id,
            version,
            lease_id: decision.lease_id,
            lease_generation: decision.lease_generation,
            decision_id: request.decision_id,
            approval_id,
        })
    }
}

struct StoredHandle {
    secret_id: [u8; 16],
    version_constraint: Option<u64>,
    purpose_scope: Vec<u8>,
    expires_at: Option<String>,
}

struct StoredSecret {
    current_version: u64,
    status: String,
    revoked_at: Option<String>,
}

struct DecisionEvidence {
    principal: String,
    lease_id: [u8; 16],
    lease_generation: u64,
    policy_bundle_id: [u8; 16],
    policy_bundle_hash: [u8; 32],
    approval_id: Option<[u8; 16]>,
    global_seq: u64,
}

fn verify_transaction_integrity(transaction: &Transaction<'_>) -> Result<(), SecretBrokerError> {
    crate::integrity::verify(transaction)
        .map_err(|error| SecretBrokerError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| SecretBrokerError::AuthoritySecurity(error.to_string()))
}

fn load_handle(
    transaction: &Transaction<'_>,
    handle_id: [u8; 16],
) -> Result<StoredHandle, SecretBrokerError> {
    let row = transaction
        .query_row(
            "SELECT secret_id, version_constraint, purpose_scope, expires_at FROM secret_handles WHERE handle_id = ?1",
            params![&handle_id[..]],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretBrokerError::HandleNotFound)?;
    if row.2.is_empty() || row.2.len() > MAX_PURPOSE_BYTES {
        return Err(SecretBrokerError::InvalidStoredRecord(
            "handle purpose scope is invalid",
        ));
    }
    Ok(StoredHandle {
        secret_id: id16(row.0, "handle secret id is not 16 bytes")?,
        version_constraint: row
            .1
            .map(|value| positive_u64(value, "handle version constraint is invalid"))
            .transpose()?,
        purpose_scope: row.2,
        expires_at: row.3,
    })
}

fn load_secret(
    transaction: &Transaction<'_>,
    secret_id: [u8; 16],
) -> Result<StoredSecret, SecretBrokerError> {
    let row = transaction
        .query_row(
            "SELECT current_version, status, revoked_at FROM secret_records WHERE secret_id = ?1",
            params![&secret_id[..]],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretBrokerError::SecretNotFound)?;
    Ok(StoredSecret {
        current_version: positive_u64(row.0, "secret current version is invalid")?,
        status: row.1,
        revoked_at: row.2,
    })
}

fn verify_version_active(
    transaction: &Transaction<'_>,
    secret_id: [u8; 16],
    version: u64,
) -> Result<(), SecretBrokerError> {
    let retired_at = transaction
        .query_row(
            "SELECT retired_at FROM secret_versions WHERE secret_id = ?1 AND version = ?2",
            params![&secret_id[..], to_i64(version)?],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .ok_or(SecretBrokerError::SecretVersionNotFound)?;
    if retired_at.is_some() {
        return Err(SecretBrokerError::SecretVersionRetired);
    }
    Ok(())
}

fn load_current_decision(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    expected_resource: &str,
) -> Result<DecisionEvidence, SecretBrokerError> {
    let row = transaction
        .query_row(
            "SELECT principal, action, resource, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, approval_id, decision, global_seq, authority_evidence_version FROM authorization_decisions WHERE decision_id = ?1",
            params![&decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                    row.get::<_, Option<Vec<u8>>>(7)?,
                    row.get::<_, Option<Vec<u8>>>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, i64>(11)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretBrokerError::DecisionNotFound)?;
    if row.1 != BROKER_ACTION
        || row.2 != expected_resource
        || row.3 != "pass"
        || row.9 != "allow"
        || row.11 < 2
    {
        return Err(SecretBrokerError::DecisionMismatch);
    }
    let global_seq = nonnegative_u64(row.10, "decision global sequence is invalid")?;
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (SELECT global_seq FROM session_events UNION ALL SELECT global_seq FROM effect_transitions UNION ALL SELECT global_seq FROM authorization_decisions)",
        [],
        |row| row.get(0),
    )?;
    if global_seq != nonnegative_u64(latest, "latest global sequence is invalid")? {
        return Err(SecretBrokerError::StaleDecision);
    }
    Ok(DecisionEvidence {
        principal: row.0,
        lease_id: id16(
            row.4.ok_or(SecretBrokerError::DecisionMismatch)?,
            "decision lease id is invalid",
        )?,
        lease_generation: positive_u64(
            row.5.ok_or(SecretBrokerError::DecisionMismatch)?,
            "decision lease generation is invalid",
        )?,
        policy_bundle_id: id16(
            row.6.ok_or(SecretBrokerError::DecisionMismatch)?,
            "decision policy bundle id is invalid",
        )?,
        policy_bundle_hash: hash32(
            row.7.ok_or(SecretBrokerError::DecisionMismatch)?,
            "decision policy bundle hash is invalid",
        )?,
        approval_id: row
            .8
            .map(|value| id16(value, "decision approval id is invalid"))
            .transpose()?,
        global_seq,
    })
}

fn verify_active_policy(
    transaction: &Transaction<'_>,
    decision: &DecisionEvidence,
) -> Result<(), SecretBrokerError> {
    let row = transaction
        .query_row(
            "SELECT policy_bundle_id, bundle_hash FROM active_policy WHERE singleton_id = 1",
            [],
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?
        .ok_or(SecretBrokerError::ActivePolicyMissing)?;
    let active_id = id16(row.0, "active policy id is invalid")?;
    let active_hash = hash32(row.1, "active policy hash is invalid")?;
    if active_id != decision.policy_bundle_id || active_hash != decision.policy_bundle_hash {
        return Err(SecretBrokerError::PolicyMismatch);
    }
    let status = transaction
        .query_row(
            "SELECT validation_status FROM policy_bundles WHERE policy_bundle_id = ?1 AND bundle_hash = ?2",
            params![&active_id[..], &active_hash[..]],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or(SecretBrokerError::PolicyBundleInvalid)?;
    if status != "validated" {
        return Err(SecretBrokerError::PolicyBundleInvalid);
    }
    Ok(())
}

fn verify_lease_chain(
    transaction: &Transaction<'_>,
    lease_id: [u8; 16],
    expected_generation: u64,
    principal: &str,
    resource: &str,
    observed_at: &str,
) -> Result<(), SecretBrokerError> {
    let mut next = Some(lease_id);
    let mut seen = HashSet::new();
    let mut depth = 0_usize;
    while let Some(current_id) = next {
        if depth >= MAX_PARENT_CHAIN_DEPTH {
            return Err(SecretBrokerError::LeaseParentTooDeep);
        }
        if !seen.insert(current_id) {
            return Err(SecretBrokerError::LeaseParentCycle);
        }
        let row = transaction
            .query_row(
                "SELECT principal_id, parent_lease_id, actions_scope, resources_scope, not_before, expires_at, generation, status, EXISTS(SELECT 1 FROM capability_revocations r WHERE r.lease_id = l.lease_id) FROM capability_leases l WHERE l.lease_id = ?1",
                params![&current_id[..]],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<Vec<u8>>>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, i64>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, i64>(8)?,
                    ))
                },
            )
            .optional()?
            .ok_or(SecretBrokerError::LeaseNotFound)?;
        if row.0 != principal {
            return Err(SecretBrokerError::LeaseMismatch);
        }
        let generation = positive_u64(row.6, "lease generation is invalid")?;
        if depth == 0 && generation != expected_generation {
            return Err(SecretBrokerError::LeaseMismatch);
        }
        if row.7 != "active" {
            return Err(SecretBrokerError::LeaseInactive);
        }
        if row.8 != 0 {
            return Err(SecretBrokerError::LeaseRevoked);
        }
        if let Some(not_before) = row.4.as_deref() {
            require_stored_time(not_before, "lease not_before is malformed")?;
            if observed_at < not_before {
                return Err(SecretBrokerError::LeaseNotYetValid);
            }
        }
        if let Some(expires_at) = row.5.as_deref() {
            require_stored_time(expires_at, "lease expiry is malformed")?;
            if observed_at >= expires_at {
                return Err(SecretBrokerError::LeaseExpired);
            }
        }
        if !scope_contains(&row.2, BROKER_ACTION)? || !scope_contains(&row.3, resource)? {
            return Err(SecretBrokerError::LeaseScopeMismatch);
        }
        next = row
            .1
            .map(|value| id16(value, "lease parent id is invalid"))
            .transpose()?;
        depth += 1;
    }
    Ok(())
}

fn verify_optional_approval(
    transaction: &Transaction<'_>,
    approval_id: Option<[u8; 16]>,
    request: &BrokerSecretUseRequest<'_>,
    resource: &str,
) -> Result<Option<[u8; 16]>, SecretBrokerError> {
    let Some(approval_id) = approval_id else {
        return Ok(None);
    };
    let row = transaction
        .query_row(
            "SELECT class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at FROM approvals WHERE approval_id = ?1",
            params![&approval_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, Option<Vec<u8>>>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Vec<u8>>(8)?,
                    row.get::<_, Vec<u8>>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<i64>>(12)?,
                    row.get::<_, Option<String>>(13)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretBrokerError::ApprovalNotFound)?;
    let class = ApprovalClass::try_from(row.0.as_str())
        .map_err(|_| SecretBrokerError::InvalidStoredRecord("approval class is invalid"))?;
    if row.7 != BROKER_RISK_CLASS {
        return Err(SecretBrokerError::ApprovalRiskMismatch);
    }
    let taint_digest = hash32(row.8, "approval taint digest is invalid")?;
    if taint_digest != request.taint_digest {
        return Err(SecretBrokerError::ApprovalTaintMismatch);
    }
    require_stored_time(&row.10, "approval issued_at is malformed")?;
    if request.observed_at < row.10.as_str() {
        return Err(SecretBrokerError::ApprovalNotYetValid);
    }
    if let Some(expires_at) = row.11.as_deref() {
        require_stored_time(expires_at, "approval expiry is malformed")?;
        if request.observed_at >= expires_at {
            return Err(SecretBrokerError::ApprovalExpired);
        }
    }
    if row.13.is_some() {
        return Err(SecretBrokerError::ApprovalRevoked);
    }

    let scope = decode_approval_scope(class, row.3, row.4, row.5, row.6)?;
    if !approval_scope_matches(&scope, request, resource)? {
        return Err(SecretBrokerError::ApprovalScopeMismatch);
    }
    let max_uses = positive_u64(
        row.12.ok_or(SecretBrokerError::ApprovalMismatch)?,
        "approval max uses is invalid",
    )?;
    let uses: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM approval_consumptions WHERE approval_id = ?1 AND state IN ('reserved', 'consumed')",
        params![&approval_id[..]],
        |row| row.get(0),
    )?;
    if nonnegative_u64(uses, "approval use count is invalid")? >= max_uses {
        return Err(SecretBrokerError::ApprovalUsageLimitReached);
    }

    let scope_digest = hash32(row.2, "approval scope digest is invalid")?;
    let parent_decision_id = id16(row.9, "approval parent decision id is invalid")?;
    let prepared = prepare_approval(
        &row.1,
        scope,
        &row.7,
        taint_digest,
        &row.10,
        row.11.as_deref(),
        max_uses,
    )
    .map_err(|_| SecretBrokerError::ApprovalMismatch)?;
    let parent = transaction
        .query_row(
            "SELECT principal, action, resource, context_hash, decision FROM authorization_decisions WHERE decision_id = ?1",
            params![&parent_decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretBrokerError::ApprovalMismatch)?;
    if parent.0 != row.1
        || parent.1 != APPROVAL_ISSUE_ACTION
        || parent.2 != prepared.resource()
        || parent.4 != "allow"
    {
        return Err(SecretBrokerError::ApprovalMismatch);
    }
    let context_hash = hash32(parent.3, "approval parent context hash is invalid")?;
    let rebound = bound_scope_digest(prepared.intent_digest(), parent_decision_id, context_hash)?;
    if rebound != scope_digest {
        return Err(SecretBrokerError::ApprovalMismatch);
    }
    Ok(Some(approval_id))
}

fn decode_approval_scope(
    class: ApprovalClass,
    action_scope: Vec<u8>,
    resource_scope: Vec<u8>,
    effect_id: Option<Vec<u8>>,
    session_id: Option<Vec<u8>>,
) -> Result<ApprovalScope, SecretBrokerError> {
    let actions = || decode_set(&action_scope, "approval action scope is invalid");
    let resources = || decode_set(&resource_scope, "approval resource scope is invalid");
    match class {
        ApprovalClass::Once => {
            let effect_id = effect_id
                .ok_or(SecretBrokerError::ApprovalMismatch)
                .and_then(|value| id16(value, "approval effect id is invalid"))?;
            if session_id.is_some() {
                return Err(SecretBrokerError::ApprovalMismatch);
            }
            Ok(ApprovalScope::once(
                EffectId(u128::from_be_bytes(effect_id)),
                utf8(&action_scope, "approval action is invalid")?,
                utf8(&resource_scope, "approval resource is invalid")?,
            )
            .map_err(|_| SecretBrokerError::ApprovalMismatch)?)
        }
        ApprovalClass::SessionScoped => {
            if effect_id.is_some() {
                return Err(SecretBrokerError::ApprovalMismatch);
            }
            let session_id = session_id
                .ok_or(SecretBrokerError::ApprovalMismatch)
                .and_then(|value| id16(value, "approval session id is invalid"))?;
            Ok(ApprovalScope::session_scoped(
                SessionId(u128::from_be_bytes(session_id)),
                &actions()?,
                &resources()?,
            )
            .map_err(|_| SecretBrokerError::ApprovalMismatch)?)
        }
        ApprovalClass::TimeBoxed => {
            if effect_id.is_some() || session_id.is_some() {
                return Err(SecretBrokerError::ApprovalMismatch);
            }
            Ok(ApprovalScope::time_boxed(&actions()?, &resources()?)
                .map_err(|_| SecretBrokerError::ApprovalMismatch)?)
        }
        ApprovalClass::OperationPattern => {
            if effect_id.is_some() || session_id.is_some() {
                return Err(SecretBrokerError::ApprovalMismatch);
            }
            Ok(ApprovalScope::operation_pattern(
                utf8(&action_scope, "approval action pattern is invalid")?,
                utf8(&resource_scope, "approval resource pattern is invalid")?,
            )
            .map_err(|_| SecretBrokerError::ApprovalMismatch)?)
        }
        ApprovalClass::RunPreauthorization => {
            if effect_id.is_some() {
                return Err(SecretBrokerError::ApprovalMismatch);
            }
            let session_id = session_id
                .map(|value| id16(value, "approval session id is invalid"))
                .transpose()?
                .map(|value| SessionId(u128::from_be_bytes(value)));
            Ok(
                ApprovalScope::run_preauthorization(session_id, &actions()?, &resources()?)
                    .map_err(|_| SecretBrokerError::ApprovalMismatch)?,
            )
        }
    }
}

fn approval_scope_matches(
    scope: &ApprovalScope,
    request: &BrokerSecretUseRequest<'_>,
    resource: &str,
) -> Result<bool, SecretBrokerError> {
    Ok(match scope {
        ApprovalScope::Once {
            effect_id,
            action,
            resource: expected_resource,
        } => {
            request.approval_effect_id == Some(*effect_id)
                && action == BROKER_ACTION
                && expected_resource == resource
        }
        ApprovalScope::SessionScoped {
            session_id,
            actions,
            resources,
        } => {
            request.approval_session_id == Some(*session_id)
                && contains(actions, BROKER_ACTION)
                && contains(resources, resource)
        }
        ApprovalScope::TimeBoxed { actions, resources } => {
            contains(actions, BROKER_ACTION) && contains(resources, resource)
        }
        ApprovalScope::OperationPattern {
            action_pattern,
            resource_pattern,
        } => {
            bounded_pattern_matches(action_pattern, BROKER_ACTION)?
                && bounded_pattern_matches(resource_pattern, resource)?
        }
        ApprovalScope::RunPreauthorization {
            session_id,
            actions,
            resources,
        } => {
            session_id.is_none_or(|expected| request.approval_session_id == Some(expected))
                && contains(actions, BROKER_ACTION)
                && contains(resources, resource)
        }
    })
}

fn consume_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    use_id: [u8; 16],
    global_seq: u64,
) -> Result<(), SecretBrokerError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"golam:secret-broker-approval-consumption:v1");
    hasher.update(&approval_id);
    hasher.update(&use_id);
    let digest = hasher.finalize();
    let mut consumption_id = [0_u8; 16];
    consumption_id.copy_from_slice(&digest.as_bytes()[..16]);
    transaction.execute(
        "INSERT INTO approval_consumptions (consumption_id, approval_id, effect_or_operation_id, reserved_global_seq, consumed_global_seq, state) VALUES (?1, ?2, ?3, ?4, ?5, 'consumed')",
        params![
            &consumption_id[..],
            &approval_id[..],
            &use_id[..],
            to_i64(global_seq)?,
            to_i64(global_seq)?,
        ],
    )?;
    append_approval_consumption_snapshot(transaction, &consumption_id)
        .map_err(|error| SecretBrokerError::AuthoritySecurity(error.to_string()))
}

fn validate_request(request: &BrokerSecretUseRequest<'_>) -> Result<(), SecretBrokerError> {
    validate_text(
        request.principal,
        MAX_PRINCIPAL_BYTES,
        "principal is invalid",
    )?;
    validate_text(request.purpose, MAX_PURPOSE_BYTES, "purpose is invalid")?;
    validate_text(
        request.destination_or_process,
        MAX_DESTINATION_BYTES,
        "destination/process is invalid",
    )?;
    if !valid_utc_second(request.observed_at) {
        return Err(SecretBrokerError::InvalidRequest(
            "observed_at must be canonical UTC-second time",
        ));
    }
    match request.locality {
        BrokerLocality::StrictLocal => {
            if !(request.destination_or_process.starts_with("process:")
                || request.destination_or_process.starts_with("service:"))
            {
                return Err(SecretBrokerError::ExternalDestinationDenied);
            }
        }
    }
    Ok(())
}

fn broker_resource(
    handle_id: [u8; 16],
    purpose: &str,
    destination: &str,
    locality: BrokerLocality,
) -> Result<String, SecretBrokerError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(BROKER_RESOURCE_DOMAIN)?;
    encoder.push_bytes(&handle_id)?;
    encoder.push_bytes(purpose.as_bytes())?;
    encoder.push_bytes(destination.as_bytes())?;
    encoder.push_u8(match locality {
        BrokerLocality::StrictLocal => 1,
    });
    let digest = blake3::hash(&encoder.finish());
    Ok(format!(
        "secret-broker:{}:{}",
        hex_bytes(&handle_id),
        hex_bytes(&digest.as_bytes()[..16])
    ))
}

fn derive_use_id(
    handle_id: [u8; 16],
    version: u64,
    decision_id: [u8; 16],
    principal: &str,
    purpose: &str,
    destination: &str,
) -> Result<[u8; 16], SecretBrokerError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(USE_ID_DOMAIN)?;
    encoder.push_bytes(&handle_id)?;
    encoder.push_u64(version);
    encoder.push_bytes(&decision_id)?;
    encoder.push_bytes(principal.as_bytes())?;
    encoder.push_bytes(purpose.as_bytes())?;
    encoder.push_bytes(destination.as_bytes())?;
    let digest = blake3::hash(&encoder.finish());
    let mut use_id = [0_u8; 16];
    use_id.copy_from_slice(&digest.as_bytes()[..16]);
    Ok(use_id)
}

fn bound_scope_digest(
    intent_digest: [u8; 32],
    parent_decision_id: [u8; 16],
    context_hash: [u8; 32],
) -> Result<[u8; 32], SecretBrokerError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(APPROVAL_BINDING_DOMAIN)?;
    encoder.push_bytes(&intent_digest)?;
    encoder.push_bytes(&parent_decision_id)?;
    encoder.push_bytes(&context_hash)?;
    Ok(*blake3::hash(&encoder.finish()).as_bytes())
}

fn scope_contains(bytes: &[u8], value: &str) -> Result<bool, SecretBrokerError> {
    if bytes.is_empty() {
        return Ok(false);
    }
    let text = utf8(bytes, "lease scope is not UTF-8")?;
    let entries = text.split('\n').collect::<Vec<_>>();
    if entries.len() > MAX_SCOPE_ITEMS || entries.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(SecretBrokerError::InvalidStoredRecord(
            "lease scope is not bounded sorted unique data",
        ));
    }
    Ok(entries.binary_search(&value).is_ok())
}

fn decode_set(bytes: &[u8], reason: &'static str) -> Result<Vec<String>, SecretBrokerError> {
    let text = utf8(bytes, reason)?;
    if text.is_empty() {
        return Err(SecretBrokerError::InvalidStoredRecord(reason));
    }
    let values = text.split('\n').map(str::to_owned).collect::<Vec<_>>();
    if values.len() > MAX_SCOPE_ITEMS || values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(SecretBrokerError::InvalidStoredRecord(reason));
    }
    Ok(values)
}

fn contains(values: &[String], value: &str) -> bool {
    values
        .binary_search_by(|candidate| candidate.as_str().cmp(value))
        .is_ok()
}

fn bounded_pattern_matches(pattern: &str, value: &str) -> Result<bool, SecretBrokerError> {
    let Some(wildcard) = pattern.find('*') else {
        return Ok(pattern == value);
    };
    if pattern[wildcard + 1..].contains('*') {
        return Err(SecretBrokerError::InvalidStoredRecord(
            "approval pattern contains multiple wildcards",
        ));
    }
    let prefix = &pattern[..wildcard];
    let suffix = &pattern[wildcard + 1..];
    Ok(value.len() >= prefix.len() + suffix.len()
        && value.starts_with(prefix)
        && value.ends_with(suffix))
}

fn require_stored_time(value: &str, reason: &'static str) -> Result<(), SecretBrokerError> {
    if valid_utc_second(value) {
        Ok(())
    } else {
        Err(SecretBrokerError::InvalidStoredRecord(reason))
    }
}

fn validate_text(
    value: &str,
    max_bytes: usize,
    reason: &'static str,
) -> Result<(), SecretBrokerError> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        Err(SecretBrokerError::InvalidRequest(reason))
    } else {
        Ok(())
    }
}

fn utf8<'a>(bytes: &'a [u8], reason: &'static str) -> Result<&'a str, SecretBrokerError> {
    std::str::from_utf8(bytes).map_err(|_| SecretBrokerError::InvalidStoredRecord(reason))
}

fn id16(value: Vec<u8>, reason: &'static str) -> Result<[u8; 16], SecretBrokerError> {
    value
        .try_into()
        .map_err(|_| SecretBrokerError::InvalidStoredRecord(reason))
}

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], SecretBrokerError> {
    value
        .try_into()
        .map_err(|_| SecretBrokerError::InvalidStoredRecord(reason))
}

fn positive_u64(value: i64, reason: &'static str) -> Result<u64, SecretBrokerError> {
    let value = nonnegative_u64(value, reason)?;
    if value == 0 {
        return Err(SecretBrokerError::InvalidStoredRecord(reason));
    }
    Ok(value)
}

fn nonnegative_u64(value: i64, reason: &'static str) -> Result<u64, SecretBrokerError> {
    u64::try_from(value).map_err(|_| SecretBrokerError::InvalidStoredRecord(reason))
}

fn to_i64(value: u64) -> Result<i64, SecretBrokerError> {
    i64::try_from(value).map_err(|_| SecretBrokerError::IntegerOverflow)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_security_write::{
        append_active_policy_snapshot, append_authorization_decision_v2_snapshot,
        append_capability_lease_snapshot, append_policy_bundle_snapshot,
        append_secret_handle_snapshot, append_secret_record_snapshot,
        append_secret_version_snapshot,
    };
    use crate::security_audit::{AuthorizationAuditInput, append_authorization_decision};
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-secret-broker-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    struct Fixture {
        handle_id: [u8; 16],
        secret_id: [u8; 16],
        lease_id: [u8; 16],
        decision_id: [u8; 16],
        resource: String,
    }

    fn seed_fixture(connection: &mut Connection) -> Fixture {
        let handle_id = [3_u8; 16];
        let secret_id = [4_u8; 16];
        let lease_id = [5_u8; 16];
        let decision_id = [6_u8; 16];
        let policy_id = [7_u8; 16];
        let policy_hash = [8_u8; 32];
        let resource = broker_resource(
            handle_id,
            "git.auth",
            "process:git",
            BrokerLocality::StrictLocal,
        )
        .unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "INSERT INTO policy_bundles (policy_bundle_id, version, schema_version, canonical_policy_bytes, bundle_hash, created_by, created_global_seq, validation_status) VALUES (?1, 1, 1, X'01', ?2, 'owner:owner', 0, 'validated')",
                params![&policy_id[..], &policy_hash[..]],
            )
            .unwrap();
        append_policy_bundle_snapshot(&transaction, &policy_id).unwrap();
        transaction
            .execute(
                "INSERT INTO active_policy (singleton_id, policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq) VALUES (1, ?1, ?2, 'owner:owner', ?3, 0)",
                params![&policy_id[..], &[9_u8; 16][..], &policy_hash[..]],
            )
            .unwrap();
        // Correct argument order after intentionally using explicit named values.
        transaction
            .execute("DELETE FROM active_policy", [])
            .unwrap();
        transaction
            .execute(
                "INSERT INTO active_policy (singleton_id, policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq) VALUES (1, ?1, ?2, 'owner:owner', ?3, 0)",
                params![&policy_id[..], &policy_hash[..], &[9_u8; 16][..]],
            )
            .unwrap();
        append_active_policy_snapshot(&transaction).unwrap();
        transaction
            .execute(
                "INSERT INTO capability_leases (lease_id, principal_id, parent_lease_id, actions_scope, resources_scope, context_constraints, issued_by, issued_global_seq, not_before, expires_at, generation, status, authority_digest) VALUES (?1, 'owner:owner', NULL, ?2, ?3, X'', 'owner:owner', 0, '2026-08-28T00:00:00Z', '2026-08-29T00:00:00Z', 1, 'active', ?4)",
                params![&lease_id[..], BROKER_ACTION.as_bytes(), resource.as_bytes(), &[10_u8; 32][..]],
            )
            .unwrap();
        append_capability_lease_snapshot(&transaction, &lease_id).unwrap();
        transaction
            .execute(
                "INSERT INTO secret_records (secret_id, classification, owner_principal, current_version, status, created_global_seq, revoked_at) VALUES (?1, 'api_credential', 'owner:owner', 1, 'active', 0, NULL)",
                params![&secret_id[..]],
            )
            .unwrap();
        append_secret_record_snapshot(&transaction, &secret_id).unwrap();
        transaction
            .execute(
                "INSERT INTO secret_versions (secret_id, version, ciphertext, nonce_or_algorithm_metadata, associated_data_hash, created_global_seq, rotated_from, retired_at) VALUES (?1, 1, X'0102', X'47535631000000000000000000000000', ?2, 0, NULL, NULL)",
                params![&secret_id[..], &[11_u8; 32][..]],
            )
            .unwrap();
        append_secret_version_snapshot(&transaction, &secret_id, 1).unwrap();
        transaction
            .execute(
                "INSERT INTO secret_handles (handle_id, secret_id, version_constraint, purpose_scope, expires_at) VALUES (?1, ?2, 1, ?3, '2026-08-29T00:00:00Z')",
                params![&handle_id[..], &secret_id[..], b"git.auth".as_slice()],
            )
            .unwrap();
        append_secret_handle_snapshot(&transaction, &handle_id).unwrap();
        transaction
            .execute(
                "INSERT INTO authorization_decisions (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, authority_evidence_version) VALUES (?1, 'owner:owner', ?2, ?3, ?4, 'allow', 'test_broker_allow', 1, 'pass', ?5, 1, ?6, ?7, X'', NULL, 2)",
                params![
                    &decision_id[..],
                    BROKER_ACTION,
                    &resource,
                    &[12_u8; 32][..],
                    &lease_id[..],
                    &policy_id[..],
                    &policy_hash[..],
                ],
            )
            .unwrap();
        append_authorization_decision(
            &transaction,
            AuthorizationAuditInput {
                decision_id: &decision_id,
                principal: "owner:owner",
                action: BROKER_ACTION,
                resource: &resource,
                context_hash: &[12_u8; 32],
                decision: "allow",
                reason_code: "test_broker_allow",
                global_seq: 1,
            },
        )
        .unwrap();
        append_authorization_decision_v2_snapshot(&transaction, &decision_id).unwrap();
        crate::integrity::verify(&transaction).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
        Fixture {
            handle_id,
            secret_id,
            lease_id,
            decision_id,
            resource,
        }
    }

    fn request(fixture: &Fixture) -> BrokerSecretUseRequest<'_> {
        BrokerSecretUseRequest {
            handle_id: fixture.handle_id,
            principal: "owner:owner",
            purpose: "git.auth",
            destination_or_process: "process:git",
            locality: BrokerLocality::StrictLocal,
            observed_at: "2026-08-28T12:00:00Z",
            decision_id: fixture.decision_id,
            approval_effect_id: None,
            approval_session_id: None,
            taint_digest: [0; 32],
        }
    }

    #[test]
    fn broker_authorization_records_metadata_without_plaintext_surface() {
        let (runtime, authority) = authority();
        let mut store = SecretBrokerStore::open(&authority).unwrap();
        let fixture = seed_fixture(&mut store.connection);
        let permit = store.authorize_brokered_use(request(&fixture)).unwrap();
        assert_eq!(permit.handle_id(), fixture.handle_id);
        assert_eq!(permit.secret_id(), fixture.secret_id);
        assert_eq!(permit.version(), 1);
        assert_eq!(permit.lease_id(), fixture.lease_id);
        assert_eq!(permit.lease_generation(), 1);
        assert_eq!(permit.decision_id(), fixture.decision_id);
        assert_eq!(permit.approval_id(), None);
        let row: (String, String, String, Option<Vec<u8>>) = store
            .connection
            .query_row(
                "SELECT purpose, destination_or_process, mode, approval_id FROM secret_use_records WHERE use_id = ?1",
                params![&permit.use_id()[..]],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(row.0, "git.auth");
        assert_eq!(row.1, "process:git");
        assert_eq!(row.2, "brokered");
        assert!(row.3.is_none());
        crate::authority_security_v2::verify(&store.connection).unwrap();
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn strict_local_rejects_external_destination_before_recording_use() {
        let (runtime, authority) = authority();
        let mut store = SecretBrokerStore::open(&authority).unwrap();
        let fixture = seed_fixture(&mut store.connection);
        let mut external = request(&fixture);
        external.destination_or_process = "https://example.invalid";
        assert!(matches!(
            store.authorize_brokered_use(external),
            Err(SecretBrokerError::ExternalDestinationDenied)
        ));
        let count: i64 = store
            .connection
            .query_row("SELECT COUNT(*) FROM secret_use_records", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn stale_handle_version_and_revoked_secret_fail_closed() {
        let (runtime, authority) = authority();
        let mut store = SecretBrokerStore::open(&authority).unwrap();
        let fixture = seed_fixture(&mut store.connection);
        let transaction = store
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "UPDATE secret_records SET current_version = 2 WHERE secret_id = ?1",
                params![&fixture.secret_id[..]],
            )
            .unwrap();
        append_secret_record_snapshot(&transaction, &fixture.secret_id).unwrap();
        transaction.commit().unwrap();
        assert!(matches!(
            store.authorize_brokered_use(request(&fixture)),
            Err(SecretBrokerError::StaleHandleVersion)
        ));

        let transaction = store
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "UPDATE secret_records SET current_version = 1, status = 'revoked', revoked_at = '2026-08-28T11:00:00Z' WHERE secret_id = ?1",
                params![&fixture.secret_id[..]],
            )
            .unwrap();
        append_secret_record_snapshot(&transaction, &fixture.secret_id).unwrap();
        transaction.commit().unwrap();
        assert!(matches!(
            store.authorize_brokered_use(request(&fixture)),
            Err(SecretBrokerError::SecretRevoked)
        ));
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn broker_resource_is_bound_to_destination_and_purpose() {
        let fixture_handle = [3_u8; 16];
        let first = broker_resource(
            fixture_handle,
            "git.auth",
            "process:git",
            BrokerLocality::StrictLocal,
        )
        .unwrap();
        let changed = broker_resource(
            fixture_handle,
            "registry.auth",
            "process:git",
            BrokerLocality::StrictLocal,
        )
        .unwrap();
        assert_ne!(first, changed);
    }
}
