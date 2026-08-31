#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::authority_security_write::{
    append_approval_consumption_snapshot, append_egress_permit_snapshot,
};
use crate::storage::{AuthorityStore, StorageError};

pub const EGRESS_PERMIT_ISSUE_ACTION: &str = "egress.permit.issue";
pub const EGRESS_PERMIT_REVOKE_ACTION: &str = "egress.permit.revoke";
pub const EGRESS_PERMIT_MUTATION_RISK_CLASS: &str = "egress_permit_mutation";

const ISSUE_INTENT_DOMAIN: &[u8] = b"golam:egress-permit-issue-intent:v1";
const ISSUE_ID_DOMAIN: &[u8] = b"golam:egress-permit-id:v1";
const REVOKE_INTENT_DOMAIN: &[u8] = b"golam:egress-permit-revoke-intent:v1";
const APPROVAL_CONSUMPTION_DOMAIN: &[u8] = b"golam:egress-permit-approval-consumption:v1";
const MAX_PRINCIPAL_BYTES: usize = 512;
const MAX_ACTION_BYTES: usize = 128;
const MAX_PURPOSE_BYTES: usize = 512;
const MAX_DESTINATION_BYTES: usize = 2_048;
const MAX_PROTOCOL_PORT_BYTES: usize = 128;
const MAX_REASON_CODE_BYTES: usize = 128;
const MAX_PARENT_CHAIN_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EgressParentLeaseBinding {
    lease_id: [u8; 16],
    generation: u64,
    authority_digest: [u8; 32],
}

