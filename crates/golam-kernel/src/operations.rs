#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::{CheckpointId, EventId, GoalId, GoalVersionId, SessionId};
use golam_ledger::checkpoint::{
    CheckpointError, CheckpointManager, CheckpointRecord, CreateCheckpoint, LoadedProjection,
};
use golam_ledger::fork::{CreateFork, ForkError, ForkManager, ForkRecord};
use golam_ledger::goal::{CreateGoalVersion, GoalDocument, GoalError, GoalManager, StoredGoalVersion};
use golam_ledger::session_read::{SessionReadError, SessionReader, SessionSummary};
use golam_ledger::storage::{AuthorityStore, CreateSession, StorageError, StoredEvent};

use crate::{
    AuthorizationContext, AuthorizationPolicy, AuthorizationRequest, KernelApi, KernelError,
    Principal, RecoveryReport, RecoveryScanner,
};

pub struct KernelCreateSession<'a> {
    pub session_id: SessionId,
    pub event_id: EventId,
    pub recorded_at: &'a str,
    pub payload: &'a [u8],
}

pub struct KernelCreateFork<'a> {
    pub child_session_id: SessionId,
    pub event_id: EventId,
    pub parent_session_id: SessionId,
    pub through_session_seq: u64,
    pub recorded_at: &'a str,
}

pub struct KernelAppendGoal<'a> {
    pub goal_version_id: GoalVersionId,
    pub goal_id: GoalId,
    pub event_id: EventId,
    pub session_id: SessionId,
    pub expected_session_seq: u64,
    pub expected_goal_version: u64,
    pub recorded_at: &'a str,
    pub document: GoalDocument<'a>,
}

pub struct KernelCreateCheckpoint<'a> {
    pub checkpoint_id: CheckpointId,
    pub created_event_id: EventId,
    pub session_id: SessionId,
    pub through_session_seq: u64,
    pub recorded_at: &'a str,
}

#[derive(Debug)]
pub enum KernelOperationError {
    Kernel(KernelError),
    Storage(StorageError),
    SessionRead(SessionReadError),
    Fork(ForkError),
    Goal(GoalError),
    Checkpoint(CheckpointError),
}

impl fmt::Display for KernelOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(error) => write!(f, "kernel operation authorization/runtime error: {error}"),
            Self::Storage(error) => write!(f, "kernel session operation error: {error}"),
            Self::SessionRead(error) => write!(f, "kernel session read error: {error}"),
            Self::Fork(error) => write!(f, "kernel fork operation error: {error}"),
            Self::Goal(error) => write!(f, "kernel goal operation error: {error}"),
            Self::Checkpoint(error) => write!(f, "kernel checkpoint operation error: {error}"),
        }
    }
}

impl Error for KernelOperationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel(error) => Some(error),
            Self::Storage(error) => Some(error),
            Self::SessionRead(error) => Some(error),
            Self::Fork(error) => Some(error),
            Self::Goal(error) => Some(error),
            Self::Checkpoint(error) => Some(error),
        }
    }
}

impl From<KernelError> for KernelOperationError {
    fn from(value: KernelError) -> Self {
        Self::Kernel(value)
    }
}

impl From<StorageError> for KernelOperationError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

impl From<SessionReadError> for KernelOperationError {
    fn from(value: SessionReadError) -> Self {
        Self::SessionRead(value)
    }
}

impl From<ForkError> for KernelOperationError {
    fn from(value: ForkError) -> Self {
        Self::Fork(value)
    }
}

impl From<GoalError> for KernelOperationError {
    fn from(value: GoalError) -> Self {
        Self::Goal(value)
    }
}

impl From<CheckpointError> for KernelOperationError {
    fn from(value: CheckpointError) -> Self {
        Self::Checkpoint(value)
    }
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn create_session(
        &mut self,
        principal: Principal<'_>,
        input: KernelCreateSession<'_>,
        scope: &str,
    ) -> Result<StoredEvent, KernelOperationError> {
        let resource = format!("session:{}", input.session_id.0);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "session.create",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        let mut store = AuthorityStore::open(self.authority.authority_db_path())?;
        Ok(store.create_session(CreateSession {
            session_id: input.session_id,
            event_id: input.event_id,
            owner_principal: principal.subject,
            actor_principal: principal.subject,
            recorded_at: input.recorded_at,
            payload: input.payload,
            security_critical: true,
        })?)
    }

