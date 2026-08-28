#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId, EventId, SessionId};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::EventKind;
use crate::authority_security_write::append_secret_handle_snapshot;
use crate::secret_mutation::{
    PreparedSecretCreate, SecretMutationError, SecretMutationOutcome, SecretMutationStore,
    prepare_secret_create,
};
use crate::storage::{AppendEvent, AuthorityStore, StorageError};

const HANDLE_ID_BYTES: usize = 16;
const MAX_ACTOR_BYTES: usize = 512;
const MAX_PURPOSE_BYTES: usize = 4096;
const MAX_TIME_BYTES: usize = 128;
const ENTRY_PROJECTION_DOMAIN: &[u8] = b"golam:designated-secret-entry:v1";
const TOMBSTONE_MARKER: &[u8] = b"<redacted-secret>";

pub(crate) struct PrepareDesignatedSecretEntryRequest<'a> {
    pub session_id: SessionId,
    pub expected_session_seq: u64,
    pub event_id: EventId,
    pub actor_principal: &'a str,
    pub owner_principal: &'a str,
    pub recorded_at: &'a str,
    pub classification: &'a str,
    pub purpose_scope: &'a str,
    pub expires_at: Option<&'a str>,
    pub value: Vec<u8>,
}

pub(crate) struct PreparedDesignatedSecretEntry {
    secret: PreparedSecretCreate,
    session_id: SessionId,
    expected_session_seq: u64,
    event_id: EventId,
    actor_principal: String,
    recorded_at: String,
    classification: String,
    purpose_scope: String,
    expires_at: Option<String>,
}

impl PreparedDesignatedSecretEntry {
    pub(crate) fn resource(&self) -> &str {
        self.secret.resource()
    }

    pub(crate) const fn intent_digest(&self) -> [u8; 32] {
        self.secret.intent_digest()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DesignatedSecretEntryReceipt {
    handle_id: [u8; HANDLE_ID_BYTES],
    version: u64,
    event_global_seq: u64,
    event_session_seq: u64,
}

impl DesignatedSecretEntryReceipt {
    pub(crate) const fn handle_id(&self) -> [u8; HANDLE_ID_BYTES] {
        self.handle_id
    }

    pub(crate) const fn version(&self) -> u64 {
        self.version
    }

    pub(crate) const fn event_global_seq(&self) -> u64 {
        self.event_global_seq
    }

    pub(crate) const fn event_session_seq(&self) -> u64 {
        self.event_session_seq
    }
}

#[derive(Debug)]
pub(crate) enum SecretEntryError {
    Mutation(SecretMutationError),
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Random(getrandom::Error),
    AuthoritySecurity(String),
    InvalidActor,
    InvalidPurpose,
    InvalidTime,
    HandleCollision,
    CreatedSecretUnavailable,
    IntegerOverflow,
}

impl fmt::Display for SecretEntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mutation(error) => write!(f, "designated secret entry mutation failed: {error}"),
            Self::Storage(error) => write!(f, "designated secret entry storage failed: {error}"),
            Self::Sqlite(error) => write!(f, "designated secret entry sqlite failed: {error}"),
            Self::Core(error) => write!(f, "designated secret entry encoding failed: {error}"),
            Self::Random(error) => {
                write!(f, "designated secret entry random source failed: {error}")
            }
            Self::AuthoritySecurity(error) => {
                write!(
                    f,
                    "designated secret entry authority-security failed: {error}"
                )
            }
            Self::InvalidActor => f.write_str("designated secret entry actor is invalid"),
            Self::InvalidPurpose => f.write_str("designated secret entry purpose is invalid"),
            Self::InvalidTime => f.write_str("designated secret entry time metadata is invalid"),
            Self::HandleCollision => f.write_str("designated secret entry handle collision"),
            Self::CreatedSecretUnavailable => {
                f.write_str("designated secret entry created secret is unavailable or stale")
            }
            Self::IntegerOverflow => f.write_str("designated secret entry integer overflow"),
        }
    }
}

