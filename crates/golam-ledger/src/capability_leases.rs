#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::error::Error;
use std::fmt;

use golam_core::authority::AuthorityLayout;
use rusqlite::{Connection, OptionalExtension, params};

const MAX_PARENT_CHAIN_DEPTH: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityLeaseRuntimeState {
    pub lease_id: [u8; 16],
    pub principal_id: String,
    pub parent_lease_id: Option<[u8; 16]>,
    pub not_before: Option<String>,
    pub expires_at: Option<String>,
    pub generation: u64,
    pub status: String,
    pub authority_digest: [u8; 32],
    pub revoked: bool,
}

#[derive(Debug)]
pub enum CapabilityLeaseRuntimeError {
    Sqlite(rusqlite::Error),
    Integrity(String),
    AuthoritySecurity(String),
    InvalidStoredRecord(&'static str),
    MissingParent,
    ParentCycle,
    ParentChainTooDeep,
}

impl fmt::Display for CapabilityLeaseRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "capability lease runtime sqlite error: {error}"),
            Self::Integrity(error) => {
                write!(f, "capability lease runtime integrity error: {error}")
            }
            Self::AuthoritySecurity(error) => {
                write!(
                    f,
                    "capability lease runtime authority-security error: {error}"
                )
            }
            Self::InvalidStoredRecord(reason) => {
                write!(
                    f,
                    "capability lease runtime stored record is invalid: {reason}"
                )
            }
            Self::MissingParent => {
                f.write_str("capability lease parent chain references a missing lease")
            }
            Self::ParentCycle => f.write_str("capability lease parent chain contains a cycle"),
            Self::ParentChainTooDeep => {
                f.write_str("capability lease parent chain exceeds the bounded depth")
            }
        }
    }
}

impl Error for CapabilityLeaseRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Integrity(_)
            | Self::AuthoritySecurity(_)
            | Self::InvalidStoredRecord(_)
            | Self::MissingParent
            | Self::ParentCycle
            | Self::ParentChainTooDeep => None,
        }
    }
}

impl From<rusqlite::Error> for CapabilityLeaseRuntimeError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

/// Loads the current lease and its complete parent chain from one coherent,
/// integrity-verified authority snapshot. The returned vector is ordered from
/// the requested lease toward its root parent. An empty vector means the
/// requested lease does not exist.
pub fn load_capability_lease_runtime_chain(
    layout: &AuthorityLayout,
    lease_id: [u8; 16],
) -> Result<Vec<CapabilityLeaseRuntimeState>, CapabilityLeaseRuntimeError> {
    let connection = Connection::open(layout.authority_db_path())?;
    connection.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA query_only = ON; PRAGMA busy_timeout = 5000; BEGIN DEFERRED;",
    )?;

    let result = load_verified_chain(&connection, lease_id);
    let rollback = connection.execute_batch("ROLLBACK;");
    match (result, rollback) {
        (Ok(chain), Ok(())) => Ok(chain),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(CapabilityLeaseRuntimeError::Sqlite(error)),
    }
}

fn load_verified_chain(
    connection: &Connection,
    lease_id: [u8; 16],
) -> Result<Vec<CapabilityLeaseRuntimeState>, CapabilityLeaseRuntimeError> {
    crate::integrity::verify(connection)
        .map_err(|error| CapabilityLeaseRuntimeError::Integrity(error.to_string()))?;
    crate::authority_security_v2::verify(connection)
        .map_err(|error| CapabilityLeaseRuntimeError::AuthoritySecurity(error.to_string()))?;

    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut next = Some(lease_id);

    while let Some(current_id) = next {
        if chain.len() >= MAX_PARENT_CHAIN_DEPTH {
            return Err(CapabilityLeaseRuntimeError::ParentChainTooDeep);
        }
        if !seen.insert(current_id) {
            return Err(CapabilityLeaseRuntimeError::ParentCycle);
        }

        let state = load_state(connection, current_id)?;
        let Some(state) = state else {
            return if chain.is_empty() {
                Ok(Vec::new())
            } else {
                Err(CapabilityLeaseRuntimeError::MissingParent)
            };
        };
        next = state.parent_lease_id;
        chain.push(state);
    }

    Ok(chain)
}

fn load_state(
    connection: &Connection,
    lease_id: [u8; 16],
) -> Result<Option<CapabilityLeaseRuntimeState>, CapabilityLeaseRuntimeError> {
    let row = connection
        .query_row(
            "SELECT l.principal_id, l.parent_lease_id, l.not_before, l.expires_at, l.generation, l.status, l.authority_digest, EXISTS(SELECT 1 FROM capability_revocations r WHERE r.lease_id = l.lease_id) FROM capability_leases l WHERE l.lease_id = ?1",
            params![&lease_id[..]],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<Vec<u8>>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Vec<u8>>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            },
        )
        .optional()?;

    row.map(|row| {
        let parent_lease_id = row.1.map(id_from_vec).transpose()?;
        let generation = u64::try_from(row.4)
            .map_err(|_| CapabilityLeaseRuntimeError::InvalidStoredRecord("negative generation"))?;
        if generation == 0 {
            return Err(CapabilityLeaseRuntimeError::InvalidStoredRecord(
                "zero generation",
            ));
        }
        let authority_digest = hash_from_vec(row.6)?;
        Ok(CapabilityLeaseRuntimeState {
            lease_id,
            principal_id: row.0,
            parent_lease_id,
            not_before: row.2,
            expires_at: row.3,
            generation,
            status: row.5,
            authority_digest,
            revoked: row.7 != 0,
        })
    })
    .transpose()
}

fn id_from_vec(value: Vec<u8>) -> Result<[u8; 16], CapabilityLeaseRuntimeError> {
    value
        .try_into()
        .map_err(|_| CapabilityLeaseRuntimeError::InvalidStoredRecord("lease id is not 16 bytes"))
}

fn hash_from_vec(value: Vec<u8>) -> Result<[u8; 32], CapabilityLeaseRuntimeError> {
    value.try_into().map_err(|_| {
        CapabilityLeaseRuntimeError::InvalidStoredRecord("authority digest is not 32 bytes")
    })
}
