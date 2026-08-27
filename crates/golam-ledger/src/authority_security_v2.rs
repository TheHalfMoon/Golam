#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use golam_core::{CanonicalEncoder, CoreError};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, params};
#[cfg(test)]
use rusqlite::Transaction;

const CHAIN_NAME: &str = "authority-security-v2";
const KEY_DOMAIN: &[u8] = b"golam:authority-security-v2:key:v1";
const PAYLOAD_DOMAIN: &[u8] = b"golam:authority-security-v2:payload:v1";
const RECORD_DOMAIN: &[u8] = b"golam:authority-security-v2:record:v1";

type SnapshotKey = (String, Vec<u8>);
type LatestSnapshots = HashMap<SnapshotKey, [u8; 32]>;
type SourceRow = (Vec<u8>, Vec<u8>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProtectedSourceKind {
    PrincipalRecord,
    PolicyBundle,
    ActivePolicy,
    CapabilityLease,
    CapabilityRevocation,
    Approval,
    ApprovalConsumption,
    TaintAttestation,
    VerifierRule,
    SecretRecord,
    SecretVersion,
    SecretHandle,
    SecretUseRecord,
    EgressPermit,
    SandboxProfile,
    SandboxAdmission,
    AuthorizationDecisionV2,
}

impl ProtectedSourceKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::PrincipalRecord => "principal_record",
            Self::PolicyBundle => "policy_bundle",
            Self::ActivePolicy => "active_policy",
            Self::CapabilityLease => "capability_lease",
            Self::CapabilityRevocation => "capability_revocation",
            Self::Approval => "approval",
            Self::ApprovalConsumption => "approval_consumption",
            Self::TaintAttestation => "taint_attestation",
            Self::VerifierRule => "verifier_rule",
            Self::SecretRecord => "secret_record",
            Self::SecretVersion => "secret_version",
            Self::SecretHandle => "secret_handle",
            Self::SecretUseRecord => "secret_use_record",
            Self::EgressPermit => "egress_permit",
            Self::SandboxProfile => "sandbox_profile",
            Self::SandboxAdmission => "sandbox_admission",
            Self::AuthorizationDecisionV2 => "authorization_decision_v2",
        }
    }
}

#[cfg(test)]
pub(crate) enum ProtectedSourceKey<'a> {
    Text(&'a str),
}

#[cfg(test)]
impl ProtectedSourceKey<'_> {
    fn values(&self) -> Vec<Value> {
        match self {
            Self::Text(value) => vec![Value::Text((*value).to_owned())],
        }
    }
}

#[derive(Debug)]
pub(crate) enum AuthoritySecurityV2Error {
    Sqlite(rusqlite::Error),
    Core(CoreError),
    InvalidSource(&'static str),
    Coverage(&'static str),
    Integrity(&'static str),
    SequenceOverflow,
}

impl fmt::Display for AuthoritySecurityV2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "authority-security-v2 sqlite error: {error}"),
            Self::Core(error) => write!(f, "authority-security-v2 encoding error: {error}"),
            Self::InvalidSource(reason) => {
                write!(f, "authority-security-v2 invalid source: {reason}")
            }
            Self::Coverage(reason) => {
                write!(f, "authority-security-v2 coverage gap: {reason}")
            }
            Self::Integrity(reason) => {
                write!(f, "authority-security-v2 integrity failure: {reason}")
            }
            Self::SequenceOverflow => f.write_str("authority-security-v2 sequence overflow"),
        }
    }
}

impl Error for AuthoritySecurityV2Error {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::InvalidSource(_)
            | Self::Coverage(_)
            | Self::Integrity(_)
            | Self::SequenceOverflow => None,
        }
    }
}

