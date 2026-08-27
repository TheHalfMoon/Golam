#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{CanonicalEncoder, CoreError};
use rusqlite::types::Value;
use rusqlite::{OptionalExtension, Params, Transaction, params};

const CHAIN_NAME: &str = "authority-security-v2";
const KEY_DOMAIN: &[u8] = b"golam:authority-security-v2:key:v1";
const PAYLOAD_DOMAIN: &[u8] = b"golam:authority-security-v2:payload:v1";
const RECORD_DOMAIN: &[u8] = b"golam:authority-security-v2:record:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProtectedMutationKind {
    PolicyBundle,
    ActivePolicy,
    CapabilityLease,
    CapabilityRevocation,
    AuthorizationDecisionV2,
    #[cfg(test)]
    Approval,
    ApprovalConsumption,
}

impl ProtectedMutationKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::PolicyBundle => "policy_bundle",
            Self::ActivePolicy => "active_policy",
            Self::CapabilityLease => "capability_lease",
            Self::CapabilityRevocation => "capability_revocation",
            Self::AuthorizationDecisionV2 => "authorization_decision_v2",
            #[cfg(test)]
            Self::Approval => "approval",
            Self::ApprovalConsumption => "approval_consumption",
        }
    }
}

#[derive(Debug)]
pub(crate) enum AuthoritySecurityWriteError {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    InvalidSource(&'static str),
    InvalidStoredHash,
    SequenceOverflow,
}

impl fmt::Display for AuthoritySecurityWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "authority-security write sqlite error: {error}"),
            Self::Core(error) => write!(f, "authority-security write encoding error: {error}"),
            Self::InvalidSource(reason) => {
                write!(f, "authority-security write invalid source: {reason}")
            }
            Self::InvalidStoredHash => {
                f.write_str("authority-security write encountered malformed stored hash")
            }
            Self::SequenceOverflow => f.write_str("authority-security write sequence overflow"),
        }
    }
}

impl Error for AuthoritySecurityWriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::InvalidSource(_) | Self::InvalidStoredHash | Self::SequenceOverflow => None,
        }
    }
}

impl From<rusqlite::Error> for AuthoritySecurityWriteError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for AuthoritySecurityWriteError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub(crate) fn append_policy_bundle_snapshot(
    transaction: &Transaction<'_>,
    policy_bundle_id: &[u8],
) -> Result<(), AuthoritySecurityWriteError> {
    let values = query_values(
        transaction,
        "SELECT policy_bundle_id, version, schema_version, canonical_policy_bytes, bundle_hash, created_by, created_global_seq, validation_status FROM policy_bundles WHERE policy_bundle_id = ?1",
        params![policy_bundle_id],
    )?;
    append_snapshot(transaction, ProtectedMutationKind::PolicyBundle, 1, &values)
}

pub(crate) fn append_active_policy_snapshot(
    transaction: &Transaction<'_>,
) -> Result<(), AuthoritySecurityWriteError> {
    let values = query_values(
        transaction,
        "SELECT singleton_id, policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq FROM active_policy WHERE singleton_id = 1",
        [],
    )?;
    append_snapshot(transaction, ProtectedMutationKind::ActivePolicy, 1, &values)
}

pub(crate) fn append_capability_lease_snapshot(
    transaction: &Transaction<'_>,
    lease_id: &[u8],
) -> Result<(), AuthoritySecurityWriteError> {
    let values = query_values(
        transaction,
        "SELECT lease_id, principal_id, parent_lease_id, actions_scope, resources_scope, context_constraints, issued_by, issued_global_seq, not_before, expires_at, generation, status, authority_digest FROM capability_leases WHERE lease_id = ?1",
        params![lease_id],
    )?;
    append_snapshot(
        transaction,
        ProtectedMutationKind::CapabilityLease,
        1,
        &values,
    )
}

