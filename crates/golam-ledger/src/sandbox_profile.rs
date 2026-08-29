#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::authority_security_write::{
    append_approval_consumption_snapshot, append_sandbox_profile_snapshot,
};
use crate::storage::{AuthorityStore, StorageError};

pub const SANDBOX_PROFILE_REGISTER_ACTION: &str = "sandbox.profile.register";
pub const SANDBOX_PROFILE_MUTATION_RISK_CLASS: &str = "sandbox_profile_mutation";

const PROFILE_INTENT_DOMAIN: &[u8] = b"golam:sandbox-profile-register-intent:v1";
const APPROVAL_CONSUMPTION_DOMAIN: &[u8] = b"golam:sandbox-profile-approval-consumption:v1";
const LIST_DOMAIN: &[u8] = b"golam:sandbox-profile-list:v1";
const MAX_PRINCIPAL_BYTES: usize = 512;
const MAX_ROOT_BYTES: usize = 2_048;
const MAX_ENV_NAME_BYTES: usize = 128;
const MAX_TOKEN_BYTES: usize = 512;
const MAX_LIST_ITEMS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxProfileClass {
    WasmWasiExtension,
    NativeUntrustedSubprocess,
    McpServer,
    SkillHelper,
    BrowserProtocolHelper,
    LocalModelSidecar,
}

