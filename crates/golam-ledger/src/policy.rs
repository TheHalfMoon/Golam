#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::authority_security_write::{
    append_active_policy_snapshot, append_approval_consumption_snapshot,
    append_policy_bundle_snapshot,
};
use crate::storage::{AuthorityStore, StorageError};

const POLICY_BUNDLE_DOMAIN: &[u8] = b"golam:policy-bundle:v1";
const POLICY_BUNDLE_ID_DOMAIN: &[u8] = b"golam:policy-bundle-id:v1";
const APPROVAL_CONSUMPTION_ID_DOMAIN: &[u8] = b"golam:policy-approval-consumption:v1";

pub const POLICY_STAGE_ACTION: &str = "policy.stage";
pub const POLICY_ACTIVATE_ACTION: &str = "policy.activate";
pub const POLICY_MUTATION_RISK_CLASS: &str = "policy_mutation";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PolicyBundleId(pub [u8; 16]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedPolicyBundle {
    policy_bundle_id: PolicyBundleId,
    schema_version: u64,
    canonical_bundle_bytes: Vec<u8>,
    bundle_hash: [u8; 32],
}

impl PreparedPolicyBundle {
    pub const fn policy_bundle_id(&self) -> PolicyBundleId {
        self.policy_bundle_id
    }

    pub const fn schema_version(&self) -> u64 {
        self.schema_version
    }

    pub const fn bundle_hash(&self) -> [u8; 32] {
        self.bundle_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyBundleRecord {
    pub policy_bundle_id: PolicyBundleId,
    pub version: u64,
    pub schema_version: u64,
    pub bundle_hash: [u8; 32],
    pub created_by: String,
    pub created_global_seq: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivePolicyRecord {
    pub policy_bundle_id: PolicyBundleId,
    pub bundle_hash: [u8; 32],
    pub activated_by: String,
    pub activation_effect_id: EffectId,
    pub activated_global_seq: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyActivationRecord {
    pub previous_policy_bundle_id: Option<PolicyBundleId>,
    pub active: ActivePolicyRecord,
}

#[derive(Debug)]
pub enum PolicyLifecycleError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidSchemaVersion,
    IntegerOverflow,
    MissingAuthorityDecision,
    AuthorityDecisionMismatch,
    StaleAuthorityDecision,
    DuplicateBundle,
    BundleNotFound,
    BundleNotValidated,
    InvalidStoredBundle,
    EffectNotFound,
    EffectMismatch,
    ApprovalNotFound,
    ApprovalMismatch,
    ApprovalAlreadyUsed,
    InvalidStoredRecord,
}

impl fmt::Display for PolicyLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "policy lifecycle authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "policy lifecycle sqlite error: {error}"),
            Self::Core(error) => write!(f, "policy lifecycle canonical encoding error: {error}"),
            Self::Integrity(error) => write!(f, "policy lifecycle canonical integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "policy lifecycle authority-security error: {error}")
            }
            Self::InvalidSchemaVersion => f.write_str("policy schema version must be non-zero"),
            Self::IntegerOverflow => f.write_str("policy lifecycle integer conversion overflow"),
            Self::MissingAuthorityDecision => {
                f.write_str("policy mutation has no durable authorization decision")
            }
            Self::AuthorityDecisionMismatch => {
                f.write_str("policy mutation authorization decision does not match the exact action/resource")
            }
            Self::StaleAuthorityDecision => {
                f.write_str("policy mutation authorization decision is not the latest canonical decision")
            }
            Self::DuplicateBundle => f.write_str("immutable policy bundle already exists"),
            Self::BundleNotFound => f.write_str("policy bundle does not exist"),
            Self::BundleNotValidated => f.write_str("policy bundle is not validated"),
            Self::InvalidStoredBundle => f.write_str("stored policy bundle integrity is invalid"),
            Self::EffectNotFound => f.write_str("policy activation effect does not exist"),
            Self::EffectMismatch => {
                f.write_str("policy activation effect is not exact, authorized at-most-once policy work")
            }
            Self::ApprovalNotFound => f.write_str("policy activation approval does not exist"),
            Self::ApprovalMismatch => {
                f.write_str("policy activation approval does not match the exact effect/action/resource")
            }
            Self::ApprovalAlreadyUsed => {
                f.write_str("policy activation one-shot approval was already consumed")
            }
            Self::InvalidStoredRecord => f.write_str("stored policy lifecycle record is malformed"),
        }
    }
}

impl Error for PolicyLifecycleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for PolicyLifecycleError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for PolicyLifecycleError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for PolicyLifecycleError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub fn prepare_policy_bundle(
    schema_version: u64,
    policy_source: &str,
    schema_source: &str,
) -> Result<PreparedPolicyBundle, PolicyLifecycleError> {
    if schema_version == 0 {
        return Err(PolicyLifecycleError::InvalidSchemaVersion);
    }
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(POLICY_BUNDLE_DOMAIN)?;
    encoder.push_u64(schema_version);
    encoder.push_bytes(policy_source.as_bytes())?;
    encoder.push_bytes(schema_source.as_bytes())?;
    let canonical_bundle_bytes = encoder.finish();
    let bundle_hash = *blake3::hash(&canonical_bundle_bytes).as_bytes();
    let policy_bundle_id = policy_bundle_id(bundle_hash);
    Ok(PreparedPolicyBundle {
        policy_bundle_id,
        schema_version,
        canonical_bundle_bytes,
        bundle_hash,
    })
}

pub fn policy_bundle_resource(policy_bundle_id: PolicyBundleId) -> String {
    let mut resource = String::from("policy-bundle:");
    for byte in policy_bundle_id.0 {
        use std::fmt::Write as _;
        let _ = write!(resource, "{byte:02x}");
    }
    resource
}

pub struct PolicyStore {
    connection: Connection,
}

impl PolicyStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, PolicyLifecycleError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn stage_prepared(
        &mut self,
        prepared: PreparedPolicyBundle,
        authority_decision_id: [u8; 16],
    ) -> Result<PolicyBundleRecord, PolicyLifecycleError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let resource = policy_bundle_resource(prepared.policy_bundle_id);
        let authority = verify_current_authority(
            &transaction,
            authority_decision_id,
            POLICY_STAGE_ACTION,
            &resource,
        )?;
        let duplicate = transaction
            .query_row(
                "SELECT 1 FROM policy_bundles WHERE policy_bundle_id = ?1 OR bundle_hash = ?2 LIMIT 1",
                params![&prepared.policy_bundle_id.0[..], &prepared.bundle_hash[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if duplicate {
            return Err(PolicyLifecycleError::DuplicateBundle);
        }
        let current_version: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM policy_bundles",
            [],
            |row| row.get(0),
        )?;
        let version = u64::try_from(current_version)
            .map_err(|_| PolicyLifecycleError::InvalidStoredRecord)?
            .checked_add(1)
            .ok_or(PolicyLifecycleError::IntegerOverflow)?;
        transaction.execute(
            "INSERT INTO policy_bundles (policy_bundle_id, version, schema_version, canonical_policy_bytes, bundle_hash, created_by, created_global_seq, validation_status) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'validated')",
            params![
                &prepared.policy_bundle_id.0[..],
                to_i64(version)?,
                to_i64(prepared.schema_version)?,
                &prepared.canonical_bundle_bytes,
                &prepared.bundle_hash[..],
                &authority.principal,
                to_i64(authority.global_seq)?,
            ],
        )?;
        append_policy_bundle_snapshot(&transaction, &prepared.policy_bundle_id.0)
            .map_err(|error| PolicyLifecycleError::AuthoritySecurity(error.to_string()))?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| PolicyLifecycleError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;
        Ok(PolicyBundleRecord {
            policy_bundle_id: prepared.policy_bundle_id,
            version,
            schema_version: prepared.schema_version,
            bundle_hash: prepared.bundle_hash,
            created_by: authority.principal,
            created_global_seq: authority.global_seq,
        })
    }

    pub fn activate(
        &mut self,
        policy_bundle_id: PolicyBundleId,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        activation_effect_id: EffectId,
    ) -> Result<PolicyActivationRecord, PolicyLifecycleError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let bundle = load_bundle(&transaction, policy_bundle_id)?;
        let resource = policy_bundle_resource(policy_bundle_id);
        let authority = verify_current_authority(
            &transaction,
            authority_decision_id,
            POLICY_ACTIVATE_ACTION,
            &resource,
        )?;
        verify_activation_effect(&transaction, activation_effect_id, &resource)?;
        verify_activation_approval(&transaction, approval_id, activation_effect_id, &resource)?;
        let previous_policy_bundle_id = transaction
            .query_row(
                "SELECT policy_bundle_id FROM active_policy WHERE singleton_id = 1",
                [],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()?
            .map(policy_bundle_id_from_vec)
            .transpose()?;
        transaction.execute(
            "INSERT INTO active_policy (singleton_id, policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq) VALUES (1, ?1, ?2, ?3, ?4, ?5) ON CONFLICT(singleton_id) DO UPDATE SET policy_bundle_id = excluded.policy_bundle_id, bundle_hash = excluded.bundle_hash, activated_by = excluded.activated_by, activation_effect_id = excluded.activation_effect_id, activated_global_seq = excluded.activated_global_seq",
            params![
                &policy_bundle_id.0[..],
                &bundle.bundle_hash[..],
                &authority.principal,
                &activation_effect_id.0.to_be_bytes()[..],
                to_i64(authority.global_seq)?,
            ],
        )?;
        append_active_policy_snapshot(&transaction)
            .map_err(|error| PolicyLifecycleError::AuthoritySecurity(error.to_string()))?;
        consume_activation_approval(
            &transaction,
            approval_id,
            activation_effect_id,
            authority.global_seq,
        )?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| PolicyLifecycleError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;
        Ok(PolicyActivationRecord {
            previous_policy_bundle_id,
            active: ActivePolicyRecord {
                policy_bundle_id,
                bundle_hash: bundle.bundle_hash,
                activated_by: authority.principal,
                activation_effect_id,
                activated_global_seq: authority.global_seq,
            },
        })
    }

    pub fn active(&self) -> Result<Option<ActivePolicyRecord>, PolicyLifecycleError> {
        let row = self
            .connection
            .query_row(
                "SELECT policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq FROM active_policy WHERE singleton_id = 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .optional()?;
        row.map(|row| {
            Ok(ActivePolicyRecord {
                policy_bundle_id: policy_bundle_id_from_vec(row.0)?,
                bundle_hash: hash_from_vec(row.1)?,
                activated_by: row.2,
                activation_effect_id: EffectId(id_from_vec(row.3)?),
                activated_global_seq: seq_from_i64(row.4)?,
            })
        })
        .transpose()
    }
}

#[derive(Debug)]
struct AuthorityEvidence {
    principal: String,
    global_seq: u64,
}

#[derive(Debug)]
struct StoredBundle {
    bundle_hash: [u8; 32],
}

fn verify_transaction_integrity(
    transaction: &Transaction<'_>,
) -> Result<(), PolicyLifecycleError> {
    crate::integrity::verify(transaction)
        .map_err(|error| PolicyLifecycleError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| PolicyLifecycleError::AuthoritySecurity(error.to_string()))
}

fn verify_current_authority(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    expected_action: &str,
    expected_resource: &str,
) -> Result<AuthorityEvidence, PolicyLifecycleError> {
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
        .ok_or(PolicyLifecycleError::MissingAuthorityDecision)?;
    if row.1 != expected_action || row.2 != expected_resource || row.3 != "allow" {
        return Err(PolicyLifecycleError::AuthorityDecisionMismatch);
    }
    let global_seq = seq_from_i64(row.4)?;
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (SELECT global_seq FROM session_events UNION ALL SELECT global_seq FROM effect_transitions UNION ALL SELECT global_seq FROM authorization_decisions)",
        [],
        |row| row.get(0),
    )?;
    if global_seq != seq_from_i64(latest)? {
        return Err(PolicyLifecycleError::StaleAuthorityDecision);
    }
    Ok(AuthorityEvidence {
        principal: row.0,
        global_seq,
    })
}

fn load_bundle(
    transaction: &Transaction<'_>,
    policy_bundle_id: PolicyBundleId,
) -> Result<StoredBundle, PolicyLifecycleError> {
    let row = transaction
        .query_row(
            "SELECT canonical_policy_bytes, bundle_hash, validation_status FROM policy_bundles WHERE policy_bundle_id = ?1",
            params![&policy_bundle_id.0[..]],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or(PolicyLifecycleError::BundleNotFound)?;
    if row.2 != "validated" {
        return Err(PolicyLifecycleError::BundleNotValidated);
    }
    let bundle_hash = hash_from_vec(row.1)?;
    if *blake3::hash(&row.0).as_bytes() != bundle_hash
        || policy_bundle_id_from_hash(bundle_hash) != policy_bundle_id
    {
        return Err(PolicyLifecycleError::InvalidStoredBundle);
    }
    Ok(StoredBundle { bundle_hash })
}

fn verify_activation_effect(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    expected_resource: &str,
) -> Result<(), PolicyLifecycleError> {
    let row = transaction
        .query_row(
            "SELECT i.action, i.resource, i.risk_class, i.execution_semantics, t.to_state FROM effect_intents i JOIN effect_transitions t ON t.effect_id = i.effect_id WHERE i.effect_id = ?1 AND t.global_seq = (SELECT MAX(t2.global_seq) FROM effect_transitions t2 WHERE t2.effect_id = i.effect_id)",
            params![&effect_id.0.to_be_bytes()[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .optional()?
        .ok_or(PolicyLifecycleError::EffectNotFound)?;
    if row.0 != POLICY_ACTIVATE_ACTION
        || row.1 != expected_resource
        || row.2 != POLICY_MUTATION_RISK_CLASS
        || row.3 != "at_most_once"
        || row.4 != "authorized"
    {
        return Err(PolicyLifecycleError::EffectMismatch);
    }
    Ok(())
}

fn verify_activation_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    expected_resource: &str,
) -> Result<(), PolicyLifecycleError> {
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
        .ok_or(PolicyLifecycleError::ApprovalNotFound)?;
    if row.0 != "ONCE"
        || row.1.as_slice() != POLICY_ACTIVATE_ACTION.as_bytes()
        || row.2.as_slice() != expected_resource.as_bytes()
        || row.3.as_deref() != Some(effect_id.0.to_be_bytes().as_slice())
        || row.4.is_some()
        || row.5 != POLICY_MUTATION_RISK_CLASS
        || row.6.is_some()
        || row.7 != Some(1)
        || row.8.is_some()
    {
        return Err(PolicyLifecycleError::ApprovalMismatch);
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
        return Err(PolicyLifecycleError::ApprovalAlreadyUsed);
    }
    Ok(())
}

fn consume_activation_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    global_seq: u64,
) -> Result<(), PolicyLifecycleError> {
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
        .map_err(|error| PolicyLifecycleError::AuthoritySecurity(error.to_string()))
}

fn policy_bundle_id(bundle_hash: [u8; 32]) -> PolicyBundleId {
    policy_bundle_id_from_hash(bundle_hash)
}

fn policy_bundle_id_from_hash(bundle_hash: [u8; 32]) -> PolicyBundleId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(POLICY_BUNDLE_ID_DOMAIN);
    hasher.update(&bundle_hash);
    let hash = hasher.finalize();
    let mut id = [0_u8; 16];
    id.copy_from_slice(&hash.as_bytes()[..16]);
    PolicyBundleId(id)
}

fn approval_consumption_id(approval_id: [u8; 16], effect_id: EffectId) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(APPROVAL_CONSUMPTION_ID_DOMAIN);
    hasher.update(&approval_id);
    hasher.update(&effect_id.0.to_be_bytes());
    let hash = hasher.finalize();
    let mut id = [0_u8; 16];
    id.copy_from_slice(&hash.as_bytes()[..16]);
    id
}

fn policy_bundle_id_from_vec(value: Vec<u8>) -> Result<PolicyBundleId, PolicyLifecycleError> {
    let id: [u8; 16] = value
        .try_into()
        .map_err(|_| PolicyLifecycleError::InvalidStoredRecord)?;
    Ok(PolicyBundleId(id))
}

fn id_from_vec(value: Vec<u8>) -> Result<u128, PolicyLifecycleError> {
    let id: [u8; 16] = value
        .try_into()
        .map_err(|_| PolicyLifecycleError::InvalidStoredRecord)?;
    Ok(u128::from_be_bytes(id))
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], PolicyLifecycleError> {
    value
        .try_into()
        .map_err(|_| PolicyLifecycleError::InvalidStoredRecord)
}

fn seq_from_i64(value: i64) -> Result<u64, PolicyLifecycleError> {
    u64::try_from(value).map_err(|_| PolicyLifecycleError::InvalidStoredRecord)
}

fn to_i64(value: u64) -> Result<i64, PolicyLifecycleError> {
    i64::try_from(value).map_err(|_| PolicyLifecycleError::IntegerOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_security_write::append_approval_snapshot;
    use crate::authorization::{
        AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionKind,
        StoredAuthorizationDecision,
    };
    use crate::dispatch::encode_effect_dependencies;
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_core::paths::RuntimeLayout;
    use golam_core::{EffectTransitionId, EventId, SessionId};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    const SCHEMA: &str = "entity User;\nentity Photo;\naction view appliesTo { principal: [User], resource: [Photo] };\n";
    const POLICY_A: &str = "permit(principal is User, action == Action::\"view\", resource is Photo);\n";
    const POLICY_B: &str = "forbid(principal is User, action == Action::\"view\", resource is Photo);\n";

    static N: AtomicU64 = AtomicU64::new(0);

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-policy-lifecycle-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn append_allow(
        log: &mut AuthorizationAuditLog,
        action: &str,
        resource: &str,
    ) -> StoredAuthorizationDecision {
        log.append(AppendAuthorizationDecision {
            principal: "owner:owner",
            action,
            resource,
            context: "scope=local-owner",
            decision: AuthorizationDecisionKind::Allow,
            reason_code: "test_current_authority",
        })
        .unwrap()
    }

    fn stage(
        authority: &AuthorityLayout,
        log: &mut AuthorizationAuditLog,
        policy: &str,
    ) -> PolicyBundleRecord {
        let prepared = prepare_policy_bundle(1, policy, SCHEMA).unwrap();
        let resource = policy_bundle_resource(prepared.policy_bundle_id());
        let decision = append_allow(log, POLICY_STAGE_ACTION, &resource);
        let mut store = PolicyStore::open(authority).unwrap();
        store
            .stage_prepared(prepared, decision.decision_id)
            .unwrap()
    }

    fn authorize_activation_effect(
        authority: &AuthorityLayout,
        effect_id: EffectId,
        resource: &str,
        bundle_hash: [u8; 32],
        id_base: u128,
    ) {
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let mut effects = EffectStore::open(authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(1),
                requested_by: "owner:owner",
                action: POLICY_ACTIVATE_ACTION,
                resource,
                risk_class: POLICY_MUTATION_RISK_CLASS,
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash: bundle_hash,
                proposed_event_id: EventId(id_base),
                transition_id: EffectTransitionId(id_base + 1),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(id_base + 2),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("policy_activation_approved"),
                evidence_ref: None,
                event_id: EventId(id_base + 3),
            })
            .unwrap();
    }

    fn seed_activation_approval(
        authority: &AuthorityLayout,
        approval_id: [u8; 16],
        effect_id: EffectId,
        resource: &str,
    ) {
        let mut connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
            )
            .unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "INSERT INTO approvals (approval_id, class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at) VALUES (?1, 'ONCE', 'owner:owner', ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, '2026-08-27T00:00:00Z', NULL, 1, NULL)",
                params![
                    &approval_id[..],
                    &[3_u8; 32][..],
                    POLICY_ACTIVATE_ACTION.as_bytes(),
                    resource.as_bytes(),
                    &effect_id.0.to_be_bytes()[..],
                    POLICY_MUTATION_RISK_CLASS,
                    &[4_u8; 32][..],
                    &[5_u8; 16][..],
                ],
            )
            .unwrap();
        append_approval_snapshot(&transaction, &approval_id).unwrap();
        transaction.commit().unwrap();
    }

    #[test]
    fn prepared_bundle_identity_is_deterministic_and_field_sensitive() {
        let first = prepare_policy_bundle(1, POLICY_A, SCHEMA).unwrap();
        assert_eq!(first, prepare_policy_bundle(1, POLICY_A, SCHEMA).unwrap());
        assert_ne!(first, prepare_policy_bundle(2, POLICY_A, SCHEMA).unwrap());
        assert_ne!(first, prepare_policy_bundle(1, POLICY_B, SCHEMA).unwrap());
        assert!(matches!(
            prepare_policy_bundle(0, POLICY_A, SCHEMA),
            Err(PolicyLifecycleError::InvalidSchemaVersion)
        ));
    }

    #[test]
    fn staging_requires_latest_exact_allow_and_versions_immutable_bundles() {
        let (runtime, authority) = authority();
        let mut log = AuthorizationAuditLog::open(&authority).unwrap();
        let first = stage(&authority, &mut log, POLICY_A);
        assert_eq!(first.version, 1);
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());

        let prepared_second = prepare_policy_bundle(1, POLICY_B, SCHEMA).unwrap();
        let second_resource = policy_bundle_resource(prepared_second.policy_bundle_id());
        let stale = append_allow(&mut log, POLICY_STAGE_ACTION, &second_resource);
        let _intervening = append_allow(&mut log, "session.read", "session:1");
        let mut store = PolicyStore::open(&authority).unwrap();
        assert!(matches!(
            store.stage_prepared(prepared_second.clone(), stale.decision_id),
            Err(PolicyLifecycleError::StaleAuthorityDecision)
        ));

        let current = append_allow(&mut log, POLICY_STAGE_ACTION, &second_resource);
        let second = store
            .stage_prepared(prepared_second, current.decision_id)
            .unwrap();
        assert_eq!(second.version, 2);
        assert_ne!(first.policy_bundle_id, second.policy_bundle_id);

        let duplicate = prepare_policy_bundle(1, POLICY_A, SCHEMA).unwrap();
        let duplicate_resource = policy_bundle_resource(duplicate.policy_bundle_id());
        let duplicate_decision = append_allow(&mut log, POLICY_STAGE_ACTION, &duplicate_resource);
        assert!(matches!(
            store.stage_prepared(duplicate, duplicate_decision.decision_id),
            Err(PolicyLifecycleError::DuplicateBundle)
        ));
        drop(store);
        drop(log);
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn activation_is_atomic_effect_bound_and_one_shot() {
        let (runtime, authority) = authority();
        let mut log = AuthorizationAuditLog::open(&authority).unwrap();
        let first = stage(&authority, &mut log, POLICY_A);
        let second = stage(&authority, &mut log, POLICY_B);

        let first_resource = policy_bundle_resource(first.policy_bundle_id);
        let first_effect = EffectId(7_001);
        let first_approval = 7_101_u128.to_be_bytes();
        authorize_activation_effect(
            &authority,
            first_effect,
            &first_resource,
            first.bundle_hash,
            7_200,
        );
        seed_activation_approval(&authority, first_approval, first_effect, &first_resource);
        let first_decision = append_allow(&mut log, POLICY_ACTIVATE_ACTION, &first_resource);
        let mut store = PolicyStore::open(&authority).unwrap();
        let activated = store
            .activate(
                first.policy_bundle_id,
                first_decision.decision_id,
                first_approval,
                first_effect,
            )
            .unwrap();
        assert_eq!(activated.previous_policy_bundle_id, None);
        assert_eq!(activated.active.policy_bundle_id, first.policy_bundle_id);
        drop(store);
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());

        let second_resource = policy_bundle_resource(second.policy_bundle_id);
        let bad_effect = EffectId(8_001);
        let bad_approval = 8_101_u128.to_be_bytes();
        authorize_activation_effect(
            &authority,
            bad_effect,
            &second_resource,
            second.bundle_hash,
            8_200,
        );
        seed_activation_approval(&authority, bad_approval, bad_effect, &first_resource);
        let bad_decision = append_allow(&mut log, POLICY_ACTIVATE_ACTION, &second_resource);
        let mut store = PolicyStore::open(&authority).unwrap();
        assert!(matches!(
            store.activate(
                second.policy_bundle_id,
                bad_decision.decision_id,
                bad_approval,
                bad_effect,
            ),
            Err(PolicyLifecycleError::ApprovalMismatch)
        ));
        assert_eq!(
            store.active().unwrap().unwrap().policy_bundle_id,
            first.policy_bundle_id
        );
        drop(store);

        let second_effect = EffectId(9_001);
        let second_approval = 9_101_u128.to_be_bytes();
        authorize_activation_effect(
            &authority,
            second_effect,
            &second_resource,
            second.bundle_hash,
            9_200,
        );
        seed_activation_approval(
            &authority,
            second_approval,
            second_effect,
            &second_resource,
        );
        let second_decision = append_allow(&mut log, POLICY_ACTIVATE_ACTION, &second_resource);
        let mut store = PolicyStore::open(&authority).unwrap();
        let activated = store
            .activate(
                second.policy_bundle_id,
                second_decision.decision_id,
                second_approval,
                second_effect,
            )
            .unwrap();
        assert_eq!(
            activated.previous_policy_bundle_id,
            Some(first.policy_bundle_id)
        );
        assert_eq!(activated.active.policy_bundle_id, second.policy_bundle_id);
        drop(store);

        let replay_decision = append_allow(&mut log, POLICY_ACTIVATE_ACTION, &first_resource);
        let mut store = PolicyStore::open(&authority).unwrap();
        assert!(matches!(
            store.activate(
                first.policy_bundle_id,
                replay_decision.decision_id,
                first_approval,
                first_effect,
            ),
            Err(PolicyLifecycleError::ApprovalAlreadyUsed)
        ));
        assert_eq!(
            store.active().unwrap().unwrap().policy_bundle_id,
            second.policy_bundle_id
        );
        drop(store);
        drop(log);
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
