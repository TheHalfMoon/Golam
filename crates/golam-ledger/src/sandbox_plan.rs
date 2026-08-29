#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::CanonicalEncoder;
use golam_core::authority::AuthorityLayout;
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::egress_permit::{load_permit, verify_lease_chain_for_use};
use crate::sandbox_profile::{
    SandboxNetworkRule, SandboxProfileClass, SandboxProfileRecord, SandboxSpawnRule, load_profile,
    sandbox_profile_resource,
};
use crate::storage::{AuthorityStore, StorageError};

pub const SANDBOX_LAUNCH_ACTION: &str = "sandbox.launch";

const PLAN_DOMAIN: &[u8] = b"golam:sandbox-launch-plan:v1";
const MAX_PRINCIPAL_BYTES: usize = 512;

#[derive(Clone, Copy, Debug)]
pub struct SandboxPlanRequest<'a> {
    pub profile_id: [u8; 16],
    pub profile_version: u64,
    pub principal_or_process: &'a str,
    pub decision_id: [u8; 16],
    pub lease_id: [u8; 16],
    pub egress_permit_id: Option<[u8; 16]>,
    pub observed_at: &'a str,
    pub strict_local: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxEgressBinding {
    pub permit_id: [u8; 16],
    pub action: String,
    pub purpose: String,
    pub destination_scope: String,
    pub protocol_port_scope: String,
    pub taint_digest: [u8; 32],
    pub secret_handle_id: Option<[u8; 16]>,
    pub expires_at: Option<String>,
    pub usage_limit: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxLaunchPlan {
    pub profile_id: [u8; 16],
    pub profile_version: u64,
    pub profile_class: SandboxProfileClass,
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
    pub principal_or_process: String,
    pub decision_id: [u8; 16],
    pub lease_id: [u8; 16],
    pub lease_generation: u64,
    pub policy_bundle_id: [u8; 16],
    pub policy_bundle_hash: [u8; 32],
    pub egress: Option<SandboxEgressBinding>,
    pub strict_local: bool,
    pub observed_at: String,
    pub plan_hash: [u8; 32],
}

#[derive(Debug)]
pub enum SandboxPlanError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Canonical(golam_core::CoreError),
    Integrity(String),
    AuthoritySecurity(String),
    Profile(String),
    InvalidPrincipal,
    InvalidTime,
    MissingDecision,
    DecisionMismatch,
    DecisionStale,
    ActivePolicyMissing,
    ActivePolicyMismatch,
    ActivePolicyInvalid,
    LeaseAuthority(String),
    StrictLocalExternalEgressDenied,
    EgressPermitRequired,
    EgressPermitForbidden,
    EgressPermitInvalid,
    EgressPermitExpired,
    EgressPermitExhausted,
}

impl fmt::Display for SandboxPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "sandbox plan authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "sandbox plan sqlite error: {error}"),
            Self::Canonical(error) => write!(f, "sandbox plan canonical encoding error: {error}"),
            Self::Integrity(error) => write!(f, "sandbox plan integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "sandbox plan authority-security error: {error}")
            }
            Self::Profile(error) => write!(f, "sandbox plan profile error: {error}"),
            Self::InvalidPrincipal => f.write_str("sandbox plan principal/process is invalid"),
            Self::InvalidTime => f.write_str("sandbox plan observed_at is invalid"),
            Self::MissingDecision => f.write_str("sandbox launch authorization decision is missing"),
            Self::DecisionMismatch => {
                f.write_str("sandbox launch authorization decision is mismatched")
            }
            Self::DecisionStale => f.write_str("sandbox launch authorization decision is stale"),
            Self::ActivePolicyMissing => f.write_str("sandbox launch active policy is missing"),
            Self::ActivePolicyMismatch => {
                f.write_str("sandbox launch decision policy is not the active policy")
            }
            Self::ActivePolicyInvalid => {
                f.write_str("sandbox launch active policy bundle is not validated")
            }
            Self::LeaseAuthority(error) => write!(f, "sandbox launch lease authority denied: {error}"),
            Self::StrictLocalExternalEgressDenied => {
                f.write_str("strict-local mode forbids an external-egress sandbox plan")
            }
            Self::EgressPermitRequired => {
                f.write_str("sandbox profile requires an exact active egress permit")
            }
            Self::EgressPermitForbidden => {
                f.write_str("sandbox profile does not permit an external egress binding")
            }
            Self::EgressPermitInvalid => f.write_str("sandbox egress permit is invalid or mismatched"),
            Self::EgressPermitExpired => f.write_str("sandbox egress permit is expired"),
            Self::EgressPermitExhausted => f.write_str("sandbox egress permit is exhausted"),
        }
    }
}