impl Error for SecretEntryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Mutation(error) => Some(error),
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Random(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SecretMutationError> for SecretEntryError {
    fn from(value: SecretMutationError) -> Self {
        Self::Mutation(value)
    }
}

impl From<StorageError> for SecretEntryError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for SecretEntryError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for SecretEntryError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<getrandom::Error> for SecretEntryError {
    fn from(value: getrandom::Error) -> Self {
        Self::Random(value)
    }
}

pub(crate) fn prepare_designated_secret_entry(
    request: PrepareDesignatedSecretEntryRequest<'_>,
) -> Result<PreparedDesignatedSecretEntry, SecretEntryError> {
    validate_actor(request.actor_principal)?;
    validate_purpose(request.purpose_scope)?;
    validate_time(request.recorded_at)?;
    if let Some(expires_at) = request.expires_at {
        validate_time(expires_at)?;
        if expires_at <= request.recorded_at {
            return Err(SecretEntryError::InvalidTime);
        }
    }

    // The complete submitted value crosses directly into the already-qualified
    // protected secret-create preparation path. No syntax detector, parser or
    // recognized-format branch participates in this guarantee.
    let secret = prepare_secret_create(
        request.classification,
        request.owner_principal,
        request.value,
    )?;

    Ok(PreparedDesignatedSecretEntry {
        secret,
        session_id: request.session_id,
        expected_session_seq: request.expected_session_seq,
        event_id: request.event_id,
        actor_principal: request.actor_principal.to_owned(),
        recorded_at: request.recorded_at.to_owned(),
        classification: request.classification.to_owned(),
        purpose_scope: request.purpose_scope.to_owned(),
        expires_at: request.expires_at.map(str::to_owned),
    })
}

pub(crate) struct SecretEntryStore<'a> {
    layout: &'a AuthorityLayout,
}

impl<'a> SecretEntryStore<'a> {
    pub(crate) const fn new(layout: &'a AuthorityLayout) -> Self {
        Self { layout }
    }

    pub(crate) fn commit(
        &self,
        prepared: PreparedDesignatedSecretEntry,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<DesignatedSecretEntryReceipt, SecretEntryError> {
        let metadata = EntryMetadata::from_prepared(&prepared);
        let mut mutations = SecretMutationStore::open(self.layout)?;
        let created = mutations.create(
            prepared.secret,
            authority_decision_id,
            approval_id,
            effect_id,
        )?;
        drop(mutations);
        self.finalize(metadata, created)
    }

    #[cfg(test)]
    pub(crate) fn commit_with_protector<P: crate::secret_vault::KeyProtector>(
        &self,
        prepared: PreparedDesignatedSecretEntry,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
        protector: P,
    ) -> Result<DesignatedSecretEntryReceipt, SecretEntryError> {
        let metadata = EntryMetadata::from_prepared(&prepared);
        let mut mutations = SecretMutationStore::open(self.layout)?;
        let created = mutations.create_with_protector(
            prepared.secret,
            authority_decision_id,
            approval_id,
            effect_id,
            protector,
        )?;
        drop(mutations);
        self.finalize(metadata, created)
    }

    fn finalize(
        &self,
        metadata: EntryMetadata,
        created: SecretMutationOutcome,
    ) -> Result<DesignatedSecretEntryReceipt, SecretEntryError> {
        let handle_id = issue_handle(self.layout, &metadata, created)?;
        let payload = projection_payload(handle_id, created.version(), &metadata)?;

        let mut store = AuthorityStore::open(self.layout.authority_db_path())?;
        let event = store.append_event(AppendEvent {
            event_id: metadata.event_id,
            session_id: metadata.session_id,
            expected_session_seq: metadata.expected_session_seq,
            kind: EventKind::SecretEntryRedacted,
            actor_principal: &metadata.actor_principal,
            recorded_at: &metadata.recorded_at,
            payload: &payload,
            security_critical: true,
        })?;
        Ok(DesignatedSecretEntryReceipt {
            handle_id,
            version: created.version(),
            event_global_seq: event.record.global_seq,
            event_session_seq: event.record.session_seq,
        })
    }
}

struct EntryMetadata {
    session_id: SessionId,
    expected_session_seq: u64,
    event_id: EventId,
    actor_principal: String,
    recorded_at: String,
    classification: String,
    purpose_scope: String,
    expires_at: Option<String>,
}

impl EntryMetadata {
    fn from_prepared(prepared: &PreparedDesignatedSecretEntry) -> Self {
        Self {
            session_id: prepared.session_id,
            expected_session_seq: prepared.expected_session_seq,
            event_id: prepared.event_id,
            actor_principal: prepared.actor_principal.clone(),
            recorded_at: prepared.recorded_at.clone(),
            classification: prepared.classification.clone(),
            purpose_scope: prepared.purpose_scope.clone(),
            expires_at: prepared.expires_at.clone(),
        }
    }
}

fn issue_handle(
    layout: &AuthorityLayout,
    metadata: &EntryMetadata,
    created: SecretMutationOutcome,
) -> Result<[u8; HANDLE_ID_BYTES], SecretEntryError> {
    let store = AuthorityStore::open(layout.authority_db_path())?;
    drop(store);
    let mut connection = Connection::open(layout.authority_db_path())?;
    connection.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
    )?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    crate::integrity::verify(&transaction)
        .map_err(|error| SecretEntryError::AuthoritySecurity(error.to_string()))?;
    crate::authority_security_v2::verify(&transaction)
        .map_err(|error| SecretEntryError::AuthoritySecurity(error.to_string()))?;