    pub fn list_sessions(
        &mut self,
        principal: Principal<'_>,
        scope: &str,
    ) -> Result<Vec<SessionSummary>, KernelOperationError> {
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "session.read",
            resource: "session:*",
            context: AuthorizationContext::local(scope),
        })?;
        Ok(SessionReader::open(&self.authority)?.list()?)
    }

    pub fn open_session(
        &mut self,
        principal: Principal<'_>,
        session_id: SessionId,
        scope: &str,
    ) -> Result<Option<SessionSummary>, KernelOperationError> {
        let resource = format!("session:{}", session_id.0);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "session.read",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        Ok(SessionReader::open(&self.authority)?.get(session_id)?)
    }

    pub fn fork_session(
        &mut self,
        principal: Principal<'_>,
        input: KernelCreateFork<'_>,
        scope: &str,
    ) -> Result<ForkRecord, KernelOperationError> {
        let resource = format!("session:{}", input.child_session_id.0);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "session.fork",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        let mut manager = ForkManager::open(self.authority.authority_db_path())?;
        Ok(manager.create(CreateFork {
            child_session_id: input.child_session_id,
            event_id: input.event_id,
            parent_session_id: input.parent_session_id,
            through_session_seq: input.through_session_seq,
            actor_principal: principal.subject,
            recorded_at: input.recorded_at,
        })?)
    }

    pub fn append_goal_version(
        &mut self,
        principal: Principal<'_>,
        input: KernelAppendGoal<'_>,
        scope: &str,
    ) -> Result<StoredGoalVersion, KernelOperationError> {
        let resource = format!("goal:{}", input.goal_id.0);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "goal.append",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        let mut manager = GoalManager::open(self.authority.authority_db_path())?;
        Ok(manager.append_version(CreateGoalVersion {
            goal_version_id: input.goal_version_id,
            goal_id: input.goal_id,
            event_id: input.event_id,
            session_id: input.session_id,
            expected_session_seq: input.expected_session_seq,
            expected_goal_version: input.expected_goal_version,
            actor_principal: principal.subject,
            recorded_at: input.recorded_at,
            document: input.document,
        })?)
    }

    pub fn create_checkpoint(
        &mut self,
        principal: Principal<'_>,
        input: KernelCreateCheckpoint<'_>,
        scope: &str,
    ) -> Result<CheckpointRecord, KernelOperationError> {
        let resource = format!("checkpoint:{}", input.checkpoint_id.0);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "checkpoint.create",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        let mut authority = AuthorityStore::open(self.authority.authority_db_path())?;
        let mut manager =
            CheckpointManager::open(self.authority.authority_db_path(), &self.runtime.artifact_dir)?;
        Ok(manager.create(
            &mut authority,
            CreateCheckpoint {
                checkpoint_id: input.checkpoint_id,
                created_event_id: input.created_event_id,
                session_id: input.session_id,
                through_session_seq: input.through_session_seq,
                actor_principal: principal.subject,
                recorded_at: input.recorded_at,
            },
        )?)
    }

    pub fn verify_checkpoint(
        &mut self,
        principal: Principal<'_>,
        checkpoint_id: CheckpointId,
        session_id: SessionId,
        through_session_seq: u64,
        scope: &str,
    ) -> Result<LoadedProjection, KernelOperationError> {
        let resource = format!("checkpoint:{}", checkpoint_id.0);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "checkpoint.verify",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        let manager =
            CheckpointManager::open(self.authority.authority_db_path(), &self.runtime.artifact_dir)?;
        Ok(manager.load_or_replay(checkpoint_id, session_id, through_session_seq)?)
    }

    pub fn replay_session(
        &mut self,
        principal: Principal<'_>,
        session_id: SessionId,
        through_session_seq: u64,
        scope: &str,
    ) -> Result<Vec<u8>, KernelOperationError> {
        let resource = format!("session:{}", session_id.0);
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "replay.run",
            resource: &resource,
            context: AuthorizationContext::local(scope),
        })?;
        let manager =
            CheckpointManager::open(self.authority.authority_db_path(), &self.runtime.artifact_dir)?;
        Ok(manager.replay_projection(session_id, through_session_seq)?)
    }

    pub fn read_recovery_status(
        &mut self,
        principal: Principal<'_>,
        scope: &str,
    ) -> Result<RecoveryReport, KernelOperationError> {
        self.require_authority(&AuthorizationRequest {
            principal,
            action: "recovery.status.read",
            resource: "recovery:status",
            context: AuthorizationContext::local(scope),
        })?;
        Ok(RecoveryScanner::scan(&self.runtime)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BootstrapPolicy, Principal};
    use golam_core::paths::RuntimeLayout;
    use golam_ledger::checkpoint::ProjectionSource;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static N: AtomicU64 = AtomicU64::new(0);

    fn runtime() -> RuntimeLayout {
        let n = N.fetch_add(1, Ordering::Relaxed);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        RuntimeLayout::initialize(std::env::temp_dir().join(format!(
            "golam-kernel-operations-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    #[test]
    fn authenticated_principal_is_bound_into_session_and_fork_operations() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let principal = Principal::local_owner("owner");
        kernel
            .create_session(
                principal,
                KernelCreateSession {
                    session_id: SessionId(1),
                    event_id: EventId(2),
                    recorded_at: "2026-08-25T11:40:00Z",
                    payload: b"root",
                },
                "local-owner",
            )
            .unwrap();
        let root = kernel
            .open_session(principal, SessionId(1), "local-owner")
            .unwrap()
            .unwrap();
        assert_eq!(root.owner_principal, "owner");
        assert_eq!(kernel.list_sessions(principal, "local-owner").unwrap().len(), 1);

        kernel
            .fork_session(
                principal,
                KernelCreateFork {
                    child_session_id: SessionId(3),
                    event_id: EventId(4),
                    parent_session_id: SessionId(1),
                    through_session_seq: 1,
                    recorded_at: "2026-08-25T11:41:00Z",
                },
                "local-owner",
            )
            .unwrap();
        let child = kernel
            .open_session(principal, SessionId(3), "local-owner")
            .unwrap()
            .unwrap();
        assert_eq!(child.owner_principal, "owner");
        assert_eq!(child.parent_session_id, Some(SessionId(1)));
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn checkpoint_and_replay_stay_behind_kernel_authorization() {
        let runtime = runtime();
        let mut kernel = KernelApi::open(&runtime, BootstrapPolicy::default()).unwrap();
        let principal = Principal::local_owner("owner");
        kernel
            .create_session(
                principal,
                KernelCreateSession {
                    session_id: SessionId(10),
                    event_id: EventId(11),
                    recorded_at: "2026-08-25T11:42:00Z",
                    payload: b"root",
                },
                "local-owner",
            )
            .unwrap();
        kernel
            .create_checkpoint(
                principal,
                KernelCreateCheckpoint {
                    checkpoint_id: CheckpointId(12),
                    created_event_id: EventId(13),
                    session_id: SessionId(10),
                    through_session_seq: 1,
                    recorded_at: "2026-08-25T11:43:00Z",
                },
                "local-owner",
            )
            .unwrap();
        let loaded = kernel
            .verify_checkpoint(
                principal,
                CheckpointId(12),
                SessionId(10),
                1,
                "local-owner",
            )
            .unwrap();
        assert_eq!(loaded.source, ProjectionSource::Checkpoint);
        let replay = kernel
            .replay_session(principal, SessionId(10), 1, "local-owner")
            .unwrap();
        assert_eq!(loaded.bytes, replay);
        assert_eq!(
            kernel
                .read_recovery_status(principal, "local-owner")
                .unwrap()
                .mode,
            crate::RecoveryMode::Normal
        );
        drop(kernel);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