impl From<rusqlite::Error> for AuthoritySecurityV2Error {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for AuthoritySecurityV2Error {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

#[derive(Clone, Copy)]
struct SourceSpec {
    kind: ProtectedSourceKind,
    key_columns: usize,
    query: &'static str,
}

const SOURCE_SPECS: &[SourceSpec] = &[
    SourceSpec {
        kind: ProtectedSourceKind::PrincipalRecord,
        key_columns: 1,
        query: "SELECT principal_id, principal_kind, owner_principal, status, attributes_version, created_global_seq, revoked_at FROM principal_records ORDER BY principal_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::PolicyBundle,
        key_columns: 1,
        query: "SELECT policy_bundle_id, version, schema_version, canonical_policy_bytes, bundle_hash, created_by, created_global_seq, validation_status FROM policy_bundles ORDER BY policy_bundle_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::ActivePolicy,
        key_columns: 1,
        query: "SELECT singleton_id, policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq FROM active_policy ORDER BY singleton_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::CapabilityLease,
        key_columns: 1,
        query: "SELECT lease_id, principal_id, parent_lease_id, actions_scope, resources_scope, context_constraints, issued_by, issued_global_seq, not_before, expires_at, generation, status, authority_digest FROM capability_leases ORDER BY lease_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::CapabilityRevocation,
        key_columns: 1,
        query: "SELECT revocation_id, lease_id, revoked_by, reason_code, revoked_global_seq, revoked_at FROM capability_revocations ORDER BY revocation_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::Approval,
        key_columns: 1,
        query: "SELECT approval_id, class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at FROM approvals ORDER BY approval_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::ApprovalConsumption,
        key_columns: 1,
        query: "SELECT consumption_id, approval_id, effect_or_operation_id, reserved_global_seq, consumed_global_seq, state FROM approval_consumptions ORDER BY consumption_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::TaintAttestation,
        key_columns: 1,
        query: "SELECT attestation_id, source_artifact_ids, source_labels, result_artifact_id, result_labels, mechanism, rule_id, principal, evidence_hash, created_global_seq FROM taint_attestations ORDER BY attestation_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::VerifierRule,
        key_columns: 1,
        query: "SELECT rule_id, kind, version, authority_source_binding, allowed_downgrades, registered_by, status, created_global_seq FROM verifier_rules ORDER BY rule_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::SecretRecord,
        key_columns: 1,
        query: "SELECT secret_id, classification, owner_principal, current_version, status, created_global_seq, revoked_at FROM secret_records ORDER BY secret_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::SecretVersion,
        key_columns: 2,
        query: "SELECT secret_id, version, ciphertext, nonce_or_algorithm_metadata, associated_data_hash, created_global_seq, rotated_from, retired_at FROM secret_versions ORDER BY secret_id, version",
    },
    SourceSpec {
        kind: ProtectedSourceKind::SecretHandle,
        key_columns: 1,
        query: "SELECT handle_id, secret_id, version_constraint, purpose_scope, expires_at FROM secret_handles ORDER BY handle_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::SecretUseRecord,
        key_columns: 1,
        query: "SELECT use_id, handle_id, principal, purpose, destination_or_process, mode, approval_id, decision_id, created_global_seq FROM secret_use_records ORDER BY use_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::EgressPermit,
        key_columns: 1,
        query: "SELECT permit_id, principal_or_process, action, purpose, destination_scope, protocol_port_scope, taint_digest, secret_handle_id, parent_lease_id, issued_at, expires_at, usage_limit, status FROM egress_permits ORDER BY permit_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::SandboxProfile,
        key_columns: 2,
        query: "SELECT profile_id, version, class, filesystem_read_roots, filesystem_write_roots, network_rule, environment_allowlist, spawn_rule, cpu_limit, memory_limit, time_limit, output_limit, device_allowlist, ipc_allowlist, inherited_handle_rules, platform_requirements, status FROM sandbox_profiles ORDER BY profile_id, version",
    },
    SourceSpec {
        kind: ProtectedSourceKind::SandboxAdmission,
        key_columns: 1,
        query: "SELECT admission_id, profile_id, profile_version, principal_or_process, lease_id, decision_id, egress_permit_id, resolved_launch_plan_hash, platform_executor, created_global_seq FROM sandbox_admissions ORDER BY admission_id",
    },
    SourceSpec {
        kind: ProtectedSourceKind::AuthorizationDecisionV2,
        key_columns: 1,
        query: "SELECT decision_id, principal, action, resource, context_hash, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, decision, reason_code, global_seq, authority_evidence_version FROM authorization_decisions WHERE authority_evidence_version >= 2 ORDER BY decision_id",
    },
];

#[cfg(test)]
pub(crate) fn append_current_snapshot(
    transaction: &Transaction<'_>,
    kind: ProtectedSourceKind,
    key: ProtectedSourceKey<'_>,
) -> Result<(), AuthoritySecurityV2Error> {
    let key_values = key.values();
    let record_id = record_id(kind, &key_values)?;
    let payload = current_payload_for_record(transaction, kind, &record_id)?;
    append_snapshot(transaction, kind, &record_id, &payload)
}

pub(crate) fn verify(connection: &Connection) -> Result<(), AuthoritySecurityV2Error> {
    ensure_table_exists(connection)?;
    let latest = verify_chain(connection)?;
    verify_current_sources(connection, &latest)
}

fn ensure_table_exists(connection: &Connection) -> Result<(), AuthoritySecurityV2Error> {
    let exists = connection
        .query_row(
            "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = 'authority_security_audit_v2' LIMIT 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if exists {
        Ok(())
    } else {
        Err(AuthoritySecurityV2Error::Integrity(
            "authority_security_audit_v2 table is missing",
        ))
    }
}

fn verify_chain(connection: &Connection) -> Result<LatestSnapshots, AuthoritySecurityV2Error> {
    let mut statement = connection.prepare(
        "SELECT audit_seq, record_kind, record_id, payload_bytes, payload_hash, previous_hash, record_hash FROM authority_security_audit_v2 ORDER BY audit_seq ASC",
    )?;
    let mut rows = statement.query([])?;
    let mut expected_seq = 1_u64;
    let mut previous_hash = None;
    let mut last = None;
    let mut latest = HashMap::new();
    while let Some(row) = rows.next()? {
        let audit_seq = u64::try_from(row.get::<_, i64>(0)?)
            .map_err(|_| AuthoritySecurityV2Error::Integrity("negative audit sequence"))?;
        if audit_seq != expected_seq {
            return Err(AuthoritySecurityV2Error::Integrity(
                "audit sequence is not contiguous",
            ));
        }
        let kind: String = row.get(1)?;
        if source_spec_by_name(&kind).is_none() {
            return Err(AuthoritySecurityV2Error::Integrity(
                "unknown protected source kind",
            ));
        }
        let record_id: Vec<u8> = row.get(2)?;
        let payload: Vec<u8> = row.get(3)?;
        let stored_payload_hash = hash_from_vec(row.get(4)?)?;
        let stored_previous_hash = optional_hash(row.get(5)?)?;
        let stored_record_hash = hash_from_vec(row.get(6)?)?;
        if stored_previous_hash != previous_hash {
            return Err(AuthoritySecurityV2Error::Integrity(
                "previous hash does not match chain head",
            ));
        }
        let computed_payload_hash = *blake3::hash(&payload).as_bytes();
        if computed_payload_hash != stored_payload_hash {
            return Err(AuthoritySecurityV2Error::Integrity(
                "snapshot payload hash mismatch",
            ));
        }
        let computed_record_hash = audit_record_hash(
            audit_seq,
            &kind,
            &record_id,
            stored_payload_hash,
            stored_previous_hash,
        )?;
        if computed_record_hash != stored_record_hash {
            return Err(AuthoritySecurityV2Error::Integrity(
                "snapshot record hash mismatch",
            ));
        }
        latest.insert((kind, record_id), stored_payload_hash);
        previous_hash = Some(stored_record_hash);
        last = Some((audit_seq, stored_record_hash));
        expected_seq = expected_seq
            .checked_add(1)
            .ok_or(AuthoritySecurityV2Error::SequenceOverflow)?;
    }
    drop(rows);
    drop(statement);
    verify_head(connection, last)?;
    Ok(latest)
}

fn verify_current_sources(
    connection: &Connection,
    latest: &LatestSnapshots,
) -> Result<(), AuthoritySecurityV2Error> {
    let mut current = HashSet::new();
    for spec in SOURCE_SPECS {
        for (record_id, payload) in source_rows(connection, *spec)? {
            let key = (spec.kind.as_str().to_owned(), record_id);
            let payload_hash = *blake3::hash(&payload).as_bytes();
            match latest.get(&key) {
                Some(stored) if *stored == payload_hash => {}
                Some(_) => {
                    return Err(AuthoritySecurityV2Error::Integrity(
                        "protected source differs from latest authenticated snapshot",
                    ));
                }
                None => {
                    return Err(AuthoritySecurityV2Error::Coverage(
                        "protected source is missing an authenticated snapshot",
                    ));
                }
            }
            current.insert(key);
        }
    }
    if latest.keys().any(|key| !current.contains(key)) {
        return Err(AuthoritySecurityV2Error::Integrity(
            "authenticated snapshot has no current protected source",
        ));
    }
    Ok(())
}

#[cfg(test)]
fn current_payload_for_record(
    connection: &Connection,
    kind: ProtectedSourceKind,
    record_id: &[u8],
) -> Result<Vec<u8>, AuthoritySecurityV2Error> {
    let spec = source_spec(kind);
    let mut found = None;
    for (candidate_id, payload) in source_rows(connection, spec)? {
        if candidate_id == record_id {
            if found.is_some() {
                return Err(AuthoritySecurityV2Error::Integrity(
                    "protected source key is not unique",
                ));
            }
            found = Some(payload);
        }
    }
    found.ok_or(AuthoritySecurityV2Error::Coverage(
        "protected source row does not exist",
    ))
}

fn source_rows(
    connection: &Connection,
    spec: SourceSpec,
) -> Result<Vec<SourceRow>, AuthoritySecurityV2Error> {
    let mut statement = connection.prepare(spec.query)?;
    let column_count = statement.column_count();
    if spec.key_columns == 0 || spec.key_columns > column_count {
        return Err(AuthoritySecurityV2Error::InvalidSource(
            "invalid protected-source key shape",
        ));
    }
    let rows = statement.query_map([], |row| {
        let mut values = Vec::with_capacity(column_count);
        for index in 0..column_count {
            values.push(row.get::<_, Value>(index)?);
        }
        Ok(values)
    })?;
    let mut result = Vec::new();
    for values in rows {
        let values = values?;
        let record_id = record_id(spec.kind, &values[..spec.key_columns])?;
        let payload = canonical_payload(spec.kind, &values)?;
        result.push((record_id, payload));
    }
    Ok(result)
}

#[cfg(test)]
fn source_spec(kind: ProtectedSourceKind) -> SourceSpec {
    SOURCE_SPECS
        .iter()
        .copied()
        .find(|spec| spec.kind == kind)
        .expect("all ProtectedSourceKind variants have a SourceSpec")
}

fn source_spec_by_name(kind: &str) -> Option<SourceSpec> {
    SOURCE_SPECS
        .iter()
        .copied()
        .find(|spec| spec.kind.as_str() == kind)
}

fn record_id(
    kind: ProtectedSourceKind,
    key_values: &[Value],
) -> Result<Vec<u8>, AuthoritySecurityV2Error> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(KEY_DOMAIN)?;
    encoder.push_bytes(kind.as_str().as_bytes())?;
    encoder.push_u64(
        u64::try_from(key_values.len())
            .map_err(|_| AuthoritySecurityV2Error::InvalidSource("too many key fields"))?,
    );
    for value in key_values {
        encode_value(&mut encoder, value)?;
    }
    Ok(blake3::hash(&encoder.finish()).as_bytes().to_vec())
}

fn canonical_payload(
    kind: ProtectedSourceKind,
    values: &[Value],
) -> Result<Vec<u8>, AuthoritySecurityV2Error> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(PAYLOAD_DOMAIN)?;
    encoder.push_bytes(kind.as_str().as_bytes())?;
    encoder.push_u64(
        u64::try_from(values.len())
            .map_err(|_| AuthoritySecurityV2Error::InvalidSource("too many source fields"))?,
    );
    for value in values {
        encode_value(&mut encoder, value)?;
    }
    Ok(encoder.finish())
}