impl SandboxProfileClass {
    const fn code(self) -> u8 {
        match self {
            Self::WasmWasiExtension => 1,
            Self::NativeUntrustedSubprocess => 2,
            Self::McpServer => 3,
            Self::SkillHelper => 4,
            Self::BrowserProtocolHelper => 5,
            Self::LocalModelSidecar => 6,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WasmWasiExtension => "wasm_wasi_extension",
            Self::NativeUntrustedSubprocess => "native_untrusted_subprocess",
            Self::McpServer => "mcp_server",
            Self::SkillHelper => "skill_helper",
            Self::BrowserProtocolHelper => "browser_protocol_helper",
            Self::LocalModelSidecar => "local_model_sidecar",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "wasm_wasi_extension" => Some(Self::WasmWasiExtension),
            "native_untrusted_subprocess" => Some(Self::NativeUntrustedSubprocess),
            "mcp_server" => Some(Self::McpServer),
            "skill_helper" => Some(Self::SkillHelper),
            "browser_protocol_helper" => Some(Self::BrowserProtocolHelper),
            "local_model_sidecar" => Some(Self::LocalModelSidecar),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxNetworkRule {
    DenyAll,
    LoopbackOnly,
    PermitRequired,
}

impl SandboxNetworkRule {
    const fn code(self) -> u8 {
        match self {
            Self::DenyAll => 1,
            Self::LoopbackOnly => 2,
            Self::PermitRequired => 3,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DenyAll => "deny_all",
            Self::LoopbackOnly => "loopback_only",
            Self::PermitRequired => "permit_required",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "deny_all" => Some(Self::DenyAll),
            "loopback_only" => Some(Self::LoopbackOnly),
            "permit_required" => Some(Self::PermitRequired),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxSpawnRule {
    Deny,
    DirectChildOnly,
    ManagedDescendants,
}

impl SandboxSpawnRule {
    const fn code(self) -> u8 {
        match self {
            Self::Deny => 1,
            Self::DirectChildOnly => 2,
            Self::ManagedDescendants => 3,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deny => "deny",
            Self::DirectChildOnly => "direct_child_only",
            Self::ManagedDescendants => "managed_descendants",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "deny" => Some(Self::Deny),
            "direct_child_only" => Some(Self::DirectChildOnly),
            "managed_descendants" => Some(Self::ManagedDescendants),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SandboxProfileDefinition<'a> {
    pub profile_id: [u8; 16],
    pub version: u64,
    pub class: SandboxProfileClass,
    pub filesystem_read_roots: &'a [&'a str],
    pub filesystem_write_roots: &'a [&'a str],
    pub network_rule: SandboxNetworkRule,
    pub environment_allowlist: &'a [&'a str],
    pub spawn_rule: SandboxSpawnRule,
    pub cpu_limit: Option<u64>,
    pub memory_limit: Option<u64>,
    pub time_limit: Option<u64>,
    pub output_limit: Option<u64>,
    pub device_allowlist: &'a [&'a str],
    pub ipc_allowlist: &'a [&'a str],
    pub inherited_handle_rules: &'a [&'a str],
    pub platform_requirements: &'a [&'a str],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedSandboxProfile {
    profile_id: [u8; 16],
    version: u64,
    class: SandboxProfileClass,
    filesystem_read_roots: Vec<String>,
    filesystem_read_roots_bytes: Vec<u8>,
    filesystem_write_roots: Vec<String>,
    filesystem_write_roots_bytes: Vec<u8>,
    network_rule: SandboxNetworkRule,
    environment_allowlist: Vec<String>,
    environment_allowlist_bytes: Vec<u8>,
    spawn_rule: SandboxSpawnRule,
    cpu_limit: Option<u64>,
    memory_limit: Option<u64>,
    time_limit: Option<u64>,
    output_limit: Option<u64>,
    device_allowlist: Vec<String>,
    device_allowlist_bytes: Vec<u8>,
    ipc_allowlist: Vec<String>,
    ipc_allowlist_bytes: Vec<u8>,
    inherited_handle_rules: Vec<String>,
    inherited_handle_rules_bytes: Vec<u8>,
    platform_requirements: Vec<String>,
    platform_requirements_bytes: Vec<u8>,
    registered_by_principal: String,
    mutation_taint_digest: [u8; 32],
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedSandboxProfile {
    pub const fn profile_id(&self) -> [u8; 16] {
        self.profile_id
    }

    pub const fn version(&self) -> u64 {
        self.version
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }

    pub const fn mutation_taint_digest(&self) -> [u8; 32] {
        self.mutation_taint_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxProfileRecord {
    pub profile_id: [u8; 16],
    pub version: u64,
    pub class: SandboxProfileClass,
    pub filesystem_read_roots: Vec<String>,
    pub filesystem_write_roots: Vec<String>,
    pub network_rule: SandboxNetworkRule,
    pub environment_allowlist: Vec<String>,
    pub spawn_rule: SandboxSpawnRule,
    pub cpu_limit: Option<u64>,
    pub memory_limit: Option<u64>,
    pub time_limit: Option<u64>,
    pub output_limit: Option<u64>,
    pub device_allowlist: Vec<String>,
    pub ipc_allowlist: Vec<String>,
    pub inherited_handle_rules: Vec<String>,
    pub platform_requirements: Vec<String>,
    pub status: String,
}

#[derive(Debug)]
pub enum SandboxProfileError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidProfileId,
    InvalidVersion,
    InvalidPrincipal,
    InvalidRoot,
    InvalidEnvironmentName,
    InvalidToken,
    TooManyItems,
    DuplicateItem,
    InvalidLimit,
    IntegerOverflow,
    MissingAuthorityDecision,
    AuthorityDecisionMismatch,
    StaleAuthorityDecision,
    EffectNotFound,
    EffectMismatch,
    ApprovalNotFound,
    ApprovalMismatch,
    ApprovalAlreadyUsed,
    DuplicateProfileVersion,
    ProfileNotFound,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for SandboxProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "sandbox profile authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "sandbox profile sqlite error: {error}"),
            Self::Core(error) => write!(f, "sandbox profile canonical encoding error: {error}"),
            Self::Integrity(error) => write!(f, "sandbox profile integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "sandbox profile authority-security error: {error}")
            }
            Self::InvalidProfileId => f.write_str("sandbox profile id must be non-zero"),
            Self::InvalidVersion => f.write_str("sandbox profile version must be non-zero"),
            Self::InvalidPrincipal => f.write_str("sandbox profile principal is not canonical"),
            Self::InvalidRoot => f.write_str("sandbox profile filesystem root is not canonical"),
            Self::InvalidEnvironmentName => {
                f.write_str("sandbox profile environment allowlist name is invalid")
            }
            Self::InvalidToken => f.write_str("sandbox profile token is invalid"),
            Self::TooManyItems => f.write_str("sandbox profile list exceeds bounded item count"),
            Self::DuplicateItem => f.write_str("sandbox profile list contains a duplicate"),
            Self::InvalidLimit => f.write_str("sandbox profile resource limit must be positive"),
            Self::IntegerOverflow => f.write_str("sandbox profile integer conversion overflow"),
            Self::MissingAuthorityDecision => {
                f.write_str("sandbox profile mutation has no durable authorization decision")
            }
            Self::AuthorityDecisionMismatch => {
                f.write_str("sandbox profile mutation authorization decision is mismatched")
            }
            Self::StaleAuthorityDecision => {
                f.write_str("sandbox profile mutation authorization decision is stale")
            }
            Self::EffectNotFound => f.write_str("sandbox profile mutation effect does not exist"),
            Self::EffectMismatch => f.write_str("sandbox profile mutation effect is mismatched"),
            Self::ApprovalNotFound => {
                f.write_str("sandbox profile mutation approval does not exist")
            }
            Self::ApprovalMismatch => {
                f.write_str("sandbox profile mutation approval is mismatched")
            }
            Self::ApprovalAlreadyUsed => {
                f.write_str("sandbox profile mutation approval was already consumed")
            }
            Self::DuplicateProfileVersion => {
                f.write_str("sandbox profile id/version already exists")
            }
            Self::ProfileNotFound => f.write_str("sandbox profile does not exist"),
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored sandbox profile record is invalid: {reason}")
            }
        }
    }
}

impl Error for SandboxProfileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for SandboxProfileError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for SandboxProfileError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for SandboxProfileError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

pub fn prepare_sandbox_profile(
    definition: SandboxProfileDefinition<'_>,
    registered_by_principal: &str,
    mutation_taint_digest: [u8; 32],
) -> Result<PreparedSandboxProfile, SandboxProfileError> {
    if definition.profile_id == [0; 16] {
        return Err(SandboxProfileError::InvalidProfileId);
    }
    if definition.version == 0 {
        return Err(SandboxProfileError::InvalidVersion);
    }
    validate_principal(registered_by_principal)?;

    let filesystem_read_roots = canonicalize_list(definition.filesystem_read_roots, validate_root)?;
    let filesystem_write_roots =
        canonicalize_list(definition.filesystem_write_roots, validate_root)?;
    let environment_allowlist =
        canonicalize_list(definition.environment_allowlist, validate_environment_name)?;
    let device_allowlist = canonicalize_list(definition.device_allowlist, validate_token)?;
    let ipc_allowlist = canonicalize_list(definition.ipc_allowlist, validate_token)?;
    let inherited_handle_rules =
        canonicalize_list(definition.inherited_handle_rules, validate_token)?;
    let platform_requirements =
        canonicalize_list(definition.platform_requirements, validate_token)?;

    validate_limit(definition.cpu_limit)?;
    validate_limit(definition.memory_limit)?;
    validate_limit(definition.time_limit)?;
    validate_limit(definition.output_limit)?;

    let filesystem_read_roots_bytes = encode_list(&filesystem_read_roots)?;
    let filesystem_write_roots_bytes = encode_list(&filesystem_write_roots)?;
    let environment_allowlist_bytes = encode_list(&environment_allowlist)?;
    let device_allowlist_bytes = encode_list(&device_allowlist)?;
    let ipc_allowlist_bytes = encode_list(&ipc_allowlist)?;
    let inherited_handle_rules_bytes = encode_list(&inherited_handle_rules)?;
    let platform_requirements_bytes = encode_list(&platform_requirements)?;

    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(PROFILE_INTENT_DOMAIN)?;
    encoder.push_bytes(&definition.profile_id)?;
    encoder.push_u64(definition.version);
    encoder.push_u8(definition.class.code());
    encoder.push_bytes(&filesystem_read_roots_bytes)?;
    encoder.push_bytes(&filesystem_write_roots_bytes)?;
    encoder.push_u8(definition.network_rule.code());
    encoder.push_bytes(&environment_allowlist_bytes)?;
    encoder.push_u8(definition.spawn_rule.code());
    encode_optional_u64(&mut encoder, definition.cpu_limit);
    encode_optional_u64(&mut encoder, definition.memory_limit);
    encode_optional_u64(&mut encoder, definition.time_limit);
    encode_optional_u64(&mut encoder, definition.output_limit);
    encoder.push_bytes(&device_allowlist_bytes)?;
    encoder.push_bytes(&ipc_allowlist_bytes)?;
    encoder.push_bytes(&inherited_handle_rules_bytes)?;
    encoder.push_bytes(&platform_requirements_bytes)?;
    encoder.push_bytes(registered_by_principal.as_bytes())?;
    encoder.push_bytes(&mutation_taint_digest)?;
    let intent_digest = crate::payload_hash(&encoder.finish());
    let resource = sandbox_profile_resource(definition.profile_id, definition.version);

    Ok(PreparedSandboxProfile {
        profile_id: definition.profile_id,
        version: definition.version,
        class: definition.class,
        filesystem_read_roots,
        filesystem_read_roots_bytes,
        filesystem_write_roots,
        filesystem_write_roots_bytes,
        network_rule: definition.network_rule,
        environment_allowlist,
        environment_allowlist_bytes,
        spawn_rule: definition.spawn_rule,
        cpu_limit: definition.cpu_limit,
        memory_limit: definition.memory_limit,
        time_limit: definition.time_limit,
        output_limit: definition.output_limit,
        device_allowlist,
        device_allowlist_bytes,
        ipc_allowlist,
        ipc_allowlist_bytes,
        inherited_handle_rules,
        inherited_handle_rules_bytes,
        platform_requirements,
        platform_requirements_bytes,
        registered_by_principal: registered_by_principal.to_owned(),
        mutation_taint_digest,
        intent_digest,
        resource,
    })
}

pub fn sandbox_profile_resource(profile_id: [u8; 16], version: u64) -> String {
    format!("sandbox-profile:{}:v{version}", hex_bytes(&profile_id))
}

pub struct SandboxProfileStore {
    connection: Connection,
}

impl SandboxProfileStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, SandboxProfileError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn register(
        &mut self,
        prepared: PreparedSandboxProfile,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<SandboxProfileRecord, SandboxProfileError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let authority = verify_current_authority(
            &transaction,
            authority_decision_id,
            &prepared.registered_by_principal,
            &prepared.resource,
        )?;
        verify_registration_effect(
            &transaction,
            effect_id,
            &prepared.resource,
            prepared.intent_digest,
        )?;
        verify_once_approval(
            &transaction,
            approval_id,
            effect_id,
            &prepared.resource,
            prepared.mutation_taint_digest,
        )?;

        let duplicate = transaction
            .query_row(
                "SELECT 1 FROM sandbox_profiles WHERE profile_id = ?1 AND version = ?2 LIMIT 1",
                params![&prepared.profile_id[..], to_i64(prepared.version)?],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if duplicate {
            return Err(SandboxProfileError::DuplicateProfileVersion);
        }

        transaction.execute(
            "INSERT INTO sandbox_profiles \
             (profile_id, version, class, filesystem_read_roots, filesystem_write_roots, network_rule, environment_allowlist, spawn_rule, cpu_limit, memory_limit, time_limit, output_limit, device_allowlist, ipc_allowlist, inherited_handle_rules, platform_requirements, status) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 'active')",
            params![
                &prepared.profile_id[..],
                to_i64(prepared.version)?,
                prepared.class.as_str(),
                &prepared.filesystem_read_roots_bytes,
                &prepared.filesystem_write_roots_bytes,
                prepared.network_rule.as_str(),
                &prepared.environment_allowlist_bytes,
                prepared.spawn_rule.as_str(),
                prepared.cpu_limit.map(to_i64).transpose()?,
                prepared.memory_limit.map(to_i64).transpose()?,
                prepared.time_limit.map(to_i64).transpose()?,
                prepared.output_limit.map(to_i64).transpose()?,
                &prepared.device_allowlist_bytes,
                &prepared.ipc_allowlist_bytes,
                &prepared.inherited_handle_rules_bytes,
                &prepared.platform_requirements_bytes,
            ],
        )?;
        append_sandbox_profile_snapshot(
            &transaction,
            &prepared.profile_id,
            to_i64(prepared.version)?,
        )
        .map_err(|error| SandboxProfileError::AuthoritySecurity(error.to_string()))?;
        consume_once_approval(&transaction, approval_id, effect_id, authority.global_seq)?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| SandboxProfileError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(SandboxProfileRecord {
            profile_id: prepared.profile_id,
            version: prepared.version,
            class: prepared.class,
            filesystem_read_roots: prepared.filesystem_read_roots,
            filesystem_write_roots: prepared.filesystem_write_roots,
            network_rule: prepared.network_rule,
            environment_allowlist: prepared.environment_allowlist,
            spawn_rule: prepared.spawn_rule,
            cpu_limit: prepared.cpu_limit,
            memory_limit: prepared.memory_limit,
            time_limit: prepared.time_limit,
            output_limit: prepared.output_limit,
            device_allowlist: prepared.device_allowlist,
            ipc_allowlist: prepared.ipc_allowlist,
            inherited_handle_rules: prepared.inherited_handle_rules,
            platform_requirements: prepared.platform_requirements,
            status: "active".to_owned(),
        })
    }

    pub fn profile(
        &self,
        profile_id: [u8; 16],
        version: u64,
    ) -> Result<SandboxProfileRecord, SandboxProfileError> {
        crate::integrity::verify(&self.connection)
            .map_err(|error| SandboxProfileError::Integrity(error.to_string()))?;
        crate::authority_security_v2::verify(&self.connection)
            .map_err(|error| SandboxProfileError::AuthoritySecurity(error.to_string()))?;
        load_profile(&self.connection, profile_id, version)
    }
}

struct AuthorityEvidence {
    global_seq: u64,
}

fn verify_transaction_integrity(transaction: &Transaction<'_>) -> Result<(), SandboxProfileError> {
    crate::integrity::verify(transaction)
        .map_err(|error| SandboxProfileError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| SandboxProfileError::AuthoritySecurity(error.to_string()))
}

fn verify_current_authority(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    expected_principal: &str,
    expected_resource: &str,
) -> Result<AuthorityEvidence, SandboxProfileError> {
    let row = transaction
        .query_row(
            "SELECT principal, action, resource, hard_guard_result, decision, global_seq, authority_evidence_version \
             FROM authorization_decisions WHERE decision_id = ?1",
            params![&decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )
        .optional()?
        .ok_or(SandboxProfileError::MissingAuthorityDecision)?;
    if row.0 != expected_principal
        || row.1 != SANDBOX_PROFILE_REGISTER_ACTION
        || row.2 != expected_resource
        || row.3 != "pass"
        || row.4 != "allow"
        || row.6 < 2
    {
        return Err(SandboxProfileError::AuthorityDecisionMismatch);
    }
    let global_seq = from_i64(row.5, "sandbox profile decision sequence is negative")?;
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (\
           SELECT global_seq FROM session_events \
           UNION ALL SELECT global_seq FROM effect_transitions \
           UNION ALL SELECT global_seq FROM authorization_decisions\
         )",
        [],
        |row| row.get(0),
    )?;
    if global_seq != from_i64(latest, "latest authority sequence is negative")? {
        return Err(SandboxProfileError::StaleAuthorityDecision);
    }
    Ok(AuthorityEvidence { global_seq })
}

fn verify_registration_effect(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    expected_resource: &str,
    expected_payload_hash: [u8; 32],
) -> Result<(), SandboxProfileError> {
    let row = transaction
        .query_row(
            "SELECT i.action, i.resource, i.risk_class, i.execution_semantics, i.payload_hash, t.to_state \
             FROM effect_intents i JOIN effect_transitions t ON t.effect_id = i.effect_id \
             WHERE i.effect_id = ?1 AND t.global_seq = \
             (SELECT MAX(t2.global_seq) FROM effect_transitions t2 WHERE t2.effect_id = i.effect_id)",
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
        .ok_or(SandboxProfileError::EffectNotFound)?;
    if row.0 != SANDBOX_PROFILE_REGISTER_ACTION
        || row.1 != expected_resource
        || row.2 != SANDBOX_PROFILE_MUTATION_RISK_CLASS
        || row.3 != "at_most_once"
        || row.4.as_slice() != expected_payload_hash
        || row.5 != "authorized"
    {
        return Err(SandboxProfileError::EffectMismatch);
    }
    Ok(())
}

fn verify_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    expected_resource: &str,
    expected_taint_digest: [u8; 32],
) -> Result<(), SandboxProfileError> {
    let row = transaction
        .query_row(
            "SELECT class, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, expires_at, max_uses, revoked_at \
             FROM approvals WHERE approval_id = ?1",
            params![&approval_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Vec<u8>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            },
        )
        .optional()?
        .ok_or(SandboxProfileError::ApprovalNotFound)?;
    if row.0 != "ONCE"
        || row.1.as_slice() != SANDBOX_PROFILE_REGISTER_ACTION.as_bytes()
        || row.2.as_slice() != expected_resource.as_bytes()
        || row.3.as_deref() != Some(effect_id.0.to_be_bytes().as_slice())
        || row.4.is_some()
        || row.5 != SANDBOX_PROFILE_MUTATION_RISK_CLASS
        || row.6.as_slice() != expected_taint_digest
        || row.7.is_some()
        || row.8 != Some(1)
        || row.9.is_some()
    {
        return Err(SandboxProfileError::ApprovalMismatch);
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
        return Err(SandboxProfileError::ApprovalAlreadyUsed);
    }
    Ok(())
}

fn consume_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    global_seq: u64,
) -> Result<(), SandboxProfileError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(APPROVAL_CONSUMPTION_DOMAIN);
    hasher.update(&approval_id);
    hasher.update(&effect_id.0.to_be_bytes());
    let digest = hasher.finalize();
    let mut consumption_id = [0_u8; 16];
    consumption_id.copy_from_slice(&digest.as_bytes()[..16]);
    transaction.execute(
        "INSERT INTO approval_consumptions \
         (consumption_id, approval_id, effect_or_operation_id, reserved_global_seq, consumed_global_seq, state) \
         VALUES (?1, ?2, ?3, ?4, ?5, 'consumed')",
        params![
            &consumption_id[..],
            &approval_id[..],
            &effect_id.0.to_be_bytes()[..],
            to_i64(global_seq)?,
            to_i64(global_seq)?,
        ],
    )?;
    append_approval_consumption_snapshot(transaction, &consumption_id)
        .map_err(|error| SandboxProfileError::AuthoritySecurity(error.to_string()))
}

