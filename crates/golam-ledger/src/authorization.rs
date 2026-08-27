#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use rusqlite::{Connection, Transaction, TransactionBehavior, params};

use crate::active_policy_integrity::{
    ActivePolicyIntegrityError, verify_path as verify_active_policy,
};
use crate::authority_security_write::append_authorization_decision_v2_snapshot;
use crate::security_audit::{self, AuthorizationAuditInput};
use crate::storage::{AuthorityStore, StorageError};

const AUTHORIZATION_DECISION_DOMAIN: &[u8] = b"golam:authorization-decision:v1";
const MATCHED_RULE_IDS_DOMAIN: &[u8] = b"golam:authorization-matched-rule-ids:v1";
const AUTHORITY_EVIDENCE_VERSION: u64 = 2;
const MAX_HARD_GUARD_RESULT_BYTES: usize = 128;
const MAX_MATCHED_RULE_IDS: usize = 64;
const MAX_MATCHED_RULE_ID_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationDecisionKind {
    Allow,
    Deny,
}

impl AuthorizationDecisionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "allow" => Some(Self::Allow),
            "deny" => Some(Self::Deny),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizationDecisionEvidence<'a> {
    pub hard_guard_result: &'a str,
    pub lease_id: Option<&'a [u8; 16]>,
    pub lease_generation: Option<u64>,
    pub policy_bundle_id: Option<&'a [u8; 16]>,
    pub policy_bundle_hash: Option<&'a [u8; 32]>,
    pub matched_rule_ids: &'a [&'a str],
    pub approval_id: Option<&'a [u8; 16]>,
}

impl<'a> AuthorizationDecisionEvidence<'a> {
    pub const fn hard_guard_only(hard_guard_result: &'a str) -> Self {
        Self {
            hard_guard_result,
            lease_id: None,
            lease_generation: None,
            policy_bundle_id: None,
            policy_bundle_hash: None,
            matched_rule_ids: &[],
            approval_id: None,
        }
    }
}

pub struct AppendAuthorizationDecision<'a> {
    pub principal: &'a str,
    pub action: &'a str,
    pub resource: &'a str,
    pub context: &'a str,
    pub evidence: AuthorizationDecisionEvidence<'a>,
    pub decision: AuthorizationDecisionKind,
    pub reason_code: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredAuthorizationDecision {
    pub decision_id: [u8; 16],
    pub principal: String,
    pub action: String,
    pub resource: String,
    pub context_hash: [u8; 32],
    pub hard_guard_result: String,
    pub lease_id: Option<[u8; 16]>,
    pub lease_generation: Option<u64>,
    pub policy_bundle_id: Option<[u8; 16]>,
    pub policy_bundle_hash: Option<[u8; 32]>,
    pub matched_rule_ids: Vec<String>,
    pub approval_id: Option<[u8; 16]>,
    pub decision: AuthorizationDecisionKind,
    pub reason_code: String,
    pub global_seq: u64,
    pub authority_evidence_version: u64,
}

#[derive(Debug)]
pub enum AuthorizationAuditError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    SecurityAudit(String),
    AuthoritySecurityV2(String),
    ActivePolicyIntegrity(ActivePolicyIntegrityError),
    InvalidMetadata,
    InvalidEvidence(&'static str),
    SequenceOverflow,
    InvalidStoredRecord,
}

impl fmt::Display for AuthorizationAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "authorization audit authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "authorization audit sqlite error: {error}"),
            Self::SecurityAudit(error) => write!(f, "authorization integrity-chain error: {error}"),
            Self::AuthoritySecurityV2(error) => {
                write!(f, "authorization authority-security-v2 error: {error}")
            }
            Self::ActivePolicyIntegrity(error) => {
                write!(f, "authorization active-policy integrity error: {error}")
            }
            Self::InvalidMetadata => f.write_str(
                "authorization audit principal, action, resource and reason code are required",
            ),
            Self::InvalidEvidence(reason) => {
                write!(f, "authorization evidence is invalid: {reason}")
            }
            Self::SequenceOverflow => f.write_str("authorization audit global sequence overflow"),
            Self::InvalidStoredRecord => {
                f.write_str("stored authorization decision record is malformed")
            }
        }
    }
}

impl Error for AuthorizationAuditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::ActivePolicyIntegrity(error) => Some(error),
            Self::SecurityAudit(_)
            | Self::AuthoritySecurityV2(_)
            | Self::InvalidMetadata
            | Self::InvalidEvidence(_)
            | Self::SequenceOverflow
            | Self::InvalidStoredRecord => None,
        }
    }
}

