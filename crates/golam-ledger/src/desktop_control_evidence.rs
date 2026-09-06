#![forbid(unsafe_code)]

mod binding_store;
mod effect_store;
mod state_store;
mod types;

use golam_core::authority::AuthorityLayout;
use rusqlite::Connection;

pub use types::{
    DesktopControlEvidenceError, DesktopEffectEvidence, DesktopEvidenceOperation,
    DesktopEvidenceStatus,
};

pub struct DesktopControlEvidenceStore {
    pub(crate) connection: Connection,
}

impl DesktopControlEvidenceStore {
    pub fn open(layout: &AuthorityLayout) -> Result<Self, DesktopControlEvidenceError> {
        let connection = Connection::open(layout.authority_db_path())?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; \
             PRAGMA synchronous = FULL; PRAGMA busy_timeout = 5000;",
        )?;
        effect_store::migrate(&connection)?;
        binding_store::migrate(&connection)?;
        state_store::migrate(&connection)?;
        Ok(Self { connection })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, DesktopControlEvidenceError> {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        effect_store::migrate(&connection)?;
        binding_store::migrate(&connection)?;
        state_store::migrate(&connection)?;
        Ok(Self { connection })
    }
}
