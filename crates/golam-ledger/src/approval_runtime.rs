#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId, SessionId};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::approval_binding::{APPROVAL_ISSUE_ACTION, ApprovalBindingError, prepare_approval};
use crate::approvals::{ApprovalClass, ApprovalScope, ApprovalScopeError};
use crate::storage::{AuthorityStore, StorageError};

const APPROVAL_BINDING_DOMAIN: &[u8] = b"golam:approval-binding:v1";
const MAX_ACTION_BYTES: usize = 128;
const MAX_RESOURCE_BYTES: usize = 2_048;
const MAX_RISK_CLASS_BYTES: usize = 128;

#[derive(Clone, Copy, Debug)]
pub struct ApprovalUseRequest<'a> {
    pub approval_id: [u8; 16],
    pub action: &'a str,
    pub resource: &'a str,
    pub effect_id: Option<EffectId>,
    pub session_id: Option<SessionId>,
    pub risk_class: &'a str,
    pub taint_digest: [u8; 32],
    pub observed_at: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovalUseEvidence {
    approval_id: [u8; 16],
    class: ApprovalClass,
    scope_digest: [u8; 32],
    parent_decision_id: [u8; 16],
    max_uses: u64,
    current_uses: u64,
}

impl ApprovalUseEvidence {
    pub const fn approval_id(self) -> [u8; 16] {
        self.approval_id
    }

    pub const fn class(self) -> ApprovalClass {
        self.class
    }

    pub const fn scope_digest(self) -> [u8; 32] {
        self.scope_digest
    }

    pub const fn parent_decision_id(self) -> [u8; 16] {
        self.parent_decision_id
    }

    pub const fn max_uses(self) -> u64 {
        self.max_uses
    }

    pub const fn current_uses(self) -> u64 {
        self.current_uses
    }
}

#[derive(Debug)]
pub enum ApprovalUseError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Scope(ApprovalScopeError),
    Binding(ApprovalBindingError),
    Integrity(String),
    AuthoritySecurity(String),
    ApprovalNotFound,
    InvalidUseRequest,
    InvalidObservedTime,
    InvalidStoredRecord(&'static str),
    ParentDecisionNotFound,
    ParentDecisionMismatch,
    BindingMismatch,
    NotYetIssued,
    Expired,
    Revoked,
    ScopeMismatch,
    RiskMismatch,
    TaintMismatch,
    UsageLimitReached,
}

impl fmt::Display for ApprovalUseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "approval use authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "approval use sqlite error: {error}"),
            Self::Core(error) => write!(f, "approval use canonical encoding error: {error}"),
            Self::Scope(error) => write!(f, "approval use scope error: {error}"),
            Self::Binding(error) => write!(f, "approval use binding error: {error}"),
            Self::Integrity(error) => write!(f, "approval use integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "approval use authority-security error: {error}")
            }
            Self::ApprovalNotFound => f.write_str("approval does not exist"),
            Self::InvalidUseRequest => f.write_str("approval use request is not canonical"),
            Self::InvalidObservedTime => {
                f.write_str("approval observed use time is not canonical UTC-second time")
            }
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored approval use record is invalid: {reason}")
            }
            Self::ParentDecisionNotFound => {
                f.write_str("approval parent authorization decision does not exist")
            }
            Self::ParentDecisionMismatch => f.write_str(
                "approval parent authorization decision does not match issuance authority",
            ),
            Self::BindingMismatch => {
                f.write_str("approval scope digest does not match its bound issuance authority")
            }
            Self::NotYetIssued => f.write_str("approval is not yet fresh for use"),
            Self::Expired => f.write_str("approval is expired"),
            Self::Revoked => f.write_str("approval is revoked"),
            Self::ScopeMismatch => f.write_str("approval does not cover the exact protected use"),
            Self::RiskMismatch => f.write_str("approval risk class does not match protected use"),
            Self::TaintMismatch => {
                f.write_str("approval taint digest does not match protected use")
            }
            Self::UsageLimitReached => f.write_str("approval usage limit has been reached"),
        }
    }
}