impl From<StorageError> for AuthorizationAuditError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for AuthorizationAuditError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<ActivePolicyIntegrityError> for AuthorizationAuditError {
    fn from(value: ActivePolicyIntegrityError) -> Self {
        Self::ActivePolicyIntegrity(value)
    }
}

pub struct AuthorizationAuditLog {
    connection: Connection,
}

impl AuthorizationAuditLog {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, AuthorizationAuditError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        verify_active_policy(layout.authority_db_path())?;
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn append(
        &mut self,
        input: AppendAuthorizationDecision<'_>,
    ) -> Result<StoredAuthorizationDecision, AuthorizationAuditError> {
        validate_input(&input)?;
        let (matched_rule_ids, encoded_rule_ids) =
            canonical_matched_rule_ids(input.evidence.matched_rule_ids)?;
        let lease_generation = input
            .evidence
            .lease_generation
            .map(i64::try_from)
            .transpose()
            .map_err(|_| AuthorizationAuditError::SequenceOverflow)?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let global_seq = next_global_seq(&transaction)?;
        let context_hash = *blake3::hash(input.context.as_bytes()).as_bytes();
        let decision_id = decision_id(&input, context_hash, global_seq);
        transaction.execute(
            "INSERT INTO authorization_decisions \
             (decision_id, principal, action, resource, context_hash, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, decision, reason_code, global_seq, authority_evidence_version) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                &decision_id[..],
                input.principal,
                input.action,
                input.resource,
                &context_hash[..],
                input.evidence.hard_guard_result,
                input.evidence.lease_id.map(|id| id.as_slice()),
                lease_generation,
                input.evidence.policy_bundle_id.map(|id| id.as_slice()),
                input.evidence.policy_bundle_hash.map(|hash| hash.as_slice()),
                &encoded_rule_ids,
                input.evidence.approval_id.map(|id| id.as_slice()),
                input.decision.as_str(),
                input.reason_code,
                i64::try_from(global_seq).map_err(|_| AuthorizationAuditError::SequenceOverflow)?,
                i64::try_from(AUTHORITY_EVIDENCE_VERSION)
                    .map_err(|_| AuthorizationAuditError::SequenceOverflow)?,
            ],
        )?;
        security_audit::append_authorization_decision(
            &transaction,
            AuthorizationAuditInput {
                decision_id: &decision_id,
                principal: input.principal,
                action: input.action,
                resource: input.resource,
                context_hash: &context_hash,
                decision: input.decision.as_str(),
                reason_code: input.reason_code,
                global_seq,
            },
        )
        .map_err(|error| AuthorizationAuditError::SecurityAudit(error.to_string()))?;
        append_authorization_decision_v2_snapshot(&transaction, &decision_id)
            .map_err(|error| AuthorizationAuditError::AuthoritySecurityV2(error.to_string()))?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| AuthorizationAuditError::AuthoritySecurityV2(error.to_string()))?;
        transaction.commit()?;

        Ok(StoredAuthorizationDecision {
            decision_id,
            principal: input.principal.to_owned(),
            action: input.action.to_owned(),
            resource: input.resource.to_owned(),
            context_hash,
            hard_guard_result: input.evidence.hard_guard_result.to_owned(),
            lease_id: input.evidence.lease_id.copied(),
            lease_generation: input.evidence.lease_generation,
            policy_bundle_id: input.evidence.policy_bundle_id.copied(),
            policy_bundle_hash: input.evidence.policy_bundle_hash.copied(),
            matched_rule_ids,
            approval_id: input.evidence.approval_id.copied(),
            decision: input.decision,
            reason_code: input.reason_code.to_owned(),
            global_seq,
            authority_evidence_version: AUTHORITY_EVIDENCE_VERSION,
        })
    }

    pub fn records(&self) -> Result<Vec<StoredAuthorizationDecision>, AuthorizationAuditError> {
        let mut statement = self.connection.prepare(
            "SELECT decision_id, principal, action, resource, context_hash, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, decision, reason_code, global_seq, authority_evidence_version \
             FROM authorization_decisions ORDER BY global_seq ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Vec<u8>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<Vec<u8>>>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<Vec<u8>>>(8)?,
                row.get::<_, Option<Vec<u8>>>(9)?,
                row.get::<_, Vec<u8>>(10)?,
                row.get::<_, Option<Vec<u8>>>(11)?,
                row.get::<_, String>(12)?,
                row.get::<_, String>(13)?,
                row.get::<_, i64>(14)?,
                row.get::<_, i64>(15)?,
            ))
        })?;
        let mut records = Vec::new();
        for row in rows {
            let (
                decision_id,
                principal,
                action,
                resource,
                context_hash,
                hard_guard_result,
                lease_id,
                lease_generation,
                policy_bundle_id,
                policy_bundle_hash,
                matched_rule_ids,
                approval_id,
                decision,
                reason_code,
                global_seq,
                authority_evidence_version,
            ) = row?;
            let authority_evidence_version = u64::try_from(authority_evidence_version)
                .map_err(|_| AuthorizationAuditError::InvalidStoredRecord)?;
            let matched_rule_ids = if authority_evidence_version >= AUTHORITY_EVIDENCE_VERSION {
                decode_matched_rule_ids(&matched_rule_ids)?
            } else if matched_rule_ids.is_empty() {
                Vec::new()
            } else {
                return Err(AuthorizationAuditError::InvalidStoredRecord);
            };
            let record = StoredAuthorizationDecision {
                decision_id: id16(decision_id)?,
                principal,
                action,
                resource,
                context_hash: hash32(context_hash)?,
                hard_guard_result,
                lease_id: optional_id16(lease_id)?,
                lease_generation: optional_positive_u64(lease_generation)?,
                policy_bundle_id: optional_id16(policy_bundle_id)?,
                policy_bundle_hash: optional_hash32(policy_bundle_hash)?,
                matched_rule_ids,
                approval_id: optional_id16(approval_id)?,
                decision: AuthorizationDecisionKind::from_str(&decision)
                    .ok_or(AuthorizationAuditError::InvalidStoredRecord)?,
                reason_code,
                global_seq: u64::try_from(global_seq)
                    .map_err(|_| AuthorizationAuditError::InvalidStoredRecord)?,
                authority_evidence_version,
            };
            validate_stored_evidence(&record)?;
            records.push(record);
        }
        Ok(records)
    }
}