impl EgressParentLeaseBinding {
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
pub struct PreparedEgressPermitIssue {
    principal_or_process: String,
    action: String,
    purpose: String,
    destination_scope: String,
    protocol_port_scope: String,
    taint_digest: [u8; 32],
    secret_handle_id: Option<[u8; 16]>,
    parent_lease: EgressParentLeaseBinding,
    issued_at: String,
    expires_at: Option<String>,
    usage_limit: Option<u64>,
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedEgressPermitIssue {
    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedEgressPermitRevocation {
    permit_id: [u8; 16],
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedEgressPermitRevocation {
    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EgressPermitRecord {
    pub permit_id: [u8; 16],
    pub principal_or_process: String,
    pub action: String,
    pub purpose: String,
    pub destination_scope: String,
    pub protocol_port_scope: String,
    pub taint_digest: [u8; 32],
    pub secret_handle_id: Option<[u8; 16]>,
    pub parent_lease_id: [u8; 16],
    pub issued_at: String,
    pub expires_at: Option<String>,
    pub usage_limit: Option<u64>,
    pub uses_consumed: u64,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EgressPermitUseReceipt {
    pub permit_id: [u8; 16],
    pub decision_id: [u8; 16],
    pub uses_consumed: u64,
    pub status: String,
}

#[derive(Debug)]
pub enum EgressPermitError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidPrincipal,
    InvalidAction,
    InvalidPurpose,
    InvalidDestination,
    InvalidProtocolPort,
    InvalidTime,
    InvalidUsageLimit,
    InvalidReasonCode,
    IntegerOverflow,
    MissingAuthorityDecision,
    AuthorityDecisionMismatch,
    StaleAuthorityDecision,
    EffectNotFound,
    EffectMismatch,
    ApprovalNotFound,
    ApprovalMismatch,
    ApprovalAlreadyUsed,
    ParentLeaseNotFound,
    ParentLeaseMismatch,
    ParentLeaseInactive,
    ParentLeaseRevoked,
    ParentLeaseNotYetValid,
    ParentLeaseExpired,
    ParentLeaseScopeMismatch,
    ParentLeaseTemporalWidening,
    ParentLeaseCycle,
    ParentLeaseTooDeep,
    DuplicatePermit,
    PermitNotFound,
    PermitInactive,
    PermitRevoked,
    PermitExpired,
    PermitScopeMismatch,
    PermitUsageExhausted,
    UseDecisionNotFound,
    UseDecisionMismatch,
    UseDecisionStale,
    ActivePolicyMissing,
    PolicyMismatch,
    PolicyBundleInvalid,
    ConcurrentUseConflict,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for EgressPermitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "egress permit authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "egress permit sqlite error: {error}"),
            Self::Core(error) => write!(f, "egress permit canonical encoding error: {error}"),
            Self::Integrity(error) => write!(f, "egress permit integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "egress permit authority-security error: {error}")
            }
            Self::InvalidPrincipal => f.write_str("egress permit principal/process is invalid"),
            Self::InvalidAction => f.write_str("egress permit action is invalid"),
            Self::InvalidPurpose => f.write_str("egress permit purpose is invalid"),
            Self::InvalidDestination => f.write_str("egress permit destination scope is invalid"),
            Self::InvalidProtocolPort => {
                f.write_str("egress permit protocol/port scope is invalid")
            }
            Self::InvalidTime => f.write_str("egress permit time is invalid"),
            Self::InvalidUsageLimit => f.write_str("egress permit usage limit is invalid"),
            Self::InvalidReasonCode => f.write_str("egress permit reason code is invalid"),
            Self::IntegerOverflow => f.write_str("egress permit integer conversion overflow"),
            Self::MissingAuthorityDecision => {
                f.write_str("egress permit mutation has no durable authorization decision")
            }
            Self::AuthorityDecisionMismatch => {
                f.write_str("egress permit mutation authorization decision is mismatched")
            }
            Self::StaleAuthorityDecision => {
                f.write_str("egress permit mutation authorization decision is stale")
            }
            Self::EffectNotFound => f.write_str("egress permit mutation effect does not exist"),
            Self::EffectMismatch => f.write_str("egress permit mutation effect is mismatched"),
            Self::ApprovalNotFound => f.write_str("egress permit mutation approval does not exist"),
            Self::ApprovalMismatch => f.write_str("egress permit mutation approval is mismatched"),
            Self::ApprovalAlreadyUsed => {
                f.write_str("egress permit mutation approval was already consumed")
            }
            Self::ParentLeaseNotFound => f.write_str("egress permit parent lease does not exist"),
            Self::ParentLeaseMismatch => {
                f.write_str("egress permit parent lease binding is mismatched")
            }
            Self::ParentLeaseInactive => f.write_str("egress permit parent lease is inactive"),
            Self::ParentLeaseRevoked => {
                f.write_str("egress permit parent lease or ancestor is revoked")
            }
            Self::ParentLeaseNotYetValid => {
                f.write_str("egress permit parent lease is not yet valid")
            }
            Self::ParentLeaseExpired => f.write_str("egress permit parent lease is expired"),
            Self::ParentLeaseScopeMismatch => {
                f.write_str("egress permit exceeds parent lease scope")
            }
            Self::ParentLeaseTemporalWidening => {
                f.write_str("egress permit lifetime exceeds parent lease lifetime")
            }
            Self::ParentLeaseCycle => {
                f.write_str("egress permit parent lease chain contains a cycle")
            }
            Self::ParentLeaseTooDeep => f.write_str("egress permit parent lease chain is too deep"),
            Self::DuplicatePermit => f.write_str("egress permit already exists"),
            Self::PermitNotFound => f.write_str("egress permit does not exist"),
            Self::PermitInactive => f.write_str("egress permit is not active"),
            Self::PermitRevoked => f.write_str("egress permit is revoked"),
            Self::PermitExpired => f.write_str("egress permit is expired"),
            Self::PermitScopeMismatch => f.write_str("egress permit does not cover exact use"),
            Self::PermitUsageExhausted => f.write_str("egress permit usage limit is exhausted"),
            Self::UseDecisionNotFound => {
                f.write_str("egress use authorization decision is missing")
            }
            Self::UseDecisionMismatch => {
                f.write_str("egress use authorization decision is mismatched")
            }
            Self::UseDecisionStale => f.write_str("egress use authorization decision is stale"),
            Self::ActivePolicyMissing => f.write_str("egress use active policy is missing"),
            Self::PolicyMismatch => f.write_str("egress use decision policy is not active"),
            Self::PolicyBundleInvalid => f.write_str("egress use active policy bundle is invalid"),
            Self::ConcurrentUseConflict => {
                f.write_str("egress permit use raced with another mutation")
            }
            Self::InvalidStoredRecord(reason) => {
                write!(f, "egress permit stored record is invalid: {reason}")
            }
        }
    }
}

impl Error for EgressPermitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for EgressPermitError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for EgressPermitError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for EgressPermitError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

#[allow(clippy::too_many_arguments)]
// Keep each authority binding explicit at this security-sensitive preparation boundary.
pub fn prepare_egress_permit_issue(
    principal_or_process: &str,
    action: &str,
    purpose: &str,
    destination_scope: &str,
    protocol_port_scope: &str,
    taint_digest: [u8; 32],
    secret_handle_id: Option<[u8; 16]>,
    parent_lease: EgressParentLeaseBinding,
    issued_at: &str,
    expires_at: Option<&str>,
    usage_limit: Option<u64>,
) -> Result<PreparedEgressPermitIssue, EgressPermitError> {
    validate_principal(principal_or_process)?;
    validate_action(action)?;
    validate_bounded_text(
        purpose,
        MAX_PURPOSE_BYTES,
        EgressPermitError::InvalidPurpose,
    )?;
    validate_bounded_text(
        destination_scope,
        MAX_DESTINATION_BYTES,
        EgressPermitError::InvalidDestination,
    )?;
    validate_protocol_port(protocol_port_scope)?;
    if parent_lease.generation == 0 {
        return Err(EgressPermitError::ParentLeaseMismatch);
    }
    if !valid_utc_second(issued_at) {
        return Err(EgressPermitError::InvalidTime);
    }
    let expires_at = match expires_at {
        Some(value) if valid_utc_second(value) && issued_at < value => Some(value.to_owned()),
        Some(_) => return Err(EgressPermitError::InvalidTime),
        None => None,
    };
    if matches!(usage_limit, Some(0)) {
        return Err(EgressPermitError::InvalidUsageLimit);
    }

    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(ISSUE_INTENT_DOMAIN)?;
    encoder.push_bytes(principal_or_process.as_bytes())?;
    encoder.push_bytes(action.as_bytes())?;
    encoder.push_bytes(purpose.as_bytes())?;
    encoder.push_bytes(destination_scope.as_bytes())?;
    encoder.push_bytes(protocol_port_scope.as_bytes())?;
    encoder.push_bytes(&taint_digest)?;
    encode_optional_id(&mut encoder, secret_handle_id)?;
    encoder.push_bytes(&parent_lease.lease_id)?;
    encoder.push_u64(parent_lease.generation);
    encoder.push_bytes(&parent_lease.authority_digest)?;
    encoder.push_bytes(issued_at.as_bytes())?;
    encode_optional_text(&mut encoder, expires_at.as_deref())?;
    encode_optional_u64(&mut encoder, usage_limit);
    let intent_digest = crate::payload_hash(&encoder.finish());
    let resource = format!("egress-permit-issue:{}", hex_bytes(&intent_digest));

    Ok(PreparedEgressPermitIssue {
        principal_or_process: principal_or_process.to_owned(),
        action: action.to_owned(),
        purpose: purpose.to_owned(),
        destination_scope: destination_scope.to_owned(),
        protocol_port_scope: protocol_port_scope.to_owned(),
        taint_digest,
        secret_handle_id,
        parent_lease,
        issued_at: issued_at.to_owned(),
        expires_at,
        usage_limit,
        intent_digest,
        resource,
    })
}

pub fn prepare_egress_permit_revocation(
    permit_id: [u8; 16],
    reason_code: &str,
) -> Result<PreparedEgressPermitRevocation, EgressPermitError> {
    validate_reason_code(reason_code)?;
    let resource = egress_permit_resource(permit_id);
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(REVOKE_INTENT_DOMAIN)?;
    encoder.push_bytes(&permit_id)?;
    encoder.push_bytes(reason_code.as_bytes())?;
    let intent_digest = crate::payload_hash(&encoder.finish());
    Ok(PreparedEgressPermitRevocation {
        permit_id,
        intent_digest,
        resource,
    })
}

pub fn egress_permit_resource(permit_id: [u8; 16]) -> String {
    format!("egress-permit:{}", hex_bytes(&permit_id))
}

pub struct EgressPermitStore {
    pub(crate) connection: Connection,
}

impl EgressPermitStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, EgressPermitError> {
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
        prepared: PreparedEgressPermitIssue,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<EgressPermitRecord, EgressPermitError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let authority = verify_current_mutation_authority(
            &transaction,
            authority_decision_id,
            EGRESS_PERMIT_ISSUE_ACTION,
            &prepared.resource,
        )?;
        verify_mutation_effect(
            &transaction,
            effect_id,
            EGRESS_PERMIT_ISSUE_ACTION,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        verify_once_approval(
            &transaction,
            approval_id,
            effect_id,
            EGRESS_PERMIT_ISSUE_ACTION,
            &prepared.resource,
        )?;
        verify_parent_lease_for_issue(&transaction, &prepared)?;

        let permit_id = derived_id(
            ISSUE_ID_DOMAIN,
            prepared.intent_digest,
            effect_id,
            authority_decision_id,
            approval_id,
        );
        let duplicate = transaction
            .query_row(
                "SELECT 1 FROM egress_permits WHERE permit_id = ?1 LIMIT 1",
                params![&permit_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if duplicate {
            return Err(EgressPermitError::DuplicatePermit);
        }

        transaction.execute(
            "INSERT INTO egress_permits (permit_id, principal_or_process, action, purpose, destination_scope, protocol_port_scope, taint_digest, secret_handle_id, parent_lease_id, issued_at, expires_at, usage_limit, status, uses_consumed) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 'active', 0)",
            params![
                &permit_id[..],
                &prepared.principal_or_process,
                &prepared.action,
                &prepared.purpose,
                prepared.destination_scope.as_bytes(),
                prepared.protocol_port_scope.as_bytes(),
                &prepared.taint_digest[..],
                prepared.secret_handle_id.map(|value| value.to_vec()),
                &prepared.parent_lease.lease_id[..],
                &prepared.issued_at,
                prepared.expires_at.as_deref(),
                prepared.usage_limit.map(to_i64).transpose()?,
            ],
        )?;
        append_egress_permit_snapshot(&transaction, &permit_id)
            .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))?;
        consume_once_approval(&transaction, approval_id, effect_id, authority.global_seq)?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(EgressPermitRecord {
            permit_id,
            principal_or_process: prepared.principal_or_process,
            action: prepared.action,
            purpose: prepared.purpose,
            destination_scope: prepared.destination_scope,
            protocol_port_scope: prepared.protocol_port_scope,
            taint_digest: prepared.taint_digest,
            secret_handle_id: prepared.secret_handle_id,
            parent_lease_id: prepared.parent_lease.lease_id,
            issued_at: prepared.issued_at,
            expires_at: prepared.expires_at,
            usage_limit: prepared.usage_limit,
            uses_consumed: 0,
            status: "active".to_owned(),
        })
    }

    pub fn revoke(
        &mut self,
        prepared: PreparedEgressPermitRevocation,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<EgressPermitRecord, EgressPermitError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let authority = verify_current_mutation_authority(
            &transaction,
            authority_decision_id,
            EGRESS_PERMIT_REVOKE_ACTION,
            &prepared.resource,
        )?;
        verify_mutation_effect(
            &transaction,
            effect_id,
            EGRESS_PERMIT_REVOKE_ACTION,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        verify_once_approval(
            &transaction,
            approval_id,
            effect_id,
            EGRESS_PERMIT_REVOKE_ACTION,
            &prepared.resource,
        )?;

        let mut record = load_permit(&transaction, prepared.permit_id)?;
        if record.status == "revoked" {
            return Err(EgressPermitError::PermitRevoked);
        }
        if record.status != "active" && record.status != "exhausted" {
            return Err(EgressPermitError::PermitInactive);
        }
        transaction.execute(
            "UPDATE egress_permits SET status = 'revoked' WHERE permit_id = ?1",
            params![&prepared.permit_id[..]],
        )?;
        append_egress_permit_snapshot(&transaction, &prepared.permit_id)
            .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))?;
        consume_once_approval(&transaction, approval_id, effect_id, authority.global_seq)?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;
        record.status = "revoked".to_owned();
        Ok(record)
    }

    #[allow(clippy::too_many_arguments)]
    // Keep the exact permit-use bindings visible at the protected execution boundary.
    pub fn authorize_use(
        &mut self,
        permit_id: [u8; 16],
        decision_id: [u8; 16],
        principal_or_process: &str,
        action: &str,
        purpose: &str,
        destination: &str,
        protocol_port: &str,
        observed_at: &str,
    ) -> Result<EgressPermitUseReceipt, EgressPermitError> {
        validate_principal(principal_or_process)?;
        validate_action(action)?;
        validate_bounded_text(
            purpose,
            MAX_PURPOSE_BYTES,
            EgressPermitError::InvalidPurpose,
        )?;
        validate_bounded_text(
            destination,
            MAX_DESTINATION_BYTES,
            EgressPermitError::InvalidDestination,
        )?;
        validate_protocol_port(protocol_port)?;
        if !valid_utc_second(observed_at) {
            return Err(EgressPermitError::InvalidTime);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let permit = load_permit(&transaction, permit_id)?;
        if permit.status == "revoked" {
            return Err(EgressPermitError::PermitRevoked);
        }
        if permit.status == "exhausted" {
            return Err(EgressPermitError::PermitUsageExhausted);
        }
        if permit.status != "active" {
            return Err(EgressPermitError::PermitInactive);
        }
        if permit.principal_or_process != principal_or_process
            || permit.action != action
            || permit.purpose != purpose
            || permit.destination_scope != destination
            || permit.protocol_port_scope != protocol_port
        {
            return Err(EgressPermitError::PermitScopeMismatch);
        }
        if observed_at < permit.issued_at.as_str() {
            return Err(EgressPermitError::PermitInactive);
        }
        if let Some(expires_at) = permit.expires_at.as_deref()
            && observed_at >= expires_at
        {
            return Err(EgressPermitError::PermitExpired);
        }
        if let Some(limit) = permit.usage_limit
            && permit.uses_consumed >= limit
        {
            return Err(EgressPermitError::PermitUsageExhausted);
        }

        let decision = load_current_use_decision(
            &transaction,
            decision_id,
            principal_or_process,
            action,
            destination,
            permit.parent_lease_id,
        )?;
        verify_active_policy(&transaction, &decision)?;
        verify_lease_chain_for_use(
            &transaction,
            permit.parent_lease_id,
            decision.lease_generation,
            principal_or_process,
            action,
            destination,
            observed_at,
        )?;

        let new_uses = permit
            .uses_consumed
            .checked_add(1)
            .ok_or(EgressPermitError::IntegerOverflow)?;
        let new_status = if permit.usage_limit == Some(new_uses) {
            "exhausted"
        } else {
            "active"
        };
        let changed = transaction.execute(
            "UPDATE egress_permits SET uses_consumed = ?1, status = ?2 WHERE permit_id = ?3 AND uses_consumed = ?4 AND status = 'active'",
            params![
                to_i64(new_uses)?,
                new_status,
                &permit_id[..],
                to_i64(permit.uses_consumed)?,
            ],
        )?;
        if changed != 1 {
            return Err(EgressPermitError::ConcurrentUseConflict);
        }
        append_egress_permit_snapshot(&transaction, &permit_id)
            .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(EgressPermitUseReceipt {
            permit_id,
            decision_id,
            uses_consumed: new_uses,
            status: new_status.to_owned(),
        })
    }
}

struct AuthorityEvidence {
    global_seq: u64,
}

pub(crate) struct UseDecisionEvidence {
    pub(crate) lease_generation: u64,
    pub(crate) policy_bundle_id: [u8; 16],
    pub(crate) policy_bundle_hash: [u8; 32],
}

fn verify_transaction_integrity(transaction: &Transaction<'_>) -> Result<(), EgressPermitError> {
    crate::integrity::verify(transaction)
        .map_err(|error| EgressPermitError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))
}

fn verify_current_mutation_authority(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    expected_action: &str,
    expected_resource: &str,
) -> Result<AuthorityEvidence, EgressPermitError> {
    let row = transaction
        .query_row(
            "SELECT action, resource, hard_guard_result, decision, global_seq, authority_evidence_version FROM authorization_decisions WHERE decision_id = ?1",
            params![&decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        )
        .optional()?
        .ok_or(EgressPermitError::MissingAuthorityDecision)?;
    if row.0 != expected_action
        || row.1 != expected_resource
        || row.2 != "pass"
        || row.3 != "allow"
        || row.5 < 2
    {
        return Err(EgressPermitError::AuthorityDecisionMismatch);
    }
    let global_seq = nonnegative_u64(row.4, "mutation decision sequence is invalid")?;
    if latest_global_seq(transaction)? != global_seq {
        return Err(EgressPermitError::StaleAuthorityDecision);
    }
    Ok(AuthorityEvidence { global_seq })
}

fn verify_mutation_effect(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    expected_action: &str,
    expected_resource: &str,
    expected_payload_hash: [u8; 32],
) -> Result<(), EgressPermitError> {
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
        .ok_or(EgressPermitError::EffectNotFound)?;
    if row.0 != expected_action
        || row.1 != expected_resource
        || row.2 != EGRESS_PERMIT_MUTATION_RISK_CLASS
        || row.3 != "at_most_once"
        || row.4.as_slice() != expected_payload_hash
        || row.5 != "authorized"
    {
        return Err(EgressPermitError::EffectMismatch);
    }
    Ok(())
}

fn verify_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    expected_action: &str,
    expected_resource: &str,
) -> Result<(), EgressPermitError> {
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
        .ok_or(EgressPermitError::ApprovalNotFound)?;
    if row.0 != "ONCE"
        || row.1.as_slice() != expected_action.as_bytes()
        || row.2.as_slice() != expected_resource.as_bytes()
        || row.3.as_deref() != Some(effect_id.0.to_be_bytes().as_slice())
        || row.4.is_some()
        || row.5 != EGRESS_PERMIT_MUTATION_RISK_CLASS
        || row.6.is_some()
        || row.7 != Some(1)
        || row.8.is_some()
    {
        return Err(EgressPermitError::ApprovalMismatch);
    }
    let used = transaction
        .query_row(
            "SELECT 1 FROM approval_consumptions WHERE approval_id = ?1 LIMIT 1",
            params![&approval_id[..]],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if used {
        return Err(EgressPermitError::ApprovalAlreadyUsed);
    }
    Ok(())
}

fn consume_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    global_seq: u64,
) -> Result<(), EgressPermitError> {
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
        .map_err(|error| EgressPermitError::AuthoritySecurity(error.to_string()))
}

fn verify_parent_lease_for_issue(
    transaction: &Transaction<'_>,
    prepared: &PreparedEgressPermitIssue,
) -> Result<(), EgressPermitError> {
    verify_lease_chain(
        transaction,
        prepared.parent_lease.lease_id,
        Some(prepared.parent_lease),
        &prepared.principal_or_process,
        &prepared.action,
        &prepared.destination_scope,
        &prepared.issued_at,
        prepared.expires_at.as_deref(),
    )
}

pub(crate) fn verify_lease_chain_for_use(
    transaction: &Transaction<'_>,
    lease_id: [u8; 16],
    expected_generation: u64,
    principal: &str,
    action: &str,
    resource: &str,
    observed_at: &str,
) -> Result<(), EgressPermitError> {
    verify_lease_chain(
        transaction,
        lease_id,
        Some(EgressParentLeaseBinding::new(
            lease_id,
            expected_generation,
            load_lease_digest(transaction, lease_id)?,
        )),
        principal,
        action,
        resource,
        observed_at,
        Some(observed_at),
    )
}

#[allow(clippy::too_many_arguments)]
// Chain validation intentionally receives each authority dimension separately.
fn verify_lease_chain(
    transaction: &Transaction<'_>,
    lease_id: [u8; 16],
    expected_binding: Option<EgressParentLeaseBinding>,
    principal: &str,
    action: &str,
    resource: &str,
    observed_at: &str,
    permit_expires_at: Option<&str>,
) -> Result<(), EgressPermitError> {
    let mut next = Some(lease_id);
    let mut seen = HashSet::new();
    let mut depth = 0_usize;
    while let Some(current_id) = next {
        if depth >= MAX_PARENT_CHAIN_DEPTH {
            return Err(EgressPermitError::ParentLeaseTooDeep);
        }
        if !seen.insert(current_id) {
            return Err(EgressPermitError::ParentLeaseCycle);
        }
        let row = transaction
            .query_row(
                "SELECT principal_id, parent_lease_id, actions_scope, resources_scope, not_before, expires_at, generation, status, authority_digest, EXISTS(SELECT 1 FROM capability_revocations r WHERE r.lease_id = l.lease_id) FROM capability_leases l WHERE l.lease_id = ?1",
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
                        row.get::<_, Vec<u8>>(8)?,
                        row.get::<_, i64>(9)?,
                    ))
                },
            )
            .optional()?
            .ok_or(EgressPermitError::ParentLeaseNotFound)?;
        if row.0 != principal {
            return Err(EgressPermitError::ParentLeaseMismatch);
        }
        let generation = positive_u64(row.6, "lease generation is invalid")?;
        let authority_digest = hash32(row.8, "lease authority digest is invalid")?;
        if depth == 0
            && let Some(binding) = expected_binding
            && (binding.generation != generation || binding.authority_digest != authority_digest)
        {
            return Err(EgressPermitError::ParentLeaseMismatch);
        }
        if row.7 != "active" {
            return Err(EgressPermitError::ParentLeaseInactive);
        }
        if row.9 != 0 {
            return Err(EgressPermitError::ParentLeaseRevoked);
        }
        if let Some(not_before) = row.4.as_deref() {
            require_stored_time(not_before, "lease not_before is malformed")?;
            if observed_at < not_before {
                return Err(EgressPermitError::ParentLeaseNotYetValid);
            }
        }
        if let Some(expires_at) = row.5.as_deref() {
            require_stored_time(expires_at, "lease expiry is malformed")?;
            if observed_at >= expires_at {
                return Err(EgressPermitError::ParentLeaseExpired);
            }
            if let Some(permit_end) = permit_expires_at {
                if permit_end > expires_at {
                    return Err(EgressPermitError::ParentLeaseTemporalWidening);
                }
            } else if permit_expires_at.is_none() {
                return Err(EgressPermitError::ParentLeaseTemporalWidening);
            }
        }
        if !scope_contains(&row.2, action)? || !scope_contains(&row.3, resource)? {
            return Err(EgressPermitError::ParentLeaseScopeMismatch);
        }
        next = row
            .1
            .map(|value| id16(value, "lease parent id is invalid"))
            .transpose()?;
        depth += 1;
    }
    Ok(())
}