fn encode_value(
    encoder: &mut CanonicalEncoder,
    value: &Value,
) -> Result<(), AuthoritySecurityV2Error> {
    match value {
        Value::Null => encoder.push_u8(0),
        Value::Integer(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value.to_be_bytes())?;
        }
        Value::Real(_) => {
            return Err(AuthoritySecurityV2Error::InvalidSource(
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

#[cfg(test)]
fn append_snapshot(
    transaction: &Transaction<'_>,
    kind: ProtectedSourceKind,
    record_id: &[u8],
    payload: &[u8],
) -> Result<(), AuthoritySecurityV2Error> {
    let previous = transaction
        .query_row(
            "SELECT last_global_seq, last_hash FROM audit_chain_heads WHERE chain_name = ?1",
            params![CHAIN_NAME],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;
    let (audit_seq, previous_hash) = match previous {
        Some((seq, hash)) => {
            let seq = u64::try_from(seq)
                .map_err(|_| AuthoritySecurityV2Error::Integrity("negative chain head sequence"))?;
            (
                seq.checked_add(1)
                    .ok_or(AuthoritySecurityV2Error::SequenceOverflow)?,
                Some(hash_from_vec(hash)?),
            )
        }
        None => (1, None),
    };
    let payload_hash = *blake3::hash(payload).as_bytes();
    let record_hash = audit_record_hash(
        audit_seq,
        kind.as_str(),
        record_id,
        payload_hash,
        previous_hash,
    )?;
    transaction.execute(
        "INSERT INTO authority_security_audit_v2 (audit_seq, record_kind, record_id, payload_bytes, payload_hash, previous_hash, record_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            i64::try_from(audit_seq).map_err(|_| AuthoritySecurityV2Error::SequenceOverflow)?,
            kind.as_str(),
            record_id,
            payload,
            &payload_hash[..],
            previous_hash.map(|hash| hash.to_vec()),
            &record_hash[..],
        ],
    )?;
    transaction.execute(
        "INSERT INTO audit_chain_heads (chain_name, last_global_seq, last_hash) VALUES (?1, ?2, ?3) ON CONFLICT(chain_name) DO UPDATE SET last_global_seq = excluded.last_global_seq, last_hash = excluded.last_hash",
        params![
            CHAIN_NAME,
            i64::try_from(audit_seq).map_err(|_| AuthoritySecurityV2Error::SequenceOverflow)?,
            &record_hash[..],
        ],
    )?;
    Ok(())
}

fn audit_record_hash(
    audit_seq: u64,
    kind: &str,
    record_id: &[u8],
    payload_hash: [u8; 32],
    previous_hash: Option<[u8; 32]>,
) -> Result<[u8; 32], AuthoritySecurityV2Error> {
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

fn verify_head(
    connection: &Connection,
    computed: Option<(u64, [u8; 32])>,
) -> Result<(), AuthoritySecurityV2Error> {
    let stored = connection
        .query_row(
            "SELECT last_global_seq, last_hash FROM audit_chain_heads WHERE chain_name = ?1",
            params![CHAIN_NAME],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?;
    let stored = match stored {
        Some((seq, hash)) => Some((
            u64::try_from(seq)
                .map_err(|_| AuthoritySecurityV2Error::Integrity("negative chain head sequence"))?,
            hash_from_vec(hash)?,
        )),
        None => None,
    };
    if stored == computed {
        Ok(())
    } else {
        Err(AuthoritySecurityV2Error::Integrity(
            "authority-security-v2 chain head mismatch",
        ))
    }
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], AuthoritySecurityV2Error> {
    value
        .try_into()
        .map_err(|_| AuthoritySecurityV2Error::Integrity("stored hash is not 32 bytes"))
}

fn optional_hash(value: Option<Vec<u8>>) -> Result<Option<[u8; 32]>, AuthoritySecurityV2Error> {
    value.map(hash_from_vec).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AuthorityStore, StorageError};
    use rusqlite::TransactionBehavior;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn db_path(label: &str) -> std::path::PathBuf {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "golam-authority-security-v2-{label}-{}-{t}-{n}.sqlite3",
            std::process::id()
        ))
    }

    fn insert_principal(connection: &Connection, status: &str) {
        connection
            .execute(
                "INSERT INTO principal_records (principal_id, principal_kind, owner_principal, status, attributes_version, created_global_seq, revoked_at) VALUES ('owner', 'local_owner', NULL, ?1, 1, 1, NULL)",
                params![status],
            )
            .unwrap();
    }

    #[test]
    fn protected_row_without_snapshot_fails_startup() {
        let path = db_path("missing");
        drop(AuthorityStore::open(&path).unwrap());
        let connection = Connection::open(&path).unwrap();
        insert_principal(&connection, "active");
        drop(connection);
        assert!(matches!(
            AuthorityStore::open(&path),
            Err(StorageError::IntegrityCheckFailed(_))
        ));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn authenticated_snapshot_detects_tampering() {
        let path = db_path("tamper");
        drop(AuthorityStore::open(&path).unwrap());
        let mut connection = Connection::open(&path).unwrap();
        insert_principal(&connection, "active");
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        append_current_snapshot(
            &transaction,
            ProtectedSourceKind::PrincipalRecord,
            ProtectedSourceKey::Text("owner"),
        )
        .unwrap();
        transaction.commit().unwrap();
        drop(connection);
        drop(AuthorityStore::open(&path).unwrap());

        let connection = Connection::open(&path).unwrap();
        connection
            .execute(
                "UPDATE principal_records SET principal_kind = 'tampered' WHERE principal_id = 'owner'",
                [],
            )
            .unwrap();
        drop(connection);
        assert!(matches!(
            AuthorityStore::open(&path),
            Err(StorageError::IntegrityCheckFailed(_))
        ));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn mutable_source_can_append_a_new_authenticated_snapshot() {
        let path = db_path("mutable");
        drop(AuthorityStore::open(&path).unwrap());
        let mut connection = Connection::open(&path).unwrap();
        insert_principal(&connection, "active");
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        append_current_snapshot(
            &transaction,
            ProtectedSourceKind::PrincipalRecord,
            ProtectedSourceKey::Text("owner"),
        )
        .unwrap();
        transaction.commit().unwrap();

        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "UPDATE principal_records SET status = 'revoked', revoked_at = '2026-08-27T00:00:00Z' WHERE principal_id = 'owner'",
                [],
            )
            .unwrap();
        append_current_snapshot(
            &transaction,
            ProtectedSourceKind::PrincipalRecord,
            ProtectedSourceKey::Text("owner"),
        )
        .unwrap();
        transaction.commit().unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM authority_security_audit_v2 WHERE record_kind = 'principal_record'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
        drop(connection);
        drop(AuthorityStore::open(&path).unwrap());
        fs::remove_file(path).unwrap();
    }
}