fn validate_input(input: &AppendAuthorizationDecision<'_>) -> Result<(), AuthorizationAuditError> {
    if input.principal.is_empty()
        || input.action.is_empty()
        || input.resource.is_empty()
        || input.reason_code.is_empty()
    {
        return Err(AuthorizationAuditError::InvalidMetadata);
    }
    validate_reference(
        input.evidence.hard_guard_result,
        MAX_HARD_GUARD_RESULT_BYTES,
        "hard guard result is empty, non-canonical or too large",
    )?;
    if input.evidence.lease_id.is_some() != input.evidence.lease_generation.is_some() {
        return Err(AuthorizationAuditError::InvalidEvidence(
            "lease id and generation must be present together",
        ));
    }
    if input.evidence.policy_bundle_id.is_some() != input.evidence.policy_bundle_hash.is_some() {
        return Err(AuthorizationAuditError::InvalidEvidence(
            "policy bundle id and hash must be present together",
        ));
    }
    if input.evidence.policy_bundle_id.is_none() && !input.evidence.matched_rule_ids.is_empty() {
        return Err(AuthorizationAuditError::InvalidEvidence(
            "matched rule ids require policy bundle evidence",
        ));
    }
    if input.evidence.hard_guard_result != "pass"
        && (input.evidence.lease_id.is_some()
            || input.evidence.policy_bundle_id.is_some()
            || !input.evidence.matched_rule_ids.is_empty()
            || input.evidence.approval_id.is_some())
    {
        return Err(AuthorizationAuditError::InvalidEvidence(
            "hard-guard denial cannot claim downstream authority evidence",
        ));
    }
    Ok(())
}

fn validate_stored_evidence(
    record: &StoredAuthorizationDecision,
) -> Result<(), AuthorizationAuditError> {
    if record.authority_evidence_version < AUTHORITY_EVIDENCE_VERSION {
        return Ok(());
    }
    validate_reference(
        &record.hard_guard_result,
        MAX_HARD_GUARD_RESULT_BYTES,
        "stored hard guard result is invalid",
    )?;
    if record.lease_id.is_some() != record.lease_generation.is_some() {
        return Err(AuthorizationAuditError::InvalidStoredRecord);
    }
    if record.policy_bundle_id.is_some() != record.policy_bundle_hash.is_some() {
        return Err(AuthorizationAuditError::InvalidStoredRecord);
    }
    if record.policy_bundle_id.is_none() && !record.matched_rule_ids.is_empty() {
        return Err(AuthorizationAuditError::InvalidStoredRecord);
    }
    if record.hard_guard_result != "pass"
        && (record.lease_id.is_some()
            || record.policy_bundle_id.is_some()
            || !record.matched_rule_ids.is_empty()
            || record.approval_id.is_some())
    {
        return Err(AuthorizationAuditError::InvalidStoredRecord);
    }
    Ok(())
}

