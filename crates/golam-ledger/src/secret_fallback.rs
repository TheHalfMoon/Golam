#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::approval_binding::{APPROVAL_ISSUE_ACTION, prepare_approval};
use crate::approvals::ApprovalScope;
use crate::authority_security_write::{
    append_approval_consumption_snapshot, append_secret_use_record_snapshot,
};
use crate::secret_vault::{KeyProtector, OsKeyProtector, SecretVault, VaultBinding, VaultError};
use crate::storage::{AuthorityStore, StorageError};

pub(crate) const FALLBACK_ACTION: &str = "secret.fallback.use";
pub(crate) const FALLBACK_RISK_CLASS: &str = "secret_fallback";
const FALLBACK_EXECUTION_SEMANTICS: &str = "at_most_once";
const PLAN_DOMAIN: &[u8] = b"golam:secret-fallback-plan:v1";
const RESOURCE_DOMAIN: &[u8] = b"golam:secret-fallback-resource:v1";
const INTENT_DOMAIN: &[u8] = b"golam:secret-fallback-intent:v1";
const USE_ID_DOMAIN: &[u8] = b"golam:secret-fallback-use-id:v1";
const APPROVAL_BINDING_DOMAIN: &[u8] = b"golam:approval-binding:v1";
const APPROVAL_CONSUMPTION_DOMAIN: &[u8] = b"golam:secret-fallback-approval-consumption:v1";
const REDACTION_MARKER: &[u8] = b"<redacted-secret>";
const SECURITY_METADATA_VERSION: u64 = 1;
const MAX_TEXT_BYTES: usize = 2_048;
const MAX_EXECUTABLE_BYTES: usize = 4_096;
const MAX_ARG_BYTES: usize = 8_192;
const MAX_ENV_VALUE_BYTES: usize = 16_384;
const MAX_LAUNCH_ITEMS: usize = 64;
const MAX_PARENT_CHAIN_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FallbackInjectionChannel {
    Stdin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FallbackInjectorCapabilities {
    pub clear_environment: bool,
    pub stdin_secret_channel: bool,
    pub closes_stdin_after_write: bool,
    pub forbids_secret_argv: bool,
    pub forbids_secret_environment: bool,
    pub no_ambient_secret_inheritance: bool,
    pub captures_stdout_stderr: bool,
}

impl FallbackInjectorCapabilities {
    fn qualified(self) -> bool {
        self.clear_environment
            && self.stdin_secret_channel
            && self.closes_stdin_after_write
            && self.forbids_secret_argv
            && self.forbids_secret_environment
            && self.no_ambient_secret_inheritance
            && self.captures_stdout_stderr
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FallbackRawOutput {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub(crate) trait FallbackInjector {
    fn executor_id(&self) -> &'static str;
    fn capabilities(&self) -> FallbackInjectorCapabilities;
    fn inject_stdin(
        &mut self,
        plan: &FallbackLaunchPlan,
        secret: &[u8],
    ) -> Result<FallbackRawOutput, Vec<u8>>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FallbackLaunchPlan {
    executable: String,
    args: Vec<String>,
    explicit_environment: Vec<(String, String)>,
    destination_or_process: String,
    executor_id: String,
    channel: FallbackInjectionChannel,
    plan_hash: [u8; 32],
}

impl FallbackLaunchPlan {
    pub(crate) fn executable(&self) -> &str {
        &self.executable
    }

    pub(crate) fn args(&self) -> &[String] {
        &self.args
    }

    pub(crate) fn explicit_environment(&self) -> &[(String, String)] {
        &self.explicit_environment
    }

    pub(crate) fn destination_or_process(&self) -> &str {
        &self.destination_or_process
    }

    pub(crate) fn executor_id(&self) -> &str {
        &self.executor_id
    }

    pub(crate) const fn channel(&self) -> FallbackInjectionChannel {
        self.channel
    }

    pub(crate) const fn plan_hash(&self) -> [u8; 32] {
        self.plan_hash
    }

    pub(crate) const fn clears_environment(&self) -> bool {
        true
    }

    pub(crate) const fn secret_in_argv(&self) -> bool {
        false
    }

    pub(crate) const fn secret_in_environment(&self) -> bool {
        false
    }

    pub(crate) const fn secret_inherited_by_descendants(&self) -> bool {
        false
    }
}

pub(crate) struct FallbackSecretUseRequest<'a> {
    pub handle_id: [u8; 16],
    pub principal: &'a str,
    pub purpose: &'a str,
    pub destination_or_process: &'a str,
    pub admission_id: [u8; 16],
    pub approval_id: [u8; 16],
    pub effect_id: EffectId,
    pub observed_at: &'a str,
    pub taint_digest: [u8; 32],
    pub executable: &'a str,
    pub args: &'a [String],
    pub explicit_environment: &'a [(String, String)],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FallbackExecutionOutput {
    use_id: [u8; 16],
    exit_code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl FallbackExecutionOutput {
    pub(crate) const fn use_id(&self) -> [u8; 16] {
        self.use_id
    }

    pub(crate) const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub(crate) fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub(crate) fn stderr(&self) -> &[u8] {
        &self.stderr
    }
}

#[derive(Debug)]
pub(crate) enum SecretFallbackError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Vault(VaultError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidRequest(&'static str),
    InjectorCapabilitiesUnavailable,
    AdmissionNotFound,
    AdmissionMismatch,
    AdmissionPlanMismatch,
    AdmissionExecutorMismatch,
    AdmissionEgressNotAllowed,
    AdmissionDecisionNotFound,
    AdmissionDecisionMismatch,
    PolicyMismatch,
    LeaseNotFound,
    LeaseMismatch,
    LeaseInactive,
    LeaseRevoked,
    LeaseNotYetValid,
    LeaseExpired,
    LeaseScopeMismatch,
    LeaseParentCycle,
    LeaseParentTooDeep,
    HandleNotFound,
    HandlePurposeMismatch,
    HandleExpired,
    SecretNotFound,
    SecretRevoked,
    StaleHandleVersion,
    SecretVersionNotFound,
    SecretVersionRetired,
    EffectNotFound,
    EffectMismatch,
    ApprovalNotFound,
    ApprovalMismatch,
    ApprovalExpired,
    ApprovalRevoked,
    ApprovalAlreadyUsed,
    DuplicateUse,
    SecretPresentInLaunchField,
    EmptySecret,
    InjectorFailed(String),
    InvalidStoredRecord(&'static str),
    IntegerOverflow,
}

impl fmt::Display for SecretFallbackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "secret fallback authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "secret fallback sqlite error: {error}"),
            Self::Core(error) => write!(f, "secret fallback encoding error: {error}"),
            Self::Vault(error) => write!(f, "secret fallback vault error: {error}"),
            Self::Integrity(error) => write!(f, "secret fallback integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "secret fallback authority-security error: {error}")
            }
            Self::InvalidRequest(reason) => write!(f, "secret fallback request is invalid: {reason}"),
            Self::InjectorCapabilitiesUnavailable => {
                f.write_str("secret fallback injector does not provide all required containment capabilities")
            }
            Self::AdmissionNotFound => f.write_str("secret fallback sandbox/process admission does not exist"),
            Self::AdmissionMismatch => f.write_str("secret fallback admission does not match the exact request"),
            Self::AdmissionPlanMismatch => f.write_str("secret fallback admission launch-plan hash mismatch"),
            Self::AdmissionExecutorMismatch => f.write_str("secret fallback admission executor mismatch"),
            Self::AdmissionEgressNotAllowed => f.write_str("secret fallback admission carries egress authority"),
            Self::AdmissionDecisionNotFound => f.write_str("secret fallback admission decision does not exist"),
            Self::AdmissionDecisionMismatch => f.write_str("secret fallback admission decision is not usable authority"),
            Self::PolicyMismatch => f.write_str("secret fallback active policy binding mismatch"),
            Self::LeaseNotFound => f.write_str("secret fallback capability lease does not exist"),
            Self::LeaseMismatch => f.write_str("secret fallback capability lease binding mismatch"),
            Self::LeaseInactive => f.write_str("secret fallback capability lease is inactive"),
            Self::LeaseRevoked => f.write_str("secret fallback capability lease is revoked"),
            Self::LeaseNotYetValid => f.write_str("secret fallback capability lease is not yet valid"),
            Self::LeaseExpired => f.write_str("secret fallback capability lease is expired"),
            Self::LeaseScopeMismatch => f.write_str("secret fallback capability lease scope mismatch"),
            Self::LeaseParentCycle => f.write_str("secret fallback capability lease parent cycle"),
            Self::LeaseParentTooDeep => f.write_str("secret fallback capability lease parent chain exceeds bound"),
            Self::HandleNotFound => f.write_str("secret fallback handle does not exist"),
            Self::HandlePurposeMismatch => f.write_str("secret fallback handle purpose does not match"),
            Self::HandleExpired => f.write_str("secret fallback handle is expired"),
            Self::SecretNotFound => f.write_str("secret fallback secret does not exist"),
            Self::SecretRevoked => f.write_str("secret fallback secret is revoked"),
            Self::StaleHandleVersion => f.write_str("secret fallback handle is pinned to a stale version"),
            Self::SecretVersionNotFound => f.write_str("secret fallback selected secret version does not exist"),
            Self::SecretVersionRetired => f.write_str("secret fallback selected secret version is retired"),
            Self::EffectNotFound => f.write_str("secret fallback effect does not exist"),
            Self::EffectMismatch => f.write_str("secret fallback effect is not exact authorized at-most-once work"),
            Self::ApprovalNotFound => f.write_str("secret fallback approval does not exist"),
            Self::ApprovalMismatch => f.write_str("secret fallback approval does not match exact fallback work"),
            Self::ApprovalExpired => f.write_str("secret fallback approval is expired"),
            Self::ApprovalRevoked => f.write_str("secret fallback approval is revoked"),
            Self::ApprovalAlreadyUsed => f.write_str("secret fallback one-shot approval is already used"),
            Self::DuplicateUse => f.write_str("secret fallback use was already recorded"),
            Self::SecretPresentInLaunchField => {
                f.write_str("secret fallback plaintext appears in a non-secret launch field")
            }
            Self::EmptySecret => f.write_str("secret fallback refuses an empty secret value"),
            Self::InjectorFailed(error) => write!(f, "secret fallback injector failed: {error}"),
            Self::InvalidStoredRecord(reason) => write!(f, "secret fallback stored record is invalid: {reason}"),
            Self::IntegerOverflow => f.write_str("secret fallback integer conversion overflow"),
        }
    }
}

impl Error for SecretFallbackError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::Vault(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for SecretFallbackError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for SecretFallbackError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for SecretFallbackError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl From<VaultError> for SecretFallbackError {
    fn from(value: VaultError) -> Self {
        Self::Vault(value)
    }
}

pub(crate) struct SecretFallbackStore<P: KeyProtector> {
    connection: Connection,
    protector: P,
}

impl SecretFallbackStore<OsKeyProtector> {
    pub(crate) fn open(layout: &AuthorityLayout) -> Result<Self, SecretFallbackError> {
        Self::open_with_protector(layout, OsKeyProtector::new())
    }
}

impl<P: KeyProtector> SecretFallbackStore<P> {
    pub(crate) fn open_with_protector(
        layout: &AuthorityLayout,
        protector: P,
    ) -> Result<Self, SecretFallbackError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self {
            connection,
            protector,
        })
    }

    pub(crate) fn execute_with_injector(
        &mut self,
        request: FallbackSecretUseRequest<'_>,
        injector: &mut impl FallbackInjector,
    ) -> Result<FallbackExecutionOutput, SecretFallbackError> {
        validate_request(&request)?;
        if !injector.capabilities().qualified() {
            return Err(SecretFallbackError::InjectorCapabilitiesUnavailable);
        }
        let plan = build_launch_plan(&request, injector.executor_id())?;
        let resource = fallback_resource(
            request.handle_id,
            request.admission_id,
            request.purpose,
            request.destination_or_process,
            plan.plan_hash,
        )?;
        let intent_digest = fallback_intent_digest(
            request.handle_id,
            request.admission_id,
            request.effect_id,
            request.principal,
            &resource,
            plan.plan_hash,
            request.taint_digest,
        )?;

        let protector = &self.protector;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let admission = verify_admission(
            &transaction,
            request.admission_id,
            request.principal,
            request.destination_or_process,
            plan.plan_hash,
            injector.executor_id(),
            &resource,
            request.observed_at,
        )?;
        verify_effect(
            &transaction,
            request.effect_id,
            &resource,
            intent_digest,
        )?;
        verify_approval(
            &transaction,
            request.approval_id,
            request.effect_id,
            &resource,
            request.taint_digest,
            request.observed_at,
        )?;
        let material = load_secret_material(
            &transaction,
            request.handle_id,
            request.purpose,
            request.observed_at,
        )?;
        let use_id = fallback_use_id(
            request.handle_id,
            material.version,
            request.admission_id,
            request.approval_id,
            request.effect_id,
        )?;
        let exists = transaction
            .query_row(
                "SELECT 1 FROM secret_use_records WHERE use_id = ?1 LIMIT 1",
                params![&use_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if exists {
            return Err(SecretFallbackError::DuplicateUse);
        }

        let vault = SecretVault::from_persisted_algorithm_metadata(
            protector,
            material
                .all_algorithm_metadata
                .iter()
                .map(Vec::as_slice),
        )?;
        let binding = VaultBinding::new(
            material.secret_id,
            material.version,
            material.classification.clone(),
            SECURITY_METADATA_VERSION,
        )?;
        let decision_id = admission.decision_id;
        let created_global_seq = admission.decision_global_seq;

        vault.with_persisted_plaintext(
            &binding,
            &material.ciphertext,
            &material.algorithm_metadata,
            material.associated_data_hash,
            |secret| {
                if secret.is_empty() {
                    return Err(SecretFallbackError::EmptySecret);
                }
                if secret_occurs_in_launch_fields(secret, &plan) {
                    return Err(SecretFallbackError::SecretPresentInLaunchField);
                }
                transaction.execute(
                    "INSERT INTO secret_use_records (use_id, handle_id, principal, purpose, destination_or_process, mode, approval_id, decision_id, created_global_seq) VALUES (?1, ?2, ?3, ?4, ?5, 'fallback_stdin', ?6, ?7, ?8)",
                    params![
                        &use_id[..],
                        &request.handle_id[..],
                        request.principal,
                        request.purpose,
                        request.destination_or_process,
                        &request.approval_id[..],
                        &decision_id[..],
                        to_i64(created_global_seq)?,
                    ],
                )?;
                append_secret_use_record_snapshot(&transaction, &use_id)
                    .map_err(|error| SecretFallbackError::AuthoritySecurity(error.to_string()))?;
                consume_approval(
                    &transaction,
                    request.approval_id,
                    request.effect_id,
                    created_global_seq,
                )?;
                crate::authority_security_v2::verify(&transaction)
                    .map_err(|error| SecretFallbackError::AuthoritySecurity(error.to_string()))?;
                transaction.commit()?;

                let raw = match injector.inject_stdin(&plan, secret) {
                    Ok(raw) => raw,
                    Err(error) => {
                        let sanitized = redact_bytes(&error, secret);
                        return Err(SecretFallbackError::InjectorFailed(
                            String::from_utf8_lossy(&sanitized).into_owned(),
                        ));
                    }
                };
                Ok(FallbackExecutionOutput {
                    use_id,
                    exit_code: raw.exit_code,
                    stdout: redact_bytes(&raw.stdout, secret),
                    stderr: redact_bytes(&raw.stderr, secret),
                })
            },
        )?
    }
}

#[derive(Clone, Debug)]
struct AdmissionEvidence {
    decision_id: [u8; 16],
    decision_global_seq: u64,
}

#[derive(Clone, Debug)]
struct SecretMaterial {
    secret_id: [u8; 16],
    version: u64,
    classification: String,
    ciphertext: Vec<u8>,
    algorithm_metadata: Vec<u8>,
    associated_data_hash: [u8; 32],
    all_algorithm_metadata: Vec<Vec<u8>>,
}

fn verify_transaction_integrity(transaction: &Transaction<'_>) -> Result<(), SecretFallbackError> {
    crate::integrity::verify(transaction)
        .map_err(|error| SecretFallbackError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| SecretFallbackError::AuthoritySecurity(error.to_string()))
}

fn verify_admission(
    transaction: &Transaction<'_>,
    admission_id: [u8; 16],
    principal: &str,
    destination: &str,
    plan_hash: [u8; 32],
    executor_id: &str,
    resource: &str,
    observed_at: &str,
) -> Result<AdmissionEvidence, SecretFallbackError> {
    let row = transaction
        .query_row(
            "SELECT profile_id, profile_version, principal_or_process, lease_id, decision_id, egress_permit_id, resolved_launch_plan_hash, platform_executor, created_global_seq FROM sandbox_admissions WHERE admission_id = ?1",
            params![&admission_id[..]],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                    row.get::<_, Option<Vec<u8>>>(5)?,
                    row.get::<_, Vec<u8>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretFallbackError::AdmissionNotFound)?;
    let profile_id = id16(row.0, "fallback admission profile id is invalid")?;
    let profile_version = positive_u64(row.1, "fallback admission profile version is invalid")?;
    if row.2 != destination {
        return Err(SecretFallbackError::AdmissionMismatch);
    }
    let lease_id = id16(row.3, "fallback admission lease id is invalid")?;
    let decision_id = id16(row.4, "fallback admission decision id is invalid")?;
    if row.5.is_some() {
        return Err(SecretFallbackError::AdmissionEgressNotAllowed);
    }
    if hash32(row.6, "fallback admission plan hash is invalid")? != plan_hash {
        return Err(SecretFallbackError::AdmissionPlanMismatch);
    }
    if row.7 != executor_id {
        return Err(SecretFallbackError::AdmissionExecutorMismatch);
    }
    let created_global_seq = nonnegative_u64(row.8, "fallback admission sequence is invalid")?;
    let profile_status = transaction
        .query_row(
            "SELECT status FROM sandbox_profiles WHERE profile_id = ?1 AND version = ?2",
            params![&profile_id[..], to_i64(profile_version)?],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or(SecretFallbackError::AdmissionMismatch)?;
    if profile_status != "active" {
        return Err(SecretFallbackError::AdmissionMismatch);
    }

    let decision = transaction
        .query_row(
            "SELECT principal, decision, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, global_seq, authority_evidence_version FROM authorization_decisions WHERE decision_id = ?1",
            params![&decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, Option<Vec<u8>>>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretFallbackError::AdmissionDecisionNotFound)?;
    if decision.0 != principal
        || decision.1 != "allow"
        || decision.2 != "pass"
        || decision.8 < 2
    {
        return Err(SecretFallbackError::AdmissionDecisionMismatch);
    }
    let decision_lease = decision
        .3
        .ok_or(SecretFallbackError::AdmissionDecisionMismatch)
        .and_then(|value| id16(value, "fallback decision lease id is invalid"))?;
    if decision_lease != lease_id {
        return Err(SecretFallbackError::AdmissionDecisionMismatch);
    }
    let lease_generation = positive_u64(
        decision
            .4
            .ok_or(SecretFallbackError::AdmissionDecisionMismatch)?,
        "fallback decision lease generation is invalid",
    )?;
    let policy_id = decision
        .5
        .ok_or(SecretFallbackError::AdmissionDecisionMismatch)
        .and_then(|value| id16(value, "fallback decision policy id is invalid"))?;
    let policy_hash = decision
        .6
        .ok_or(SecretFallbackError::AdmissionDecisionMismatch)
        .and_then(|value| hash32(value, "fallback decision policy hash is invalid"))?;
    let decision_global_seq = nonnegative_u64(
        decision.7,
        "fallback admission decision sequence is invalid",
    )?;
    if created_global_seq != decision_global_seq {
        return Err(SecretFallbackError::AdmissionDecisionMismatch);
    }
    verify_active_policy(transaction, policy_id, policy_hash)?;
    verify_lease_chain(
        transaction,
        lease_id,
        lease_generation,
        principal,
        resource,
        observed_at,
    )?;
    Ok(AdmissionEvidence {
        decision_id,
        decision_global_seq,
    })
}

fn verify_active_policy(
    transaction: &Transaction<'_>,
    expected_id: [u8; 16],
    expected_hash: [u8; 32],
) -> Result<(), SecretFallbackError> {
    let active = transaction
        .query_row(
            "SELECT policy_bundle_id, bundle_hash FROM active_policy WHERE singleton_id = 1",
            [],
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?
        .ok_or(SecretFallbackError::PolicyMismatch)?;
    if id16(active.0, "active policy id is invalid")? != expected_id
        || hash32(active.1, "active policy hash is invalid")? != expected_hash
    {
        return Err(SecretFallbackError::PolicyMismatch);
    }
    let validated = transaction
        .query_row(
            "SELECT 1 FROM policy_bundles WHERE policy_bundle_id = ?1 AND bundle_hash = ?2 AND validation_status = 'validated' LIMIT 1",
            params![&expected_id[..], &expected_hash[..]],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if !validated {
        return Err(SecretFallbackError::PolicyMismatch);
    }
    Ok(())
}

fn verify_lease_chain(
    transaction: &Transaction<'_>,
    lease_id: [u8; 16],
    expected_generation: u64,
    principal: &str,
    resource: &str,
    observed_at: &str,
) -> Result<(), SecretFallbackError> {
    let mut next = Some(lease_id);
    let mut seen = HashSet::new();
    let mut depth = 0_usize;
    while let Some(current_id) = next {
        if depth >= MAX_PARENT_CHAIN_DEPTH {
            return Err(SecretFallbackError::LeaseParentTooDeep);
        }
        if !seen.insert(current_id) {
            return Err(SecretFallbackError::LeaseParentCycle);
        }
        let row = transaction
            .query_row(
                "SELECT principal_id, parent_lease_id, actions_scope, resources_scope, not_before, expires_at, generation, status, EXISTS(SELECT 1 FROM capability_revocations r WHERE r.lease_id = l.lease_id) FROM capability_leases l WHERE l.lease_id = ?1",
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
                        row.get::<_, i64>(8)?,
                    ))
                },
            )
            .optional()?
            .ok_or(SecretFallbackError::LeaseNotFound)?;
        if row.0 != principal {
            return Err(SecretFallbackError::LeaseMismatch);
        }
        let generation = positive_u64(row.6, "fallback lease generation is invalid")?;
        if depth == 0 && generation != expected_generation {
            return Err(SecretFallbackError::LeaseMismatch);
        }
        if row.7 != "active" {
            return Err(SecretFallbackError::LeaseInactive);
        }
        if row.8 != 0 {
            return Err(SecretFallbackError::LeaseRevoked);
        }
        if let Some(not_before) = row.4.as_deref() {
            require_stored_time(not_before, "fallback lease not_before is malformed")?;
            if observed_at < not_before {
                return Err(SecretFallbackError::LeaseNotYetValid);
            }
        }
        if let Some(expires_at) = row.5.as_deref() {
            require_stored_time(expires_at, "fallback lease expires_at is malformed")?;
            if observed_at >= expires_at {
                return Err(SecretFallbackError::LeaseExpired);
            }
        }
        if !scope_contains(&row.2, FALLBACK_ACTION)? || !scope_contains(&row.3, resource)? {
            return Err(SecretFallbackError::LeaseScopeMismatch);
        }
        next = row
            .1
            .map(|value| id16(value, "fallback parent lease id is invalid"))
            .transpose()?;
        depth += 1;
    }
    Ok(())
}

fn verify_effect(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    resource: &str,
    intent_digest: [u8; 32],
) -> Result<(), SecretFallbackError> {
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
        .ok_or(SecretFallbackError::EffectNotFound)?;
    if row.0 != FALLBACK_ACTION
        || row.1 != resource
        || row.2 != FALLBACK_RISK_CLASS
        || row.3 != FALLBACK_EXECUTION_SEMANTICS
        || row.4.as_slice() != intent_digest
        || row.5 != "authorized"
    {
        return Err(SecretFallbackError::EffectMismatch);
    }
    Ok(())
}

fn verify_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    resource: &str,
    taint_digest: [u8; 32],
    observed_at: &str,
) -> Result<(), SecretFallbackError> {
    let row = transaction
        .query_row(
            "SELECT class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at FROM approvals WHERE approval_id = ?1",
            params![&approval_id[..]],
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
        .ok_or(SecretFallbackError::ApprovalNotFound)?;
    if row.0 != "ONCE"
        || row.3.as_slice() != FALLBACK_ACTION.as_bytes()
        || row.4.as_slice() != resource.as_bytes()
        || row.5.as_deref() != Some(effect_id.0.to_be_bytes().as_slice())
        || row.6.is_some()
        || row.7 != FALLBACK_RISK_CLASS
        || hash32(row.8, "fallback approval taint digest is invalid")? != taint_digest
        || row.12 != Some(1)
    {
        return Err(SecretFallbackError::ApprovalMismatch);
    }
    require_stored_time(&row.10, "fallback approval issued_at is malformed")?;
    if observed_at < row.10.as_str() {
        return Err(SecretFallbackError::ApprovalMismatch);
    }
    if let Some(expires_at) = row.11.as_deref() {
        require_stored_time(expires_at, "fallback approval expiry is malformed")?;
        if observed_at >= expires_at {
            return Err(SecretFallbackError::ApprovalExpired);
        }
    }
    if row.13.is_some() {
        return Err(SecretFallbackError::ApprovalRevoked);
    }
    let used = transaction
        .query_row(
            "SELECT 1 FROM approval_consumptions WHERE approval_id = ?1 AND state IN ('reserved', 'consumed') LIMIT 1",
            params![&approval_id[..]],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if used {
        return Err(SecretFallbackError::ApprovalAlreadyUsed);
    }

    let parent_decision_id = id16(row.9, "fallback approval parent decision id is invalid")?;
    let scope = ApprovalScope::once(effect_id, FALLBACK_ACTION, resource)
        .map_err(|_| SecretFallbackError::ApprovalMismatch)?;
    let prepared = prepare_approval(
        &row.1,
        scope,
        &row.7,
        taint_digest,
        &row.10,
        row.11.as_deref(),
        1,
    )
    .map_err(|_| SecretFallbackError::ApprovalMismatch)?;
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
        .ok_or(SecretFallbackError::ApprovalMismatch)?;
    if parent.0 != row.1
        || parent.1 != APPROVAL_ISSUE_ACTION
        || parent.2 != prepared.resource()
        || parent.4 != "allow"
    {
        return Err(SecretFallbackError::ApprovalMismatch);
    }
    let parent_context_hash = hash32(parent.3, "fallback approval parent context hash is invalid")?;
    let rebound = bound_scope_digest(
        prepared.intent_digest(),
        parent_decision_id,
        parent_context_hash,
    )?;
    if rebound != hash32(row.2, "fallback approval scope digest is invalid")? {
        return Err(SecretFallbackError::ApprovalMismatch);
    }
    Ok(())
}

fn load_secret_material(
    transaction: &Transaction<'_>,
    handle_id: [u8; 16],
    purpose: &str,
    observed_at: &str,
) -> Result<SecretMaterial, SecretFallbackError> {
    let handle = transaction
        .query_row(
            "SELECT secret_id, version_constraint, purpose_scope, expires_at FROM secret_handles WHERE handle_id = ?1",
            params![&handle_id[..]],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretFallbackError::HandleNotFound)?;
    if handle.2.as_slice() != purpose.as_bytes() {
        return Err(SecretFallbackError::HandlePurposeMismatch);
    }
    if let Some(expires_at) = handle.3.as_deref() {
        require_stored_time(expires_at, "fallback handle expiry is malformed")?;
        if observed_at >= expires_at {
            return Err(SecretFallbackError::HandleExpired);
        }
    }
    let secret_id = id16(handle.0, "fallback secret id is invalid")?;
    let record = transaction
        .query_row(
            "SELECT classification, current_version, status, revoked_at FROM secret_records WHERE secret_id = ?1",
            params![&secret_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretFallbackError::SecretNotFound)?;
    if record.2 != "active" || record.3.is_some() {
        return Err(SecretFallbackError::SecretRevoked);
    }
    let current_version = positive_u64(record.1, "fallback current secret version is invalid")?;
    let version = match handle.1 {
        Some(value) => {
            let pinned = positive_u64(value, "fallback handle version constraint is invalid")?;
            if pinned != current_version {
                return Err(SecretFallbackError::StaleHandleVersion);
            }
            pinned
        }
        None => current_version,
    };
    let version_row = transaction
        .query_row(
            "SELECT ciphertext, nonce_or_algorithm_metadata, associated_data_hash, retired_at FROM secret_versions WHERE secret_id = ?1 AND version = ?2",
            params![&secret_id[..], to_i64(version)?],
            |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or(SecretFallbackError::SecretVersionNotFound)?;
    if version_row.3.is_some() {
        return Err(SecretFallbackError::SecretVersionRetired);
    }
    let mut statement = transaction.prepare(
        "SELECT nonce_or_algorithm_metadata FROM secret_versions ORDER BY secret_id, version",
    )?;
    let all_algorithm_metadata = statement
        .query_map([], |row| row.get::<_, Vec<u8>>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(statement);
    Ok(SecretMaterial {
        secret_id,
        version,
        classification: record.0,
        ciphertext: version_row.0,
        algorithm_metadata: version_row.1,
        associated_data_hash: hash32(
            version_row.2,
            "fallback secret associated-data hash is invalid",
        )?,
        all_algorithm_metadata,
    })
}

fn consume_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    global_seq: u64,
) -> Result<(), SecretFallbackError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(APPROVAL_CONSUMPTION_DOMAIN);
    hasher.update(&approval_id);
    hasher.update(&effect_id.0.to_be_bytes());
    let digest = hasher.finalize();
    let mut consumption_id = [0_u8; 16];
    consumption_id.copy_from_slice(&digest.as_bytes()[..16]);
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
        .map_err(|error| SecretFallbackError::AuthoritySecurity(error.to_string()))
}

fn build_launch_plan(
    request: &FallbackSecretUseRequest<'_>,
    executor_id: &str,
) -> Result<FallbackLaunchPlan, SecretFallbackError> {
    validate_text(
        request.executable,
        MAX_EXECUTABLE_BYTES,
        "executable is empty, oversized or non-canonical",
    )?;
    validate_text(
        executor_id,
        MAX_TEXT_BYTES,
        "executor id is empty, oversized or non-canonical",
    )?;
    if request.args.len() > MAX_LAUNCH_ITEMS || request.explicit_environment.len() > MAX_LAUNCH_ITEMS {
        return Err(SecretFallbackError::InvalidRequest(
            "launch item count exceeds bound",
        ));
    }
    let mut args = Vec::with_capacity(request.args.len());
    for arg in request.args {
        validate_argument(arg)?;
        args.push(arg.clone());
    }
    let mut explicit_environment = Vec::with_capacity(request.explicit_environment.len());
    for (name, value) in request.explicit_environment {
        validate_environment_name(name)?;
        validate_text(
            value,
            MAX_ENV_VALUE_BYTES,
            "environment value is empty, oversized or non-canonical",
        )?;
        explicit_environment.push((name.clone(), value.clone()));
    }
    explicit_environment.sort_by(|left, right| left.0.cmp(&right.0));
    if explicit_environment
        .windows(2)
        .any(|pair| pair[0].0 == pair[1].0)
    {
        return Err(SecretFallbackError::InvalidRequest(
            "environment contains duplicate names",
        ));
    }
    let mut plan = FallbackLaunchPlan {
        executable: request.executable.to_owned(),
        args,
        explicit_environment,
        destination_or_process: request.destination_or_process.to_owned(),
        executor_id: executor_id.to_owned(),
        channel: FallbackInjectionChannel::Stdin,
        plan_hash: [0_u8; 32],
    };
    plan.plan_hash = launch_plan_hash(&plan)?;
    Ok(plan)
}

fn launch_plan_hash(plan: &FallbackLaunchPlan) -> Result<[u8; 32], SecretFallbackError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(PLAN_DOMAIN)?;
    encoder.push_bytes(plan.executable.as_bytes())?;
    encoder.push_u64(u64::try_from(plan.args.len()).map_err(|_| SecretFallbackError::IntegerOverflow)?);
    for arg in &plan.args {
        encoder.push_bytes(arg.as_bytes())?;
    }
    encoder.push_u64(
        u64::try_from(plan.explicit_environment.len())
            .map_err(|_| SecretFallbackError::IntegerOverflow)?,
    );
    for (name, value) in &plan.explicit_environment {
        encoder.push_bytes(name.as_bytes())?;
        encoder.push_bytes(value.as_bytes())?;
    }
    encoder.push_bytes(plan.destination_or_process.as_bytes())?;
    encoder.push_bytes(plan.executor_id.as_bytes())?;
    encoder.push_u8(1);
    encoder.push_u8(1);
    encoder.push_u8(0);
    encoder.push_u8(0);
    encoder.push_u8(0);
    Ok(*blake3::hash(&encoder.finish()).as_bytes())
}

fn fallback_resource(
    handle_id: [u8; 16],
    admission_id: [u8; 16],
    purpose: &str,
    destination: &str,
    plan_hash: [u8; 32],
) -> Result<String, SecretFallbackError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(RESOURCE_DOMAIN)?;
    encoder.push_bytes(&handle_id)?;
    encoder.push_bytes(&admission_id)?;
    encoder.push_bytes(purpose.as_bytes())?;
    encoder.push_bytes(destination.as_bytes())?;
    encoder.push_bytes(&plan_hash)?;
    let digest = blake3::hash(&encoder.finish());
    Ok(format!(
        "secret-fallback:{}:{}",
        hex_bytes(&handle_id),
        hex_bytes(&digest.as_bytes()[..16])
    ))
}

fn fallback_intent_digest(
    handle_id: [u8; 16],
    admission_id: [u8; 16],
    effect_id: EffectId,
    principal: &str,
    resource: &str,
    plan_hash: [u8; 32],
    taint_digest: [u8; 32],
) -> Result<[u8; 32], SecretFallbackError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(INTENT_DOMAIN)?;
    encoder.push_bytes(&handle_id)?;
    encoder.push_bytes(&admission_id)?;
    encoder.push_u128(effect_id.0);
    encoder.push_bytes(principal.as_bytes())?;
    encoder.push_bytes(resource.as_bytes())?;
    encoder.push_bytes(&plan_hash)?;
    encoder.push_bytes(&taint_digest)?;
    Ok(*blake3::hash(&encoder.finish()).as_bytes())
}

fn fallback_use_id(
    handle_id: [u8; 16],
    version: u64,
    admission_id: [u8; 16],
    approval_id: [u8; 16],
    effect_id: EffectId,
) -> Result<[u8; 16], SecretFallbackError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(USE_ID_DOMAIN)?;
    encoder.push_bytes(&handle_id)?;
    encoder.push_u64(version);
    encoder.push_bytes(&admission_id)?;
    encoder.push_bytes(&approval_id)?;
    encoder.push_u128(effect_id.0);
    let digest = blake3::hash(&encoder.finish());
    let mut id = [0_u8; 16];
    id.copy_from_slice(&digest.as_bytes()[..16]);
    Ok(id)
}

fn bound_scope_digest(
    intent_digest: [u8; 32],
    parent_decision_id: [u8; 16],
    context_hash: [u8; 32],
) -> Result<[u8; 32], SecretFallbackError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(APPROVAL_BINDING_DOMAIN)?;
    encoder.push_bytes(&intent_digest)?;
    encoder.push_bytes(&parent_decision_id)?;
    encoder.push_bytes(&context_hash)?;
    Ok(*blake3::hash(&encoder.finish()).as_bytes())
}

fn secret_occurs_in_launch_fields(secret: &[u8], plan: &FallbackLaunchPlan) -> bool {
    contains_bytes(plan.executable.as_bytes(), secret)
        || plan.args.iter().any(|arg| contains_bytes(arg.as_bytes(), secret))
        || plan.explicit_environment.iter().any(|(name, value)| {
            contains_bytes(name.as_bytes(), secret) || contains_bytes(value.as_bytes(), secret)
        })
}

fn redact_bytes(input: &[u8], secret: &[u8]) -> Vec<u8> {
    if secret.is_empty() {
        return input.to_vec();
    }
    let mut output = Vec::with_capacity(input.len());
    let mut cursor = 0_usize;
    while cursor < input.len() {
        if input[cursor..].starts_with(secret) {
            output.extend_from_slice(REDACTION_MARKER);
            cursor += secret.len();
        } else {
            output.push(input[cursor]);
            cursor += 1;
        }
    }
    output
}

fn contains_bytes(value: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && value.windows(needle.len()).any(|window| window == needle)
}

fn validate_request(request: &FallbackSecretUseRequest<'_>) -> Result<(), SecretFallbackError> {
    validate_text(request.principal, MAX_TEXT_BYTES, "principal is invalid")?;
    validate_text(request.purpose, MAX_TEXT_BYTES, "purpose is invalid")?;
    validate_text(
        request.destination_or_process,
        MAX_TEXT_BYTES,
        "destination/process is invalid",
    )?;
    if !(request.destination_or_process.starts_with("process:")
        || request.destination_or_process.starts_with("service:"))
    {
        return Err(SecretFallbackError::InvalidRequest(
            "fallback destination must be strict-local process/service identity",
        ));
    }
    if !valid_utc_second(request.observed_at) {
        return Err(SecretFallbackError::InvalidRequest(
            "observed_at must be canonical UTC-second time",
        ));
    }
    Ok(())
}

fn validate_argument(value: &str) -> Result<(), SecretFallbackError> {
    if value.len() > MAX_ARG_BYTES || value.chars().any(|ch| ch == '\0' || ch.is_control()) {
        return Err(SecretFallbackError::InvalidRequest(
            "argument is oversized or contains control data",
        ));
    }
    Ok(())
}

fn validate_environment_name(value: &str) -> Result<(), SecretFallbackError> {
    let bytes = value.as_bytes();
    if value.is_empty()
        || value.len() > 128
        || !(bytes[0].is_ascii_uppercase() || bytes[0] == b'_')
        || bytes
            .iter()
            .any(|byte| !(byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_'))
    {
        return Err(SecretFallbackError::InvalidRequest(
            "environment name is not canonical",
        ));
    }
    Ok(())
}

fn validate_text(
    value: &str,
    max_bytes: usize,
    reason: &'static str,
) -> Result<(), SecretFallbackError> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(SecretFallbackError::InvalidRequest(reason));
    }
    Ok(())
}

fn scope_contains(bytes: &[u8], expected: &str) -> Result<bool, SecretFallbackError> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        SecretFallbackError::InvalidStoredRecord("fallback lease scope is not UTF-8")
    })?;
    if text.is_empty() {
        return Ok(false);
    }
    let entries = text.split('\n').collect::<Vec<_>>();
    if entries.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(SecretFallbackError::InvalidStoredRecord(
            "fallback lease scope is not strictly sorted and unique",
        ));
    }
    Ok(entries.binary_search(&expected).is_ok())
}

