#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use golam_core::taint::{TaintLabel, TaintSet};
use golam_core::{CanonicalEncoder, CoreError, EffectId};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};

use crate::authority_security_write::{
    append_approval_consumption_snapshot, append_taint_attestation_snapshot,
};
use crate::storage::{AuthorityStore, StorageError};
use crate::verifier_registry::TAINT_AUTHORITY_MUTATION_RISK_CLASS;

pub const TAINT_DOWNGRADE_ACTION: &str = "taint.downgrade";
pub const TAINT_SECRET_ELIMINATION_ACTION: &str = "taint.secret_eliminate";

const MAX_SOURCE_ARTIFACTS: usize = 64;
const MAX_PRINCIPAL_BYTES: usize = 512;
const MAX_AUTHORITY_SOURCE_BINDING_BYTES: usize = 4096;
const SOURCE_ARTIFACTS_DOMAIN: &[u8] = b"golam:taint-attestation-sources:v1";
const ATTESTATION_ID_DOMAIN: &[u8] = b"golam:taint-attestation-id:v1";
const ATTESTATION_INTENT_DOMAIN: &[u8] = b"golam:taint-attestation-intent:v1";
const APPROVAL_CONSUMPTION_DOMAIN: &[u8] = b"golam:taint-downgrade-approval-consumption:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaintDowngradeMechanism {
    HumanApproval,
    DeterministicVerifier,
    SecretEliminationSanitizer,
}

impl TaintDowngradeMechanism {
    const fn code(self) -> u8 {
        match self {
            Self::HumanApproval => 1,
            Self::DeterministicVerifier => 2,
            Self::SecretEliminationSanitizer => 3,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::HumanApproval => "human_approval",
            Self::DeterministicVerifier => "deterministic_verifier",
            Self::SecretEliminationSanitizer => "secret_elimination_sanitizer",
        }
    }