fn canonical_matched_rule_ids(
    rule_ids: &[&str],
) -> Result<(Vec<String>, Vec<u8>), AuthorizationAuditError> {
    if rule_ids.len() > MAX_MATCHED_RULE_IDS {
        return Err(AuthorizationAuditError::InvalidEvidence(
            "too many matched rule ids",
        ));
    }
    let mut canonical = rule_ids.to_vec();
    canonical.sort_unstable();
    for (index, rule_id) in canonical.iter().enumerate() {
        validate_reference(
            rule_id,
            MAX_MATCHED_RULE_ID_BYTES,
            "matched rule id is empty, non-canonical or too large",
        )?;
        if index > 0 && canonical[index - 1] == *rule_id {
            return Err(AuthorizationAuditError::InvalidEvidence(
                "matched rule ids contain a duplicate",
            ));
        }
    }
    let count = u16::try_from(canonical.len())
        .map_err(|_| AuthorizationAuditError::InvalidEvidence("too many matched rule ids"))?;
    let mut encoded = Vec::with_capacity(
        MATCHED_RULE_IDS_DOMAIN.len()
            + 2
            + canonical.iter().map(|rule| 2 + rule.len()).sum::<usize>(),
    );
    encoded.extend_from_slice(MATCHED_RULE_IDS_DOMAIN);
    encoded.extend_from_slice(&count.to_be_bytes());
    for rule_id in &canonical {
        let bytes = rule_id.as_bytes();
        let len = u16::try_from(bytes.len()).map_err(|_| {
            AuthorizationAuditError::InvalidEvidence("matched rule id is too large")
        })?;
        encoded.extend_from_slice(&len.to_be_bytes());
        encoded.extend_from_slice(bytes);
    }
    Ok((canonical.into_iter().map(str::to_owned).collect(), encoded))
}

fn decode_matched_rule_ids(bytes: &[u8]) -> Result<Vec<String>, AuthorizationAuditError> {
    if !bytes.starts_with(MATCHED_RULE_IDS_DOMAIN) {
        return Err(AuthorizationAuditError::InvalidStoredRecord);
    }
    let mut offset = MATCHED_RULE_IDS_DOMAIN.len();
    let count = usize::from(take_u16(bytes, &mut offset)?);
    if count > MAX_MATCHED_RULE_IDS {
        return Err(AuthorizationAuditError::InvalidStoredRecord);
    }
    let mut rule_ids = Vec::with_capacity(count);
    for _ in 0..count {
        let len = usize::from(take_u16(bytes, &mut offset)?);
        let end = offset
            .checked_add(len)
            .ok_or(AuthorizationAuditError::InvalidStoredRecord)?;
        let value = bytes
            .get(offset..end)
            .ok_or(AuthorizationAuditError::InvalidStoredRecord)?;
        offset = end;
        let rule_id = std::str::from_utf8(value)
            .map_err(|_| AuthorizationAuditError::InvalidStoredRecord)?
            .to_owned();
        validate_reference(
            &rule_id,
            MAX_MATCHED_RULE_ID_BYTES,
            "stored matched rule id is invalid",
        )?;
        if rule_ids.last().is_some_and(|previous| previous >= &rule_id) {
            return Err(AuthorizationAuditError::InvalidStoredRecord);
        }
        rule_ids.push(rule_id);
    }
    if offset != bytes.len() {
        return Err(AuthorizationAuditError::InvalidStoredRecord);
    }
    Ok(rule_ids)
}

fn validate_reference(
    value: &str,
    max_bytes: usize,
    reason: &'static str,
) -> Result<(), AuthorizationAuditError> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(AuthorizationAuditError::InvalidEvidence(reason));
    }
    Ok(())
}