fn id16(value: Vec<u8>, reason: &'static str) -> Result<[u8; 16], SecretFallbackError> {
    value
        .try_into()
        .map_err(|_| SecretFallbackError::InvalidStoredRecord(reason))
}

fn hash32(value: Vec<u8>, reason: &'static str) -> Result<[u8; 32], SecretFallbackError> {
    value
        .try_into()
        .map_err(|_| SecretFallbackError::InvalidStoredRecord(reason))
}

fn nonnegative_u64(value: i64, reason: &'static str) -> Result<u64, SecretFallbackError> {
    u64::try_from(value).map_err(|_| SecretFallbackError::InvalidStoredRecord(reason))
}

fn positive_u64(value: i64, reason: &'static str) -> Result<u64, SecretFallbackError> {
    let value = nonnegative_u64(value, reason)?;
    if value == 0 {
        return Err(SecretFallbackError::InvalidStoredRecord(reason));
    }
    Ok(value)
}

fn to_i64(value: u64) -> Result<i64, SecretFallbackError> {
    i64::try_from(value).map_err(|_| SecretFallbackError::IntegerOverflow)
}

fn require_stored_time(value: &str, reason: &'static str) -> Result<(), SecretFallbackError> {
    if valid_utc_second(value) {
        Ok(())
    } else {
        Err(SecretFallbackError::InvalidStoredRecord(reason))
    }
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

fn hex_bytes(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_security_write::{
        append_active_policy_snapshot, append_approval_snapshot,
        append_authorization_decision_v2_snapshot, append_capability_lease_snapshot,
        append_policy_bundle_snapshot, append_sandbox_admission_snapshot,
        append_sandbox_profile_snapshot, append_secret_handle_snapshot,
        append_secret_record_snapshot, append_secret_version_snapshot,
    };
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use crate::security_audit::{AuthorizationAuditInput, append_authorization_decision};
    use golam_core::paths::RuntimeLayout;
    use golam_core::{EffectTransitionId, EventId, SessionId};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    use zeroize::Zeroizing;

    const CANARY: &[u8] = b"golam-t003-054-deterministic-canary-secret";
    const EXECUTOR_ID: &str = "golam-test-stdin-fallback-v1";
    static N: AtomicU64 = AtomicU64::new(0);

    #[derive(Clone)]
    struct TestProtector {
        key: [u8; 32],
    }

    impl KeyProtector for TestProtector {
        fn load_master_key(
            &self,
        ) -> Result<Zeroizing<Vec<u8>>, crate::secret_vault::KeyProtectionError> {
            Ok(Zeroizing::new(self.key.to_vec()))
        }

        fn store_master_key(
            &self,
            _key: &[u8],
        ) -> Result<(), crate::secret_vault::KeyProtectionError> {
            Err(crate::secret_vault::KeyProtectionError::Unsupported)
        }
    }

    struct RecordingInjector {
        capabilities: FallbackInjectorCapabilities,
    }

    impl RecordingInjector {
        fn qualified() -> Self {
            Self {
                capabilities: FallbackInjectorCapabilities {
                    clear_environment: true,
                    stdin_secret_channel: true,
                    closes_stdin_after_write: true,
                    forbids_secret_argv: true,
                    forbids_secret_environment: true,
                    no_ambient_secret_inheritance: true,
                    captures_stdout_stderr: true,
                },
            }
        }
    }

    impl FallbackInjector for RecordingInjector {
        fn executor_id(&self) -> &'static str {
            EXECUTOR_ID
        }

        fn capabilities(&self) -> FallbackInjectorCapabilities {
            self.capabilities
        }

        fn inject_stdin(
            &mut self,
            plan: &FallbackLaunchPlan,
            secret: &[u8],
        ) -> Result<FallbackRawOutput, Vec<u8>> {
            assert!(plan.clears_environment());
            assert!(!plan.secret_in_argv());
            assert!(!plan.secret_in_environment());
            assert!(!plan.secret_inherited_by_descendants());
            assert_eq!(plan.channel(), FallbackInjectionChannel::Stdin);
            assert!(!plan
                .args()
                .iter()
                .any(|arg| contains_bytes(arg.as_bytes(), secret)));
            assert!(!plan.explicit_environment().iter().any(|(_, value)| {
                contains_bytes(value.as_bytes(), secret)
            }));
            let mut stdout = b"stdout-before:".to_vec();
            stdout.extend_from_slice(secret);
            stdout.extend_from_slice(b":stdout-after");
            let mut stderr = secret.to_vec();
            stderr.extend_from_slice(b":stderr");
            Ok(FallbackRawOutput {
                exit_code: Some(0),
                stdout,
                stderr,
            })
        }
    }

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-secret-fallback-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    struct Fixture {
        handle_id: [u8; 16],
        admission_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
        plan: FallbackLaunchPlan,
        resource: String,
        protector: TestProtector,
    }

    fn base_request<'a>(fixture: &'a Fixture, args: &'a [String]) -> FallbackSecretUseRequest<'a> {
        FallbackSecretUseRequest {
            handle_id: fixture.handle_id,
            principal: "owner:owner",
            purpose: "git.auth",
            destination_or_process: "process:git",
            admission_id: fixture.admission_id,
            approval_id: fixture.approval_id,
            effect_id: fixture.effect_id,
            observed_at: "2026-08-28T12:00:00Z",
            taint_digest: [0_u8; 32],
            executable: "test-helper",
            args,
            explicit_environment: &[("GOLAM_TEST_MODE".to_owned(), "1".to_owned())],
        }
    }

    fn seed_fixture(authority: &AuthorityLayout) -> Fixture {
        let handle_id = [31_u8; 16];
        let secret_id = [32_u8; 16];
        let lease_id = [33_u8; 16];
        let policy_id = [34_u8; 16];
        let policy_hash = [35_u8; 32];
        let profile_id = [36_u8; 16];
        let admission_id = [37_u8; 16];
        let admission_decision_id = [38_u8; 16];
        let approval_parent_decision_id = [39_u8; 16];
        let approval_id = [40_u8; 16];
        let effect_id = EffectId(41);
        let protector = TestProtector { key: [42_u8; 32] };
        let empty_args: Vec<String> = Vec::new();
        let request = FallbackSecretUseRequest {
            handle_id,
            principal: "owner:owner",
            purpose: "git.auth",
            destination_or_process: "process:git",
            admission_id,
            approval_id,
            effect_id,
            observed_at: "2026-08-28T12:00:00Z",
            taint_digest: [0_u8; 32],
            executable: "test-helper",
            args: &empty_args,
            explicit_environment: &[("GOLAM_TEST_MODE".to_owned(), "1".to_owned())],
        };
        let plan = build_launch_plan(&request, EXECUTOR_ID).unwrap();
        let resource = fallback_resource(
            handle_id,
            admission_id,
            request.purpose,
            request.destination_or_process,
            plan.plan_hash(),
        )
        .unwrap();
        let intent_digest = fallback_intent_digest(
            handle_id,
            admission_id,
            effect_id,
            request.principal,
            &resource,
            plan.plan_hash(),
            request.taint_digest,
        )
        .unwrap();

        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        let mut effects = EffectStore::open(authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(1),
                requested_by: "owner:owner",
                action: FALLBACK_ACTION,
                resource: &resource,
                risk_class: FALLBACK_RISK_CLASS,
                execution_semantics: FALLBACK_EXECUTION_SEMANTICS,
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: b"[]",
                payload_hash: intent_digest,
                proposed_event_id: EventId(410),
                transition_id: EffectTransitionId(411),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(412),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("secret_fallback_authorized"),
                evidence_ref: None,
                event_id: EventId(413),
            })
            .unwrap();
        drop(effects);

        let connection = Connection::open(authority.authority_db_path()).unwrap();
        let transaction = connection
            .unchecked_transaction()
            .unwrap();
        transaction
            .execute(
                "INSERT INTO policy_bundles (policy_bundle_id, version, schema_version, canonical_policy_bytes, bundle_hash, created_by, created_global_seq, validation_status) VALUES (?1, 1, 1, X'01', ?2, 'owner:owner', 0, 'validated')",
                params![&policy_id[..], &policy_hash[..]],
            )
            .unwrap();
        append_policy_bundle_snapshot(&transaction, &policy_id).unwrap();
        transaction
            .execute(
                "INSERT INTO active_policy (singleton_id, policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq) VALUES (1, ?1, ?2, 'owner:owner', ?3, 0)",
                params![&policy_id[..], &policy_hash[..], &[43_u8; 16][..]],
            )
            .unwrap();
        append_active_policy_snapshot(&transaction).unwrap();
        transaction
            .execute(
                "INSERT INTO capability_leases (lease_id, principal_id, parent_lease_id, actions_scope, resources_scope, context_constraints, issued_by, issued_global_seq, not_before, expires_at, generation, status, authority_digest) VALUES (?1, 'owner:owner', NULL, ?2, ?3, X'', 'owner:owner', 0, '2026-08-28T00:00:00Z', '2026-08-29T00:00:00Z', 1, 'active', ?4)",
                params![&lease_id[..], FALLBACK_ACTION.as_bytes(), resource.as_bytes(), &[44_u8; 32][..]],
            )
            .unwrap();
        append_capability_lease_snapshot(&transaction, &lease_id).unwrap();

        let vault = SecretVault::from_persisted_algorithm_metadata(protector.clone(), []).unwrap();
        let binding = VaultBinding::new(secret_id, 1, "api_credential", 1).unwrap();
        let encrypted = vault.seal(&binding, CANARY).unwrap();
        transaction
            .execute(
                "INSERT INTO secret_records (secret_id, classification, owner_principal, current_version, status, created_global_seq, revoked_at) VALUES (?1, 'api_credential', 'owner:owner', 1, 'active', 0, NULL)",
                params![&secret_id[..]],
            )
            .unwrap();
        append_secret_record_snapshot(&transaction, &secret_id).unwrap();
        transaction
            .execute(
                "INSERT INTO secret_versions (secret_id, version, ciphertext, nonce_or_algorithm_metadata, associated_data_hash, created_global_seq, rotated_from, retired_at) VALUES (?1, 1, ?2, ?3, ?4, 0, NULL, NULL)",
                params![
                    &secret_id[..],
                    encrypted.ciphertext(),
                    encrypted.algorithm_metadata(),
                    &encrypted.associated_data_hash()[..],
                ],
            )
            .unwrap();
        append_secret_version_snapshot(&transaction, &secret_id, 1).unwrap();
        transaction
            .execute(
                "INSERT INTO secret_handles (handle_id, secret_id, version_constraint, purpose_scope, expires_at) VALUES (?1, ?2, 1, ?3, '2026-08-29T00:00:00Z')",
                params![&handle_id[..], &secret_id[..], b"git.auth".as_slice()],
            )
            .unwrap();
        append_secret_handle_snapshot(&transaction, &handle_id).unwrap();

        transaction
            .execute(
                "INSERT INTO sandbox_profiles (profile_id, version, class, filesystem_read_roots, filesystem_write_roots, network_rule, environment_allowlist, spawn_rule, cpu_limit, memory_limit, time_limit, output_limit, device_allowlist, ipc_allowlist, inherited_handle_rules, platform_requirements, status) VALUES (?1, 1, 'fallback_test', X'', X'', X'00', X'', X'00', NULL, NULL, NULL, NULL, X'', X'', X'', X'', 'active')",
                params![&profile_id[..]],
            )
            .unwrap();
        append_sandbox_profile_snapshot(&transaction, &profile_id, 1).unwrap();

        let approval_scope = ApprovalScope::once(effect_id, FALLBACK_ACTION, &resource).unwrap();
        let prepared = prepare_approval(
            "owner:owner",
            approval_scope,
            FALLBACK_RISK_CLASS,
            [0_u8; 32],
            "2026-08-28T00:00:00Z",
            None,
            1,
        )
        .unwrap();
        let approval_context_hash = [45_u8; 32];
        transaction
            .execute(
                "INSERT INTO authorization_decisions (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq, hard_guard_result, matched_rule_ids, authority_evidence_version) VALUES (?1, 'owner:owner', ?2, ?3, ?4, 'allow', 'test_fallback_approval_issue', 3, 'pass', X'', 2)",
                params![
                    &approval_parent_decision_id[..],
                    APPROVAL_ISSUE_ACTION,
                    prepared.resource(),
                    &approval_context_hash[..],
                ],
            )
            .unwrap();
        append_authorization_decision(
            &transaction,
            AuthorizationAuditInput {
                decision_id: &approval_parent_decision_id,
                principal: "owner:owner",
                action: APPROVAL_ISSUE_ACTION,
                resource: prepared.resource(),
                context_hash: &approval_context_hash,
                decision: "allow",
                reason_code: "test_fallback_approval_issue",
                global_seq: 3,
            },
        )
        .unwrap();
        append_authorization_decision_v2_snapshot(&transaction, &approval_parent_decision_id).unwrap();
        let scope_digest = bound_scope_digest(
            prepared.intent_digest(),
            approval_parent_decision_id,
            approval_context_hash,
        )
        .unwrap();
        transaction
            .execute(
                "INSERT INTO approvals (approval_id, class, approver_principal, scope_digest, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, parent_decision_id, issued_at, expires_at, max_uses, revoked_at) VALUES (?1, 'ONCE', 'owner:owner', ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, '2026-08-28T00:00:00Z', NULL, 1, NULL)",
                params![
                    &approval_id[..],
                    &scope_digest[..],
                    FALLBACK_ACTION.as_bytes(),
                    resource.as_bytes(),
                    &effect_id.0.to_be_bytes()[..],
                    FALLBACK_RISK_CLASS,
                    &[0_u8; 32][..],
                    &approval_parent_decision_id[..],
                ],
            )
            .unwrap();
        append_approval_snapshot(&transaction, &approval_id).unwrap();

        transaction
            .execute(
                "INSERT INTO authorization_decisions (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, authority_evidence_version) VALUES (?1, 'owner:owner', 'sandbox.admit', 'sandbox-admission:test', ?2, 'allow', 'test_fallback_admission', 4, 'pass', ?3, 1, ?4, ?5, X'', NULL, 2)",
                params![
                    &admission_decision_id[..],
                    &[46_u8; 32][..],
                    &lease_id[..],
                    &policy_id[..],
                    &policy_hash[..],
                ],
            )
            .unwrap();
        append_authorization_decision(
            &transaction,
            AuthorizationAuditInput {
                decision_id: &admission_decision_id,
                principal: "owner:owner",
                action: "sandbox.admit",
                resource: "sandbox-admission:test",
                context_hash: &[46_u8; 32],
                decision: "allow",
                reason_code: "test_fallback_admission",
                global_seq: 4,
            },
        )
        .unwrap();
        append_authorization_decision_v2_snapshot(&transaction, &admission_decision_id).unwrap();
        transaction
            .execute(
                "INSERT INTO sandbox_admissions (admission_id, profile_id, profile_version, principal_or_process, lease_id, decision_id, egress_permit_id, resolved_launch_plan_hash, platform_executor, created_global_seq) VALUES (?1, ?2, 1, 'process:git', ?3, ?4, NULL, ?5, ?6, 4)",
                params![
                    &admission_id[..],
                    &profile_id[..],
                    &lease_id[..],
                    &admission_decision_id[..],
                    &plan.plan_hash()[..],
                    EXECUTOR_ID,
                ],
            )
            .unwrap();
        append_sandbox_admission_snapshot(&transaction, &admission_id).unwrap();
        crate::integrity::verify(&transaction).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();

        Fixture {
            handle_id,
            admission_id,
            approval_id,
            effect_id,
            plan,
            resource,
            protector,
        }
    }

    #[test]
    fn fallback_redacts_canary_and_consumes_exact_once_approval() {
        let (runtime, authority) = authority();
        let fixture = seed_fixture(&authority);
        let mut store = SecretFallbackStore::open_with_protector(
            &authority,
            fixture.protector.clone(),
        )
        .unwrap();
        let args: Vec<String> = Vec::new();
        let mut injector = RecordingInjector::qualified();
        let output = store
            .execute_with_injector(base_request(&fixture, &args), &mut injector)
            .unwrap();
        assert_eq!(output.exit_code(), Some(0));
        assert!(!contains_bytes(output.stdout(), CANARY));
        assert!(!contains_bytes(output.stderr(), CANARY));
        assert!(contains_bytes(output.stdout(), REDACTION_MARKER));
        assert!(contains_bytes(output.stderr(), REDACTION_MARKER));
        let row: (String, String) = store
            .connection
            .query_row(
                "SELECT mode, destination_or_process FROM secret_use_records WHERE use_id = ?1",
                params![&output.use_id()[..]],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(row.0, "fallback_stdin");
        assert_eq!(row.1, "process:git");
        let consumption_count: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM approval_consumptions WHERE approval_id = ?1 AND state = 'consumed'",
                params![&fixture.approval_id[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(consumption_count, 1);
        assert!(matches!(
            store.execute_with_injector(base_request(&fixture, &args), &mut injector),
            Err(SecretFallbackError::ApprovalAlreadyUsed)
        ));
        crate::authority_security_v2::verify(&store.connection).unwrap();
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn secret_in_argv_is_rejected_before_use_or_approval_consumption() {
        let (runtime, authority) = authority();
        let fixture = seed_fixture(&authority);
        let mut store = SecretFallbackStore::open_with_protector(
            &authority,
            fixture.protector.clone(),
        )
        .unwrap();
        let args = vec![String::from_utf8(CANARY.to_vec()).unwrap()];
        let mut injector = RecordingInjector::qualified();
        assert!(matches!(
            store.execute_with_injector(base_request(&fixture, &args), &mut injector),
            Err(SecretFallbackError::SecretPresentInLaunchField)
        ));
        let use_count: i64 = store
            .connection
            .query_row("SELECT COUNT(*) FROM secret_use_records", [], |row| row.get(0))
            .unwrap();
        let consumption_count: i64 = store
            .connection
            .query_row("SELECT COUNT(*) FROM approval_consumptions", [], |row| row.get(0))
            .unwrap();
        assert_eq!(use_count, 0);
        assert_eq!(consumption_count, 0);
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn admission_plan_and_injector_capabilities_fail_closed() {
        let (runtime, authority) = authority();
        let fixture = seed_fixture(&authority);
        let mut store = SecretFallbackStore::open_with_protector(
            &authority,
            fixture.protector.clone(),
        )
        .unwrap();
        let args: Vec<String> = Vec::new();
        let mut weak = RecordingInjector::qualified();
        weak.capabilities.no_ambient_secret_inheritance = false;
        assert!(matches!(
            store.execute_with_injector(base_request(&fixture, &args), &mut weak),
            Err(SecretFallbackError::InjectorCapabilitiesUnavailable)
        ));

        let transaction = store
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "UPDATE sandbox_admissions SET resolved_launch_plan_hash = ?2 WHERE admission_id = ?1",
                params![&fixture.admission_id[..], &[99_u8; 32][..]],
            )
            .unwrap();
        append_sandbox_admission_snapshot(&transaction, &fixture.admission_id).unwrap();
        transaction.commit().unwrap();
        let mut injector = RecordingInjector::qualified();
        assert!(matches!(
            store.execute_with_injector(base_request(&fixture, &args), &mut injector),
            Err(SecretFallbackError::AdmissionPlanMismatch)
        ));
        drop(store);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn launch_plan_is_stdin_only_and_hash_sensitive() {
        let (runtime, authority) = authority();
        let fixture = seed_fixture(&authority);
        assert_eq!(fixture.plan.channel(), FallbackInjectionChannel::Stdin);
        assert!(fixture.plan.clears_environment());
        assert!(!fixture.plan.secret_in_argv());
        assert!(!fixture.plan.secret_in_environment());
        assert!(!fixture.plan.secret_inherited_by_descendants());
        let changed_args = vec!["--different".to_owned()];
        let request = base_request(&fixture, &changed_args);
        let changed = build_launch_plan(&request, EXECUTOR_ID).unwrap();
        assert_ne!(fixture.plan.plan_hash(), changed.plan_hash());
        assert!(!fixture.resource.is_empty());
        drop(AuthorityStore::open(authority.authority_db_path()).unwrap());
        fs::remove_dir_all(runtime.root).unwrap();
    }
}