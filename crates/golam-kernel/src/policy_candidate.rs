#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::str::FromStr;

use cedar_policy::{PolicySet, Schema, ValidationMode, Validator};

pub const MAX_CANDIDATE_POLICY_BYTES: usize = 131_072;
pub const MAX_CANDIDATE_SCHEMA_BYTES: usize = 131_072;
pub const MAX_CANDIDATE_DIAGNOSTICS: usize = 16;
pub const MAX_CANDIDATE_DIAGNOSTIC_CHARS: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidatePolicyFailureKind {
    PolicyTooLarge,
    SchemaTooLarge,
    PolicyParse,
    SchemaParse,
    SchemaWarning,
    PolicyValidation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidatePolicyError {
    kind: CandidatePolicyFailureKind,
    diagnostics: Vec<String>,
}

impl CandidatePolicyError {
    pub const fn kind(&self) -> CandidatePolicyFailureKind {
        self.kind
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    fn single(kind: CandidatePolicyFailureKind, diagnostic: impl fmt::Display) -> Self {
        Self {
            kind,
            diagnostics: vec![bounded_diagnostic(diagnostic)],
        }
    }

    fn from_diagnostics<T, I>(kind: CandidatePolicyFailureKind, diagnostics: I) -> Self
    where
        T: fmt::Display,
        I: IntoIterator<Item = T>,
    {
        Self {
            kind,
            diagnostics: diagnostics
                .into_iter()
                .take(MAX_CANDIDATE_DIAGNOSTICS)
                .map(bounded_diagnostic)
                .collect(),
        }
    }
}

impl fmt::Display for CandidatePolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "candidate policy rejected: {:?}", self.kind)?;
        if let Some(first) = self.diagnostics.first() {
            write!(f, ": {first}")?;
        }
        Ok(())
    }
}

impl Error for CandidatePolicyError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPolicyCandidate {
    policy_source: String,
    schema_source: String,
}

impl ValidatedPolicyCandidate {
    pub fn policy_source(&self) -> &str {
        &self.policy_source
    }

    pub fn schema_source(&self) -> &str {
        &self.schema_source
    }

    pub fn into_sources(self) -> (String, String) {
        (self.policy_source, self.schema_source)
    }
}

pub fn validate_policy_candidate(
    policy_source: &str,
    schema_source: &str,
) -> Result<ValidatedPolicyCandidate, CandidatePolicyError> {
    if policy_source.len() > MAX_CANDIDATE_POLICY_BYTES {
        return Err(CandidatePolicyError::single(
            CandidatePolicyFailureKind::PolicyTooLarge,
            format_args!(
                "policy source is {} bytes; maximum is {}",
                policy_source.len(),
                MAX_CANDIDATE_POLICY_BYTES
            ),
        ));
    }
    if schema_source.len() > MAX_CANDIDATE_SCHEMA_BYTES {
        return Err(CandidatePolicyError::single(
            CandidatePolicyFailureKind::SchemaTooLarge,
            format_args!(
                "schema source is {} bytes; maximum is {}",
                schema_source.len(),
                MAX_CANDIDATE_SCHEMA_BYTES
            ),
        ));
    }

    let policy_set = PolicySet::from_str(policy_source).map_err(|error| {
        CandidatePolicyError::single(CandidatePolicyFailureKind::PolicyParse, error)
    })?;
    let (schema, schema_warnings) =
        Schema::from_cedarschema_str(schema_source).map_err(|error| {
            CandidatePolicyError::single(CandidatePolicyFailureKind::SchemaParse, error)
        })?;

    let warnings = schema_warnings.into_iter().collect::<Vec<_>>();
    if !warnings.is_empty() {
        return Err(CandidatePolicyError::from_diagnostics(
            CandidatePolicyFailureKind::SchemaWarning,
            warnings,
        ));
    }

    let result = Validator::new(schema).validate(&policy_set, ValidationMode::Strict);
    if !result.validation_passed_without_warnings() {
        let diagnostics = result
            .validation_errors()
            .map(|error| error.to_string())
            .chain(
                result
                    .validation_warnings()
                    .map(|warning| warning.to_string()),
            );
        return Err(CandidatePolicyError::from_diagnostics(
            CandidatePolicyFailureKind::PolicyValidation,
            diagnostics,
        ));
    }

    Ok(ValidatedPolicyCandidate {
        policy_source: policy_source.to_owned(),
        schema_source: schema_source.to_owned(),
    })
}

fn bounded_diagnostic(diagnostic: impl fmt::Display) -> String {
    diagnostic
        .to_string()
        .chars()
        .take(MAX_CANDIDATE_DIAGNOSTIC_CHARS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCHEMA: &str = r#"
entity User;
entity Photo;
action view appliesTo { principal: [User], resource: [Photo] };
"#;

    const POLICY: &str = r#"
permit(
    principal is User,
    action == Action::"view",
    resource is Photo
);
"#;

    #[test]
    fn valid_candidate_passes_strict_validation_and_preserves_exact_sources() {
        let candidate = validate_policy_candidate(POLICY, SCHEMA).unwrap();
        assert_eq!(candidate.policy_source(), POLICY);
        assert_eq!(candidate.schema_source(), SCHEMA);
        assert_eq!(
            candidate.clone().into_sources(),
            (POLICY.to_owned(), SCHEMA.to_owned())
        );
    }

    #[test]
    fn malformed_policy_and_schema_fail_closed() {
        let policy_error = validate_policy_candidate("permit(", SCHEMA).unwrap_err();
        assert_eq!(policy_error.kind(), CandidatePolicyFailureKind::PolicyParse);
        assert!(!policy_error.diagnostics().is_empty());

        let schema_error = validate_policy_candidate(POLICY, "entity").unwrap_err();
        assert_eq!(schema_error.kind(), CandidatePolicyFailureKind::SchemaParse);
        assert!(!schema_error.diagnostics().is_empty());
    }

    #[test]
    fn schema_invalid_policy_is_rejected() {
        let invalid = r#"
permit(
    principal is User,
    action == Action::"delete",
    resource is Photo
);
"#;
        let error = validate_policy_candidate(invalid, SCHEMA).unwrap_err();
        assert_eq!(error.kind(), CandidatePolicyFailureKind::PolicyValidation);
        assert!(!error.diagnostics().is_empty());
    }

    #[test]
    fn candidate_inputs_are_bounded_before_cedar_parsing() {
        let policy = " ".repeat(MAX_CANDIDATE_POLICY_BYTES + 1);
        let error = validate_policy_candidate(&policy, SCHEMA).unwrap_err();
        assert_eq!(error.kind(), CandidatePolicyFailureKind::PolicyTooLarge);

        let schema = " ".repeat(MAX_CANDIDATE_SCHEMA_BYTES + 1);
        let error = validate_policy_candidate(POLICY, &schema).unwrap_err();
        assert_eq!(error.kind(), CandidatePolicyFailureKind::SchemaTooLarge);
    }

    #[test]
    fn diagnostics_are_bounded_in_count_and_text() {
        let long_action = "x".repeat(MAX_CANDIDATE_DIAGNOSTIC_CHARS * 4);
        let policy = format!(
            "permit(principal is User, action == Action::\"{long_action}\", resource is Photo);"
        );
        let error = validate_policy_candidate(&policy, SCHEMA).unwrap_err();
        assert_eq!(error.kind(), CandidatePolicyFailureKind::PolicyValidation);
        assert!(error.diagnostics().len() <= MAX_CANDIDATE_DIAGNOSTICS);
        assert!(
            error
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.chars().count() <= MAX_CANDIDATE_DIAGNOSTIC_CHARS)
        );
    }
}