fn take_u16(bytes: &[u8], offset: &mut usize) -> Result<u16, AuthorizationAuditError> {
    let end = offset
        .checked_add(2)
        .ok_or(AuthorizationAuditError::InvalidStoredRecord)?;
    let value: [u8; 2] = bytes
        .get(*offset..end)
        .ok_or(AuthorizationAuditError::InvalidStoredRecord)?
        .try_into()
        .map_err(|_| AuthorizationAuditError::InvalidStoredRecord)?;
    *offset = end;
    Ok(u16::from_be_bytes(value))
}

fn next_global_seq(transaction: &Transaction<'_>) -> Result<u64, AuthorizationAuditError> {
    let current: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (\
           SELECT global_seq FROM session_events \
           UNION ALL SELECT global_seq FROM effect_transitions \
           UNION ALL SELECT global_seq FROM authorization_decisions\
         )",
        [],
        |row| row.get(0),
    )?;
    u64::try_from(current)
        .map_err(|_| AuthorizationAuditError::InvalidStoredRecord)?
        .checked_add(1)
        .ok_or(AuthorizationAuditError::SequenceOverflow)
}

fn decision_id(
    input: &AppendAuthorizationDecision<'_>,
    context_hash: [u8; 32],
    global_seq: u64,
) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(AUTHORIZATION_DECISION_DOMAIN);
    hasher.update(&global_seq.to_be_bytes());
    hash_len_prefixed(&mut hasher, input.principal.as_bytes());
    hash_len_prefixed(&mut hasher, input.action.as_bytes());
    hash_len_prefixed(&mut hasher, input.resource.as_bytes());
    hasher.update(&context_hash);
    hash_len_prefixed(&mut hasher, input.decision.as_str().as_bytes());
    hash_len_prefixed(&mut hasher, input.reason_code.as_bytes());
    let hash = hasher.finalize();
    let mut id = [0_u8; 16];
    id.copy_from_slice(&hash.as_bytes()[..16]);
    id
}

fn hash_len_prefixed(hasher: &mut blake3::Hasher, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_be_bytes());
    hasher.update(value);
}

fn id16(value: Vec<u8>) -> Result<[u8; 16], AuthorizationAuditError> {
    value
        .try_into()
        .map_err(|_| AuthorizationAuditError::InvalidStoredRecord)
}

fn hash32(value: Vec<u8>) -> Result<[u8; 32], AuthorizationAuditError> {
    value
        .try_into()
        .map_err(|_| AuthorizationAuditError::InvalidStoredRecord)
}

fn optional_id16(value: Option<Vec<u8>>) -> Result<Option<[u8; 16]>, AuthorizationAuditError> {
    value.map(id16).transpose()
}

fn optional_hash32(value: Option<Vec<u8>>) -> Result<Option<[u8; 32]>, AuthorizationAuditError> {
    value.map(hash32).transpose()
}