impl Error for ApprovalUseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Scope(error) => Some(error),
            Self::Binding(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for ApprovalUseError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for ApprovalUseError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for ApprovalUseError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<ApprovalScopeError> for ApprovalUseError {
    fn from(value: ApprovalScopeError) -> Self {
        Self::Scope(value)
    }
}

impl From<ApprovalBindingError> for ApprovalUseError {
    fn from(value: ApprovalBindingError) -> Self {
        Self::Binding(value)
    }
}

pub struct ApprovalUseStore {
    connection: Connection,
}

impl ApprovalUseStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, ApprovalUseError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    /// Revalidates protected approval state against the exact use immediately
    /// before execution. This is read-only evidence; it does not reserve or
    /// consume approval authority. Atomic reservation/consumption is owned by
    /// T003-033 and remains required for consumptive execution.
    pub fn validate(
        &mut self,
        request: ApprovalUseRequest<'_>,
    ) -> Result<ApprovalUseEvidence, ApprovalUseError> {
        validate_use_request(&request)?;
        if !valid_utc_second(request.observed_at) {
            return Err(ApprovalUseError::InvalidObservedTime);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Deferred)?;
        crate::integrity::verify(&transaction)
            .map_err(|error| ApprovalUseError::Integrity(error.to_string()))?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| ApprovalUseError::AuthoritySecurity(error.to_string()))?;

        let row = transaction
            .query_row(
                "SELECT class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at FROM approvals WHERE approval_id = ?1",
                params![&request.approval_id[..]],
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
            .ok_or(ApprovalUseError::ApprovalNotFound)?;

        let class = ApprovalClass::try_from(row.0.as_str())?;
        let scope_digest = hash32(row.2, "approval scope digest is not 32 bytes")?;
        let scope = decode_scope(class, row.3, row.4, row.5, row.6)?;
        let taint_digest = hash32(row.8, "approval taint digest is not 32 bytes")?;
        let parent_decision_id = id16(row.9, "approval parent decision id is not 16 bytes")?;
        let max_uses = positive_u64(row.12, "approval max_uses is missing or invalid")?;

        if !valid_utc_second(&row.10) {
            return Err(ApprovalUseError::InvalidStoredRecord(
                "approval issued_at is malformed",
            ));
        }
        if request.observed_at < row.10.as_str() {
            return Err(ApprovalUseError::NotYetIssued);
        }
        if let Some(expires_at) = row.11.as_deref() {
            if !valid_utc_second(expires_at) || row.10.as_str() >= expires_at {
                return Err(ApprovalUseError::InvalidStoredRecord(
                    "approval expiry is malformed or not after issuance",
                ));
            }
            if request.observed_at >= expires_at {
                return Err(ApprovalUseError::Expired);
            }
        }
        if let Some(revoked_at) = row.13.as_deref() {
            if !valid_utc_second(revoked_at) || revoked_at < row.10.as_str() {
                return Err(ApprovalUseError::InvalidStoredRecord(
                    "approval revoked_at is malformed or predates issuance",
                ));
            }
            return Err(ApprovalUseError::Revoked);
        }

        if row.7 != request.risk_class {
            return Err(ApprovalUseError::RiskMismatch);
        }
        if taint_digest != request.taint_digest {
            return Err(ApprovalUseError::TaintMismatch);
        }
        if !scope_matches(&scope, &request)? {
            return Err(ApprovalUseError::ScopeMismatch);
        }

        let prepared = prepare_approval(
            &row.1,
            scope,
            &row.7,
            taint_digest,
            &row.10,
            row.11.as_deref(),
            max_uses,
        )?;
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
            .ok_or(ApprovalUseError::ParentDecisionNotFound)?;
        if parent.0 != row.1
            || parent.1 != APPROVAL_ISSUE_ACTION
            || parent.2 != prepared.resource()
            || parent.4 != "allow"
        {
            return Err(ApprovalUseError::ParentDecisionMismatch);
        }
        let context_hash = hash32(
            parent.3,
            "approval parent authorization context hash is not 32 bytes",
        )?;
        let rebound =
            bound_scope_digest(prepared.intent_digest(), parent_decision_id, context_hash)?;
        if rebound != scope_digest {
            return Err(ApprovalUseError::BindingMismatch);
        }

        let mut statement = transaction.prepare(
            "SELECT state FROM approval_consumptions WHERE approval_id = ?1 ORDER BY reserved_global_seq ASC, consumption_id ASC",
        )?;
        let states = statement.query_map(params![&request.approval_id[..]], |row| {
            row.get::<_, String>(0)
        })?;
        let mut current_uses = 0_u64;
        for state in states {
            match state?.as_str() {
                "reserved" | "consumed" => {
                    current_uses = current_uses.checked_add(1).ok_or(
                        ApprovalUseError::InvalidStoredRecord("approval use count overflow"),
                    )?;
                }
                "released" => {}
                _ => {
                    return Err(ApprovalUseError::InvalidStoredRecord(
                        "approval consumption state is unknown",
                    ));
                }
            }
        }
        drop(statement);
        if current_uses >= max_uses {
            return Err(ApprovalUseError::UsageLimitReached);
        }

        transaction.commit()?;
        Ok(ApprovalUseEvidence {
            approval_id: request.approval_id,
            class,
            scope_digest,
            parent_decision_id,
            max_uses,
            current_uses,
        })
    }
}

