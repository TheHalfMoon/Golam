#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{CanonicalEncoder, CoreError, EffectId, SessionId};

const MAX_SCOPE_ITEMS: usize = 32;
const MAX_ACTION_BYTES: usize = 128;
const MAX_RESOURCE_BYTES: usize = 2_048;
const MAX_PATTERN_BYTES: usize = 2_048;
const MAX_CANONICAL_SCOPE_BYTES: usize = 131_072;
const APPROVAL_SCOPE_DOMAIN: &[u8] = b"golam:approval-scope:v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalClass {
    Once,
    SessionScoped,
    TimeBoxed,
    OperationPattern,
    RunPreauthorization,
}

impl ApprovalClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Once => "ONCE",
            Self::SessionScoped => "SESSION_SCOPED",
            Self::TimeBoxed => "TIME_BOXED",
            Self::OperationPattern => "OPERATION_PATTERN",
            Self::RunPreauthorization => "RUN_PREAUTHORIZATION",
        }
    }

    const fn code(self) -> u8 {
        match self {
            Self::Once => 1,
            Self::SessionScoped => 2,
            Self::TimeBoxed => 3,
            Self::OperationPattern => 4,
            Self::RunPreauthorization => 5,
        }
    }
}

impl TryFrom<&str> for ApprovalClass {
    type Error = ApprovalScopeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ONCE" => Ok(Self::Once),
            "SESSION_SCOPED" => Ok(Self::SessionScoped),
            "TIME_BOXED" => Ok(Self::TimeBoxed),
            "OPERATION_PATTERN" => Ok(Self::OperationPattern),
            "RUN_PREAUTHORIZATION" => Ok(Self::RunPreauthorization),
            _ => Err(ApprovalScopeError::InvalidClass),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApprovalScope {
    Once {
        effect_id: EffectId,
    },
    SessionScoped {
        session_id: SessionId,
        actions: Vec<String>,
        resources: Vec<String>,
    },
    TimeBoxed {
        actions: Vec<String>,
        resources: Vec<String>,
    },
    OperationPattern {
        action_pattern: String,
        resource_pattern: String,
    },
    RunPreauthorization {
        session_id: Option<SessionId>,
        actions: Vec<String>,
        resources: Vec<String>,
    },
}

impl ApprovalScope {
    pub const fn once(effect_id: EffectId) -> Self {
        Self::Once { effect_id }
    }

    pub fn session_scoped(
        session_id: SessionId,
        actions: &[String],
        resources: &[String],
    ) -> Result<Self, ApprovalScopeError> {
        Ok(Self::SessionScoped {
            session_id,
            actions: normalize_scope_values(actions, MAX_ACTION_BYTES, "action")?,
            resources: normalize_scope_values(resources, MAX_RESOURCE_BYTES, "resource")?,
        })
    }

    pub fn time_boxed(
        actions: &[String],
        resources: &[String],
    ) -> Result<Self, ApprovalScopeError> {
        Ok(Self::TimeBoxed {
            actions: normalize_scope_values(actions, MAX_ACTION_BYTES, "action")?,
            resources: normalize_scope_values(resources, MAX_RESOURCE_BYTES, "resource")?,
        })
    }

    pub fn operation_pattern(
        action_pattern: &str,
        resource_pattern: &str,
    ) -> Result<Self, ApprovalScopeError> {
        validate_pattern(action_pattern, "action pattern")?;
        validate_pattern(resource_pattern, "resource pattern")?;
        Ok(Self::OperationPattern {
            action_pattern: action_pattern.to_owned(),
            resource_pattern: resource_pattern.to_owned(),
        })
    }

    pub fn run_preauthorization(
        session_id: Option<SessionId>,
        actions: &[String],
        resources: &[String],
    ) -> Result<Self, ApprovalScopeError> {
        Ok(Self::RunPreauthorization {
            session_id,
            actions: normalize_scope_values(actions, MAX_ACTION_BYTES, "action")?,
            resources: normalize_scope_values(resources, MAX_RESOURCE_BYTES, "resource")?,
        })
    }

