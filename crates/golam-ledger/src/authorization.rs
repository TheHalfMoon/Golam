#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use rusqlite::{Connection, Transaction, TransactionBehavior, params};

use crate::security_audit::{self, AuthorizationAuditInput};
use crate::storage::{AuthorityStore, StorageError};

const AUTHORIZATION_DECISION_DOMAIN: &[u8] = b"golam:authorization-decision:v1";

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

pub struct AppendAuthorizationDecision<'a> {
    pub principal: &'a str,
    pub action: &'a str,
    pub resource: &'a str,
    pub context: &'a str,
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
    pub decision: AuthorizationDecisionKind,
    pub reason_code: String,
    pub global_seq: u64,
}

#[derive(Debug)]
pub enum AuthorizationAuditError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    SecurityAudit(String),
    InvalidMetadata,
    SequenceOverflow,
    InvalidStoredRecord,
}

impl fmt::Display for AuthorizationAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "authorization audit authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "authorization audit sqlite error: {error}"),
            Self::SecurityAudit(error) => write!(f, "authorization integrity-chain error: {error}"),
            Self::InvalidMetadata => f.write_str(
                "authorization audit principal, action, resource and reason code are required",
            ),
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
            Self::SecurityAudit(_)
            | Self::InvalidMetadata
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

pub struct AuthorizationAuditLog {
    connection: Connection,
}

impl AuthorizationAuditLog {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, AuthorizationAuditError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
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
        if input.principal.is_empty()
            || input.action.is_empty()
            || input.resource.is_empty()
            || input.reason_code.is_empty()
        {
            return Err(AuthorizationAuditError::InvalidMetadata);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let global_seq = next_global_seq(&transaction)?;
        let context_hash = *blake3::hash(input.context.as_bytes()).as_bytes();
        let decision_id = decision_id(&input, context_hash, global_seq);
        transaction.execute(
            "INSERT INTO authorization_decisions \
             (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &decision_id[..],
                input.principal,
                input.action,
                input.resource,
                &context_hash[..],
                input.decision.as_str(),
                input.reason_code,
                i64::try_from(global_seq).map_err(|_| AuthorizationAuditError::SequenceOverflow)?,
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
        transaction.commit()?;

        Ok(StoredAuthorizationDecision {
            decision_id,
            principal: input.principal.to_owned(),
            action: input.action.to_owned(),
            resource: input.resource.to_owned(),
            context_hash,
            decision: input.decision,
            reason_code: input.reason_code.to_owned(),
            global_seq,
        })
    }

    pub fn records(&self) -> Result<Vec<StoredAuthorizationDecision>, AuthorizationAuditError> {
        let mut statement = self.connection.prepare(
            "SELECT decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq \
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
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
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
                decision,
                reason_code,
                global_seq,
            ) = row?;
            records.push(StoredAuthorizationDecision {
                decision_id: decision_id
                    .try_into()
                    .map_err(|_| AuthorizationAuditError::InvalidStoredRecord)?,
                principal,
                action,
                resource,
                context_hash: context_hash
                    .try_into()
                    .map_err(|_| AuthorizationAuditError::InvalidStoredRecord)?,
                decision: AuthorizationDecisionKind::from_str(&decision)
                    .ok_or(AuthorizationAuditError::InvalidStoredRecord)?,
                reason_code,
                global_seq: u64::try_from(global_seq)
                    .map_err(|_| AuthorizationAuditError::InvalidStoredRecord)?,
            });
        }
        Ok(records)
    }
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

    #[test]
    fn decisions_are_durable_ordered_and_context_hashed() {
        let (runtime, authority) = authority();
        let mut log = AuthorizationAuditLog::open(&authority).unwrap();
        let allow = log
            .append(AppendAuthorizationDecision {
                principal: "owner:local",
                action: "session.create",
                resource: "session:new",
                context: "authenticated-local",
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
                decision: AuthorizationDecisionKind::Deny,
                reason_code: "strict_local_egress_denied",
            })
            .unwrap();
        assert_eq!(allow.global_seq, 1);
        assert_eq!(deny.global_seq, 2);
        assert_ne!(allow.decision_id, deny.decision_id);
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
    fn empty_security_metadata_is_rejected() {
        let (runtime, authority) = authority();
        let mut log = AuthorizationAuditLog::open(&authority).unwrap();
        assert!(matches!(
            log.append(AppendAuthorizationDecision {
                principal: "",
                action: "session.create",
                resource: "session:new",
                context: "",
                decision: AuthorizationDecisionKind::Deny,
                reason_code: "",
            }),
            Err(AuthorizationAuditError::InvalidMetadata)
        ));
        drop(log);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