impl Error for SandboxPlanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Canonical(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for SandboxPlanError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for SandboxPlanError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<golam_core::CoreError> for SandboxPlanError {
    fn from(value: golam_core::CoreError) -> Self {
        Self::Canonical(value)
    }
}

pub struct SandboxPlanCompiler {
    connection: Connection,
}

impl SandboxPlanCompiler {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, SandboxPlanError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn compile(
        &mut self,
        request: SandboxPlanRequest<'_>,
    ) -> Result<SandboxLaunchPlan, SandboxPlanError> {
        validate_principal(request.principal_or_process)?;
        if !valid_utc_second(request.observed_at) {
            return Err(SandboxPlanError::InvalidTime);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Deferred)?;
        crate::integrity::verify(&transaction)
            .map_err(|error| SandboxPlanError::Integrity(error.to_string()))?;
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| SandboxPlanError::AuthoritySecurity(error.to_string()))?;

        let profile = load_profile(&transaction, request.profile_id, request.profile_version)
            .map_err(|error| SandboxPlanError::Profile(error.to_string()))?;
        let resource = sandbox_profile_resource(request.profile_id, request.profile_version);
        let authority = load_current_launch_authority(
            &transaction,
            request.decision_id,
            request.principal_or_process,
            &resource,
            request.lease_id,
        )?;
        verify_active_policy(
            &transaction,
            authority.policy_bundle_id,
            authority.policy_bundle_hash,
        )?;
        verify_lease_chain_for_use(
            &transaction,
            request.lease_id,
            authority.lease_generation,
            request.principal_or_process,
            SANDBOX_LAUNCH_ACTION,
            &resource,
            request.observed_at,
        )
        .map_err(|error| SandboxPlanError::LeaseAuthority(error.to_string()))?;

        let egress = compile_egress_binding(
            &transaction,
            &profile,
            request.principal_or_process,
            request.lease_id,
            authority.lease_generation,
            request.egress_permit_id,
            request.observed_at,
            request.strict_local,
        )?;
        let plan_hash = plan_hash(&profile, &request, &authority, egress.as_ref())?;

        Ok(SandboxLaunchPlan {
            profile_id: profile.profile_id,
            profile_version: profile.version,
            profile_class: profile.class,
            filesystem_read_roots: profile.filesystem_read_roots,
            filesystem_write_roots: profile.filesystem_write_roots,
            network_rule: profile.network_rule,
            environment_allowlist: profile.environment_allowlist,
            spawn_rule: profile.spawn_rule,
            cpu_limit: profile.cpu_limit,
            memory_limit: profile.memory_limit,
            time_limit: profile.time_limit,
            output_limit: profile.output_limit,
            device_allowlist: profile.device_allowlist,
            ipc_allowlist: profile.ipc_allowlist,
            inherited_handle_rules: profile.inherited_handle_rules,
            platform_requirements: profile.platform_requirements,
            principal_or_process: request.principal_or_process.to_owned(),
            decision_id: request.decision_id,
            lease_id: request.lease_id,
            lease_generation: authority.lease_generation,
            policy_bundle_id: authority.policy_bundle_id,
            policy_bundle_hash: authority.policy_bundle_hash,
            egress,
            strict_local: request.strict_local,
            observed_at: request.observed_at.to_owned(),
            plan_hash,
        })
    }
}

#[derive(Clone, Copy)]
struct LaunchAuthority {
    lease_generation: u64,
    policy_bundle_id: [u8; 16],
    policy_bundle_hash: [u8; 32],
}