pub(crate) fn load_profile(
    connection: &Connection,
    profile_id: [u8; 16],
    version: u64,
) -> Result<SandboxProfileRecord, SandboxProfileError> {
    let row = connection
        .query_row(
            "SELECT class, filesystem_read_roots, filesystem_write_roots, network_rule, environment_allowlist, spawn_rule, \
             cpu_limit, memory_limit, time_limit, output_limit, device_allowlist, ipc_allowlist, inherited_handle_rules, platform_requirements, status \
             FROM sandbox_profiles WHERE profile_id = ?1 AND version = ?2",
            params![&profile_id[..], to_i64(version)?],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, Vec<u8>>(10)?,
                    row.get::<_, Vec<u8>>(11)?,
                    row.get::<_, Vec<u8>>(12)?,
                    row.get::<_, Vec<u8>>(13)?,
                    row.get::<_, String>(14)?,
                ))
            },
        )
        .optional()?
        .ok_or(SandboxProfileError::ProfileNotFound)?;

    let class = SandboxProfileClass::from_str(&row.0)
        .ok_or(SandboxProfileError::InvalidStoredRecord("invalid class"))?;
    let network_rule = SandboxNetworkRule::from_str(&row.3).ok_or(
        SandboxProfileError::InvalidStoredRecord("invalid network rule"),
    )?;
    let spawn_rule = SandboxSpawnRule::from_str(&row.5).ok_or(
        SandboxProfileError::InvalidStoredRecord("invalid spawn rule"),
    )?;
    if row.14 != "active" {
        return Err(SandboxProfileError::InvalidStoredRecord(
            "unsupported profile status",
        ));
    }

    let filesystem_read_roots = decode_list(&row.1, validate_root)?;
    let filesystem_write_roots = decode_list(&row.2, validate_root)?;
    let environment_allowlist = decode_list(&row.4, validate_environment_name)?;
    let device_allowlist = decode_list(&row.10, validate_token)?;
    let ipc_allowlist = decode_list(&row.11, validate_token)?;
    let inherited_handle_rules = decode_list(&row.12, validate_token)?;
    let platform_requirements = decode_list(&row.13, validate_token)?;

    Ok(SandboxProfileRecord {
        profile_id,
        version,
        class,
        filesystem_read_roots,
        filesystem_write_roots,
        network_rule,
        environment_allowlist,
        spawn_rule,
        cpu_limit: optional_u64(row.6)?,
        memory_limit: optional_u64(row.7)?,
        time_limit: optional_u64(row.8)?,
        output_limit: optional_u64(row.9)?,
        device_allowlist,
        ipc_allowlist,
        inherited_handle_rules,
        platform_requirements,
        status: row.14,
    })
}

