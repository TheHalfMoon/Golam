#![forbid(unsafe_code)]

use golam_core::SessionId;
use golam_core::harness_state::{ModelEvent, RequestAttempt};
use golam_core::model_backend::{HarnessEvidenceSink, HarnessEvidenceSinkError};

use crate::harness_evidence::HarnessEvidenceStore;

impl HarnessEvidenceSink for HarnessEvidenceStore {
    fn persist_prepared_attempt(
        &mut self,
        session_id: SessionId,
        attempt: &RequestAttempt,
        record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceSinkError> {
        HarnessEvidenceStore::persist_prepared_attempt(self, session_id, attempt, record_bytes)
            .map_err(|error| HarnessEvidenceSinkError::new(error.to_string()))
    }

    fn persist_attempt_state(
        &mut self,
        session_id: SessionId,
        attempt: &RequestAttempt,
        record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceSinkError> {
        HarnessEvidenceStore::persist_attempt_state(self, session_id, attempt, record_bytes)
            .map_err(|error| HarnessEvidenceSinkError::new(error.to_string()))
    }

    fn append_model_event(
        &mut self,
        event: &ModelEvent,
        record_bytes: &[u8],
    ) -> Result<(), HarnessEvidenceSinkError> {
        HarnessEvidenceStore::append_model_event(self, event, record_bytes)
            .map_err(|error| HarnessEvidenceSinkError::new(error.to_string()))
    }
}