fn decode_scope(
    class: ApprovalClass,
    action_scope: Vec<u8>,
    resource_scope: Vec<u8>,
    effect_id: Option<Vec<u8>>,
    session_id: Option<Vec<u8>>,
) -> Result<ApprovalScope, ApprovalUseError> {
    match class {
        ApprovalClass::Once => {
            let effect_id = effect_id
                .ok_or(ApprovalUseError::InvalidStoredRecord(
                    "ONCE approval is missing effect id",
                ))
                .and_then(|value| id16(value, "ONCE approval effect id is not 16 bytes"))?;
            if session_id.is_some() {
                return Err(ApprovalUseError::InvalidStoredRecord(
                    "ONCE approval has an unexpected session id",
                ));
            }
            let action = utf8(&action_scope, "approval action scope is not UTF-8")?;
            let resource = utf8(&resource_scope, "approval resource scope is not UTF-8")?;
            Ok(ApprovalScope::once(
                EffectId(u128::from_be_bytes(effect_id)),
                action,
                resource,
            )?)
        }
        ApprovalClass::SessionScoped => {
            if effect_id.is_some() {
                return Err(ApprovalUseError::InvalidStoredRecord(
                    "SESSION_SCOPED approval has an unexpected effect id",
                ));
            }
            let session_id = session_id
                .ok_or(ApprovalUseError::InvalidStoredRecord(
                    "SESSION_SCOPED approval is missing session id",
                ))
                .and_then(|value| id16(value, "approval session id is not 16 bytes"))?;
            let actions = decode_set(&action_scope, "approval action scope is not canonical")?;
            let resources =
                decode_set(&resource_scope, "approval resource scope is not canonical")?;
            let scope = ApprovalScope::session_scoped(
                SessionId(u128::from_be_bytes(session_id)),
                &actions,
                &resources,
            )?;
            require_canonical_sets(&scope, &action_scope, &resource_scope)?;
            Ok(scope)
        }
        ApprovalClass::TimeBoxed => {
            if effect_id.is_some() || session_id.is_some() {
                return Err(ApprovalUseError::InvalidStoredRecord(
                    "TIME_BOXED approval has unexpected effect/session binding",
                ));
            }
            let actions = decode_set(&action_scope, "approval action scope is not canonical")?;
            let resources =
                decode_set(&resource_scope, "approval resource scope is not canonical")?;
            let scope = ApprovalScope::time_boxed(&actions, &resources)?;
            require_canonical_sets(&scope, &action_scope, &resource_scope)?;
            Ok(scope)
        }
        ApprovalClass::OperationPattern => {
            if effect_id.is_some() || session_id.is_some() {
                return Err(ApprovalUseError::InvalidStoredRecord(
                    "OPERATION_PATTERN approval has unexpected effect/session binding",
                ));
            }
            let action = utf8(&action_scope, "approval action pattern is not UTF-8")?;
            let resource = utf8(&resource_scope, "approval resource pattern is not UTF-8")?;
            Ok(ApprovalScope::operation_pattern(action, resource)?)
        }
        ApprovalClass::RunPreauthorization => {
            if effect_id.is_some() {
                return Err(ApprovalUseError::InvalidStoredRecord(
                    "RUN_PREAUTHORIZATION approval has an unexpected effect id",
                ));
            }
            let session_id = session_id
                .map(|value| id16(value, "approval session id is not 16 bytes"))
                .transpose()?
                .map(|value| SessionId(u128::from_be_bytes(value)));
            let actions = decode_set(&action_scope, "approval action scope is not canonical")?;
            let resources =
                decode_set(&resource_scope, "approval resource scope is not canonical")?;
            let scope = ApprovalScope::run_preauthorization(session_id, &actions, &resources)?;
            require_canonical_sets(&scope, &action_scope, &resource_scope)?;
            Ok(scope)
        }
    }
}