fn canonicalize_list(
    values: &[&str],
    validator: fn(&str) -> Result<(), SandboxProfileError>,
) -> Result<Vec<String>, SandboxProfileError> {
    if values.len() > MAX_LIST_ITEMS {
        return Err(SandboxProfileError::TooManyItems);
    }
    let mut canonical = Vec::with_capacity(values.len());
    for value in values {
        validator(value)?;
        canonical.push((*value).to_owned());
    }
    canonical.sort_unstable();
    if canonical.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(SandboxProfileError::DuplicateItem);
    }
    Ok(canonical)
}

fn encode_list(values: &[String]) -> Result<Vec<u8>, SandboxProfileError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(LIST_DOMAIN)?;
    encoder
        .push_u64(u64::try_from(values.len()).map_err(|_| SandboxProfileError::IntegerOverflow)?);
    for value in values {
        encoder.push_bytes(value.as_bytes())?;
    }
    Ok(encoder.finish())
}

fn decode_list(
    bytes: &[u8],
    validator: fn(&str) -> Result<(), SandboxProfileError>,
) -> Result<Vec<String>, SandboxProfileError> {
    // CanonicalEncoder is intentionally write-only. This bounded parser mirrors
    // its length-prefixed byte representation for this private list domain.
    let mut offset = 0_usize;
    let domain = take_bytes(bytes, &mut offset)?;
    if domain != LIST_DOMAIN {
        return Err(SandboxProfileError::InvalidStoredRecord(
            "invalid list encoding domain",
        ));
    }
    let count = usize::try_from(take_u64(bytes, &mut offset)?)
        .map_err(|_| SandboxProfileError::InvalidStoredRecord("list count overflow"))?;
    if count > MAX_LIST_ITEMS {
        return Err(SandboxProfileError::InvalidStoredRecord(
            "stored list exceeds item bound",
        ));
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let raw = take_bytes(bytes, &mut offset)?;
        let value = std::str::from_utf8(raw)
            .map_err(|_| SandboxProfileError::InvalidStoredRecord("list value is not utf-8"))?
            .to_owned();
        validator(&value)
            .map_err(|_| SandboxProfileError::InvalidStoredRecord("list value is invalid"))?;
        if values.last().is_some_and(|previous| previous >= &value) {
            return Err(SandboxProfileError::InvalidStoredRecord(
                "stored list is not strictly canonical",
            ));
        }
        values.push(value);
    }
    if offset != bytes.len() {
        return Err(SandboxProfileError::InvalidStoredRecord(
            "trailing bytes in list encoding",
        ));
    }
    Ok(values)
}