fn optional_positive_u64(value: Option<i64>) -> Result<Option<u64>, AuthorizationAuditError> {
    value
        .map(|value| {
            let value =
                u64::try_from(value).map_err(|_| AuthorizationAuditError::InvalidStoredRecord)?;
            if value == 0 {
                Err(AuthorizationAuditError::InvalidStoredRecord)
            } else {
                Ok(value)
            }
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
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
            "golam-authorization-audit-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn hard_guard(result: &str) -> AuthorizationDecisionEvidence<'_> {
        AuthorizationDecisionEvidence::hard_guard_only(result)
    }

    #[test]
    fn decisions_are_durable_ordered_and_context_hashed_with_v2_evidence() {
        let (runtime, authority) = authority();
        let mut log = AuthorizationAuditLog::open(&authority).unwrap();
        let allow = log
            .append(AppendAuthorizationDecision {
                principal: "owner:local",
                action: "session.create",
                resource: "session:new",
                context: "authenticated-local",
                evidence: hard_guard("pass"),
                decision: AuthorizationDecisionKind::Allow,
                reason_code: "bootstrap_owner_session",
            })
            .unwrap();
        let deny = log
            .append(AppendAuthorizationDecision {
                principal: "client:7",
                action: "network.egress",
                resource: "https://example.invalid",
                context: "strict-local",
                evidence: hard_guard("strict_local_egress_denied"),
                decision: AuthorizationDecisionKind::Deny,
                reason_code: "strict_local_egress_denied",
            })
            .unwrap();
        assert_eq!(allow.global_seq, 1);
        assert_eq!(deny.global_seq, 2);
        assert_ne!(allow.decision_id, deny.decision_id);
        assert_eq!(allow.hard_guard_result, "pass");
        assert_eq!(deny.hard_guard_result, "strict_local_egress_denied");
        assert_eq!(allow.authority_evidence_version, 2);
        assert_eq!(
            allow.context_hash,
            *blake3::hash(b"authenticated-local").as_bytes()
        );
        assert_eq!(log.records().unwrap(), vec![allow, deny]);
        drop(log);

        let reopened = AuthorizationAuditLog::open(&authority).unwrap();
        assert_eq!(reopened.records().unwrap().len(), 2);
        drop(reopened);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn exact_lease_policy_rule_and_approval_evidence_is_canonicalized() {
        let (runtime, authority) = authority();
        let mut log = AuthorizationAuditLog::open(&authority).unwrap();
        let lease_id = [1_u8; 16];
        let policy_bundle_id = [2_u8; 16];
        let policy_bundle_hash = [3_u8; 32];
        let approval_id = [4_u8; 16];
        let rules = ["rule:z", "rule:a"];
        let stored = log
            .append(AppendAuthorizationDecision {
                principal: "owner:local",
                action: "effect.execute",
                resource: "effect:7",
                context: "local-approved",
                evidence: AuthorizationDecisionEvidence {
                    hard_guard_result: "pass",
                    lease_id: Some(&lease_id),
                    lease_generation: Some(7),
                    policy_bundle_id: Some(&policy_bundle_id),
                    policy_bundle_hash: Some(&policy_bundle_hash),
                    matched_rule_ids: &rules,
                    approval_id: Some(&approval_id),
                },
                decision: AuthorizationDecisionKind::Allow,
                reason_code: "policy_allow",
            })
            .unwrap();
        assert_eq!(stored.lease_id, Some(lease_id));
        assert_eq!(stored.lease_generation, Some(7));
        assert_eq!(stored.policy_bundle_id, Some(policy_bundle_id));
        assert_eq!(stored.policy_bundle_hash, Some(policy_bundle_hash));
        assert_eq!(stored.matched_rule_ids, vec!["rule:a", "rule:z"]);
        assert_eq!(stored.approval_id, Some(approval_id));
        assert_eq!(log.records().unwrap(), vec![stored]);
        drop(log);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn partial_or_downstream_after_hard_deny_evidence_is_rejected() {
        let (runtime, authority) = authority();
        let mut log = AuthorizationAuditLog::open(&authority).unwrap();
        let lease_id = [1_u8; 16];
        assert!(matches!(
            log.append(AppendAuthorizationDecision {
                principal: "owner:local",
                action: "effect.execute",
                resource: "effect:7",
                context: "local",
                evidence: AuthorizationDecisionEvidence {
                    hard_guard_result: "pass",
                    lease_id: Some(&lease_id),
                    lease_generation: None,
                    policy_bundle_id: None,
                    policy_bundle_hash: None,
                    matched_rule_ids: &[],
                    approval_id: None,
                },
                decision: AuthorizationDecisionKind::Deny,
                reason_code: "invalid",
            }),
            Err(AuthorizationAuditError::InvalidEvidence(_))
        ));
        assert_eq!(log.records().unwrap().len(), 0);
        drop(log);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn authority_security_v2_detects_tampered_decision_evidence_on_restart() {
        let (runtime, authority) = authority();
        let mut log = AuthorizationAuditLog::open(&authority).unwrap();
        log.append(AppendAuthorizationDecision {
            principal: "owner:local",
            action: "session.read",
            resource: "session:1",
            context: "local-owner",
            evidence: hard_guard("pass"),
            decision: AuthorizationDecisionKind::Allow,
            reason_code: "test_allow",
        })
        .unwrap();
        drop(log);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute(
                "UPDATE authorization_decisions SET hard_guard_result = 'forged'",
                [],
            )
            .unwrap();
        drop(connection);
        assert!(matches!(
            AuthorizationAuditLog::open(&authority),
            Err(AuthorizationAuditError::Storage(_))
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn empty_security_metadata_is_rejected() {
        let (runtime, authority) = authority();
        let mut log = AuthorizationAuditLog::open(&authority).unwrap();
        assert!(matches!(
            log.append(AppendAuthorizationDecision {
                principal: "",
                action: "session.create",
                resource: "session:new",
                context: "",
                evidence: hard_guard("pass"),
                decision: AuthorizationDecisionKind::Deny,
                reason_code: "",
            }),
            Err(AuthorizationAuditError::InvalidMetadata)
        ));
        drop(log);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
