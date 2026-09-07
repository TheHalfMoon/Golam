#![forbid(unsafe_code)]

mod binding_store;
mod effect_store;
mod state_store;
mod types;

use golam_core::authority::AuthorityLayout;
use golam_core::desktop_control::DesktopControlLeaseId;
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

    pub fn load_lease_ids(
        &self,
    ) -> Result<Vec<DesktopControlLeaseId>, DesktopControlEvidenceError> {
        let mut statement = self
            .connection
            .prepare("SELECT lease_id FROM desktop_control_lease_state ORDER BY lease_id ASC")?;
        let rows = statement.query_map([], |row| row.get::<_, Vec<u8>>(0))?;
        let mut lease_ids = Vec::new();
        for row in rows {
            let bytes: [u8; 16] = row?
                .try_into()
                .map_err(|_| DesktopControlEvidenceError::InvalidStoredRecord("lease id"))?;
            let value = u128::from_be_bytes(bytes);
            if value == 0 {
                return Err(DesktopControlEvidenceError::InvalidStoredRecord("lease id"));
            }
            lease_ids.push(DesktopControlLeaseId::from_u128(value));
        }
        Ok(lease_ids)
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