    const fn action(self) -> &'static str {
        match self {
            Self::HumanApproval | Self::DeterministicVerifier => TAINT_DOWNGRADE_ACTION,
            Self::SecretEliminationSanitizer => TAINT_SECRET_ELIMINATION_ACTION,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeterministicVerifierEvidence<'a> {
    pub rule_id: [u8; 16],
    pub authority_source_binding: &'a [u8],
    pub evidence_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecretEliminationSanitizerEvidence<'a> {
    pub rule_id: [u8; 16],
    pub authority_source_binding: &'a [u8],
    pub evidence_hash: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedTaintAttestation {
    attestation_id: [u8; 16],
    source_artifact_ids: Vec<[u8; 32]>,
    source_artifact_ids_bytes: Vec<u8>,
    source_labels: TaintSet,
    source_labels_bytes: Vec<u8>,
    result_artifact_id: [u8; 32],
    result_labels: TaintSet,
    result_labels_bytes: Vec<u8>,
    removed_labels: TaintSet,
    mechanism: TaintDowngradeMechanism,
    verifier_rule_id: Option<[u8; 16]>,
    authority_source_binding: Option<Vec<u8>>,
    requested_by_principal: String,
    evidence_hash: [u8; 32],
    source_taint_digest: [u8; 32],
    intent_digest: [u8; 32],
    resource: String,
}

impl PreparedTaintAttestation {
    pub const fn attestation_id(&self) -> [u8; 16] {
        self.attestation_id
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub const fn source_taint_digest(&self) -> [u8; 32] {
        self.source_taint_digest
    }

    pub const fn intent_digest(&self) -> [u8; 32] {
        self.intent_digest
    }

    pub const fn removed_labels(&self) -> TaintSet {
        self.removed_labels
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaintAttestationRecord {
    pub attestation_id: [u8; 16],
    pub source_artifact_ids: Vec<[u8; 32]>,
    pub source_labels: TaintSet,
    pub result_artifact_id: [u8; 32],
    pub result_labels: TaintSet,
    pub mechanism: TaintDowngradeMechanism,
    /// For deterministic mechanisms this is the registered rule ID. For human
    /// approval the fixed schema field carries the exact consumed approval ID
    /// so the authority reference remains directly auditable.
    pub rule_id: [u8; 16],
    pub principal: Option<String>,
    pub evidence_hash: [u8; 32],
    pub created_global_seq: u64,
}

#[derive(Debug)]
pub enum TaintAttestationError {
    Storage(StorageError),
    Sqlite(rusqlite::Error),
    Core(CoreError),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidPrincipal,
    EmptySourceArtifacts,
    TooManySourceArtifacts,
    DuplicateSourceArtifact,
    ResultArtifactMustBeNew,
    EmptySourceLabels,
    ResultLabelsNotSubset,
    NoDowngrade,
    SecretDerivedRequiresSanitizer,
    SanitizerSourceMustBeSecretDerived,
    SanitizerResultStillSecretDerived,
    MissingEvidence,
    InvalidAuthoritySourceBinding,
    WrongMechanism,
    IntegerOverflow,
    MissingAuthorityDecision,
    AuthorityDecisionMismatch,
    StaleAuthorityDecision,
    EffectNotFound,
    EffectMismatch,
    ApprovalNotFound,
    ApprovalMismatch,
    ApprovalAlreadyUsed,
    VerifierRuleNotFound,
    VerifierRuleInactive,
    VerifierRuleKindMismatch,
    VerifierSourceBindingMismatch,
    VerifierRuleDoesNotAuthorizeDowngrade,
    DuplicateAttestation,
    InvalidStoredRecord(&'static str),
}

impl fmt::Display for TaintAttestationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "taint attestation authority-store error: {error}"),
            Self::Sqlite(error) => write!(f, "taint attestation sqlite error: {error}"),
            Self::Core(error) => write!(f, "taint attestation canonical encoding error: {error}"),
            Self::Integrity(error) => write!(f, "taint attestation integrity error: {error}"),
            Self::AuthoritySecurity(error) => {
                write!(f, "taint attestation authority-security error: {error}")
            }
            Self::InvalidPrincipal => f.write_str("taint attestation principal is not canonical"),
            Self::EmptySourceArtifacts => f.write_str("taint attestation requires a source artifact"),
            Self::TooManySourceArtifacts => {
                f.write_str("taint attestation source artifact set is too large")
            }
            Self::DuplicateSourceArtifact => {
                f.write_str("taint attestation source artifacts contain a duplicate")
            }
            Self::ResultArtifactMustBeNew => {
                f.write_str("taint downgrade must produce a distinct result artifact")
            }
            Self::EmptySourceLabels => f.write_str("taint downgrade source labels cannot be empty"),
            Self::ResultLabelsNotSubset => {
                f.write_str("taint downgrade result labels must be a subset of source labels")
            }
            Self::NoDowngrade => f.write_str("taint attestation does not remove any source label"),
            Self::SecretDerivedRequiresSanitizer => f.write_str(
                "SECRET_DERIVED can only be removed by the separately authorized secret-elimination sanitizer path",
            ),
            Self::SanitizerSourceMustBeSecretDerived => f.write_str(
                "secret-elimination sanitizer source must carry SECRET_DERIVED provenance",
            ),
            Self::SanitizerResultStillSecretDerived => f.write_str(
                "secret-elimination sanitizer result must not carry SECRET_DERIVED provenance",
            ),
            Self::MissingEvidence => f.write_str("taint downgrade requires non-empty evidence"),
            Self::InvalidAuthoritySourceBinding => {
                f.write_str("deterministic authority-source binding is invalid or too large")
            }
            Self::WrongMechanism => {
                f.write_str("taint attestation commit method does not match prepared mechanism")
            }
            Self::IntegerOverflow => f.write_str("taint attestation integer conversion overflow"),
            Self::MissingAuthorityDecision => {
                f.write_str("taint downgrade has no durable authorization decision")
            }
            Self::AuthorityDecisionMismatch => {
                f.write_str("taint downgrade decision does not match exact principal/action/resource")
            }
            Self::StaleAuthorityDecision => {
                f.write_str("taint downgrade authorization decision is stale")
            }
            Self::EffectNotFound => f.write_str("taint downgrade effect does not exist"),
            Self::EffectMismatch => f.write_str(
                "taint downgrade effect is not exact authorized at-most-once protected work",
            ),
            Self::ApprovalNotFound => f.write_str("human taint downgrade approval does not exist"),
            Self::ApprovalMismatch => f.write_str(
                "human taint downgrade approval does not match exact effect/risk/provenance",
            ),
            Self::ApprovalAlreadyUsed => {
                f.write_str("human taint downgrade one-shot approval was already consumed")
            }
            Self::VerifierRuleNotFound => f.write_str("registered taint authority rule does not exist"),
            Self::VerifierRuleInactive => f.write_str("registered taint authority rule is not active"),
            Self::VerifierRuleKindMismatch => {
                f.write_str("registered taint authority rule has the wrong mechanism kind")
            }
            Self::VerifierSourceBindingMismatch => f.write_str(
                "verification evidence is not bound to the registered authoritative source",
            ),
            Self::VerifierRuleDoesNotAuthorizeDowngrade => f.write_str(
                "registered taint authority rule does not authorize every requested label removal",
            ),
            Self::DuplicateAttestation => f.write_str("taint attestation already exists"),
            Self::InvalidStoredRecord(reason) => {
                write!(f, "stored taint attestation state is invalid: {reason}")
            }
        }
    }
}

impl Error for TaintAttestationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StorageError> for TaintAttestationError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<rusqlite::Error> for TaintAttestationError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<CoreError> for TaintAttestationError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

#[derive(Clone, Copy)]
enum SecretDerivedRemovalPolicy {
    Forbid,
    RequireElimination,
}

pub fn prepare_human_downgrade(
    source_artifact_ids: impl IntoIterator<Item = [u8; 32]>,
    source_labels: TaintSet,
    result_artifact_id: [u8; 32],
    result_labels: TaintSet,
    requested_by_principal: &str,
    evidence_hash: [u8; 32],
) -> Result<PreparedTaintAttestation, TaintAttestationError> {
    prepare(
        source_artifact_ids,
        source_labels,
        result_artifact_id,
        result_labels,
        requested_by_principal,
        TaintDowngradeMechanism::HumanApproval,
        SecretDerivedRemovalPolicy::Forbid,
        None,
        None,
        evidence_hash,
    )
}

pub fn prepare_deterministic_verifier_downgrade(
    source_artifact_ids: impl IntoIterator<Item = [u8; 32]>,
    source_labels: TaintSet,
    result_artifact_id: [u8; 32],
    result_labels: TaintSet,
    requested_by_principal: &str,
    verifier_evidence: DeterministicVerifierEvidence<'_>,
) -> Result<PreparedTaintAttestation, TaintAttestationError> {
    validate_authority_source_binding(verifier_evidence.authority_source_binding)?;
    prepare(
        source_artifact_ids,
        source_labels,
        result_artifact_id,
        result_labels,
        requested_by_principal,
        TaintDowngradeMechanism::DeterministicVerifier,
        SecretDerivedRemovalPolicy::Forbid,
        Some(verifier_evidence.rule_id),
        Some(verifier_evidence.authority_source_binding.to_vec()),
        verifier_evidence.evidence_hash,
    )
}

pub fn prepare_secret_elimination_sanitizer(
    source_artifact_ids: impl IntoIterator<Item = [u8; 32]>,
    source_labels: TaintSet,
    result_artifact_id: [u8; 32],
    result_labels: TaintSet,
    requested_by_principal: &str,
    sanitizer_evidence: SecretEliminationSanitizerEvidence<'_>,
) -> Result<PreparedTaintAttestation, TaintAttestationError> {
    validate_authority_source_binding(sanitizer_evidence.authority_source_binding)?;
    prepare(
        source_artifact_ids,
        source_labels,
        result_artifact_id,
        result_labels,
        requested_by_principal,
        TaintDowngradeMechanism::SecretEliminationSanitizer,
        SecretDerivedRemovalPolicy::RequireElimination,
        Some(sanitizer_evidence.rule_id),
        Some(sanitizer_evidence.authority_source_binding.to_vec()),
        sanitizer_evidence.evidence_hash,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare(
    source_artifact_ids: impl IntoIterator<Item = [u8; 32]>,
    source_labels: TaintSet,
    result_artifact_id: [u8; 32],
    result_labels: TaintSet,
    requested_by_principal: &str,
    mechanism: TaintDowngradeMechanism,
    secret_derived_policy: SecretDerivedRemovalPolicy,
    verifier_rule_id: Option<[u8; 16]>,
    authority_source_binding: Option<Vec<u8>>,
    evidence_hash: [u8; 32],
) -> Result<PreparedTaintAttestation, TaintAttestationError> {
    validate_principal(requested_by_principal)?;
    if source_labels.is_empty() {
        return Err(TaintAttestationError::EmptySourceLabels);
    }
    if !source_labels.contains_all(result_labels) {
        return Err(TaintAttestationError::ResultLabelsNotSubset);
    }
    let removed_labels = removed_labels(source_labels, result_labels);
    if removed_labels.is_empty() {
        return Err(TaintAttestationError::NoDowngrade);
    }
    match secret_derived_policy {
        SecretDerivedRemovalPolicy::Forbid => {
            if removed_labels.contains(TaintLabel::SecretDerived) {
                return Err(TaintAttestationError::SecretDerivedRequiresSanitizer);
            }
        }
        SecretDerivedRemovalPolicy::RequireElimination => {
            if !source_labels.contains(TaintLabel::SecretDerived) {
                return Err(TaintAttestationError::SanitizerSourceMustBeSecretDerived);
            }
            if result_labels.contains(TaintLabel::SecretDerived) {
                return Err(TaintAttestationError::SanitizerResultStillSecretDerived);
            }
        }
    }
    if evidence_hash == [0; 32] {
        return Err(TaintAttestationError::MissingEvidence);
    }

    let mut source_artifact_ids = source_artifact_ids.into_iter().collect::<Vec<_>>();
    if source_artifact_ids.is_empty() {
        return Err(TaintAttestationError::EmptySourceArtifacts);
    }
    if source_artifact_ids.len() > MAX_SOURCE_ARTIFACTS {
        return Err(TaintAttestationError::TooManySourceArtifacts);
    }
    source_artifact_ids.sort_unstable();
    if source_artifact_ids
        .windows(2)
        .any(|pair| pair[0] == pair[1])
    {
        return Err(TaintAttestationError::DuplicateSourceArtifact);
    }
    if source_artifact_ids.contains(&result_artifact_id) {
        return Err(TaintAttestationError::ResultArtifactMustBeNew);
    }

    let source_artifact_ids_bytes = encode_source_artifact_ids(&source_artifact_ids)?;
    let source_labels_bytes = source_labels.canonical_bytes()?;
    let result_labels_bytes = result_labels.canonical_bytes()?;
    let source_taint_digest = *blake3::hash(&source_labels_bytes).as_bytes();

    let mut identity = CanonicalEncoder::new();
    identity.push_bytes(ATTESTATION_ID_DOMAIN)?;
    identity.push_bytes(&source_artifact_ids_bytes)?;
    identity.push_bytes(&source_labels_bytes)?;
    identity.push_bytes(&result_artifact_id)?;
    identity.push_bytes(&result_labels_bytes)?;
    identity.push_u8(mechanism.code());
    encode_optional_rule(&mut identity, verifier_rule_id)?;
    encode_optional_bytes(&mut identity, authority_source_binding.as_deref())?;
    identity.push_bytes(requested_by_principal.as_bytes())?;
    identity.push_bytes(&evidence_hash)?;
    let identity_hash = *blake3::hash(&identity.finish()).as_bytes();
    let mut attestation_id = [0_u8; 16];
    attestation_id.copy_from_slice(&identity_hash[..16]);
    let resource = taint_attestation_resource(attestation_id);

    let mut intent = CanonicalEncoder::new();
    intent.push_bytes(ATTESTATION_INTENT_DOMAIN)?;
    intent.push_bytes(&attestation_id)?;
    intent.push_bytes(requested_by_principal.as_bytes())?;
    intent.push_bytes(&source_taint_digest)?;
    let intent_digest = *blake3::hash(&intent.finish()).as_bytes();

    Ok(PreparedTaintAttestation {
        attestation_id,
        source_artifact_ids,
        source_artifact_ids_bytes,
        source_labels,
        source_labels_bytes,
        result_artifact_id,
        result_labels,
        result_labels_bytes,
        removed_labels,
        mechanism,
        verifier_rule_id,
        authority_source_binding,
        requested_by_principal: requested_by_principal.to_owned(),
        evidence_hash,
        source_taint_digest,
        intent_digest,
        resource,
    })
}

pub fn taint_attestation_resource(attestation_id: [u8; 16]) -> String {
    format!("taint-attestation:{}", hex_bytes(&attestation_id))
}

pub struct TaintAttestationStore {
    connection: Connection,
}

impl TaintAttestationStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, TaintAttestationError> {
        let store = AuthorityStore::open(layout.authority_db_path())?;
        drop(store);
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        Ok(Self { connection })
    }

    pub fn attest_human(
        &mut self,
        prepared: PreparedTaintAttestation,
        authority_decision_id: [u8; 16],
        approval_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<TaintAttestationRecord, TaintAttestationError> {
        if prepared.mechanism != TaintDowngradeMechanism::HumanApproval {
            return Err(TaintAttestationError::WrongMechanism);
        }
        self.commit(
            prepared,
            authority_decision_id,
            effect_id,
            MechanismAuthority::HumanApproval(approval_id),
        )
    }

    pub fn attest_deterministic_verifier(
        &mut self,
        prepared: PreparedTaintAttestation,
        authority_decision_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<TaintAttestationRecord, TaintAttestationError> {
        if prepared.mechanism != TaintDowngradeMechanism::DeterministicVerifier {
            return Err(TaintAttestationError::WrongMechanism);
        }
        let rule_id = prepared
            .verifier_rule_id
            .ok_or(TaintAttestationError::WrongMechanism)?;
        self.commit(
            prepared,
            authority_decision_id,
            effect_id,
            MechanismAuthority::DeterministicVerifier(rule_id),
        )
    }

    pub fn attest_secret_elimination_sanitizer(
        &mut self,
        prepared: PreparedTaintAttestation,
        authority_decision_id: [u8; 16],
        effect_id: EffectId,
    ) -> Result<TaintAttestationRecord, TaintAttestationError> {
        if prepared.mechanism != TaintDowngradeMechanism::SecretEliminationSanitizer {
            return Err(TaintAttestationError::WrongMechanism);
        }
        let rule_id = prepared
            .verifier_rule_id
            .ok_or(TaintAttestationError::WrongMechanism)?;
        self.commit(
            prepared,
            authority_decision_id,
            effect_id,
            MechanismAuthority::SecretEliminationSanitizer(rule_id),
        )
    }

    fn commit(
        &mut self,
        prepared: PreparedTaintAttestation,
        authority_decision_id: [u8; 16],
        effect_id: EffectId,
        mechanism_authority: MechanismAuthority,
    ) -> Result<TaintAttestationRecord, TaintAttestationError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        verify_transaction_integrity(&transaction)?;
        let expected_action = prepared.mechanism.action();
        let authority = verify_current_authority(
            &transaction,
            authority_decision_id,
            &prepared.requested_by_principal,
            expected_action,
            &prepared.resource,
        )?;
        verify_taint_effect(
            &transaction,
            effect_id,
            expected_action,
            &prepared.resource,
            prepared.intent_digest,
        )?;

        let (rule_id, principal, approval_to_consume) = match mechanism_authority {
            MechanismAuthority::HumanApproval(approval_id) => {
                verify_once_human_approval(
                    &transaction,
                    approval_id,
                    effect_id,
                    &prepared.resource,
                    &prepared.requested_by_principal,
                    prepared.source_taint_digest,
                )?;
                (
                    approval_id,
                    Some(authority.principal.clone()),
                    Some(approval_id),
                )
            }
            MechanismAuthority::DeterministicVerifier(rule_id) => {
                let expected_binding = prepared
                    .authority_source_binding
                    .as_deref()
                    .ok_or(TaintAttestationError::WrongMechanism)?;
                verify_active_registered_rule(
                    &transaction,
                    rule_id,
                    "deterministic_verifier",
                    expected_binding,
                    prepared.removed_labels,
                )?;
                (rule_id, None, None)
            }
            MechanismAuthority::SecretEliminationSanitizer(rule_id) => {
                let expected_binding = prepared
                    .authority_source_binding
                    .as_deref()
                    .ok_or(TaintAttestationError::WrongMechanism)?;
                verify_active_registered_rule(
                    &transaction,
                    rule_id,
                    "secret_elimination_sanitizer",
                    expected_binding,
                    prepared.removed_labels,
                )?;
                (rule_id, None, None)
            }
        };

        let duplicate = transaction
            .query_row(
                "SELECT 1 FROM taint_attestations WHERE attestation_id = ?1 LIMIT 1",
                params![&prepared.attestation_id[..]],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some();
        if duplicate {
            return Err(TaintAttestationError::DuplicateAttestation);
        }

        transaction.execute(
            "INSERT INTO taint_attestations (attestation_id, source_artifact_ids, source_labels, result_artifact_id, result_labels, mechanism, rule_id, principal, evidence_hash, created_global_seq) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                &prepared.attestation_id[..],
                &prepared.source_artifact_ids_bytes,
                &prepared.source_labels_bytes,
                &prepared.result_artifact_id[..],
                &prepared.result_labels_bytes,
                prepared.mechanism.as_str(),
                &rule_id[..],
                principal.as_deref(),
                &prepared.evidence_hash[..],
                to_i64(authority.global_seq)?,
            ],
        )?;
        append_taint_attestation_snapshot(&transaction, &prepared.attestation_id)
            .map_err(|error| TaintAttestationError::AuthoritySecurity(error.to_string()))?;
        if let Some(approval_id) = approval_to_consume {
            consume_once_approval(&transaction, approval_id, effect_id, authority.global_seq)?;
        }
        crate::authority_security_v2::verify(&transaction)
            .map_err(|error| TaintAttestationError::AuthoritySecurity(error.to_string()))?;
        transaction.commit()?;

        Ok(TaintAttestationRecord {
            attestation_id: prepared.attestation_id,
            source_artifact_ids: prepared.source_artifact_ids,
            source_labels: prepared.source_labels,
            result_artifact_id: prepared.result_artifact_id,
            result_labels: prepared.result_labels,
            mechanism: prepared.mechanism,
            rule_id,
            principal,
            evidence_hash: prepared.evidence_hash,
            created_global_seq: authority.global_seq,
        })
    }
}

#[derive(Clone, Copy)]
enum MechanismAuthority {
    HumanApproval([u8; 16]),
    DeterministicVerifier([u8; 16]),
    SecretEliminationSanitizer([u8; 16]),
}

struct AuthorityEvidence {
    principal: String,
    global_seq: u64,
}

fn verify_transaction_integrity(
    transaction: &Transaction<'_>,
) -> Result<(), TaintAttestationError> {
    crate::integrity::verify(transaction)
        .map_err(|error| TaintAttestationError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(transaction)
        .map_err(|error| TaintAttestationError::AuthoritySecurity(error.to_string()))
}

fn verify_current_authority(
    transaction: &Transaction<'_>,
    decision_id: [u8; 16],
    expected_principal: &str,
    expected_action: &str,
    expected_resource: &str,
) -> Result<AuthorityEvidence, TaintAttestationError> {
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
        .ok_or(TaintAttestationError::MissingAuthorityDecision)?;
    if row.0 != expected_principal
        || row.1 != expected_action
        || row.2 != expected_resource
        || row.3 != "allow"
    {
        return Err(TaintAttestationError::AuthorityDecisionMismatch);
    }
    let global_seq = from_i64(row.4, "taint authority decision sequence is negative")?;
    let latest: i64 = transaction.query_row(
        "SELECT COALESCE(MAX(global_seq), 0) FROM (SELECT global_seq FROM session_events UNION ALL SELECT global_seq FROM effect_transitions UNION ALL SELECT global_seq FROM authorization_decisions)",
        [],
        |row| row.get(0),
    )?;
    if global_seq != from_i64(latest, "latest authority sequence is negative")? {
        return Err(TaintAttestationError::StaleAuthorityDecision);
    }
    Ok(AuthorityEvidence {
        principal: row.0,
        global_seq,
    })
}

fn verify_taint_effect(
    transaction: &Transaction<'_>,
    effect_id: EffectId,
    expected_action: &str,
    expected_resource: &str,
    expected_payload_hash: [u8; 32],
) -> Result<(), TaintAttestationError> {
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
        .ok_or(TaintAttestationError::EffectNotFound)?;
    if row.0 != expected_action
        || row.1 != expected_resource
        || row.2 != TAINT_AUTHORITY_MUTATION_RISK_CLASS
        || row.3 != "at_most_once"
        || row.4.as_slice() != expected_payload_hash
        || row.5 != "authorized"
    {
        return Err(TaintAttestationError::EffectMismatch);
    }
    Ok(())
}

fn verify_once_human_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    expected_resource: &str,
    expected_principal: &str,
    expected_taint_digest: [u8; 32],
) -> Result<(), TaintAttestationError> {
    let row = transaction
        .query_row(
            "SELECT class, approver_principal, action_scope, resource_scope, effect_id, session_id, risk_class, taint_digest, expires_at, max_uses, revoked_at FROM approvals WHERE approval_id = ?1",
            params![&approval_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, Option<Vec<u8>>>(4)?,
                    row.get::<_, Option<Vec<u8>>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Vec<u8>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                ))
            },
        )
        .optional()?
        .ok_or(TaintAttestationError::ApprovalNotFound)?;
    if row.0 != "ONCE"
        || row.1 != expected_principal
        || row.2.as_slice() != TAINT_DOWNGRADE_ACTION.as_bytes()
        || row.3.as_slice() != expected_resource.as_bytes()
        || row.4.as_deref() != Some(effect_id.0.to_be_bytes().as_slice())
        || row.5.is_some()
        || row.6 != TAINT_AUTHORITY_MUTATION_RISK_CLASS
        || row.7.as_slice() != expected_taint_digest
        || row.8.is_some()
        || row.9 != Some(1)
        || row.10.is_some()
    {
        return Err(TaintAttestationError::ApprovalMismatch);
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
        return Err(TaintAttestationError::ApprovalAlreadyUsed);
    }
    Ok(())
}

fn verify_active_registered_rule(
    transaction: &Transaction<'_>,
    rule_id: [u8; 16],
    expected_kind: &str,
    expected_authority_source_binding: &[u8],
    removed_labels: TaintSet,
) -> Result<(), TaintAttestationError> {
    let row = transaction
        .query_row(
            "SELECT kind, authority_source_binding, allowed_downgrades, status FROM verifier_rules WHERE rule_id = ?1",
            params![&rule_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or(TaintAttestationError::VerifierRuleNotFound)?;
    if row.3 != "active" {
        return Err(TaintAttestationError::VerifierRuleInactive);
    }
    if row.0 != expected_kind {
        return Err(TaintAttestationError::VerifierRuleKindMismatch);
    }
    if row.1.as_slice() != expected_authority_source_binding {
        return Err(TaintAttestationError::VerifierSourceBindingMismatch);
    }
    let allowed = TaintSet::from_canonical_bytes(&row.2)?;
    if !allowed.contains_all(removed_labels) {
        return Err(TaintAttestationError::VerifierRuleDoesNotAuthorizeDowngrade);
    }
    Ok(())
}

fn consume_once_approval(
    transaction: &Transaction<'_>,
    approval_id: [u8; 16],
    effect_id: EffectId,
    global_seq: u64,
) -> Result<(), TaintAttestationError> {
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
        .map_err(|error| TaintAttestationError::AuthoritySecurity(error.to_string()))
}

fn removed_labels(source: TaintSet, result: TaintSet) -> TaintSet {
    TaintSet::from_labels(source.labels().filter(|label| !result.contains(*label)))
}

fn encode_source_artifact_ids(ids: &[[u8; 32]]) -> Result<Vec<u8>, CoreError> {
    let mut encoder = CanonicalEncoder::new();
    encoder.push_bytes(SOURCE_ARTIFACTS_DOMAIN)?;
    encoder.push_u64(u64::try_from(ids.len()).map_err(|_| CoreError::CanonicalLengthOverflow)?);
    for id in ids {
        encoder.push_bytes(id)?;
    }
    Ok(encoder.finish())
}

fn encode_optional_rule(
    encoder: &mut CanonicalEncoder,
    rule_id: Option<[u8; 16]>,
) -> Result<(), CoreError> {
    match rule_id {
        Some(rule_id) => {
            encoder.push_u8(1);
            encoder.push_bytes(&rule_id)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn encode_optional_bytes(
    encoder: &mut CanonicalEncoder,
    value: Option<&[u8]>,
) -> Result<(), CoreError> {
    match value {
        Some(value) => {
            encoder.push_u8(1);
            encoder.push_bytes(value)?;
        }
        None => encoder.push_u8(0),
    }
    Ok(())
}

fn validate_principal(value: &str) -> Result<(), TaintAttestationError> {
    if value.is_empty()
        || value.len() > MAX_PRINCIPAL_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(TaintAttestationError::InvalidPrincipal);
    }
    Ok(())
}

fn validate_authority_source_binding(value: &[u8]) -> Result<(), TaintAttestationError> {
    if value.is_empty() || value.len() > MAX_AUTHORITY_SOURCE_BINDING_BYTES {
        return Err(TaintAttestationError::InvalidAuthoritySourceBinding);
    }
    Ok(())
}

fn from_i64(value: i64, reason: &'static str) -> Result<u64, TaintAttestationError> {
    u64::try_from(value).map_err(|_| TaintAttestationError::InvalidStoredRecord(reason))
}

fn to_i64(value: u64) -> Result<i64, TaintAttestationError> {
    i64::try_from(value).map_err(|_| TaintAttestationError::IntegerOverflow)
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
    use crate::authority_security_write::append_verifier_rule_snapshot;
    use crate::authorization::{
        AppendAuthorizationDecision, AuthorizationAuditLog, AuthorizationDecisionEvidence,
        AuthorizationDecisionKind,
    };
    use crate::dispatch::encode_effect_dependencies;
    use crate::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_core::paths::RuntimeLayout;
    use golam_core::taint::{
        CanonicalMemoryAdmissionError, validate_canonical_long_term_memory_admission,
    };
    use golam_core::{EffectTransitionId, EventId, SessionId};
    use rusqlite::TransactionBehavior;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);
    static RECORD_N: AtomicU64 = AtomicU64::new(0);

    fn next_id() -> u128 {
        4_000_000 + u128::from(RECORD_N.fetch_add(1, Ordering::Relaxed))
    }

    fn authority() -> (RuntimeLayout, AuthorityLayout) {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let runtime = RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-taint-attestation-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        (runtime, authority)
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

    fn issue_human_approval(
        authority: &AuthorityLayout,
        prepared: &PreparedTaintAttestation,
        effect_id: EffectId,
    ) -> [u8; 16] {
        let approval = prepare_approval(
            "owner:owner",
            ApprovalScope::once(effect_id, TAINT_DOWNGRADE_ACTION, prepared.resource()).unwrap(),
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            prepared.source_taint_digest(),
            "2026-08-28T00:00:00Z",
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
            "test_taint_downgrade_approval_issue",
        );
        let decision = append_allow(
            authority,
            APPROVAL_ISSUE_ACTION,
            approval.resource(),
            "test_taint_downgrade_approval_authority",
        );
        ApprovalStore::open(authority)
            .unwrap()
            .issue(approval, decision, approval_effect_id)
            .unwrap()
            .approval_id
    }

    fn install_active_verifier_rule(
        authority: &AuthorityLayout,
        rule_id: [u8; 16],
        kind: &str,
        binding: &[u8],
        allowed: TaintSet,
    ) {
        let store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        drop(store);
        let mut connection = Connection::open(authority.authority_db_path()).unwrap();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute(
                "INSERT INTO verifier_rules (rule_id, kind, version, authority_source_binding, allowed_downgrades, registered_by, status, created_global_seq) VALUES (?1, ?2, 1, ?3, ?4, 'owner:owner', 'active', 1)",
                params![
                    &rule_id[..],
                    kind,
                    binding,
                    allowed.canonical_bytes().unwrap()
                ],
            )
            .unwrap();
        append_verifier_rule_snapshot(&transaction, &rule_id).unwrap();
        crate::authority_security_v2::verify(&transaction).unwrap();
        transaction.commit().unwrap();
    }

    #[test]
    fn normal_downgrade_is_strictly_derived_and_cannot_clear_secret_derived() {
        let source = TaintSet::from_labels([
            TaintLabel::WebUntrusted,
            TaintLabel::ModelGenerated,
            TaintLabel::SecretDerived,
        ]);
        let source_id = [1; 32];

        assert!(matches!(
            prepare_human_downgrade(
                [source_id],
                source,
                source_id,
                TaintSet::from_labels([TaintLabel::ModelGenerated, TaintLabel::SecretDerived]),
                "owner:owner",
                [7; 32],
            ),
            Err(TaintAttestationError::ResultArtifactMustBeNew)
        ));

        assert!(matches!(
            prepare_human_downgrade(
                [source_id],
                source,
                [2; 32],
                TaintSet::from_labels([TaintLabel::ModelGenerated]),
                "owner:owner",
                [7; 32],
            ),
            Err(TaintAttestationError::SecretDerivedRequiresSanitizer)
        ));

        assert!(matches!(
            prepare_human_downgrade(
                [source_id],
                TaintSet::from_labels([TaintLabel::WebUntrusted]),
                [3; 32],
                TaintSet::from_labels([TaintLabel::WebUntrusted, TaintLabel::LocalTrusted]),
                "owner:owner",
                [7; 32],
            ),
            Err(TaintAttestationError::ResultLabelsNotSubset)
        ));
    }

    #[test]
    fn human_downgrade_creates_new_attestation_and_consumes_exact_approval() {
        let (runtime, authority) = authority();
        let source_labels =
            TaintSet::from_labels([TaintLabel::WebUntrusted, TaintLabel::ModelGenerated]);
        let result_labels = TaintSet::from_labels([TaintLabel::ModelGenerated]);
        let prepared = prepare_human_downgrade(
            [[1; 32]],
            source_labels,
            [2; 32],
            result_labels,
            "owner:owner",
            [9; 32],
        )
        .unwrap();
        let effect_id = EffectId(next_id());
        let approval_id = issue_human_approval(&authority, &prepared, effect_id);
        create_authorized_effect(
            &authority,
            effect_id,
            TAINT_DOWNGRADE_ACTION,
            prepared.resource(),
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            prepared.intent_digest(),
            "owner:owner",
            "test_human_taint_downgrade",
        );
        let decision = append_allow(
            &authority,
            TAINT_DOWNGRADE_ACTION,
            prepared.resource(),
            "test_human_taint_downgrade_authority",
        );

        let record = TaintAttestationStore::open(&authority)
            .unwrap()
            .attest_human(prepared, decision, approval_id, effect_id)
            .unwrap();
        assert_eq!(record.mechanism, TaintDowngradeMechanism::HumanApproval);
        assert_eq!(record.rule_id, approval_id);
        assert_eq!(record.principal.as_deref(), Some("owner:owner"));
        assert!(record.source_labels.contains(TaintLabel::WebUntrusted));
        assert!(!record.result_labels.contains(TaintLabel::WebUntrusted));
        assert!(record.result_labels.contains(TaintLabel::ModelGenerated));
        AuthorityStore::open(authority.authority_db_path()).unwrap();

        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn deterministic_downgrade_requires_active_rule_binding_and_allowed_removal() {
        let (runtime, authority) = authority();
        let rule_id = [5; 16];
        let binding = b"authoritative-source:v1";
        install_active_verifier_rule(
            &authority,
            rule_id,
            "deterministic_verifier",
            binding,
            TaintSet::from_labels([TaintLabel::WebUntrusted]),
        );

        let source_labels =
            TaintSet::from_labels([TaintLabel::WebUntrusted, TaintLabel::ModelGenerated]);
        let prepared = prepare_deterministic_verifier_downgrade(
            [[10; 32]],
            source_labels,
            [11; 32],
            TaintSet::from_labels([TaintLabel::ModelGenerated]),
            "owner:owner",
            DeterministicVerifierEvidence {
                rule_id,
                authority_source_binding: binding,
                evidence_hash: [12; 32],
            },
        )
        .unwrap();
        let effect_id = EffectId(next_id());
        create_authorized_effect(
            &authority,
            effect_id,
            TAINT_DOWNGRADE_ACTION,
            prepared.resource(),
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            prepared.intent_digest(),
            "owner:owner",
            "test_deterministic_taint_downgrade",
        );
        let decision = append_allow(
            &authority,
            TAINT_DOWNGRADE_ACTION,
            prepared.resource(),
            "test_deterministic_taint_downgrade_authority",
        );
        let record = TaintAttestationStore::open(&authority)
            .unwrap()
            .attest_deterministic_verifier(prepared, decision, effect_id)
            .unwrap();
        assert_eq!(record.rule_id, rule_id);
        assert_eq!(record.principal, None);
        assert!(record.source_labels.contains(TaintLabel::WebUntrusted));
        assert!(!record.result_labels.contains(TaintLabel::WebUntrusted));

        let overreach = prepare_deterministic_verifier_downgrade(
            [[20; 32]],
            source_labels,
            [21; 32],
            TaintSet::empty(),
            "owner:owner",
            DeterministicVerifierEvidence {
                rule_id,
                authority_source_binding: binding,
                evidence_hash: [22; 32],
            },
        )
        .unwrap();
        let overreach_effect = EffectId(next_id());
        create_authorized_effect(
            &authority,
            overreach_effect,
            TAINT_DOWNGRADE_ACTION,
            overreach.resource(),
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            overreach.intent_digest(),
            "owner:owner",
            "test_deterministic_taint_overreach",
        );
        let overreach_decision = append_allow(
            &authority,
            TAINT_DOWNGRADE_ACTION,
            overreach.resource(),
            "test_deterministic_taint_overreach_authority",
        );
        assert!(matches!(
            TaintAttestationStore::open(&authority)
                .unwrap()
                .attest_deterministic_verifier(overreach, overreach_decision, overreach_effect,),
            Err(TaintAttestationError::VerifierRuleDoesNotAuthorizeDowngrade)
        ));
        AuthorityStore::open(authority.authority_db_path()).unwrap();

        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn sanitizer_preparation_requires_actual_secret_elimination() {
        let evidence = SecretEliminationSanitizerEvidence {
            rule_id: [31; 16],
            authority_source_binding: b"secret-schema:v1",
            evidence_hash: [32; 32],
        };
        assert!(matches!(
            prepare_secret_elimination_sanitizer(
                [[30; 32]],
                TaintSet::from_labels([TaintLabel::WebUntrusted]),
                [31; 32],
                TaintSet::empty(),
                "owner:owner",
                evidence,
            ),
            Err(TaintAttestationError::SanitizerSourceMustBeSecretDerived)
        ));

        assert!(matches!(
            prepare_secret_elimination_sanitizer(
                [[32; 32]],
                TaintSet::from_labels([TaintLabel::SecretDerived, TaintLabel::ModelGenerated,]),
                [33; 32],
                TaintSet::from_labels([TaintLabel::SecretDerived, TaintLabel::ModelGenerated,]),
                "owner:owner",
                evidence,
            ),
            Err(TaintAttestationError::NoDowngrade)
        ));

        assert!(matches!(
            prepare_secret_elimination_sanitizer(
                [[34; 32]],
                TaintSet::from_labels([
                    TaintLabel::SecretDerived,
                    TaintLabel::ModelGenerated,
                    TaintLabel::WebUntrusted,
                ]),
                [35; 32],
                TaintSet::from_labels([TaintLabel::SecretDerived, TaintLabel::ModelGenerated,]),
                "owner:owner",
                evidence,
            ),
            Err(TaintAttestationError::SanitizerResultStillSecretDerived)
        ));
    }

    #[test]
    fn secret_elimination_sanitizer_creates_separate_memory_admissible_evidence() {
        let (runtime, authority) = authority();
        let rule_id = [40; 16];
        let binding = b"secret-schema:v1";
        install_active_verifier_rule(
            &authority,
            rule_id,
            "secret_elimination_sanitizer",
            binding,
            TaintSet::from_labels([TaintLabel::SecretDerived]),
        );

        let source_labels =
            TaintSet::from_labels([TaintLabel::SecretDerived, TaintLabel::ModelGenerated]);
        let result_labels = TaintSet::from_labels([TaintLabel::ModelGenerated]);
        assert_eq!(
            validate_canonical_long_term_memory_admission(source_labels),
            Err(CanonicalMemoryAdmissionError::SecretDerived)
        );
        let prepared = prepare_secret_elimination_sanitizer(
            [[40; 32]],
            source_labels,
            [41; 32],
            result_labels,
            "owner:owner",
            SecretEliminationSanitizerEvidence {
                rule_id,
                authority_source_binding: binding,
                evidence_hash: [42; 32],
            },
        )
        .unwrap();
        let effect_id = EffectId(next_id());
        create_authorized_effect(
            &authority,
            effect_id,
            TAINT_SECRET_ELIMINATION_ACTION,
            prepared.resource(),
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            prepared.intent_digest(),
            "owner:owner",
            "test_secret_elimination_sanitizer",
        );
        let decision = append_allow(
            &authority,
            TAINT_SECRET_ELIMINATION_ACTION,
            prepared.resource(),
            "test_secret_elimination_sanitizer_authority",
        );
        let record = TaintAttestationStore::open(&authority)
            .unwrap()
            .attest_secret_elimination_sanitizer(prepared, decision, effect_id)
            .unwrap();

        assert_eq!(
            record.mechanism,
            TaintDowngradeMechanism::SecretEliminationSanitizer
        );
        assert_eq!(record.rule_id, rule_id);
        assert_eq!(record.principal, None);
        assert!(record.source_labels.contains(TaintLabel::SecretDerived));
        assert!(!record.result_labels.contains(TaintLabel::SecretDerived));
        assert_eq!(
            validate_canonical_long_term_memory_admission(record.result_labels),
            Ok(())
        );
        AuthorityStore::open(authority.authority_db_path()).unwrap();

        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn sanitizer_requires_sanitizer_kind_and_cannot_overremove_labels() {
        let (runtime, authority) = authority();
        let binding = b"secret-schema:v1";
        let wrong_kind_rule = [50; 16];
        install_active_verifier_rule(
            &authority,
            wrong_kind_rule,
            "deterministic_verifier",
            binding,
            TaintSet::from_labels([TaintLabel::SecretDerived]),
        );
        let source_labels =
            TaintSet::from_labels([TaintLabel::SecretDerived, TaintLabel::WebUntrusted]);
        let wrong_kind = prepare_secret_elimination_sanitizer(
            [[50; 32]],
            source_labels,
            [51; 32],
            TaintSet::from_labels([TaintLabel::WebUntrusted]),
            "owner:owner",
            SecretEliminationSanitizerEvidence {
                rule_id: wrong_kind_rule,
                authority_source_binding: binding,
                evidence_hash: [52; 32],
            },
        )
        .unwrap();
        let wrong_kind_effect = EffectId(next_id());
        create_authorized_effect(
            &authority,
            wrong_kind_effect,
            TAINT_SECRET_ELIMINATION_ACTION,
            wrong_kind.resource(),
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            wrong_kind.intent_digest(),
            "owner:owner",
            "test_secret_elimination_wrong_kind",
        );
        let wrong_kind_decision = append_allow(
            &authority,
            TAINT_SECRET_ELIMINATION_ACTION,
            wrong_kind.resource(),
            "test_secret_elimination_wrong_kind_authority",
        );
        assert!(matches!(
            TaintAttestationStore::open(&authority)
                .unwrap()
                .attest_secret_elimination_sanitizer(
                    wrong_kind,
                    wrong_kind_decision,
                    wrong_kind_effect,
                ),
            Err(TaintAttestationError::VerifierRuleKindMismatch)
        ));

        let sanitizer_rule = [53; 16];
        install_active_verifier_rule(
            &authority,
            sanitizer_rule,
            "secret_elimination_sanitizer",
            binding,
            TaintSet::from_labels([TaintLabel::SecretDerived]),
        );
        let overreach = prepare_secret_elimination_sanitizer(
            [[53; 32]],
            source_labels,
            [54; 32],
            TaintSet::empty(),
            "owner:owner",
            SecretEliminationSanitizerEvidence {
                rule_id: sanitizer_rule,
                authority_source_binding: binding,
                evidence_hash: [55; 32],
            },
        )
        .unwrap();
        let overreach_effect = EffectId(next_id());
        create_authorized_effect(
            &authority,
            overreach_effect,
            TAINT_SECRET_ELIMINATION_ACTION,
            overreach.resource(),
            TAINT_AUTHORITY_MUTATION_RISK_CLASS,
            overreach.intent_digest(),
            "owner:owner",
            "test_secret_elimination_overreach",
        );
        let overreach_decision = append_allow(
            &authority,
            TAINT_SECRET_ELIMINATION_ACTION,
            overreach.resource(),
            "test_secret_elimination_overreach_authority",
        );
        assert!(matches!(
            TaintAttestationStore::open(&authority)
                .unwrap()
                .attest_secret_elimination_sanitizer(
                    overreach,
                    overreach_decision,
                    overreach_effect,
                ),
            Err(TaintAttestationError::VerifierRuleDoesNotAuthorizeDowngrade)
        ));
        AuthorityStore::open(authority.authority_db_path()).unwrap();

        fs::remove_dir_all(runtime.root).unwrap();
    }
}