    let active = transaction
        .query_row(
            "SELECT current_version, status, revoked_at FROM secret_records WHERE secret_id = ?1",
            params![&created.secret_id()[..]],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()?;
    let Some((current_version, status, revoked_at)) = active else {
        return Err(SecretEntryError::CreatedSecretUnavailable);
    };
    if u64::try_from(current_version).ok() != Some(created.version())
        || status != "active"
        || revoked_at.is_some()
    {
        return Err(SecretEntryError::CreatedSecretUnavailable);
    }

    let mut handle_id = [0_u8; HANDLE_ID_BYTES];
    getrandom::fill(&mut handle_id)?;
    let exists = transaction
        .query_row(
            "SELECT 1 FROM secret_handles WHERE handle_id = ?1 LIMIT 1",
            params![&handle_id[..]],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if exists {
        return Err(SecretEntryError::HandleCollision);
    }
    transaction.execute(
        "INSERT INTO secret_handles (handle_id, secret_id, version_constraint, purpose_scope, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            &handle_id[..],
            &created.secret_id()[..],
            to_i64(created.version())?,
            metadata.purpose_scope.as_bytes(),
            metadata.expires_at.as_deref(),
        ],
    )?;
    append_secret_handle_snapshot(&transaction, &handle_id)
        .map_err(|error| SecretEntryError::AuthoritySecurity(error.to_string()))?;
    crate::authority_security_v2::verify(&transaction)
        .map_err(|error| SecretEntryError::AuthoritySecurity(error.to_string()))?;
    transaction.commit()?;
    Ok(handle_id)
}

fn projection_payload(
    handle_id: [u8; HANDLE_ID_BYTES],
    version: u64,
    metadata: &EntryMetadata,
) -> Result<Vec<u8>, SecretEntryError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(ENTRY_PROJECTION_DOMAIN)?;
    encoder.push_bytes(&handle_id)?;
    encoder.push_bytes(TOMBSTONE_MARKER)?;
    encoder.push_bytes(metadata.classification.as_bytes())?;
    encoder.push_bytes(metadata.purpose_scope.as_bytes())?;
    encoder.push_u64(version);
    match metadata.expires_at.as_deref() {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value.as_bytes())?;
        }
        None => encoder.push_u8(0),
    }
    Ok(encoder.finish())
}

fn validate_actor(value: &str) -> Result<(), SecretEntryError> {
    if value.is_empty()
        || value.len() > MAX_ACTOR_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(SecretEntryError::InvalidActor);
    }
    Ok(())
}

fn validate_purpose(value: &str) -> Result<(), SecretEntryError> {
    if value.is_empty()
        || value.len() > MAX_PURPOSE_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(SecretEntryError::InvalidPurpose);
    }
    Ok(())
}