fn take_bytes<'a>(bytes: &'a [u8], offset: &mut usize) -> Result<&'a [u8], SandboxProfileError> {
    let end = offset
        .checked_add(4)
        .ok_or(SandboxProfileError::InvalidStoredRecord("length overflow"))?;
    let raw_len: [u8; 4] = bytes
        .get(*offset..end)
        .ok_or(SandboxProfileError::InvalidStoredRecord(
            "truncated byte length",
        ))?
        .try_into()
        .map_err(|_| SandboxProfileError::InvalidStoredRecord("invalid byte length"))?;
    *offset = end;
    let len = u32::from_be_bytes(raw_len) as usize;
    let end = offset
        .checked_add(len)
        .ok_or(SandboxProfileError::InvalidStoredRecord("length overflow"))?;
    let value = bytes
        .get(*offset..end)
        .ok_or(SandboxProfileError::InvalidStoredRecord(
            "truncated list encoding",
        ))?;
    *offset = end;
    Ok(value)
}

fn take_u64(bytes: &[u8], offset: &mut usize) -> Result<u64, SandboxProfileError> {
    let end = offset
        .checked_add(8)
        .ok_or(SandboxProfileError::InvalidStoredRecord("length overflow"))?;
    let value: [u8; 8] = bytes
        .get(*offset..end)
        .ok_or(SandboxProfileError::InvalidStoredRecord(
            "truncated integer encoding",
        ))?
        .try_into()
        .map_err(|_| SandboxProfileError::InvalidStoredRecord("invalid integer encoding"))?;
    *offset = end;
    Ok(u64::from_be_bytes(value))
}