fn load_lease_digest(
    transaction: &Transaction<'_>,
    lease_id: [u8; 16],
) -> Result<[u8; 32], EgressPermitError> {
    let value = transaction
        .query_row(
            "SELECT authority_digest FROM capability_leases WHERE lease_id = ?1",
            params![&lease_id[..]],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .optional()?
        .ok_or(EgressPermitError::ParentLeaseNotFound)?;
    hash32(value, "lease authority digest is invalid")
}

pub(crate) fn load_permit(
    transaction: &Transaction<'_>,
    permit_id: [u8; 16],
) -> Result<EgressPermitRecord, EgressPermitError> {
    let row = transaction
        .query_row(
            "SELECT principal_or_process, action, purpose, destination_scope, protocol_port_scope, taint_digest, secret_handle_id, parent_lease_id, issued_at, expires_at, usage_limit, status, uses_consumed FROM egress_permits WHERE permit_id = ?1",
            params![&permit_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, Vec<u8>>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                    row.get::<_, Vec<u8>>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<i64>>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, i64>(12)?,
                ))
            },
        )
        .optional()?
        .ok_or(EgressPermitError::PermitNotFound)?;
    let destination_scope =
        stored_text(row.3, MAX_DESTINATION_BYTES, "destination scope is invalid")?;
    let protocol_port_scope = stored_text(
        row.4,
        MAX_PROTOCOL_PORT_BYTES,
        "protocol/port scope is invalid",
    )?;
    let taint_digest = hash32(row.5, "taint digest is invalid")?;
    let secret_handle_id = row
        .6
        .map(|value| id16(value, "secret handle id is invalid"))
        .transpose()?;
    let parent_lease_id = id16(row.7, "parent lease id is invalid")?;
    require_stored_time(&row.8, "permit issued_at is malformed")?;
    if let Some(expires_at) = row.9.as_deref() {
        require_stored_time(expires_at, "permit expires_at is malformed")?;
        if row.8.as_str() >= expires_at {
            return Err(EgressPermitError::InvalidStoredRecord(
                "permit lifetime is invalid",
            ));
        }
    }
    let usage_limit = row
        .10
        .map(|value| positive_u64(value, "usage limit is invalid"))
        .transpose()?;
    let uses_consumed = nonnegative_u64(row.12, "uses_consumed is invalid")?;
    if usage_limit.is_some_and(|limit| uses_consumed > limit) {
        return Err(EgressPermitError::InvalidStoredRecord(
            "uses_consumed exceeds usage_limit",
        ));
    }
    Ok(EgressPermitRecord {
        permit_id,
        principal_or_process: row.0,
        action: row.1,
        purpose: row.2,
        destination_scope,
        protocol_port_scope,
        taint_digest,
        secret_handle_id,
        parent_lease_id,
        issued_at: row.8,
        expires_at: row.9,
        usage_limit,
        uses_consumed,
        status: row.11,
    })
}