fn validate_time(value: &str) -> Result<(), SecretEntryError> {
    if value.len() > MAX_TIME_BYTES || !valid_utc_second(value) {
        return Err(SecretEntryError::InvalidTime);
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

fn decimal(bytes: &[u8], start: usize, end: usize) -> u32 {
    bytes[start..end]
        .iter()
        .fold(0_u32, |value, byte| value * 10 + u32::from(*byte - b'0'))
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn to_i64(value: u64) -> Result<i64, SecretEntryError> {
    i64::try_from(value).map_err(|_| SecretEntryError::IntegerOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_security_write::{
        append_approval_snapshot, append_authorization_decision_v2_snapshot,
    };
    use crate::secret_mutation::{SECRET_CREATE_ACTION, SECRET_MUTATION_RISK_CLASS};
    use crate::secret_vault::{KeyProtectionError, KeyProtector};
    use crate::security_audit::{
        AuthorizationAuditInput, EffectIntentAuditInput, EffectTransitionAuditInput,
        append_authorization_decision, append_effect_intent, append_effect_transition,
    };
    use crate::storage::CreateSession;
    use golam_core::paths::RuntimeLayout;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    use zeroize::Zeroizing;

    const UNKNOWN_FORMAT_CANARY: &[u8] =
        b"orchid::seven-moons::not-a-recognized-token-format::T003-055";
    static N: AtomicU64 = AtomicU64::new(0);

    #[derive(Clone)]
    struct TestProtector {
        key: [u8; 32],
    }

    impl KeyProtector for TestProtector {
        fn load_master_key(&self) -> Result<Zeroizing<Vec<u8>>, KeyProtectionError> {
            Ok(Zeroizing::new(self.key.to_vec()))
        }

        fn store_master_key(&self, _key: &[u8]) -> Result<(), KeyProtectionError> {
            Err(KeyProtectionError::Unsupported)
        }
    }

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(
            std::env::temp_dir().join(format!("golam-secret-entry-{}-{t}-{n}", std::process::id())),
        )
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    struct WorkIds {
        effect: EffectId,
        decision: [u8; 16],
        approval: [u8; 16],
    }

    fn install_authorized_create(
        authority: &AuthorityLayout,
        prepared: &PreparedDesignatedSecretEntry,
    ) -> WorkIds {
        let effect = EffectId(5500);
        let effect_bytes = effect.0.to_be_bytes();
        let transition_id = [55_u8; 16];
        let decision = [56_u8; 16];
        let approval = [57_u8; 16];
        let session_id = [58_u8; 16];
        let proposed_event_id = [59_u8; 16];
        let transition_event_id = [60_u8; 16];
        let mut connection = Connection::open(authority.authority_db_path()).unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "INSERT INTO effect_intents (effect_id, session_id, requested_by, action, resource, risk_class, execution_semantics, idempotency_key, preconditions, dependencies, payload_hash, proposed_event_id) VALUES (?1, ?2, 'owner:owner', ?3, ?4, ?5, 'at_most_once', NULL, X'', X'', ?6, ?7)",
                params![
                    &effect_bytes[..],
                    &session_id[..],
                    SECRET_CREATE_ACTION,
                    prepared.resource(),
                    SECRET_MUTATION_RISK_CLASS,
                    &prepared.intent_digest()[..],
                    &proposed_event_id[..],
                ],
            )
            .unwrap();
        append_effect_intent(
            &transaction,
            EffectIntentAuditInput {
                effect_id: &effect_bytes,
                session_id: &session_id,
                requested_by: "owner:owner",
                action: SECRET_CREATE_ACTION,
                resource: prepared.resource(),
                risk_class: SECRET_MUTATION_RISK_CLASS,
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"",
                dependencies: b"",
                payload_hash: &prepared.intent_digest(),
                proposed_event_id: &proposed_event_id,
            },
        )
        .unwrap();
        transaction
            .execute(
                "INSERT INTO effect_transitions (transition_id, effect_id, global_seq, from_state, to_state, attempt_id, reason_code, evidence_ref, event_id) VALUES (?1, ?2, 2, NULL, 'authorized', NULL, NULL, NULL, ?3)",
                params![&transition_id[..], &effect_bytes[..], &transition_event_id[..]],
            )
            .unwrap();
        append_effect_transition(
            &transaction,
            EffectTransitionAuditInput {
                transition_id: &transition_id,
                effect_id: &effect_bytes,
                global_seq: 2,
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
                "INSERT INTO authorization_decisions (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, authority_evidence_version) VALUES (?1, 'owner:owner', ?2, ?3, ?4, 'allow', 'test_designated_secret_entry', 3, 'allow', NULL, NULL, NULL, NULL, X'', ?5, 2)",
                params![
                    &decision[..],
                    SECRET_CREATE_ACTION,
                    prepared.resource(),
                    &[0_u8; 32][..],
                    &approval[..],
                ],
            )
            .unwrap();
        append_authorization_decision(
            &transaction,
            AuthorizationAuditInput {
                decision_id: &decision,
                principal: "owner:owner",
                action: SECRET_CREATE_ACTION,
                resource: prepared.resource(),
                context_hash: &[0_u8; 32],
                decision: "allow",
                reason_code: "test_designated_secret_entry",
                global_seq: 3,
            },
        )
        .unwrap();
        append_authorization_decision_v2_snapshot(&transaction, &decision).unwrap();
        transaction
            .execute(
                "INSERT INTO approvals (approval_id, class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at) VALUES (?1, 'ONCE', 'owner:owner', ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, '2026-08-28T00:00:00Z', NULL, 1, NULL)",
                params![
                    &approval[..],
                    &[1_u8; 32][..],
                    SECRET_CREATE_ACTION.as_bytes(),
                    prepared.resource().as_bytes(),
                    &effect_bytes[..],
                    SECRET_MUTATION_RISK_CLASS,
                    &[0_u8; 32][..],
                    &decision[..],
                ],
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

    fn create_session(authority: &AuthorityLayout) -> SessionId {
        let session_id = SessionId(9001);
        let mut store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        store
            .create_session(CreateSession {
                session_id,
                event_id: EventId(9002),
                owner_principal: "owner:owner",
                actor_principal: "owner:owner",
                recorded_at: "2026-08-28T00:00:00Z",
                payload: b"designated-secret-entry-session",
                security_critical: false,
            })
            .unwrap();
        session_id
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle)
    }

    #[test]
    fn unknown_format_designated_value_becomes_handle_and_redacted_projection_only() {
        let (runtime, authority) = authority();
        let session_id = create_session(&authority);
        let prepared = prepare_designated_secret_entry(PrepareDesignatedSecretEntryRequest {
            session_id,
            expected_session_seq: 1,
            event_id: EventId(9003),
            actor_principal: "owner:owner",
            owner_principal: "owner:owner",
            recorded_at: "2026-08-28T00:01:00Z",
            classification: "api_credential",
            purpose_scope: "git.auth",
            expires_at: Some("2026-08-29T00:00:00Z"),
            value: UNKNOWN_FORMAT_CANARY.to_vec(),
        })
        .unwrap();
        let work = install_authorized_create(&authority, &prepared);
        let receipt = SecretEntryStore::new(&authority)
            .commit_with_protector(
                prepared,
                work.decision,
                work.approval,
                work.effect,
                TestProtector { key: [77_u8; 32] },
            )
            .unwrap();
        assert_eq!(receipt.version(), 1);
        assert_eq!(receipt.event_session_seq(), 2);
        assert_eq!(receipt.event_global_seq(), 4);

        let connection = Connection::open(authority.authority_db_path()).unwrap();
        let payload: Vec<u8> = connection
            .query_row(
                "SELECT payload_bytes FROM session_events WHERE event_id = ?1",
                params![&EventId(9003).0.to_be_bytes()[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!contains(&payload, UNKNOWN_FORMAT_CANARY));
        assert!(contains(&payload, TOMBSTONE_MARKER));
        assert!(contains(&payload, &receipt.handle_id()));
        let event_type: i64 = connection
            .query_row(
                "SELECT event_type FROM session_events WHERE event_id = ?1",
                params![&EventId(9003).0.to_be_bytes()[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(event_type, i64::from(EventKind::SecretEntryRedacted.code()));

        let ciphertext: Vec<u8> = connection
            .query_row("SELECT ciphertext FROM secret_versions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(!contains(&ciphertext, UNKNOWN_FORMAT_CANARY));
        let handle_secret_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM secret_handles h JOIN secret_records s ON s.secret_id = h.secret_id WHERE h.handle_id = ?1 AND h.version_constraint = 1 AND h.purpose_scope = ?2 AND s.status = 'active'",
                params![&receipt.handle_id()[..], b"git.auth".as_slice()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(handle_secret_count, 1);
        crate::integrity::verify(&connection).unwrap();
        crate::authority_security_v2::verify(&connection).unwrap();
        drop(connection);

        let catalog = crate::secrets::SecretCatalog::open(&authority).unwrap();
        let handle = catalog.handle(receipt.handle_id()).unwrap().unwrap();
        assert_eq!(handle.handle_id(), receipt.handle_id());
        assert_eq!(handle.version_constraint(), Some(1));
        assert_eq!(handle.purpose_scope(), b"git.auth");
        drop(catalog);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn preparation_is_detector_independent_and_rejects_only_metadata_or_secret_bounds() {
        let (_runtime, authority) = authority();
        let session_id = create_session(&authority);
        let arbitrary_binary = vec![0, 1, 2, 0, 255, 19, 88, 0, 7];
        let prepared = prepare_designated_secret_entry(PrepareDesignatedSecretEntryRequest {
            session_id,
            expected_session_seq: 1,
            event_id: EventId(9010),
            actor_principal: "owner:owner",
            owner_principal: "owner:owner",
            recorded_at: "2026-08-28T00:02:00Z",
            classification: "opaque_user_secret",
            purpose_scope: "local.test",
            expires_at: None,
            value: arbitrary_binary,
        })
        .unwrap();
        assert!(prepared.resource().starts_with("secret-create:"));
    }
}