fn validate_root(value: &str) -> Result<(), SandboxProfileError> {
    if value.is_empty()
        || value.len() > MAX_ROOT_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
        || value.contains('\\')
        || value.contains("//")
        || value
            .split('/')
            .any(|segment| segment == "." || segment == "..")
    {
        return Err(SandboxProfileError::InvalidRoot);
    }
    let unix_absolute = value.starts_with('/');
    let windows_absolute = {
        let bytes = value.as_bytes();
        bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/'
    };
    if !unix_absolute && !windows_absolute {
        return Err(SandboxProfileError::InvalidRoot);
    }
    Ok(())
}

fn validate_environment_name(value: &str) -> Result<(), SandboxProfileError> {
    if value.is_empty() || value.len() > MAX_ENV_NAME_BYTES {
        return Err(SandboxProfileError::InvalidEnvironmentName);
    }
    let mut bytes = value.bytes();
    let first = bytes
        .next()
        .ok_or(SandboxProfileError::InvalidEnvironmentName)?;
    if !(first.is_ascii_alphabetic() || first == b'_')
        || bytes.any(|byte| !(byte.is_ascii_alphanumeric() || byte == b'_'))
    {
        return Err(SandboxProfileError::InvalidEnvironmentName);
    }
    Ok(())
}

fn validate_token(value: &str) -> Result<(), SandboxProfileError> {
    if value.is_empty()
        || value.len() > MAX_TOKEN_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'_' | b'-' | b'.' | b':' | b'/' | b'@' | b'*')
        })
    {
        return Err(SandboxProfileError::InvalidToken);
    }
    Ok(())
}

fn validate_principal(value: &str) -> Result<(), SandboxProfileError> {
    if value.is_empty()
        || value.len() > MAX_PRINCIPAL_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(SandboxProfileError::InvalidPrincipal);
    }
    Ok(())
}