fn require_canonical_sets(
    scope: &ApprovalScope,
    stored_actions: &[u8],
    stored_resources: &[u8],
) -> Result<(), ApprovalUseError> {
    let (actions, resources) = match scope {
        ApprovalScope::SessionScoped {
            actions, resources, ..
        }
        | ApprovalScope::TimeBoxed { actions, resources }
        | ApprovalScope::RunPreauthorization {
            actions, resources, ..
        } => (actions, resources),
        _ => return Ok(()),
    };
    if encode_set(actions) != stored_actions || encode_set(resources) != stored_resources {
        return Err(ApprovalUseError::InvalidStoredRecord(
            "approval set scope is not strictly sorted and unique",
        ));
    }
    Ok(())
}

fn scope_matches(
    scope: &ApprovalScope,
    request: &ApprovalUseRequest<'_>,
) -> Result<bool, ApprovalUseError> {
    Ok(match scope {
        ApprovalScope::Once {
            effect_id,
            action,
            resource,
        } => {
            request.effect_id == Some(*effect_id)
                && request.action == action
                && request.resource == resource
        }
        ApprovalScope::SessionScoped {
            session_id,
            actions,
            resources,
        } => {
            request.session_id == Some(*session_id)
                && contains(actions, request.action)
                && contains(resources, request.resource)
        }
        ApprovalScope::TimeBoxed { actions, resources } => {
            contains(actions, request.action) && contains(resources, request.resource)
        }
        ApprovalScope::OperationPattern {
            action_pattern,
            resource_pattern,
        } => {
            bounded_pattern_matches(action_pattern, request.action)?
                && bounded_pattern_matches(resource_pattern, request.resource)?
        }
        ApprovalScope::RunPreauthorization {
            session_id,
            actions,
            resources,
        } => {
            session_id.is_none_or(|expected| request.session_id == Some(expected))
                && contains(actions, request.action)
                && contains(resources, request.resource)
        }
    })
}

fn bounded_pattern_matches(pattern: &str, value: &str) -> Result<bool, ApprovalUseError> {
    let Some(wildcard) = pattern.find('*') else {
        return Ok(pattern == value);
    };
    if pattern[wildcard + 1..].contains('*') {
        return Err(ApprovalUseError::InvalidStoredRecord(
            "approval operation pattern contains multiple wildcards",
        ));
    }
    let prefix = &pattern[..wildcard];
    let suffix = &pattern[wildcard + 1..];
    Ok(value.len() >= prefix.len() + suffix.len()
        && value.starts_with(prefix)
        && value.ends_with(suffix))
}

