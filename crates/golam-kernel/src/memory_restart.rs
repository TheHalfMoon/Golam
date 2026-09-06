#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::memory_storage::{MemoryLayout, MemoryLayoutError};
use golam_ledger::memory_restart::{MemoryRestartError, MemoryRestartStore};
use golam_ledger::memory_writer_authority::{
    MemoryWriterAuthorityError, MemoryWriterAuthorityStore,
};

use crate::{AuthorizationPolicy, KernelApi};

pub use golam_ledger::memory_restart::{
    MemoryRestartCase, MemoryRestartObservation, MemoryRestartResolution,
};

#[derive(Debug)]
pub enum ManagedMemoryRestartError {
    Layout(MemoryLayoutError),
    WriterAuthority(MemoryWriterAuthorityError),
    Restart(MemoryRestartError),
}

impl fmt::Display for ManagedMemoryRestartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Layout(error) => write!(f, "managed-memory restart layout failed: {error}"),
            Self::WriterAuthority(error) => {
                write!(
                    f,
                    "managed-memory restart authority initialization failed: {error}"
                )
            }
            Self::Restart(error) => write!(f, "managed-memory restart failed: {error}"),
        }
    }
}

impl Error for ManagedMemoryRestartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Layout(error) => Some(error),
            Self::WriterAuthority(error) => Some(error),
            Self::Restart(error) => Some(error),
        }
    }
}

impl From<MemoryLayoutError> for ManagedMemoryRestartError {
    fn from(value: MemoryLayoutError) -> Self {
        Self::Layout(value)
    }
}

impl From<MemoryWriterAuthorityError> for ManagedMemoryRestartError {
    fn from(value: MemoryWriterAuthorityError) -> Self {
        Self::WriterAuthority(value)
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
        drop(MemoryWriterAuthorityStore::open(&self.authority)?);
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
        drop(MemoryWriterAuthorityStore::open(&self.authority)?);
        let restart = MemoryRestartStore::open(&self.authority, &memory)?;
        Ok(restart.reconcile(case, observation, finished_at, terminal_at_unix_ms)?)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use golam_core::paths::RuntimeLayout;

    use super::*;
    use crate::DenyByDefault;

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> RuntimeLayout {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after UNIX epoch")
            .as_nanos();
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-memory-restart-kernel-{}-{nanos}-{counter}",
            std::process::id()
        )))
        .expect("test runtime must initialize")
    }

    #[test]
    fn clean_runtime_initializes_restart_schema_and_has_no_pending_cases() {
        let runtime = runtime();
        let kernel = KernelApi::open(&runtime, DenyByDefault).expect("clean kernel must open");
        assert!(
            kernel
                .pending_managed_memory_restart_cases()
                .expect("clean restart scan must succeed")
                .is_empty()
        );
        drop(kernel);
        fs::remove_dir_all(runtime.root).expect("test runtime cleanup must succeed");
    }
}
