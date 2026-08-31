#![forbid(unsafe_code)]

use crate::harness::RequestAttemptId;
use crate::harness_state::{ModelEventKind, ModelRequest};

pub const MAX_BACKEND_EMISSION_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelBackendFailureClass {
    Transient,
    Deterministic,
    ContextOverflow,
    Crashed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelBackendError {
    pub class: ModelBackendFailureClass,
    pub message: String,
}

impl ModelBackendError {
    pub fn new(class: ModelBackendFailureClass, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendEmission {
    pub sequence: u64,
    pub kind: ModelEventKind,
    pub payload: Vec<u8>,
}

impl BackendEmission {
    pub fn validate(&self) -> Result<(), ModelBackendError> {
        if self.payload.len() > MAX_BACKEND_EMISSION_BYTES {
            return Err(ModelBackendError::new(
                ModelBackendFailureClass::Deterministic,
                "backend emission exceeds bounded payload size",
            ));
        }
        Ok(())
    }
}

pub trait ModelBackendSession {
    fn backend_instance_ref(&self) -> &str;

    fn next_emission(&mut self) -> Result<Option<BackendEmission>, ModelBackendError>;

    fn request_cancel(&mut self) -> Result<(), ModelBackendError>;
}

pub trait ModelBackend {
    type Session: ModelBackendSession;

    fn start(&mut self, request: &ModelRequest) -> Result<Self::Session, ModelBackendError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendRequestBinding {
    pub request_attempt_id: RequestAttemptId,
}

impl BackendRequestBinding {
    pub const fn from_request(request: &ModelRequest) -> Self {
        Self {
            request_attempt_id: request.request_attempt_id,
        }
    }
}
