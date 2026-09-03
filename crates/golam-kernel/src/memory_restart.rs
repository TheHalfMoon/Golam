#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::memory_storage::{MemoryLayout, MemoryLayoutError};
use golam_ledger::memory_restart::{MemoryRestartError, MemoryRestartStore};

use crate::{AuthorizationPolicy, KernelApi};

pub use golam_ledger::memory_restart::{
    MemoryRestartCase, MemoryRestartObservation, MemoryRestartResolution,
};

#[derive(Debug)]
pub enum ManagedMemoryRestartError {
    Layout(MemoryLayoutError),
    Restart(MemoryRestartError),
}

impl fmt::Display for ManagedMemoryRestartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Layout(error) => write!(f, "managed-memory restart layout failed: {error}"),
            Self::Restart(error) => write!(f, "managed-memory restart failed: {error}"),
        }
    }
}

impl Error for ManagedMemoryRestartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Layout(error) => Some(error),
            Self::Restart(error) => Some(error),
        }
    }
}

impl From<MemoryLayoutError> for ManagedMemoryRestartError {
    fn from(value: MemoryLayoutError) -> Self {
        Self::Layout(value)
    }
}

impl From<MemoryRestartError> for ManagedMemoryRestartError {
    fn from(value: MemoryRestartError) -> Self {
        Self::Restart(value)
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    /// Returns every durable PREPARED managed-memory effect whose Effect Gate
    /// state is not terminal. The returned cases are observations only; they do
    /// not grant mutation authority.
    pub fn pending_managed_memory_restart_cases(
        &self,
    ) -> Result<Vec<MemoryRestartCase>, ManagedMemoryRestartError> {
        let memory = MemoryLayout::initialize(&self.runtime)?;
        let restart = MemoryRestartStore::open(&self.authority, &memory)?;
        Ok(restart.pending_cases()?)
    }

    /// Reconciles one exact restart case against a bounded Markdown observation.
    /// All authority/effect/operational writes remain behind KernelApi.
    pub fn reconcile_managed_memory_restart_case(
        &mut self,
        case: &MemoryRestartCase,
        observation: &MemoryRestartObservation,
        finished_at: &str,
        terminal_at_unix_ms: u64,
    ) -> Result<MemoryRestartResolution, ManagedMemoryRestartError> {
        let memory = MemoryLayout::initialize(&self.runtime)?;
        let restart = MemoryRestartStore::open(&self.authority, &memory)?;
        Ok(restart.reconcile(
            case,
            observation,
            finished_at,
            terminal_at_unix_ms,
        )?)
    }
}