fn load_current_launch_authority(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    principal: &str,
    resource: &str,
    lease_id: [u8; 16],
) -> Result<LaunchAuthority, SandboxPlanError> {
    let row = transaction
        .query_row(
            "SELECT principal, action, resource, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, decision, global_seq, authority_evidence_version \
             FROM authorization_decisions WHERE decision_id = ?1",
            params![&decision_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, Option<Vec<u8>>>(6)?,
                    row.get::<_, Option<Vec<u8>>>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, i64>(9)?,
                    row.get::<_, i64>(10)?,
                ))
            },
        )
        .optional()?
        .ok_or(SandboxPlanError::MissingDecision)?;
    if row.0 != principal
        || row.1 != SANDBOX_LAUNCH_ACTION
        || row.2 != resource
        || row.3 != "pass"
        || row.8 != "allow"
        || row.10 < 2
    {
        return Err(SandboxPlanError::DecisionMismatch);
    }
    let stored_lease = id16(
        row.4.ok_or(SandboxPlanError::DecisionMismatch)?,
        SandboxPlanError::DecisionMismatch,
    )?;
    if stored_lease != lease_id {
        return Err(SandboxPlanError::DecisionMismatch);
    }
    let lease_generation = positive_u64(
        row.5.ok_or(SandboxPlanError::DecisionMismatch)?,
        SandboxPlanError::DecisionMismatch,
    )?;
    let policy_bundle_id = id16(
        row.6.ok_or(SandboxPlanError::DecisionMismatch)?,
        SandboxPlanError::DecisionMismatch,
    )?;
    let policy_bundle_hash = hash32(
        row.7.ok_or(SandboxPlanError::DecisionMismatch)?,
        SandboxPlanError::DecisionMismatch,
    )?;
    let decision_seq = nonnegative_u64(row.9, SandboxPlanError::DecisionMismatch)?;
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (\
           SELECT global_seq FROM session_events \
           UNION ALL SELECT global_seq FROM effect_transitions \
           UNION ALL SELECT global_seq FROM authorization_decisions\
         )",
        [],
        |row| row.get(0),
    )?;
    if decision_seq != nonnegative_u64(latest, SandboxPlanError::DecisionMismatch)? {
        return Err(SandboxPlanError::DecisionStale);
    }
    Ok(LaunchAuthority {
        lease_generation,
        policy_bundle_id,
        policy_bundle_hash,
    })
}