    pub const fn class(&self) -> ApprovalClass {
        match self {
            Self::Once { .. } => ApprovalClass::Once,
            Self::SessionScoped { .. } => ApprovalClass::SessionScoped,
            Self::TimeBoxed { .. } => ApprovalClass::TimeBoxed,
            Self::OperationPattern { .. } => ApprovalClass::OperationPattern,
            Self::RunPreauthorization { .. } => ApprovalClass::RunPreauthorization,
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ApprovalScopeError> {
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(APPROVAL_SCOPE_DOMAIN)?;
        encoder.push_u8(self.class().code());
        match self {
            Self::Once { effect_id } => {
                encoder.push_u128(effect_id.0);
            }
            Self::SessionScoped {
                session_id,
                actions,
                resources,
            } => {
                encoder.push_u128(session_id.0);
                encode_scope_values(&mut encoder, actions)?;
                encode_scope_values(&mut encoder, resources)?;
            }
            Self::TimeBoxed { actions, resources } => {
                encode_scope_values(&mut encoder, actions)?;
                encode_scope_values(&mut encoder, resources)?;
            }
            Self::OperationPattern {
                action_pattern,
                resource_pattern,
            } => {
                encoder.push_bytes(action_pattern.as_bytes())?;
                encoder.push_bytes(resource_pattern.as_bytes())?;
            }
            Self::RunPreauthorization {
                session_id,
                actions,
                resources,
            } => {
                match session_id {
                    Some(session_id) => {
                        encoder.push_u8(1);
                        encoder.push_u128(session_id.0);
                    }
                    None => encoder.push_u8(0),
                }
                encode_scope_values(&mut encoder, actions)?;
                encode_scope_values(&mut encoder, resources)?;
            }
        }
        let bytes = encoder.finish();
        if bytes.len() > MAX_CANONICAL_SCOPE_BYTES {
            return Err(ApprovalScopeError::CanonicalScopeTooLarge);
        }
        Ok(bytes)
    }

    pub fn scope_digest(&self) -> Result<[u8; 32], ApprovalScopeError> {
        Ok(*blake3::hash(&self.canonical_bytes()?).as_bytes())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalRecord {
    approval_id: [u8; 16],
    approver_principal: String,
    scope: ApprovalScope,
    scope_digest: [u8; 32],
    risk_class: String,
    taint_digest: [u8; 32],
    parent_decision_id: [u8; 16],
    issued_at: String,
    expires_at: Option<String>,
    max_uses: Option<u64>,
    revoked_at: Option<String>,
}

impl ApprovalRecord {
    pub const fn approval_id(&self) -> [u8; 16] {
        self.approval_id
    }

    pub const fn class(&self) -> ApprovalClass {
        self.scope.class()
    }

    pub fn approver_principal(&self) -> &str {
        &self.approver_principal
    }

    pub const fn scope(&self) -> &ApprovalScope {
        &self.scope
    }

    pub const fn scope_digest(&self) -> [u8; 32] {
        self.scope_digest
    }

    pub fn risk_class(&self) -> &str {
        &self.risk_class
    }

    pub const fn taint_digest(&self) -> [u8; 32] {
        self.taint_digest
    }

    pub const fn parent_decision_id(&self) -> [u8; 16] {
        self.parent_decision_id
    }

    pub fn issued_at(&self) -> &str {
        &self.issued_at
    }

    pub fn expires_at(&self) -> Option<&str> {
        self.expires_at.as_deref()
    }

    pub const fn max_uses(&self) -> Option<u64> {
        self.max_uses
    }

    pub fn revoked_at(&self) -> Option<&str> {
        self.revoked_at.as_deref()
    }
}

#[derive(Debug)]
pub enum ApprovalScopeError {
    Core(CoreError),
    InvalidClass,
    EmptyScope(&'static str),
    TooManyScopeItems(&'static str),
    InvalidScopeValue(&'static str),
    InvalidPattern(&'static str),
    CanonicalScopeTooLarge,
}

impl fmt::Display for ApprovalScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(error) => write!(f, "approval scope canonical encoding error: {error}"),
            Self::InvalidClass => f.write_str("approval class is not canonical"),
            Self::EmptyScope(kind) => write!(f, "approval {kind} scope is empty"),
            Self::TooManyScopeItems(kind) => write!(f, "approval {kind} scope has too many items"),
            Self::InvalidScopeValue(kind) => write!(f, "approval {kind} scope value is invalid"),
            Self::InvalidPattern(kind) => write!(f, "approval {kind} is invalid"),
            Self::CanonicalScopeTooLarge => f.write_str("approval canonical scope is too large"),
        }
    }
}

impl Error for ApprovalScopeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            Self::InvalidClass
            | Self::EmptyScope(_)
            | Self::TooManyScopeItems(_)
            | Self::InvalidScopeValue(_)
            | Self::InvalidPattern(_)
            | Self::CanonicalScopeTooLarge => None,
        }
    }
}

impl From<CoreError> for ApprovalScopeError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

fn normalize_scope_values(
    values: &[String],
    max_value_bytes: usize,
    kind: &'static str,
) -> Result<Vec<String>, ApprovalScopeError> {
    if values.is_empty() {
        return Err(ApprovalScopeError::EmptyScope(kind));
    }
    if values.len() > MAX_SCOPE_ITEMS {
        return Err(ApprovalScopeError::TooManyScopeItems(kind));
    }
    let mut normalized = values.to_vec();
    normalized.sort();
    normalized.dedup();
    for value in &normalized {
        if value.is_empty() || value.len() > max_value_bytes || value.chars().any(char::is_control)
        {
            return Err(ApprovalScopeError::InvalidScopeValue(kind));
        }
    }
    Ok(normalized)
}

fn validate_pattern(value: &str, kind: &'static str) -> Result<(), ApprovalScopeError> {
    if value.is_empty() || value.len() > MAX_PATTERN_BYTES || value.chars().any(char::is_control) {
        return Err(ApprovalScopeError::InvalidPattern(kind));
    }
    Ok(())
}

fn encode_scope_values(
    encoder: &mut CanonicalEncoder,
    values: &[String],
) -> Result<(), ApprovalScopeError> {
    encoder.push_u64(
        u64::try_from(values.len()).map_err(|_| ApprovalScopeError::CanonicalScopeTooLarge)?,
    );
    for value in values {
        encoder.push_bytes(value.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn class_round_trip_covers_all_approval_classes() {
        for class in [
            ApprovalClass::Once,
            ApprovalClass::SessionScoped,
            ApprovalClass::TimeBoxed,
            ApprovalClass::OperationPattern,
            ApprovalClass::RunPreauthorization,
        ] {
            assert_eq!(ApprovalClass::try_from(class.as_str()).unwrap(), class);
        }
        assert!(matches!(
            ApprovalClass::try_from("ALWAYS"),
            Err(ApprovalScopeError::InvalidClass)
        ));
    }

    #[test]
    fn every_class_has_a_distinct_canonical_scope() {
        let scopes = [
            ApprovalScope::once(EffectId(1)),
            ApprovalScope::session_scoped(
                SessionId(7),
                &values(&["session.read"]),
                &values(&["session:7"]),
            )
            .unwrap(),
            ApprovalScope::time_boxed(&values(&["session.read"]), &values(&["session:7"])).unwrap(),
            ApprovalScope::operation_pattern("effect.*", "session:7/*").unwrap(),
            ApprovalScope::run_preauthorization(
                Some(SessionId(7)),
                &values(&["effect.simulate"]),
                &values(&["session:7"]),
            )
            .unwrap(),
        ];
        let mut digests = scopes
            .iter()
            .map(ApprovalScope::scope_digest)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        digests.sort();
        digests.dedup();
        assert_eq!(digests.len(), scopes.len());
    }

    #[test]
    fn set_scopes_are_order_independent_and_deduplicated() {
        let first = ApprovalScope::session_scoped(
            SessionId(9),
            &values(&["session.read", "session.create", "session.read"]),
            &values(&["session:2", "session:1"]),
        )
        .unwrap();
        let second = ApprovalScope::session_scoped(
            SessionId(9),
            &values(&["session.create", "session.read"]),
            &values(&["session:1", "session:2"]),
        )
        .unwrap();
        assert_eq!(
            first.scope_digest().unwrap(),
            second.scope_digest().unwrap()
        );
    }

    #[test]
    fn empty_unbounded_or_control_character_scopes_fail_closed() {
        assert!(matches!(
            ApprovalScope::time_boxed(&[], &values(&["session:1"])),
            Err(ApprovalScopeError::EmptyScope("action"))
        ));
        let too_many = (0..=MAX_SCOPE_ITEMS)
            .map(|index| format!("session.{index}"))
            .collect::<Vec<_>>();
        assert!(matches!(
            ApprovalScope::time_boxed(&too_many, &values(&["session:1"])),
            Err(ApprovalScopeError::TooManyScopeItems("action"))
        ));
        assert!(matches!(
            ApprovalScope::operation_pattern("effect.*\n", "session:1/*"),
            Err(ApprovalScopeError::InvalidPattern("action pattern"))
        ));
    }

    #[test]
    fn run_preauthorization_canonicalizes_optional_session_binding() {
        let bounded = ApprovalScope::run_preauthorization(
            Some(SessionId(5)),
            &values(&["effect.simulate"]),
            &values(&["session:5"]),
        )
        .unwrap();
        let cross_session = ApprovalScope::run_preauthorization(
            None,
            &values(&["effect.simulate"]),
            &values(&["session:5"]),
        )
        .unwrap();
        assert_ne!(
            bounded.scope_digest().unwrap(),
            cross_session.scope_digest().unwrap()
        );
    }
}