fn validate_limit(value: Option<u64>) -> Result<(), SandboxProfileError> {
    if matches!(value, Some(0)) || value.is_some_and(|value| value > i64::MAX as u64) {
        return Err(SandboxProfileError::InvalidLimit);
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

fn optional_u64(value: Option<i64>) -> Result<Option<u64>, SandboxProfileError> {
    value
        .map(|value| {
            let value = from_i64(value, "stored resource limit is negative")?;
            validate_limit(Some(value))?;
            Ok(value)
        })
        .transpose()
}

fn from_i64(value: i64, reason: &'static str) -> Result<u64, SandboxProfileError> {
    u64::try_from(value).map_err(|_| SandboxProfileError::InvalidStoredRecord(reason))
}

fn to_i64(value: u64) -> Result<i64, SandboxProfileError> {
    i64::try_from(value).map_err(|_| SandboxProfileError::IntegerOverflow)
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[(byte >> 4) as usize]));
        output.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approval_binding::{
        APPROVAL_ISSUE_ACTION, APPROVAL_MUTATION_RISK_CLASS, ApprovalStore, prepare_approval,
    };
    use crate::approvals::ApprovalScope;
    use crate::authorization::{
        AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionEvidence,
        AuthorizationDecisionKind,
    };
    use crate::dispatch::encode_effect_dependencies;
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_core::paths::RuntimeLayout;
    use golam_core::{EffectTransitionId, EventId, SessionId};
    use rusqlite::Connection;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);
    static RECORD_N: AtomicU64 = AtomicU64::new(0);

    fn next_id() -> u128 {
        8_000_000 + u128::from(RECORD_N.fetch_add(1, Ordering::Relaxed))
    }

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-sandbox-profile-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn definition<'a>(
        read_roots: &'a [&'a str],
        write_roots: &'a [&'a str],
        env: &'a [&'a str],
        devices: &'a [&'a str],
        ipc: &'a [&'a str],
        handles: &'a [&'a str],
        platforms: &'a [&'a str],
    ) -> SandboxProfileDefinition<'a> {
        SandboxProfileDefinition {
            profile_id: [7; 16],
            version: 1,
            class: SandboxProfileClass::NativeUntrustedSubprocess,
            filesystem_read_roots: read_roots,
            filesystem_write_roots: write_roots,
            network_rule: SandboxNetworkRule::DenyAll,
            environment_allowlist: env,
            spawn_rule: SandboxSpawnRule::Deny,
            cpu_limit: Some(500),
            memory_limit: Some(256 * 1024 * 1024),
            time_limit: Some(30_000),
            output_limit: Some(1024 * 1024),
            device_allowlist: devices,
            ipc_allowlist: ipc,
            inherited_handle_rules: handles,
            platform_requirements: platforms,
        }
    }

    fn prepared() -> PreparedSandboxProfile {
        prepare_sandbox_profile(
            definition(
                &["/workspace/input", "/workspace"],
                &["/workspace/output"],
                &["LANG", "TZ"],
                &["device:clock"],
                &["ipc:golam-broker"],
                &["secret-handle:explicit"],
                &["native:process-containment"],
            ),
            "owner:owner",
            [9; 32],
        )
        .unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    fn create_authorized_effect(
        authority: &AuthorityLayout,
        effect_id: EffectId,
        action: &str,
        resource: &str,
        risk_class: &str,
        payload_hash: [u8; 32],
        requested_by: &str,
        reason: &str,
    ) {
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let mut store = EffectStore::open(authority).unwrap();
        store
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(1),
                requested_by,
                action,
                resource,
                risk_class,
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash,
                proposed_event_id: EventId(next_id()),
                transition_id: EffectTransitionId(next_id()),
            })
            .unwrap();
        store
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(next_id()),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some(reason),
                evidence_ref: None,
                event_id: EventId(next_id()),
            })
            .unwrap();
    }

    fn append_allow(
        authority: &AuthorityLayout,
        action: &str,
        resource: &str,
        reason: &str,
    ) -> [u8; 16] {
        AuthorizationAuditLog::open(authority)
            .unwrap()
            .append(AppendAuthorizationDecision {
                principal: "owner:owner",
                action,
                resource,
                context: "scope=local-owner",
                evidence: AuthorizationDecisionEvidence::hard_guard_only("pass"),
                decision: AuthorizationDecisionKind::Allow,
                reason_code: reason,
            })
            .unwrap()
            .decision_id
    }

    fn issue_registration_approval(
        authority: &AuthorityLayout,
        prepared: &PreparedSandboxProfile,
        effect_id: EffectId,
    ) -> [u8; 16] {
        let approval = prepare_approval(
            "owner:owner",
            ApprovalScope::once(
                effect_id,
                SANDBOX_PROFILE_REGISTER_ACTION,
                prepared.resource(),
            )
            .unwrap(),
            SANDBOX_PROFILE_MUTATION_RISK_CLASS,
            prepared.mutation_taint_digest(),
            "2026-08-29T00:00:00Z",
            None,
            1,
        )
        .unwrap();
        let approval_effect_id = EffectId(next_id());
        create_authorized_effect(
            authority,
            approval_effect_id,
            APPROVAL_ISSUE_ACTION,
            approval.resource(),
            APPROVAL_MUTATION_RISK_CLASS,
            approval.intent_digest(),
            "owner:owner",
            "test_sandbox_profile_approval_issue",
        );
        let decision = append_allow(
            authority,
            APPROVAL_ISSUE_ACTION,
            approval.resource(),
            "test_sandbox_profile_approval_authority",
        );
        ApprovalStore::open(authority)
            .unwrap()
            .issue(approval, decision, approval_effect_id)
            .unwrap()
            .approval_id
    }

    fn register(authority: &AuthorityLayout) -> SandboxProfileRecord {
        let prepared = prepared();
        let effect_id = EffectId(next_id());
        let approval_id = issue_registration_approval(authority, &prepared, effect_id);
        create_authorized_effect(
            authority,
            effect_id,
            SANDBOX_PROFILE_REGISTER_ACTION,
            prepared.resource(),
            SANDBOX_PROFILE_MUTATION_RISK_CLASS,
            prepared.intent_digest(),
            "owner:owner",
            "test_sandbox_profile_registration_effect",
        );
        let decision = append_allow(
            authority,
            SANDBOX_PROFILE_REGISTER_ACTION,
            prepared.resource(),
            "test_sandbox_profile_registration_authority",
        );
        SandboxProfileStore::open(authority)
            .unwrap()
            .register(prepared, decision, approval_id, effect_id)
            .unwrap()
    }

    #[test]
    fn preparation_is_deterministic_and_rejects_noncanonical_inputs() {
        let first = prepared();
        let second = prepare_sandbox_profile(
            definition(
                &["/workspace", "/workspace/input"],
                &["/workspace/output"],
                &["TZ", "LANG"],
                &["device:clock"],
                &["ipc:golam-broker"],
                &["secret-handle:explicit"],
                &["native:process-containment"],
            ),
            "owner:owner",
            [9; 32],
        )
        .unwrap();
        assert_eq!(first.intent_digest(), second.intent_digest());
        assert_eq!(
            first.filesystem_read_roots,
            vec!["/workspace".to_owned(), "/workspace/input".to_owned()]
        );
        assert_eq!(
            first.environment_allowlist,
            vec!["LANG".to_owned(), "TZ".to_owned()]
        );

        let duplicate_env = ["LANG", "LANG"];
        assert!(matches!(
            prepare_sandbox_profile(
                definition(&["/workspace"], &[], &duplicate_env, &[], &[], &[], &[]),
                "owner:owner",
                [1; 32],
            ),
            Err(SandboxProfileError::DuplicateItem)
        ));

        let traversal = ["../escape"];
        assert!(matches!(
            prepare_sandbox_profile(
                definition(&traversal, &[], &[], &[], &[], &[], &[]),
                "owner:owner",
                [1; 32],
            ),
            Err(SandboxProfileError::InvalidRoot)
        ));

        let mut zero_limit = definition(&["/workspace"], &[], &[], &[], &[], &[], &[]);
        zero_limit.cpu_limit = Some(0);
        assert!(matches!(
            prepare_sandbox_profile(zero_limit, "owner:owner", [1; 32]),
            Err(SandboxProfileError::InvalidLimit)
        ));
    }

    #[test]
    fn protected_registration_requires_exact_authority_effect_approval_and_integrity() {
        let (runtime, authority) = authority();
        let record = register(&authority);
        assert_eq!(record.profile_id, [7; 16]);
        assert_eq!(record.version, 1);
        assert_eq!(record.network_rule, SandboxNetworkRule::DenyAll);
        assert_eq!(record.spawn_rule, SandboxSpawnRule::Deny);
        assert_eq!(record.status, "active");

        let reopened = SandboxProfileStore::open(&authority).unwrap();
        assert_eq!(reopened.profile([7; 16], 1).unwrap(), record);
        drop(reopened);

        let authority_store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        authority_store.verify_integrity().unwrap();
        drop(authority_store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn stale_or_mismatched_authority_cannot_register_profile() {
        let (runtime, authority) = authority();
        let prepared = prepared();
        let effect_id = EffectId(next_id());
        let approval_id = issue_registration_approval(&authority, &prepared, effect_id);
        create_authorized_effect(
            &authority,
            effect_id,
            SANDBOX_PROFILE_REGISTER_ACTION,
            prepared.resource(),
            SANDBOX_PROFILE_MUTATION_RISK_CLASS,
            prepared.intent_digest(),
            "owner:owner",
            "test_sandbox_profile_registration_effect",
        );
        let wrong_decision = append_allow(
            &authority,
            "sandbox.profile.read",
            prepared.resource(),
            "test_wrong_sandbox_profile_authority",
        );
        assert!(matches!(
            SandboxProfileStore::open(&authority).unwrap().register(
                prepared,
                wrong_decision,
                approval_id,
                effect_id
            ),
            Err(SandboxProfileError::AuthorityDecisionMismatch)
        ));
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn duplicate_version_and_post_commit_tamper_fail_closed() {
        let (runtime, authority) = authority();
        let record = register(&authority);

        let prepared = prepared();
        let effect_id = EffectId(next_id());
        let approval_id = issue_registration_approval(&authority, &prepared, effect_id);
        create_authorized_effect(
            &authority,
            effect_id,
            SANDBOX_PROFILE_REGISTER_ACTION,
            prepared.resource(),
            SANDBOX_PROFILE_MUTATION_RISK_CLASS,
            prepared.intent_digest(),
            "owner:owner",
            "test_duplicate_sandbox_profile_effect",
        );
        let decision = append_allow(
            &authority,
            SANDBOX_PROFILE_REGISTER_ACTION,
            prepared.resource(),
            "test_duplicate_sandbox_profile_authority",
        );
        assert!(matches!(
            SandboxProfileStore::open(&authority).unwrap().register(
                prepared,
                decision,
                approval_id,
                effect_id
            ),
            Err(SandboxProfileError::DuplicateProfileVersion)
        ));

        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute(
                "UPDATE sandbox_profiles SET network_rule = 'permit_required' WHERE profile_id = ?1 AND version = 1",
                params![&record.profile_id[..]],
            )
            .unwrap();
        drop(connection);
        assert!(AuthorityStore::open(authority.authority_db_path()).is_err());
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