fn load_current_use_decision(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    principal: &str,
    action: &str,
    resource: &str,
    parent_lease_id: [u8; 16],
) -> Result<UseDecisionEvidence, EgressPermitError> {
    let row = transaction
        .query_row(
            "SELECT principal, action, resource, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, decision, global_seq, authority_evidence_version FROM authorization_decisions WHERE decision_id = ?1",
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
                    row.get::<_, String>(8)?,
                    row.get::<_, i64>(9)?,
                    row.get::<_, i64>(10)?,
                ))
            },
        )
        .optional()?
        .ok_or(EgressPermitError::UseDecisionNotFound)?;
    if row.0 != principal
        || row.1 != action
        || row.2 != resource
        || row.3 != "pass"
        || row.8 != "allow"
        || row.10 < 2
    {
        return Err(EgressPermitError::UseDecisionMismatch);
    }
    let lease_id = id16(
        row.4.ok_or(EgressPermitError::UseDecisionMismatch)?,
        "decision lease id is invalid",
    )?;
    if lease_id != parent_lease_id {
        return Err(EgressPermitError::UseDecisionMismatch);
    }
    let lease_generation = positive_u64(
        row.5.ok_or(EgressPermitError::UseDecisionMismatch)?,
        "decision lease generation is invalid",
    )?;
    let policy_bundle_id = id16(
        row.6.ok_or(EgressPermitError::UseDecisionMismatch)?,
        "decision policy bundle id is invalid",
    )?;
    let policy_bundle_hash = hash32(
        row.7.ok_or(EgressPermitError::UseDecisionMismatch)?,
        "decision policy bundle hash is invalid",
    )?;
    let global_seq = nonnegative_u64(row.9, "use decision sequence is invalid")?;
    if latest_global_seq(transaction)? != global_seq {
        return Err(EgressPermitError::UseDecisionStale);
    }
    Ok(UseDecisionEvidence {
        lease_generation,
        policy_bundle_id,
        policy_bundle_hash,
    })
}