fn validate_use_request(request: &ApprovalUseRequest<'_>) -> Result<(), ApprovalUseError> {
    if !valid_action(request.action)
        || !valid_resource(request.resource)
        || !valid_risk_class(request.risk_class)
    {
        return Err(ApprovalUseError::InvalidUseRequest);
    }
    Ok(())
}

fn valid_action(value: &str) -> bool {
    let bytes = value.as_bytes();
    !value.is_empty()
        && value.len() <= MAX_ACTION_BYTES
        && bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && !bytes.windows(2).any(|pair| pair == b"..")
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-')
        })
}

fn valid_resource(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_RESOURCE_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn valid_risk_class(value: &str) -> bool {
    let bytes = value.as_bytes();
    !value.is_empty()
        && value.len() <= MAX_RISK_CLASS_BYTES
        && bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(*byte, b'_' | b'-')
        })
}

fn bound_scope_digest(
    intent_digest: [u8; 32],
    parent_decision_id: [u8; 16],
    context_hash: [u8; 32],
) -> Result<[u8; 32], ApprovalUseError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(APPROVAL_BINDING_DOMAIN)?;
    encoder.push_bytes(&intent_digest)?;
    encoder.push_bytes(&parent_decision_id)?;
    encoder.push_bytes(&context_hash)?;
    Ok(*blake3::hash(&encoder.finish()).as_bytes())
}

fn decode_set(bytes: &[u8], reason: &'static str) -> Result<Vec<String>, ApprovalUseError> {
    let text = utf8(bytes, reason)?;
    if text.is_empty() {
        return Err(ApprovalUseError::InvalidStoredRecord(reason));
    }
    Ok(text.split('\n').map(str::to_owned).collect())
}

fn encode_set(values: &[String]) -> Vec<u8> {
    values.join("\n").into_bytes()
}

fn contains(values: &[String], value: &str) -> bool {
    values
        .binary_search_by(|candidate| candidate.as_str().cmp(value))
        .is_ok()
}

fn utf8<'a>(bytes: &'a [u8], reason: &'static str) -> Result<&'a str, ApprovalUseError> {
    std::str::from_utf8(bytes).map_err(|_| ApprovalUseError::InvalidStoredRecord(reason))
}

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], ApprovalUseError> {
    value
        .try_into()
        .map_err(|_| ApprovalUseError::InvalidStoredRecord(reason))
}

fn id16(value: Vec<u8>, reason: &'static str) -> Result<[u8; 16], ApprovalUseError> {
    value
        .try_into()
        .map_err(|_| ApprovalUseError::InvalidStoredRecord(reason))
}

fn positive_u64(value: Option<i64>, reason: &'static str) -> Result<u64, ApprovalUseError> {
    let value = value.ok_or(ApprovalUseError::InvalidStoredRecord(reason))?;
    let value = u64::try_from(value).map_err(|_| ApprovalUseError::InvalidStoredRecord(reason))?;
    if value == 0 {
        return Err(ApprovalUseError::InvalidStoredRecord(reason));
    }
    Ok(value)
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

    #[test]
    fn bounded_operation_pattern_supports_one_wildcard_and_rejects_ambiguous_patterns() {
        assert!(bounded_pattern_matches("effect.*", "effect.simulate").unwrap());
        assert!(bounded_pattern_matches("session:*:write", "session:7:write").unwrap());
        assert!(!bounded_pattern_matches("session:*:write", "session:7:read").unwrap());
        assert!(matches!(
            bounded_pattern_matches("**", "anything"),
            Err(ApprovalUseError::InvalidStoredRecord(_))
        ));
    }

    #[test]
    fn freshness_uses_half_open_expiry_boundary() {
        assert!(valid_utc_second("2026-08-27T00:00:00Z"));
        assert!(!valid_utc_second("2026-08-27T00:00:60Z"));
        assert!("2026-08-27T00:59:59Z" < "2026-08-27T01:00:00Z");
    }
}
