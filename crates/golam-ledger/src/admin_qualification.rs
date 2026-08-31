#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{EventId, SessionId};

use crate::secret_entry::{PrepareDesignatedSecretEntryRequest, prepare_designated_secret_entry};

const UNKNOWN_FORMAT_CANARY: &[u8] =
    b"golam-spec003-t003081-canary::opaque-unknown-format::f8d2e4c1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretCanaryQualificationError {
    PreparationFailed,
}

impl fmt::Display for SecretCanaryQualificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PreparationFailed => {
                f.write_str("deterministic secret-canary preparation failed closed")
            }
        }
    }
}

impl Error for SecretCanaryQualificationError {}

/// Exercises the explicit designated-secret preparation boundary with a fixed
/// unknown-format canary. The prepared value is never committed, returned,
/// logged or formatted; dropping it zeroizes the protected plaintext owner.
pub fn qualify_designated_secret_canary() -> Result<(), SecretCanaryQualificationError> {
    let prepared = prepare_designated_secret_entry(PrepareDesignatedSecretEntryRequest {
        session_id: SessionId(1),
        expected_session_seq: 0,
        event_id: EventId(1),
        actor_principal: "owner:qualification",
        owner_principal: "owner:qualification",
        recorded_at: "2026-08-29T00:00:00Z",
        classification: "qualification_canary",
        purpose_scope: "qualification-only",
        expires_at: None,
        value: UNKNOWN_FORMAT_CANARY.to_vec(),
    })
    .map_err(|_| SecretCanaryQualificationError::PreparationFailed)?;
    drop(prepared);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canary_qualification_uses_designated_secret_path_without_exposing_value() {
        qualify_designated_secret_canary().unwrap();
        let rendered = SecretCanaryQualificationError::PreparationFailed.to_string();
        assert!(!rendered.contains("f8d2e4c1"));
        assert!(!rendered.contains("opaque-unknown-format"));
    }
}