pub(crate) fn verify_active_policy(
    transaction: &Transaction<'_>,
    decision: &UseDecisionEvidence,
) -> Result<(), EgressPermitError> {
    let row = transaction
        .query_row(
            "SELECT policy_bundle_id, bundle_hash FROM active_policy WHERE singleton_id = 1",
            [],
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?
        .ok_or(EgressPermitError::ActivePolicyMissing)?;
    let active_id = id16(row.0, "active policy id is invalid")?;
    let active_hash = hash32(row.1, "active policy hash is invalid")?;
    if active_id != decision.policy_bundle_id || active_hash != decision.policy_bundle_hash {
        return Err(EgressPermitError::PolicyMismatch);
    }
    let status = transaction
        .query_row(
            "SELECT validation_status FROM policy_bundles WHERE policy_bundle_id = ?1 AND bundle_hash = ?2",
            params![&active_id[..], &active_hash[..]],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or(EgressPermitError::PolicyBundleInvalid)?;
    if status != "validated" {
        return Err(EgressPermitError::PolicyBundleInvalid);
    }
    Ok(())
}

pub(crate) fn latest_global_seq(transaction: &Transaction<'_>) -> Result<u64, EgressPermitError> {
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (SELECT global_seq FROM session_events UNION ALL SELECT global_seq FROM effect_transitions UNION ALL SELECT global_seq FROM authorization_decisions)",
        [],
        |row| row.get(0),
    )?;
    nonnegative_u64(latest, "latest global sequence is invalid")
}

fn scope_contains(bytes: &[u8], expected: &str) -> Result<bool, EgressPermitError> {
    if bytes.is_empty() {
        return Ok(false);
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| EgressPermitError::InvalidStoredRecord("lease scope is not UTF-8"))?;
    Ok(text.split('\n').any(|entry| entry == expected))
}

fn validate_principal(value: &str) -> Result<(), EgressPermitError> {
    let known_prefix = ["owner:", "client:", "kernel:", "test:", "process:"]
        .iter()
        .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len());
    if !known_prefix
        || value.len() > MAX_PRINCIPAL_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(EgressPermitError::InvalidPrincipal);
    }
    Ok(())
}