fn verify_active_policy(
    transaction: &Transaction<'_>,
    expected_id: [u8; 16],
    expected_hash: [u8; 32],
) -> Result<(), SandboxPlanError> {
    let row = transaction
        .query_row(
            "SELECT policy_bundle_id, bundle_hash FROM active_policy WHERE singleton_id = 1",
            [],
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .optional()?
        .ok_or(SandboxPlanError::ActivePolicyMissing)?;
    if id16(row.0, SandboxPlanError::ActivePolicyMismatch)? != expected_id
        || hash32(row.1, SandboxPlanError::ActivePolicyMismatch)? != expected_hash
    {
        return Err(SandboxPlanError::ActivePolicyMismatch);
    }
    let status = transaction
        .query_row(
            "SELECT validation_status FROM policy_bundles WHERE policy_bundle_id = ?1 AND bundle_hash = ?2",
            params![&expected_id[..], &expected_hash[..]],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or(SandboxPlanError::ActivePolicyInvalid)?;
    if status != "validated" {
        return Err(SandboxPlanError::ActivePolicyInvalid);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compile_egress_binding(
    transaction: &Transaction<'_>,
    profile: &SandboxProfileRecord,
    principal: &str,
    lease_id: [u8; 16],
    lease_generation: u64,
    permit_id: Option<[u8; 16]>,
    observed_at: &str,
    strict_local: bool,
) -> Result<Option<SandboxEgressBinding>, SandboxPlanError> {
    if strict_local && profile.network_rule == SandboxNetworkRule::PermitRequired {
        return Err(SandboxPlanError::StrictLocalExternalEgressDenied);
    }
    match profile.network_rule {
        SandboxNetworkRule::DenyAll | SandboxNetworkRule::LoopbackOnly => {
            if permit_id.is_some() {
                return Err(SandboxPlanError::EgressPermitForbidden);
            }
            Ok(None)
        }
        SandboxNetworkRule::PermitRequired => {
            let permit_id = permit_id.ok_or(SandboxPlanError::EgressPermitRequired)?;
            let permit = load_permit(transaction, permit_id)
                .map_err(|_| SandboxPlanError::EgressPermitInvalid)?;
            if permit.status == "exhausted" {
                return Err(SandboxPlanError::EgressPermitExhausted);
            }
            if permit.status != "active"
                || permit.principal_or_process != principal
                || permit.parent_lease_id != lease_id
                || !(permit.action == "network.egress"
                    || permit.action.starts_with("network.egress."))
                || observed_at < permit.issued_at.as_str()
            {
                return Err(SandboxPlanError::EgressPermitInvalid);
            }
            if let Some(expires_at) = permit.expires_at.as_deref()
                && observed_at >= expires_at
            {
                return Err(SandboxPlanError::EgressPermitExpired);
            }
            if permit
                .usage_limit
                .is_some_and(|limit| permit.uses_consumed >= limit)
            {
                return Err(SandboxPlanError::EgressPermitExhausted);
            }
            verify_lease_chain_for_use(
                transaction,
                lease_id,
                lease_generation,
                principal,
                &permit.action,
                &permit.destination_scope,
                observed_at,
            )
            .map_err(|error| SandboxPlanError::LeaseAuthority(error.to_string()))?;
            Ok(Some(SandboxEgressBinding {
                permit_id,
                action: permit.action,
                purpose: permit.purpose,
                destination_scope: permit.destination_scope,
                protocol_port_scope: permit.protocol_port_scope,
                taint_digest: permit.taint_digest,
                secret_handle_id: permit.secret_handle_id,
                expires_at: permit.expires_at,
                usage_limit: permit.usage_limit,
            }))
        }
    }
}

fn plan_hash(
    profile: &SandboxProfileRecord,
    request: &SandboxPlanRequest<'_>,
    authority: &LaunchAuthority,
    egress: Option<&SandboxEgressBinding>,
) -> Result<[u8; 32], SandboxPlanError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(PLAN_DOMAIN)?;
    encoder.push_bytes(&profile.profile_id)?;
    encoder.push_u64(profile.version);
    encoder.push_bytes(profile.class.as_str().as_bytes())?;
    encode_strings(&mut encoder, &profile.filesystem_read_roots)?;
    encode_strings(&mut encoder, &profile.filesystem_write_roots)?;
    encoder.push_bytes(profile.network_rule.as_str().as_bytes())?;
    encode_strings(&mut encoder, &profile.environment_allowlist)?;
    encoder.push_bytes(profile.spawn_rule.as_str().as_bytes())?;
    encode_optional_u64(&mut encoder, profile.cpu_limit);
    encode_optional_u64(&mut encoder, profile.memory_limit);
    encode_optional_u64(&mut encoder, profile.time_limit);
    encode_optional_u64(&mut encoder, profile.output_limit);
    encode_strings(&mut encoder, &profile.device_allowlist)?;
    encode_strings(&mut encoder, &profile.ipc_allowlist)?;
    encode_strings(&mut encoder, &profile.inherited_handle_rules)?;
    encode_strings(&mut encoder, &profile.platform_requirements)?;
    encoder.push_bytes(request.principal_or_process.as_bytes())?;
    encoder.push_bytes(&request.decision_id)?;
    encoder.push_bytes(&request.lease_id)?;
    encoder.push_u64(authority.lease_generation);
    encoder.push_bytes(&authority.policy_bundle_id)?;
    encoder.push_bytes(&authority.policy_bundle_hash)?;
    encoder.push_u8(u8::from(request.strict_local));
    encoder.push_bytes(request.observed_at.as_bytes())?;
    match egress {
        Some(binding) => {
            encoder.push_u8(1);
            encoder.push_bytes(&binding.permit_id)?;
            encoder.push_bytes(binding.action.as_bytes())?;
            encoder.push_bytes(binding.purpose.as_bytes())?;
            encoder.push_bytes(binding.destination_scope.as_bytes())?;
            encoder.push_bytes(binding.protocol_port_scope.as_bytes())?;
            encoder.push_bytes(&binding.taint_digest)?;
            encode_optional_id(&mut encoder, binding.secret_handle_id)?;
            encode_optional_text(&mut encoder, binding.expires_at.as_deref())?;
            encode_optional_u64(&mut encoder, binding.usage_limit);
        }
        None => encoder.push_u8(0),
    }
    Ok(crate::payload_hash(&encoder.finish()))
}

fn encode_strings(
    encoder: &mut CanonicalEncoder,
    values: &[String],
) -> Result<(), SandboxPlanError> {
    encoder.push_u64(
        u64::try_from(values.len()).map_err(|_| SandboxPlanError::DecisionMismatch)?,
    );
    for value in values {
        encoder.push_bytes(value.as_bytes())?;
    }
    Ok(())
}

fn encode_optional_id(
    encoder: &mut CanonicalEncoder,
    value: Option<[u8; 16]>,
) -> Result<(), SandboxPlanError> {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(&value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn encode_optional_text(
    encoder: &mut CanonicalEncoder,
    value: Option<&str>,
) -> Result<(), SandboxPlanError> {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value.as_bytes())?;
        }
        None => encoder.push_u8(0),
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

fn validate_principal(value: &str) -> Result<(), SandboxPlanError> {
    let known_prefix = ["owner:", "client:", "kernel:", "test:", "process:"]
        .iter()
        .any(|prefix| value.starts_with(prefix) && value.len() > prefix.len());
    if !known_prefix
        || value.len() > MAX_PRINCIPAL_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(SandboxPlanError::InvalidPrincipal);
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

fn id16(value: Vec<u8>, error: SandboxPlanError) -> Result<[u8; 16], SandboxPlanError> {
    value.try_into().map_err(|_| error)
}

fn hash32(value: Vec<u8>, error: SandboxPlanError) -> Result<[u8; 32], SandboxPlanError> {
    value.try_into().map_err(|_| error)
}

fn positive_u64(value: i64, error: SandboxPlanError) -> Result<u64, SandboxPlanError> {
    let value = nonnegative_u64(value, error)?;
    if value == 0 {
        return Err(SandboxPlanError::DecisionMismatch);
    }
    Ok(value)
}

fn nonnegative_u64(value: i64, error: SandboxPlanError) -> Result<u64, SandboxPlanError> {
    u64::try_from(value).map_err(|_| error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_security_write::{
        append_active_policy_snapshot, append_authorization_decision_v2_snapshot,
        append_capability_lease_snapshot, append_egress_permit_snapshot, append_policy_bundle_snapshot,
        append_sandbox_profile_snapshot,
    };
    use crate::security_audit::{AuthorizationAuditInput, append_authorization_decision};
    use golam_core::paths::RuntimeLayout;
    use rusqlite::TransactionBehavior;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);
    const PRINCIPAL: &str = "owner:owner";
    const PROFILE_ID: [u8; 16] = [7; 16];
    const POLICY_ID: [u8; 16] = [31; 16];
    const POLICY_HASH: [u8; 32] = [32; 32];
    const LEASE_ID: [u8; 16] = [41; 16];
    const LEASE_DIGEST: [u8; 32] = [42; 32];
    const DECISION_ID: [u8; 16] = [51; 16];
    const PERMIT_ID: [u8; 16] = [61; 16];
    const OBSERVED_AT: &str = "2026-08-29T05:00:00Z";

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-sandbox-plan-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
    }

    fn encoded_empty_list() -> Vec<u8> {
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(b"golam:sandbox-profile-list:v1").unwrap();
        encoder.push_u64(0);
        encoder.finish()
    }

    fn install_base_authority(authority: &AuthorityLayout) {
        let mut connection = Connection::open(authority.authority_db_path()).unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let resource = sandbox_profile_resource(PROFILE_ID, 1);
        let empty = encoded_empty_list();
        transaction
            .execute(
                "INSERT INTO sandbox_profiles \
                 (profile_id, version, class, filesystem_read_roots, filesystem_write_roots, network_rule, environment_allowlist, spawn_rule, cpu_limit, memory_limit, time_limit, output_limit, device_allowlist, ipc_allowlist, inherited_handle_rules, platform_requirements, status) \
                 VALUES (?1, 1, 'native_untrusted_subprocess', ?2, ?2, 'deny_all', ?2, 'deny', 500, 268435456, 30000, 1048576, ?2, ?2, ?2, ?2, 'active')",
                params![&PROFILE_ID[..], &empty],
            )
            .unwrap();
        append_sandbox_profile_snapshot(&transaction, &PROFILE_ID, 1).unwrap();
        transaction
            .execute(
                "INSERT INTO policy_bundles \
                 (policy_bundle_id, version, schema_version, canonical_policy_bytes, bundle_hash, created_by, created_global_seq, validation_status) \
                 VALUES (?1, 1, 1, X'01', ?2, ?3, 1, 'validated')",
                params![&POLICY_ID[..], &POLICY_HASH[..], PRINCIPAL],
            )
            .unwrap();
        append_policy_bundle_snapshot(&transaction, &POLICY_ID).unwrap();
        transaction
            .execute(
                "INSERT INTO active_policy \
                 (singleton_id, policy_bundle_id, bundle_hash, activated_by, activation_effect_id, activated_global_seq) \
                 VALUES (1, ?1, ?2, ?3, ?4, 1)",
                params![&POLICY_ID[..], &POLICY_HASH[..], PRINCIPAL, &[33_u8; 16][..]],
            )
            .unwrap();
        append_active_policy_snapshot(&transaction).unwrap();
        let actions = b"network.egress.connect\nsandbox.launch";
        let resources = format!("https://example.invalid\n{resource}");
        transaction
            .execute(
                "INSERT INTO capability_leases \
                 (lease_id, principal_id, parent_lease_id, actions_scope, resources_scope, context_constraints, issued_by, issued_global_seq, not_before, expires_at, generation, status, authority_digest) \
                 VALUES (?1, ?2, NULL, ?3, ?4, X'', ?2, 1, NULL, '2026-08-30T00:00:00Z', 1, 'active', ?5)",
                params![&LEASE_ID[..], PRINCIPAL, &actions[..], resources.as_bytes(), &LEASE_DIGEST[..]],
            )
            .unwrap();
        append_capability_lease_snapshot(&transaction, &LEASE_ID).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
        install_launch_decision(authority, DECISION_ID);
    }

    fn install_launch_decision(authority: &AuthorityLayout, decision_id: [u8; 16]) {
        let mut connection = Connection::open(authority.authority_db_path()).unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let latest: i64 = transaction
            .query_row(
                "SELECT COALESCE(MAX(global_seq), 0) FROM (\
                   SELECT global_seq FROM session_events \
                   UNION ALL SELECT global_seq FROM effect_transitions \
                   UNION ALL SELECT global_seq FROM authorization_decisions\
                 )",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let global_seq = u64::try_from(latest).unwrap() + 1;
        let resource = sandbox_profile_resource(PROFILE_ID, 1);
        transaction
            .execute(
                "INSERT INTO authorization_decisions \
                 (decision_id, principal, action, resource, context_hash, decision, reason_code, global_seq, hard_guard_result, lease_id, lease_generation, policy_bundle_id, policy_bundle_hash, matched_rule_ids, approval_id, authority_evidence_version) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 'allow', 'sandbox_test_allow', ?6, 'pass', ?7, 1, ?8, ?9, X'', NULL, 2)",
                params![
                    &decision_id[..],
                    PRINCIPAL,
                    SANDBOX_LAUNCH_ACTION,
                    &resource,
                    &[0_u8; 32][..],
                    i64::try_from(global_seq).unwrap(),
                    &LEASE_ID[..],
                    &POLICY_ID[..],
                    &POLICY_HASH[..]
                ],
            )
            .unwrap();
        append_authorization_decision(
            &transaction,
            AuthorizationAuditInput {
                decision_id: &decision_id,
                principal: PRINCIPAL,
                action: SANDBOX_LAUNCH_ACTION,
                resource: &resource,
                context_hash: &[0_u8; 32],
                decision: "allow",
                reason_code: "sandbox_test_allow",
                global_seq,
            },
        )
        .unwrap();
        append_authorization_decision_v2_snapshot(&transaction, &decision_id).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
    }

    fn set_profile_network_rule(authority: &AuthorityLayout, rule: &str) {
        let mut connection = Connection::open(authority.authority_db_path()).unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "UPDATE sandbox_profiles SET network_rule = ?1 WHERE profile_id = ?2 AND version = 1",
                params![rule, &PROFILE_ID[..]],
            )
            .unwrap();
        append_sandbox_profile_snapshot(&transaction, &PROFILE_ID, 1).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
    }

    fn install_egress_permit(authority: &AuthorityLayout) {
        let mut connection = Connection::open(authority.authority_db_path()).unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "INSERT INTO egress_permits \
                 (permit_id, principal_or_process, action, purpose, destination_scope, protocol_port_scope, taint_digest, secret_handle_id, parent_lease_id, issued_at, expires_at, usage_limit, status, uses_consumed) \
                 VALUES (?1, ?2, 'network.egress.connect', 'sandbox-fixture', 'https://example.invalid', 'https:443', ?3, NULL, ?4, '2026-08-29T00:00:00Z', '2026-08-30T00:00:00Z', 3, 'active', 0)",
                params![&PERMIT_ID[..], PRINCIPAL, &[71_u8; 32][..], &LEASE_ID[..]],
            )
            .unwrap();
        append_egress_permit_snapshot(&transaction, &PERMIT_ID).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
    }

    fn request(decision_id: [u8; 16]) -> SandboxPlanRequest<'static> {
        SandboxPlanRequest {
            profile_id: PROFILE_ID,
            profile_version: 1,
            principal_or_process: PRINCIPAL,
            decision_id,
            lease_id: LEASE_ID,
            egress_permit_id: None,
            observed_at: OBSERVED_AT,
            strict_local: false,
        }
    }

    #[test]
    fn compile_is_deterministic_and_binds_current_profile_lease_and_policy() {
        let (runtime, authority) = authority();
        install_base_authority(&authority);
        let mut compiler = SandboxPlanCompiler::open(&authority).unwrap();
        let first = compiler.compile(request(DECISION_ID)).unwrap();
        let second = compiler.compile(request(DECISION_ID)).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.profile_id, PROFILE_ID);
        assert_eq!(first.lease_id, LEASE_ID);
        assert_eq!(first.policy_bundle_id, POLICY_ID);
        assert_eq!(first.policy_bundle_hash, POLICY_HASH);
        assert_eq!(first.network_rule, SandboxNetworkRule::DenyAll);
        assert!(first.egress.is_none());
        drop(compiler);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn stale_or_mismatched_launch_decision_denies() {
        let (runtime, authority) = authority();
        install_base_authority(&authority);
        install_launch_decision(&authority, [52; 16]);
        let mut compiler = SandboxPlanCompiler::open(&authority).unwrap();
        assert!(matches!(
            compiler.compile(request(DECISION_ID)),
            Err(SandboxPlanError::DecisionStale)
        ));
        let mut wrong = request([52; 16]);
        wrong.lease_id = [99; 16];
        assert!(matches!(
            compiler.compile(wrong),
            Err(SandboxPlanError::DecisionMismatch)
        ));
        drop(compiler);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn egress_authority_is_exact_intersection_and_is_not_consumed_by_compilation() {
        let (runtime, authority) = authority();
        install_base_authority(&authority);
        set_profile_network_rule(&authority, "permit_required");
        install_egress_permit(&authority);
        let mut compiler = SandboxPlanCompiler::open(&authority).unwrap();

        assert!(matches!(
            compiler.compile(request(DECISION_ID)),
            Err(SandboxPlanError::EgressPermitRequired)
        ));
        let mut strict = request(DECISION_ID);
        strict.egress_permit_id = Some(PERMIT_ID);
        strict.strict_local = true;
        assert!(matches!(
            compiler.compile(strict),
            Err(SandboxPlanError::StrictLocalExternalEgressDenied)
        ));
        let mut permitted = request(DECISION_ID);
        permitted.egress_permit_id = Some(PERMIT_ID);
        let plan = compiler.compile(permitted).unwrap();
        assert_eq!(plan.egress.as_ref().unwrap().permit_id, PERMIT_ID);
        assert_eq!(plan.egress.as_ref().unwrap().destination_scope, "https://example.invalid");
        drop(compiler);

        let connection = Connection::open(authority.authority_db_path()).unwrap();
        let uses: i64 = connection
            .query_row(
                "SELECT uses_consumed FROM egress_permits WHERE permit_id = ?1",
                params![&PERMIT_ID[..]],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(uses, 0);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn deny_all_profile_rejects_ambient_egress_binding() {
        let (runtime, authority) = authority();
        install_base_authority(&authority);
        install_egress_permit(&authority);
        let mut compiler = SandboxPlanCompiler::open(&authority).unwrap();
        let mut supplied = request(DECISION_ID);
        supplied.egress_permit_id = Some(PERMIT_ID);
        assert!(matches!(
            compiler.compile(supplied),
            Err(SandboxPlanError::EgressPermitForbidden)
        ));
        drop(compiler);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