pub(crate) fn append_capability_revocation_snapshot(
    transaction: &Transaction<'_>,
    revocation_id: &[u8],
) -> Result<(), AuthoritySecurityWriteError> {
    let values = query_values(
        transaction,
        "SELECT revocation_id, lease_id, revoked_by, reason_code, revoked_global_seq, revoked_at FROM capability_revocations WHERE revocation_id = ?1",
        params![revocation_id],
    )?;
    append_snapshot(
        transaction,
        ProtectedMutationKind::CapabilityRevocation,
        1,
        &values,
    )
}

pub(crate) fn append_authorization_decision_v2_snapshot(
    transaction: &Transaction<'_>,
    decision_id: &[u8],
) -> Result<(), AuthoritySecurityWriteError> {
    let values = query_values(
        transaction,
        "SELECT decision_id, principal, action, resource, context_hash, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, decision, reason_code, global_seq, authority_evidence_version FROM authorization_decisions WHERE decision_id = ?1 AND authority_evidence_version >= 2",
        params![decision_id],
    )?;
    append_snapshot(
        transaction,
        ProtectedMutationKind::AuthorizationDecisionV2,
        1,
        &values,
    )
}

#[cfg(test)]
pub(crate) fn append_approval_snapshot(
    transaction: &Transaction<'_>,
    approval_id: &[u8],
) -> Result<(), AuthoritySecurityWriteError> {
    let values = query_values(
        transaction,
        "SELECT approval_id, class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at FROM approvals WHERE approval_id = ?1",
        params![approval_id],
    )?;
    append_snapshot(transaction, ProtectedMutationKind::Approval, 1, &values)
}

pub(crate) fn append_approval_consumption_snapshot(
    transaction: &Transaction<'_>,
    consumption_id: &[u8],
) -> Result<(), AuthoritySecurityWriteError> {
    let values = query_values(
        transaction,
        "SELECT consumption_id, approval_id, effect_or_operation_id, reserved_global_seq, consumed_global_seq, state FROM approval_consumptions WHERE consumption_id = ?1",
        params![consumption_id],
    )?;
    append_snapshot(
        transaction,
        ProtectedMutationKind::ApprovalConsumption,
        1,
        &values,
    )
}

fn query_values<P: Params>(
    transaction: &Transaction<'_>,
    sql: &str,
    parameters: P,
) -> Result<Vec<Value>, AuthoritySecurityWriteError> {
    let mut statement = transaction.prepare(sql)?;
    let column_count = statement.column_count();
    let values = statement
        .query_row(parameters, |row| {
            let mut values = Vec::with_capacity(column_count);
            for index in 0..column_count {
                values.push(row.get::<_, Value>(index)?);
            }
            Ok(values)
        })
        .optional()?
        .ok_or(AuthoritySecurityWriteError::InvalidSource(
            "protected source row does not exist",
        ))?;
    Ok(values)
}

