#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::approval_binding::{APPROVAL_ISSUE_ACTION, ApprovalBindingError, prepare_approval};
use crate::approval_runtime::ApprovalUseRequest;
use crate::approvals::{ApprovalScope, ApprovalScopeError};
use crate::authority_security_write::append_approval_consumption_snapshot;
use crate::storage::{AuthorityStore, StorageError};

const APPROVAL_BINDING_DOMAIN: &[u8] = b"golam:approval-binding:v1";
const APPROVAL_CONSUMPTION_ID_DOMAIN: &[u8] = b"golam:approval-consumption:v1";
const MAX_ACTION_BYTES: usize = 128;
const MAX_RESOURCE_BYTES: usize = 2_048;
const MAX_RISK_CLASS_BYTES: usize = 128;
const EXECUTION_PROGRESS_STATES: &[&str] = &[
    "executing",
    "succeeded",
    "failed",
    "unknown_outcome",
    "reconciling",
    "manual_review",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovalReservation {
    consumption_id: [u8; 16],
    approval_id: [u8; 16],
    effect_id: EffectId,
    reserved_global_seq: u64,
}

impl ApprovalReservation {
    pub const fn consumption_id(self) -> [u8; 16] {
        self.consumption_id
    }

    pub const fn approval_id(self) -> [u8; 16] {
        self.approval_id
    }

    pub const fn effect_id(self) -> EffectId {
        self.effect_id
    }

    pub const fn reserved_global_seq(self) -> u64 {
        self.reserved_global_seq
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovalConsumption {
    reservation: ApprovalReservation,
    consumed_global_seq: u64,
}

impl ApprovalConsumption {
    pub const fn reservation(self) -> ApprovalReservation {
        self.reservation
    }

    pub const fn consumed_global_seq(self) -> u64 {
        self.consumed_global_seq
    }
}

#[derive(Debug)]
pub enum ApprovalConsumptionError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Scope(ApprovalScopeError),
    Binding(ApprovalBindingError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidUseRequest,
    InvalidObservedTime,
    ApprovalNotFound,
    ApprovalNotOnce,
    ApprovalMismatch,
    ParentDecisionNotFound,
    ParentDecisionMismatch,
    BindingMismatch,
    NotYetIssued,
    Expired,
    Revoked,
    EffectNotFound,
    EffectMismatch,
    EffectNotReady,
    EffectNotProgressed,
    ReservationAlreadyExists,
    ReservationNotFound,
    ReservationMismatch,
    ReservationStateMismatch,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for ApprovalConsumptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "approval consumption authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "approval consumption sqlite error: {error}"),
            Self::Core(error) => write!(f, "approval consumption canonical encoding error: {error}"),
            Self::Scope(error) => write!(f, "approval consumption scope error: {error}"),
            Self::Binding(error) => write!(f, "approval consumption binding error: {error}"),
            Self::Integrity(error) => write!(f, "approval consumption integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "approval consumption authority-security error: {error}")
            }
            Self::InvalidUseRequest => f.write_str("ONCE approval use request is not canonical"),
            Self::InvalidObservedTime => {
                f.write_str("ONCE approval observed use time is not canonical UTC-second time")
            }
            Self::ApprovalNotFound => f.write_str("ONCE approval does not exist"),
            Self::ApprovalNotOnce => f.write_str("approval is not ONCE authority"),
            Self::ApprovalMismatch => {
                f.write_str("ONCE approval does not match the exact protected effect use")
            }
            Self::ParentDecisionNotFound => {
                f.write_str("ONCE approval parent authorization decision does not exist")
            }
            Self::ParentDecisionMismatch => f.write_str(
                "ONCE approval parent authorization decision does not match issuance authority",
            ),
            Self::BindingMismatch => {
                f.write_str("ONCE approval scope digest does not match its issuance binding")
            }
            Self::NotYetIssued => f.write_str("ONCE approval is not yet fresh for use"),
            Self::Expired => f.write_str("ONCE approval is expired"),
            Self::Revoked => f.write_str("ONCE approval is revoked"),
            Self::EffectNotFound => f.write_str("ONCE approval target effect does not exist"),
            Self::EffectMismatch => {
                f.write_str("ONCE approval target effect does not match action/resource/risk")
            }
            Self::EffectNotReady => {
                f.write_str("ONCE approval target effect is not immediately execution-ready")
            }
            Self::EffectNotProgressed => f.write_str(
                "ONCE approval reservation cannot be consumed before protected execution progresses",
            ),
            Self::ReservationAlreadyExists => f.write_str(
                "ONCE approval already has durable reservation or consumption state; retry is blocked",
            ),
            Self::ReservationNotFound => f.write_str("ONCE approval reservation does not exist"),
            Self::ReservationMismatch => {
                f.write_str("ONCE approval reservation evidence does not match protected state")
            }
            Self::ReservationStateMismatch => {
                f.write_str("ONCE approval reservation state is not consumable")
            }
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored ONCE approval consumption record is invalid: {reason}")
            }
        }
    }
}