fn validate_action(value: &str) -> Result<(), EgressPermitError> {
    if value.len() > MAX_ACTION_BYTES
        || !(value == "network.egress" || value.starts_with("network.egress."))
    {
        return Err(EgressPermitError::InvalidAction);
    }
    let bytes = value.as_bytes();
    if !bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        || bytes.windows(2).any(|pair| pair == b"..")
        || bytes.iter().any(|byte| {
            !(byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-'))
        })
    {
        return Err(EgressPermitError::InvalidAction);
    }
    Ok(())
}

fn validate_bounded_text(
    value: &str,
    max_bytes: usize,
    error: EgressPermitError,
) -> Result<(), EgressPermitError> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(error);
    }
    Ok(())
}

fn validate_protocol_port(value: &str) -> Result<(), EgressPermitError> {
    if value.is_empty() || value.len() > MAX_PROTOCOL_PORT_BYTES {
        return Err(EgressPermitError::InvalidProtocolPort);
    }
    let Some((protocol, port)) = value.split_once(':') else {
        return Err(EgressPermitError::InvalidProtocolPort);
    };
    if protocol.is_empty()
        || protocol
            .bytes()
            .any(|byte| !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'))
    {
        return Err(EgressPermitError::InvalidProtocolPort);
    }
    let port = port
        .parse::<u16>()
        .map_err(|_| EgressPermitError::InvalidProtocolPort)?;
    if port == 0 || value.matches(':').count() != 1 {
        return Err(EgressPermitError::InvalidProtocolPort);
    }
    Ok(())
}

fn validate_reason_code(value: &str) -> Result<(), EgressPermitError> {
    if value.is_empty() || value.len() > MAX_REASON_CODE_BYTES {
        return Err(EgressPermitError::InvalidReasonCode);
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
        return Err(EgressPermitError::InvalidReasonCode);
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

fn require_stored_time(value: &str, reason: &'static str) -> Result<(), EgressPermitError> {
    if valid_utc_second(value) {
        Ok(())
    } else {
        Err(EgressPermitError::InvalidStoredRecord(reason))
    }
}

fn decimal(bytes: &[u8], start: usize, end: usize) -> u32 {
    bytes[start..end]
        .iter()
        .fold(0_u32, |value, byte| value * 10 + u32::from(*byte - b'0'))
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn encode_optional_id(
    encoder: &mut CanonicalEncoder,
    value: Option<[u8; 16]>,
) -> Result<(), EgressPermitError> {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn encode_optional_text(
    encoder: &mut CanonicalEncoder,
    value: Option<&str>,
) -> Result<(), EgressPermitError> {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value.as_bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn encode_optional_u64(encoder: &mut CanonicalEncoder, value: Option<u64>) {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_u64(value);
        }
        None => encoder.push_u8(0),
    }
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

fn stored_text(
    value: Vec<u8>,
    max_bytes: usize,
    reason: &'static str,
) -> Result<String, EgressPermitError> {
    if value.is_empty() || value.len() > max_bytes {
        return Err(EgressPermitError::InvalidStoredRecord(reason));
    }
    let value =
        String::from_utf8(value).map_err(|_| EgressPermitError::InvalidStoredRecord(reason))?;
    if value.trim() != value || value.chars().any(char::is_control) {
        return Err(EgressPermitError::InvalidStoredRecord(reason));
    }
    Ok(value)
}

fn id16(value: Vec<u8>, reason: &'static str) -> Result<[u8; 16], EgressPermitError> {
    value
        .try_into()
        .map_err(|_| EgressPermitError::InvalidStoredRecord(reason))
}

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], EgressPermitError> {
    value
        .try_into()
        .map_err(|_| EgressPermitError::InvalidStoredRecord(reason))
}

fn positive_u64(value: i64, reason: &'static str) -> Result<u64, EgressPermitError> {
    let value = nonnegative_u64(value, reason)?;
    if value == 0 {
        return Err(EgressPermitError::InvalidStoredRecord(reason));
    }
    Ok(value)
}

fn nonnegative_u64(value: i64, reason: &'static str) -> Result<u64, EgressPermitError> {
    u64::try_from(value).map_err(|_| EgressPermitError::InvalidStoredRecord(reason))
}

fn to_i64(value: u64) -> Result<i64, EgressPermitError> {
    i64::try_from(value).map_err(|_| EgressPermitError::IntegerOverflow)
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::authority_security_write::{
        append_active_policy_snapshot, append_approval_snapshot,
        append_authorization_decision_v2_snapshot, append_capability_lease_snapshot,
        append_policy_bundle_snapshot,
    };
    use crate::security_audit::{
        AuthorizationAuditInput, EffectIntentAuditInput, EffectTransitionAuditInput,
        append_authorization_decision, append_effect_intent, append_effect_transition,
    };
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);
    pub(crate) const PRINCIPAL: &str = "owner:owner";
    pub(crate) const ACTION: &str = "network.egress.connect";
    const DESTINATION: &str = "https://example.invalid";
    const PROTOCOL_PORT: &str = "https:443";
    pub(crate) const PURPOSE: &str = "fixture-fetch";
    pub(crate) const POLICY_ID: [u8; 16] = [31; 16];
    pub(crate) const POLICY_HASH: [u8; 32] = [32; 32];
    pub(crate) const LEASE_ID: [u8; 16] = [41; 16];
    const LEASE_DIGEST: [u8; 32] = [42; 32];

    pub(crate) fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-egress-permit-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    pub(crate) fn install_policy_and_parent_lease(connection: &mut Connection) {
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let resources_scope = format!(
            "{DESTINATION}\nhttps://203.0.113.10:443\nhttps://203.0.113.11:443\nhttps://10.0.0.7:443"
        );
        transaction
            .execute(
                "INSERT INTO policy_bundles (policy_bundle_id, version, schema_version, canonical_policy_bytes, bundle_hash, created_by, created_global_seq, validation_status) VALUES (?1, 1, 1, X'01', ?2, ?3, 1, 'validated')",
                params![&POLICY_ID[..], &POLICY_HASH[..], PRINCIPAL],
            )
            .unwrap();
        append_policy_bundle_snapshot(&transaction, &POLICY_ID).unwrap();
        transaction
            .execute(
                "INSERT INTO active_policy (singleton_id, policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq) VALUES (1, ?1, ?2, ?3, ?4, 1)",
                params![&POLICY_ID[..], &POLICY_HASH[..], PRINCIPAL, &[33_u8; 16][..]],
            )
            .unwrap();
        append_active_policy_snapshot(&transaction).unwrap();
        transaction
            .execute(
                "INSERT INTO capability_leases (lease_id, principal_id, parent_lease_id, actions_scope, resources_scope, context_constraints, issued_by, issued_global_seq, not_before, expires_at, generation, status, authority_digest) VALUES (?1, ?2, NULL, ?3, ?4, X'', ?2, 1, NULL, '2026-08-30T00:00:00Z', 1, 'active', ?5)",
                params![&LEASE_ID[..], PRINCIPAL, ACTION.as_bytes(), resources_scope.as_bytes(), &LEASE_DIGEST[..]],
            )
            .unwrap();
        append_capability_lease_snapshot(&transaction, &LEASE_ID).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
    }

    pub(crate) struct WorkIds {
        pub(crate) effect: EffectId,
        pub(crate) decision: [u8; 16],
        pub(crate) approval: [u8; 16],
    }

    pub(crate) fn install_mutation_work(
        connection: &mut Connection,
        base_global_seq: u64,
        discriminator: u8,
        action: &str,
        resource: &str,
        payload_hash: [u8; 32],
    ) -> WorkIds {
        let effect = EffectId(u128::from(discriminator) + 4000);
        let effect_bytes = effect.0.to_be_bytes();
        let transition_id = [discriminator; 16];
        let decision = [discriminator.wrapping_add(40); 16];
        let approval = [discriminator.wrapping_add(80); 16];
        let session_id = [discriminator.wrapping_add(1); 16];
        let proposed_event_id = [discriminator.wrapping_add(2); 16];
        let transition_event_id = [discriminator.wrapping_add(3); 16];
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "INSERT INTO effect_intents (effect_id, session_id, requested_by, action, resource, risk_class, execution_semantics, idempotency_key, preconditions, dependencies, payload_hash, proposed_event_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'at_most_once', NULL, X'', X'', ?7, ?8)",
                params![&effect_bytes[..], &session_id[..], PRINCIPAL, action, resource, EGRESS_PERMIT_MUTATION_RISK_CLASS, &payload_hash[..], &proposed_event_id[..]],
            )
            .unwrap();
        append_effect_intent(
            &transaction,
            EffectIntentAuditInput {
                effect_id: &effect_bytes,
                session_id: &session_id,
                requested_by: PRINCIPAL,
                action,
                resource,
                risk_class: EGRESS_PERMIT_MUTATION_RISK_CLASS,
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"",
                dependencies: b"",
                payload_hash: &payload_hash,
                proposed_event_id: &proposed_event_id,
            },
        )
        .unwrap();
        transaction
            .execute(
                "INSERT INTO effect_transitions (transition_id, effect_id, global_seq, from_state, to_state, attempt_id, reason_code, evidence_ref, event_id) VALUES (?1, ?2, ?3, NULL, 'authorized', NULL, NULL, NULL, ?4)",
                params![&transition_id[..], &effect_bytes[..], to_i64(base_global_seq).unwrap(), &transition_event_id[..]],
            )
            .unwrap();
        append_effect_transition(
            &transaction,
            EffectTransitionAuditInput {
                transition_id: &transition_id,
                effect_id: &effect_bytes,
                global_seq: base_global_seq,
                from_state: None,
                to_state: "authorized",
                attempt_id: None,
                reason_code: None,
                evidence_ref: None,
                event_id: &transition_event_id,
            },
        )
        .unwrap();
        transaction
            .execute(
                "INSERT INTO authorization_decisions (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, authority_evidence_version) VALUES (?1, ?2, ?3, ?4, ?5, 'allow', 'test_allow', ?6, 'pass', NULL, NULL, NULL, NULL, X'', ?7, 2)",
                params![&decision[..], PRINCIPAL, action, resource, &[0_u8; 32][..], to_i64(base_global_seq + 1).unwrap(), &approval[..]],
            )
            .unwrap();
        append_authorization_decision(
            &transaction,
            AuthorizationAuditInput {
                decision_id: &decision,
                principal: PRINCIPAL,
                action,
                resource,
                context_hash: &[0_u8; 32],
                decision: "allow",
                reason_code: "test_allow",
                global_seq: base_global_seq + 1,
            },
        )
        .unwrap();
        append_authorization_decision_v2_snapshot(&transaction, &decision).unwrap();
        transaction
            .execute(
                "INSERT INTO approvals (approval_id, class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at) VALUES (?1, 'ONCE', ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8, ?9, '2026-08-28T00:00:00Z', NULL, 1, NULL)",
                params![&approval[..], PRINCIPAL, &[1_u8; 32][..], action.as_bytes(), resource.as_bytes(), &effect_bytes[..], EGRESS_PERMIT_MUTATION_RISK_CLASS, &[0_u8; 32][..], &decision[..]],
            )
            .unwrap();
        append_approval_snapshot(&transaction, &approval).unwrap();
        crate::integrity::verify(&transaction).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
        WorkIds {
            effect,
            decision,
            approval,
        }
    }

    fn install_use_decision(
        connection: &mut Connection,
        global_seq: u64,
        discriminator: u8,
    ) -> [u8; 16] {
        let decision = [discriminator; 16];
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "INSERT INTO authorization_decisions (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, authority_evidence_version) VALUES (?1, ?2, ?3, ?4, ?5, 'allow', 'egress_test_allow', ?6, 'pass', ?7, 1, ?8, ?9, X'', NULL, 2)",
                params![&decision[..], PRINCIPAL, ACTION, DESTINATION, &[0_u8; 32][..], to_i64(global_seq).unwrap(), &LEASE_ID[..], &POLICY_ID[..], &POLICY_HASH[..]],
            )
            .unwrap();
        append_authorization_decision(
            &transaction,
            AuthorizationAuditInput {
                decision_id: &decision,
                principal: PRINCIPAL,
                action: ACTION,
                resource: DESTINATION,
                context_hash: &[0_u8; 32],
                decision: "allow",
                reason_code: "egress_test_allow",
                global_seq,
            },
        )
        .unwrap();
        append_authorization_decision_v2_snapshot(&transaction, &decision).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
        decision
    }

    pub(crate) fn prepared(limit: Option<u64>) -> PreparedEgressPermitIssue {
        prepare_egress_permit_issue(
            PRINCIPAL,
            ACTION,
            PURPOSE,
            DESTINATION,
            PROTOCOL_PORT,
            [61; 32],
            None,
            EgressParentLeaseBinding::new(LEASE_ID, 1, LEASE_DIGEST),
            "2026-08-28T01:00:00Z",
            Some("2026-08-29T00:00:00Z"),
            limit,
        )
        .unwrap()
    }

    #[test]
    fn preparation_is_deterministic_and_bounded() {
        let first = prepared(Some(2));
        let second = prepared(Some(2));
        assert_eq!(first.intent_digest(), second.intent_digest());
        assert_eq!(first.resource(), second.resource());
        assert!(matches!(
            prepare_egress_permit_issue(
                PRINCIPAL,
                "session.read",
                PURPOSE,
                DESTINATION,
                PROTOCOL_PORT,
                [61; 32],
                None,
                EgressParentLeaseBinding::new(LEASE_ID, 1, LEASE_DIGEST),
                "2026-08-28T01:00:00Z",
                None,
                Some(1),
            ),
            Err(EgressPermitError::InvalidAction)
        ));
        assert!(matches!(
            prepare_egress_permit_issue(
                PRINCIPAL,
                ACTION,
                PURPOSE,
                DESTINATION,
                "https:0",
                [61; 32],
                None,
                EgressParentLeaseBinding::new(LEASE_ID, 1, LEASE_DIGEST),
                "2026-08-28T01:00:00Z",
                None,
                Some(1),
            ),
            Err(EgressPermitError::InvalidProtocolPort)
        ));
    }

    #[test]
    fn protected_issue_use_accounting_and_revocation_are_atomic() {
        let (runtime, authority) = authority();
        let mut store = EgressPermitStore::open(&authority).unwrap();
        install_policy_and_parent_lease(&mut store.connection);

        let issue = prepared(Some(3));
        let work = install_mutation_work(
            &mut store.connection,
            1,
            11,
            EGRESS_PERMIT_ISSUE_ACTION,
            issue.resource(),
            issue.intent_digest(),
        );
        let permit = store
            .issue(issue, work.decision, work.approval, work.effect)
            .unwrap();
        assert_eq!(permit.uses_consumed, 0);
        assert_eq!(permit.status, "active");
        let issue_consumptions: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM approval_consumptions WHERE approval_id = ?1",
                params![&work.approval[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(issue_consumptions, 1);

        let use_decision = install_use_decision(&mut store.connection, 3, 91);
        let receipt = store
            .authorize_use(
                permit.permit_id,
                use_decision,
                PRINCIPAL,
                ACTION,
                PURPOSE,
                DESTINATION,
                PROTOCOL_PORT,
                "2026-08-28T02:00:00Z",
            )
            .unwrap();
        assert_eq!(receipt.uses_consumed, 1);
        assert_eq!(receipt.status, "active");

        let revoke = prepare_egress_permit_revocation(permit.permit_id, "owner_revoked").unwrap();
        let revoke_work = install_mutation_work(
            &mut store.connection,
            4,
            12,
            EGRESS_PERMIT_REVOKE_ACTION,
            revoke.resource(),
            revoke.intent_digest(),
        );
        let revoked = store
            .revoke(
                revoke,
                revoke_work.decision,
                revoke_work.approval,
                revoke_work.effect,
            )
            .unwrap();
        assert_eq!(revoked.status, "revoked");
        assert!(matches!(
            store.authorize_use(
                permit.permit_id,
                revoke_work.decision,
                PRINCIPAL,
                ACTION,
                PURPOSE,
                DESTINATION,
                PROTOCOL_PORT,
                "2026-08-28T03:00:00Z",
            ),
            Err(EgressPermitError::PermitRevoked)
        ));
        crate::integrity::verify(&store.connection).unwrap();
        crate::authority_security_v2::verify(&store.connection).unwrap();
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn bounded_use_limit_exhausts_without_scope_widening() {
        let (runtime, authority) = authority();
        let mut store = EgressPermitStore::open(&authority).unwrap();
        install_policy_and_parent_lease(&mut store.connection);
        let issue = prepared(Some(2));
        let work = install_mutation_work(
            &mut store.connection,
            1,
            21,
            EGRESS_PERMIT_ISSUE_ACTION,
            issue.resource(),
            issue.intent_digest(),
        );
        let permit = store
            .issue(issue, work.decision, work.approval, work.effect)
            .unwrap();
        let decision = install_use_decision(&mut store.connection, 3, 92);

        let first = store
            .authorize_use(
                permit.permit_id,
                decision,
                PRINCIPAL,
                ACTION,
                PURPOSE,
                DESTINATION,
                PROTOCOL_PORT,
                "2026-08-28T02:00:00Z",
            )
            .unwrap();
        assert_eq!(first.uses_consumed, 1);
        let second = store
            .authorize_use(
                permit.permit_id,
                decision,
                PRINCIPAL,
                ACTION,
                PURPOSE,
                DESTINATION,
                PROTOCOL_PORT,
                "2026-08-28T02:01:00Z",
            )
            .unwrap();
        assert_eq!(second.uses_consumed, 2);
        assert_eq!(second.status, "exhausted");
        assert!(matches!(
            store.authorize_use(
                permit.permit_id,
                decision,
                PRINCIPAL,
                ACTION,
                PURPOSE,
                DESTINATION,
                PROTOCOL_PORT,
                "2026-08-28T02:02:00Z",
            ),
            Err(EgressPermitError::PermitUsageExhausted)
        ));
        assert!(matches!(
            store.authorize_use(
                permit.permit_id,
                decision,
                PRINCIPAL,
                ACTION,
                PURPOSE,
                "https://other.invalid",
                PROTOCOL_PORT,
                "2026-08-28T02:02:00Z",
            ),
            Err(EgressPermitError::PermitUsageExhausted | EgressPermitError::PermitScopeMismatch)
        ));
        crate::authority_security_v2::verify(&store.connection).unwrap();
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