fn append_snapshot(
    transaction: &Transaction<'_>,
    kind: ProtectedMutationKind,
    key_columns: usize,
    values: &[Value],
) -> Result<(), AuthoritySecurityWriteError> {
    if key_columns == 0 || key_columns > values.len() {
        return Err(AuthoritySecurityWriteError::InvalidSource(
            "invalid protected-source key shape",
        ));
    }
    let record_id = record_id(kind, &values[..key_columns])?;
    let payload = canonical_payload(kind, values)?;
    let previous = transaction
        .query_row(
            "SELECT last_global_seq, last_hash FROM audit_chain_heads WHERE chain_name = ?1",
            params![CHAIN_NAME],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;
    let (audit_seq, previous_hash) = match previous {
        Some((sequence, hash)) => {
            let sequence = u64::try_from(sequence)
                .map_err(|_| AuthoritySecurityWriteError::SequenceOverflow)?;
            let previous_hash = hash_from_vec(hash)?;
            (
                sequence
                    .checked_add(1)
                    .ok_or(AuthoritySecurityWriteError::SequenceOverflow)?,
                Some(previous_hash),
            )
        }
        None => (1, None),
    };
    let payload_hash = *blake3::hash(&payload).as_bytes();
    let record_hash = audit_record_hash(
        audit_seq,
        kind.as_str(),
        &record_id,
        payload_hash,
        previous_hash,
    )?;
    transaction.execute(
        "INSERT INTO authority_security_audit_v2 (audit_seq, record_kind, record_id, payload_bytes, payload_hash, previous_hash, record_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            i64::try_from(audit_seq).map_err(|_| AuthoritySecurityWriteError::SequenceOverflow)?,
            kind.as_str(),
            &record_id,
            &payload,
            &payload_hash[..],
            previous_hash.map(|hash| hash.to_vec()),
            &record_hash[..],
        ],
    )?;
    transaction.execute(
        "INSERT INTO audit_chain_heads (chain_name, last_global_seq, last_hash) VALUES (?1, ?2, ?3) ON CONFLICT(chain_name) DO UPDATE SET last_global_seq = excluded.last_global_seq, last_hash = excluded.last_hash",
        params![
            CHAIN_NAME,
            i64::try_from(audit_seq).map_err(|_| AuthoritySecurityWriteError::SequenceOverflow)?,
            &record_hash[..],
        ],
    )?;
    Ok(())
}

fn record_id(
    kind: ProtectedMutationKind,
    key_values: &[Value],
) -> Result<Vec<u8>, AuthoritySecurityWriteError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KEY_DOMAIN)?;
    encoder.push_bytes(kind.as_str().as_bytes())?;
    encoder.push_u64(
        u64::try_from(key_values.len())
            .map_err(|_| AuthoritySecurityWriteError::InvalidSource("too many key fields"))?,
    );
    for value in key_values {
        encode_value(&mut encoder, value)?;
    }
    Ok(blake3::hash(&encoder.finish()).as_bytes().to_vec())
}

fn canonical_payload(
    kind: ProtectedMutationKind,
    values: &[Value],
) -> Result<Vec<u8>, AuthoritySecurityWriteError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(PAYLOAD_DOMAIN)?;
    encoder.push_bytes(kind.as_str().as_bytes())?;
    encoder.push_u64(
        u64::try_from(values.len())
            .map_err(|_| AuthoritySecurityWriteError::InvalidSource("too many source fields"))?,
    );
    for value in values {
        encode_value(&mut encoder, value)?;
    }
    Ok(encoder.finish())
}

fn encode_value(
    encoder: &mut CanonicalEncoder,
    value: &Value,
) -> Result<(), AuthoritySecurityWriteError> {
    match value {
        Value::Null => encoder.push_u8(0),
        Value::Integer(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value.to_be_bytes())?;
        }
        Value::Real(_) => {
            return Err(AuthoritySecurityWriteError::InvalidSource(
                "floating-point protected authority fields are forbidden",
            ));
        }
        Value::Text(value) => {
            encoder.push_u8(2);
            encoder.push_bytes(value.as_bytes())?;
        }
        Value::Blob(value) => {
            encoder.push_u8(3);
            encoder.push_bytes(value)?;
        }
    }
    Ok(())
}

fn audit_record_hash(
    audit_seq: u64,
    kind: &str,
    record_id: &[u8],
    payload_hash: [u8; 32],
    previous_hash: Option<[u8; 32]>,
) -> Result<[u8; 32], AuthoritySecurityWriteError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(RECORD_DOMAIN)?;
    encoder.push_u64(audit_seq);
    encoder.push_bytes(kind.as_bytes())?;
    encoder.push_bytes(record_id)?;
    encoder.push_bytes(&payload_hash)?;
    match previous_hash {
        Some(hash) => {
            encoder.push_u8(1);
            encoder.push_bytes(&hash)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(*blake3::hash(&encoder.finish()).as_bytes())
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], AuthoritySecurityWriteError> {
    value
        .try_into()
        .map_err(|_| AuthoritySecurityWriteError::InvalidStoredHash)
}