impl Error for ApprovalConsumptionError {
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

impl From<StorageError> for ApprovalConsumptionError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for ApprovalConsumptionError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for ApprovalConsumptionError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<ApprovalScopeError> for ApprovalConsumptionError {
    fn from(value: ApprovalScopeError) -> Self {
        Self::Scope(value)
    }
}

impl From<ApprovalBindingError> for ApprovalConsumptionError {
    fn from(value: ApprovalBindingError) -> Self {
        Self::Binding(value)
    }
}

pub struct ApprovalConsumptionStore {
    connection: Connection,
}

impl ApprovalConsumptionStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, ApprovalConsumptionError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    /// Atomically revalidates and reserves exactly one ONCE approval use.
    ///
    /// The write transaction is acquired before freshness/scope/effect checks,
    /// so a competing reservation cannot pass the same pre-use snapshot. Any
    /// durable reservation, including one left by a crash, blocks blind retry.
    pub fn reserve_once(
        &mut self,
        request: ApprovalUseRequest<'_>,
    ) -> Result<ApprovalReservation, ApprovalConsumptionError> {
        validate_request(&request)?;
        if !valid_utc_second(request.observed_at) {
            return Err(ApprovalConsumptionError::InvalidObservedTime);
        }
        let effect_id = request
            .effect_id
            .ok_or(ApprovalConsumptionError::InvalidUseRequest)?;
        if request.session_id.is_some() {
            return Err(ApprovalConsumptionError::InvalidUseRequest);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        verify_once_approval(&transaction, &request, effect_id)?;
        if transaction
            .query_row(
                "SELECT 1 FROM approval_consumptions WHERE approval_id = ?1 LIMIT 1",
                params![&request.approval_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some()
        {
            return Err(ApprovalConsumptionError::ReservationAlreadyExists);
        }

        let reserved_global_seq = verify_effect_ready(&transaction, &request, effect_id)?;
        let consumption_id = approval_consumption_id(request.approval_id, effect_id);
        transaction.execute(
            "INSERT INTO approval_consumptions (consumption_id, approval_id, effect_or_operation_id, reserved_global_seq, consumed_global_seq, state) VALUES (?1, ?2, ?3, ?4, NULL, 'reserved')",
            params![
                &consumption_id[..],
                &request.approval_id[..],
                &effect_id.0.to_be_bytes()[..],
                to_i64(reserved_global_seq)?,
            ],
        )?;
        append_approval_consumption_snapshot(&transaction, &consumption_id)
            .map_err(|error| ApprovalConsumptionError::AuthoritySecurity(error.to_string()))?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| ApprovalConsumptionError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(ApprovalReservation {
            consumption_id,
            approval_id: request.approval_id,
            effect_id,
            reserved_global_seq,
        })
    }

    /// Marks a durable ONCE reservation consumed only after the bound effect
    /// has progressed beyond its reserved `authorized` transition. Repeating
    /// consumption for the same sealed reservation is idempotent and never
    /// creates a second approval use.
    pub fn consume_once(
        &mut self,
        reservation: ApprovalReservation,
    ) -> Result<ApprovalConsumption, ApprovalConsumptionError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let row = transaction
            .query_row(
                "SELECT approval_id, effect_or_operation_id, reserved_global_seq, consumed_global_seq, state FROM approval_consumptions WHERE consumption_id = ?1",
                params![&reservation.consumption_id[..]],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, Option<i64>>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()?
            .ok_or(ApprovalConsumptionError::ReservationNotFound)?;
        let approval_id = id16(row.0, "reservation approval id is not 16 bytes")?;
        let effect_id = id16(row.1, "reservation effect id is not 16 bytes")?;
        let reserved_global_seq = from_i64(row.2, "reservation global sequence is invalid")?;
        if approval_id != reservation.approval_id
            || effect_id != reservation.effect_id.0.to_be_bytes()
            || reserved_global_seq != reservation.reserved_global_seq
        {
            return Err(ApprovalConsumptionError::ReservationMismatch);
        }

        if row.4 == "consumed" {
            let consumed_global_seq = row
                .3
                .ok_or(ApprovalConsumptionError::InvalidStoredRecord(
                    "consumed reservation is missing consumed global sequence",
                ))
                .and_then(|value| from_i64(value, "consumed global sequence is invalid"))?;
            if consumed_global_seq <= reserved_global_seq {
                return Err(ApprovalConsumptionError::InvalidStoredRecord(
                    "consumed global sequence does not follow reservation",
                ));
            }
            transaction.commit()?;
            return Ok(ApprovalConsumption {
                reservation,
                consumed_global_seq,
            });
        }
        if row.4 != "reserved" || row.3.is_some() {
            return Err(ApprovalConsumptionError::ReservationStateMismatch);
        }

        let consumed_global_seq =
            verify_effect_progress(&transaction, reservation.effect_id, reserved_global_seq)?;
        let updated = transaction.execute(
            "UPDATE approval_consumptions SET consumed_global_seq = ?1, state = 'consumed' WHERE consumption_id = ?2 AND state = 'reserved' AND consumed_global_seq IS NULL",
            params![
                to_i64(consumed_global_seq)?,
                &reservation.consumption_id[..],
            ],
        )?;
        if updated != 1 {
            return Err(ApprovalConsumptionError::ReservationStateMismatch);
        }
        append_approval_consumption_snapshot(&transaction, &reservation.consumption_id)
            .map_err(|error| ApprovalConsumptionError::AuthoritySecurity(error.to_string()))?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| ApprovalConsumptionError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(ApprovalConsumption {
            reservation,
            consumed_global_seq,
        })
    }
}

fn verify_transaction_integrity(
    transaction: &Transaction<'_>,
) -> Result<(), ApprovalConsumptionError> {
    crate::integrity::verify(transaction)
        .map_err(|error| ApprovalConsumptionError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| ApprovalConsumptionError::AuthoritySecurity(error.to_string()))
}

fn verify_once_approval(
    transaction: &Transaction<'_>,
    request: &ApprovalUseRequest<'_>,
    effect_id: EffectId,
) -> Result<(), ApprovalConsumptionError> {
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
        .ok_or(ApprovalConsumptionError::ApprovalNotFound)?;
    if row.0 != "ONCE" {
        return Err(ApprovalConsumptionError::ApprovalNotOnce);
    }
    let stored_effect = row
        .5
        .ok_or(ApprovalConsumptionError::InvalidStoredRecord(
            "ONCE approval is missing effect id",
        ))
        .and_then(|value| id16(value, "ONCE approval effect id is not 16 bytes"))?;
    if row.6.is_some()
        || row.3.as_slice() != request.action.as_bytes()
        || row.4.as_slice() != request.resource.as_bytes()
        || stored_effect != effect_id.0.to_be_bytes()
        || row.7 != request.risk_class
    {
        return Err(ApprovalConsumptionError::ApprovalMismatch);
    }
    let taint_digest = hash32(row.8, "approval taint digest is not 32 bytes")?;
    if taint_digest != request.taint_digest {
        return Err(ApprovalConsumptionError::ApprovalMismatch);
    }
    let max_uses = row
        .12
        .ok_or(ApprovalConsumptionError::InvalidStoredRecord(
            "ONCE approval max_uses is missing",
        ))
        .and_then(|value| from_i64(value, "ONCE approval max_uses is invalid"))?;
    if max_uses != 1 {
        return Err(ApprovalConsumptionError::InvalidStoredRecord(
            "ONCE approval max_uses is not one",
        ));
    }
    if !valid_utc_second(&row.10) {
        return Err(ApprovalConsumptionError::InvalidStoredRecord(
            "ONCE approval issued_at is malformed",
        ));
    }
    if request.observed_at < row.10.as_str() {
        return Err(ApprovalConsumptionError::NotYetIssued);
    }
    if let Some(expires_at) = row.11.as_deref() {
        if !valid_utc_second(expires_at) || row.10.as_str() >= expires_at {
            return Err(ApprovalConsumptionError::InvalidStoredRecord(
                "ONCE approval expiry is malformed or not after issuance",
            ));
        }
        if request.observed_at >= expires_at {
            return Err(ApprovalConsumptionError::Expired);
        }
    }
    if let Some(revoked_at) = row.13.as_deref() {
        if !valid_utc_second(revoked_at) || revoked_at < row.10.as_str() {
            return Err(ApprovalConsumptionError::InvalidStoredRecord(
                "ONCE approval revoked_at is malformed or predates issuance",
            ));
        }
        return Err(ApprovalConsumptionError::Revoked);
    }

    let scope = ApprovalScope::once(effect_id, request.action, request.resource)?;
    let prepared = prepare_approval(
        &row.1,
        scope,
        &row.7,
        taint_digest,
        &row.10,
        row.11.as_deref(),
        max_uses,
    )?;
    let scope_digest = hash32(row.2, "approval scope digest is not 32 bytes")?;
    let parent_decision_id = id16(row.9, "approval parent decision id is not 16 bytes")?;
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
        .ok_or(ApprovalConsumptionError::ParentDecisionNotFound)?;
    if parent.0 != row.1
        || parent.1 != APPROVAL_ISSUE_ACTION
        || parent.2 != prepared.resource()
        || parent.4 != "allow"
    {
        return Err(ApprovalConsumptionError::ParentDecisionMismatch);
    }
    let context_hash = hash32(
        parent.3,
        "approval parent authorization context hash is not 32 bytes",
    )?;
    let rebound = bound_scope_digest(
        prepared.intent_digest(),
        parent_decision_id,
        context_hash,
    )?;
    if rebound != scope_digest {
        return Err(ApprovalConsumptionError::BindingMismatch);
    }
    Ok(())
}

fn verify_effect_ready(
    transaction: &Transaction<'_>,
    request: &ApprovalUseRequest<'_>,
    effect_id: EffectId,
) -> Result<u64, ApprovalConsumptionError> {
    let row = transaction
        .query_row(
            "SELECT i.action, i.resource, i.risk_class, t.to_state, t.global_seq FROM effect_intents i JOIN effect_transitions t ON t.effect_id = i.effect_id WHERE i.effect_id = ?1 AND t.global_seq = (SELECT MAX(t2.global_seq) FROM effect_transitions t2 WHERE t2.effect_id = i.effect_id)",
            params![&effect_id.0.to_be_bytes()[..]],
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
        .ok_or(ApprovalConsumptionError::EffectNotFound)?;
    if row.0 != request.action || row.1 != request.resource || row.2 != request.risk_class {
        return Err(ApprovalConsumptionError::EffectMismatch);
    }
    if row.3 != "authorized" {
        return Err(ApprovalConsumptionError::EffectNotReady);
    }
    from_i64(row.4, "effect authorization global sequence is invalid")
}

fn verify_effect_progress(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    reserved_global_seq: u64,
) -> Result<u64, ApprovalConsumptionError> {
    let row = transaction
        .query_row(
            "SELECT to_state, global_seq FROM effect_transitions WHERE effect_id = ?1 ORDER BY global_seq DESC LIMIT 1",
            params![&effect_id.0.to_be_bytes()[..]],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()?
        .ok_or(ApprovalConsumptionError::EffectNotFound)?;
    let global_seq = from_i64(row.1, "effect progress global sequence is invalid")?;
    if global_seq <= reserved_global_seq || !EXECUTION_PROGRESS_STATES.contains(&row.0.as_str()) {
        return Err(ApprovalConsumptionError::EffectNotProgressed);
    }
    Ok(global_seq)
}

fn validate_request(request: &ApprovalUseRequest<'_>) -> Result<(), ApprovalConsumptionError> {
    if !valid_action(request.action)
        || !valid_resource(request.resource)
        || !valid_risk_class(request.risk_class)
    {
        return Err(ApprovalConsumptionError::InvalidUseRequest);
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
) -> Result<[u8; 32], ApprovalConsumptionError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(APPROVAL_BINDING_DOMAIN)?;
    encoder.push_bytes(&intent_digest)?;
    encoder.push_bytes(&parent_decision_id)?;
    encoder.push_bytes(&context_hash)?;
    Ok(*blake3::hash(&encoder.finish()).as_bytes())
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

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], ApprovalConsumptionError> {
    value
        .try_into()
        .map_err(|_| ApprovalConsumptionError::InvalidStoredRecord(reason))
}

fn id16(value: Vec<u8>, reason: &'static str) -> Result<[u8; 16], ApprovalConsumptionError> {
    value
        .try_into()
        .map_err(|_| ApprovalConsumptionError::InvalidStoredRecord(reason))
}

fn from_i64(value: i64, reason: &'static str) -> Result<u64, ApprovalConsumptionError> {
    u64::try_from(value).map_err(|_| ApprovalConsumptionError::InvalidStoredRecord(reason))
}

fn to_i64(value: u64) -> Result<i64, ApprovalConsumptionError> {
    i64::try_from(value).map_err(|_| {
        ApprovalConsumptionError::InvalidStoredRecord("approval consumption sequence overflows i64")
    })
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
    use crate::approval_binding::{APPROVAL_MUTATION_RISK_CLASS, ApprovalStore, PreparedApproval};
    use crate::approval_runtime::{ApprovalUseError, ApprovalUseStore};
    use crate::authorization::{
        AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionEvidence,
        AuthorizationDecisionKind,
    };
    use crate::dispatch::encode_effect_dependencies;
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_core::paths::RuntimeLayout;
    use golam_core::{EffectTransitionId, EventId, SessionId};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Barrier};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-approval-consumption-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn authorize_issue_effect(
        authority: &AuthorityLayout,
        prepared: &PreparedApproval,
        effect_id: EffectId,
    ) {
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let mut effects = EffectStore::open(authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(1),
                requested_by: "owner:owner",
                action: APPROVAL_ISSUE_ACTION,
                resource: prepared.resource(),
                risk_class: APPROVAL_MUTATION_RISK_CLASS,
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash: prepared.intent_digest(),
                proposed_event_id: EventId(effect_id.0 + 100),
                transition_id: EffectTransitionId(effect_id.0 + 101),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(effect_id.0 + 102),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("approval_issue_authorized"),
                evidence_ref: None,
                event_id: EventId(effect_id.0 + 103),
            })
            .unwrap();
    }

    fn create_target_effect(authority: &AuthorityLayout, effect_id: EffectId) {
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let mut effects = EffectStore::open(authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(7),
                requested_by: "owner:owner",
                action: "effect.simulate",
                resource: "session:7",
                risk_class: "irreversible_effect",
                execution_semantics: "irreversible",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash: [7; 32],
                proposed_event_id: EventId(effect_id.0 + 200),
                transition_id: EffectTransitionId(effect_id.0 + 201),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(effect_id.0 + 202),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("target_effect_authorized"),
                evidence_ref: None,
                event_id: EventId(effect_id.0 + 203),
            })
            .unwrap();
    }

    fn issue_bound_once_approval(authority: &AuthorityLayout, effect_id: EffectId) -> [u8; 16] {
        let prepared = prepare_approval(
            "owner:owner",
            ApprovalScope::once(effect_id, "effect.simulate", "session:7").unwrap(),
            "irreversible_effect",
            [9; 32],
            "2026-08-27T00:00:00Z",
            Some("2026-08-27T01:00:00Z"),
            1,
        )
        .unwrap();
        let issue_effect = EffectId(1_000 + effect_id.0);
        authorize_issue_effect(authority, &prepared, issue_effect);
        let mut log = AuthorizationAuditLog::open(authority).unwrap();
        let decision = log
            .append(AppendAuthorizationDecision {
                principal: "owner:owner",
                action: APPROVAL_ISSUE_ACTION,
                resource: prepared.resource(),
                context: "scope=local-owner",
                evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
                decision: AuthorizationDecisionKind::Allow,
                reason_code: "test_once_reservation_parent_authority",
            })
            .unwrap();
        drop(log);
        ApprovalStore::open(authority)
            .unwrap()
            .issue(prepared, decision.decision_id, issue_effect)
            .unwrap()
            .approval_id()
    }

    fn exact_use(approval_id: [u8; 16], effect_id: EffectId) -> ApprovalUseRequest<'static> {
        ApprovalUseRequest {
            approval_id,
            action: "effect.simulate",
            resource: "session:7",
            effect_id: Some(effect_id),
            session_id: None,
            risk_class: "irreversible_effect",
            taint_digest: [9; 32],
            observed_at: "2026-08-27T00:30:00Z",
        }
    }

    #[test]
    fn durable_reservation_blocks_retry_after_store_reopen() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(700);
        create_target_effect(&authority, effect_id);
        let approval_id = issue_bound_once_approval(&authority, effect_id);
        let mut store = ApprovalConsumptionStore::open(&authority).unwrap();
        let reservation = store.reserve_once(exact_use(approval_id, effect_id)).unwrap();
        assert_eq!(reservation.approval_id(), approval_id);
        assert_eq!(reservation.effect_id(), effect_id);
        drop(store);

        let mut reopened = ApprovalConsumptionStore::open(&authority).unwrap();
        assert!(matches!(
            reopened.reserve_once(exact_use(approval_id, effect_id)),
            Err(ApprovalConsumptionError::ReservationAlreadyExists)
        ));
        drop(reopened);
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn competing_once_reservations_allow_exactly_one_writer() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(710);
        create_target_effect(&authority, effect_id);
        let approval_id = issue_bound_once_approval(&authority, effect_id);
        let barrier = Arc::new(Barrier::new(3));
        let first_authority = authority.clone();
        let second_authority = authority.clone();
        let first_barrier = Arc::clone(&barrier);
        let second_barrier = Arc::clone(&barrier);

        let (first, second) = std::thread::scope(|scope| {
            let first = scope.spawn(move || {
                let mut store = ApprovalConsumptionStore::open(&first_authority).unwrap();
                first_barrier.wait();
                match store.reserve_once(exact_use(approval_id, effect_id)) {
                    Ok(_) => true,
                    Err(ApprovalConsumptionError::ReservationAlreadyExists) => false,
                    Err(error) => panic!("unexpected first reservation result: {error}"),
                }
            });
            let second = scope.spawn(move || {
                let mut store = ApprovalConsumptionStore::open(&second_authority).unwrap();
                second_barrier.wait();
                match store.reserve_once(exact_use(approval_id, effect_id)) {
                    Ok(_) => true,
                    Err(ApprovalConsumptionError::ReservationAlreadyExists) => false,
                    Err(error) => panic!("unexpected second reservation result: {error}"),
                }
            });
            barrier.wait();
            (first.join().unwrap(), second.join().unwrap())
        });
        assert_ne!(first, second);
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn consumption_requires_effect_progress_and_is_idempotent_for_same_reservation() {
        let (runtime, authority) = authority();
        let effect_id = EffectId(720);
        create_target_effect(&authority, effect_id);
        let approval_id = issue_bound_once_approval(&authority, effect_id);
        let mut store = ApprovalConsumptionStore::open(&authority).unwrap();
        let reservation = store.reserve_once(exact_use(approval_id, effect_id)).unwrap();
        assert!(matches!(
            store.consume_once(reservation),
            Err(ApprovalConsumptionError::EffectNotProgressed)
        ));

        let mut effects = EffectStore::open(&authority).unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(effect_id.0 + 204),
                effect_id,
                expected_state: "authorized",
                next_state: "executing",
                attempt_id: None,
                reason_code: Some("protected_execution_started"),
                evidence_ref: None,
                event_id: EventId(effect_id.0 + 205),
            })
            .unwrap();
        drop(effects);

        let consumed = store.consume_once(reservation).unwrap();
        assert!(consumed.consumed_global_seq() > reservation.reserved_global_seq());
        assert_eq!(store.consume_once(reservation).unwrap(), consumed);
        drop(store);

        let mut use_store = ApprovalUseStore::open(&authority).unwrap();
        assert!(matches!(
            use_store.validate(exact_use(approval_id, effect_id)),
            Err(ApprovalUseError::UsageLimitReached)
        ));
        drop(use_store);
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
