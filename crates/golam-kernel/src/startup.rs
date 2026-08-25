#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use golam_core::paths::RuntimeLayout;
use golam_ledger::recovery::{RecoveryError, RecoveryMode, RecoveryReport, RecoveryScanner};

use crate::{AuthorizationPolicy, KernelApi, KernelError};

pub enum KernelStartup<P> {
    Serving {
        kernel: KernelApi<P>,
        report: RecoveryReport,
    },
    RecoveryOnly(RecoveryReport),
    Quarantined(RecoveryReport),
}

impl<P> KernelStartup<P> {
    pub fn report(&self) -> &RecoveryReport {
        match self {
            Self::Serving { report, .. } | Self::RecoveryOnly(report) | Self::Quarantined(report) => {
                report
            }
        }
    }
}

#[derive(Debug)]
pub enum KernelStartupError {
    Recovery(RecoveryError),
    Kernel(KernelError),
}

impl fmt::Display for KernelStartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recovery(error) => write!(f, "kernel startup recovery scan failed: {error}"),
            Self::Kernel(error) => write!(f, "kernel startup failed: {error}"),
        }
    }
}

impl Error for KernelStartupError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Recovery(error) => Some(error),
            Self::Kernel(error) => Some(error),
        }
    }
}

impl From<RecoveryError> for KernelStartupError {
    fn from(value: RecoveryError) -> Self {
        Self::Recovery(value)
    }
}

impl From<KernelError> for KernelStartupError {
    fn from(value: KernelError) -> Self {
        Self::Kernel(value)
    }
}

pub fn start_kernel<P: AuthorizationPolicy>(
    runtime: &RuntimeLayout,
    policy: P,
) -> Result<KernelStartup<P>, KernelStartupError> {
    let report = RecoveryScanner::scan(runtime)?;
    match report.mode {
        RecoveryMode::Normal => Ok(KernelStartup::Serving {
            kernel: KernelApi::open_after_recovery(runtime, policy)?,
            report,
        }),
        RecoveryMode::RecoveryOnly => Ok(KernelStartup::RecoveryOnly(report)),
        RecoveryMode::Quarantine => Ok(KernelStartup::Quarantined(report)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DenyByDefault;
    use golam_core::authority::AuthorityLayout;
    use golam_core::{EffectId, EffectTransitionId, EventId, SessionId};
    use golam_ledger::dispatch::encode_effect_dependencies;
    use golam_ledger::effects::{CompareAndSwapEffect, EffectStore, ProposeEffect};
    use golam_ledger::storage::{AuthorityStore, CreateSession};
    use rusqlite::{Connection, params};
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
            "golam-kernel-startup-{}-{t}-{n}",
            std::process::id()
        )))
        .unwrap()
    }

    #[test]
    fn clean_startup_returns_serving_kernel() {
        let runtime = runtime();
        let startup = start_kernel(&runtime, DenyByDefault).unwrap();
        assert!(matches!(startup, KernelStartup::Serving { .. }));
        assert_eq!(startup.report().mode, RecoveryMode::Normal);
        drop(startup);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn incoherent_effect_blocks_direct_open_and_returns_recovery_only() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let effect_id = EffectId(30);
        let dependencies = encode_effect_dependencies(&[]).unwrap();
        let mut effects = EffectStore::open(&authority).unwrap();
        effects
            .propose(ProposeEffect {
                effect_id,
                session_id: SessionId(31),
                requested_by: "owner",
                action: "sim.write",
                resource: "sim:startup",
                risk_class: "synthetic",
                execution_semantics: "at_most_once",
                idempotency_key: None,
                preconditions: b"[]",
                dependencies: &dependencies,
                payload_hash: [6; 32],
                proposed_event_id: EventId(32),
                transition_id: EffectTransitionId(33),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(34),
                effect_id,
                expected_state: "proposed",
                next_state: "authorized",
                attempt_id: None,
                reason_code: Some("test_authorized"),
                evidence_ref: None,
                event_id: EventId(35),
            })
            .unwrap();
        effects
            .compare_and_swap(CompareAndSwapEffect {
                transition_id: EffectTransitionId(36),
                effect_id,
                expected_state: "authorized",
                next_state: "executing",
                attempt_id: None,
                reason_code: Some("incoherent_test"),
                evidence_ref: None,
                event_id: EventId(37),
            })
            .unwrap();
        drop(effects);

        assert!(matches!(
            KernelApi::open(&runtime, DenyByDefault),
            Err(KernelError::RecoveryRequired(ref report))
                if report.mode == RecoveryMode::RecoveryOnly
        ));
        let startup = start_kernel(&runtime, DenyByDefault).unwrap();
        assert!(matches!(startup, KernelStartup::RecoveryOnly(_)));
        assert_eq!(startup.report().mode, RecoveryMode::RecoveryOnly);
        drop(startup);
        fs::remove_dir_all(runtime.root).unwrap();
    }

    #[test]
    fn corrupted_authority_returns_quarantine_without_kernel() {
        let runtime = runtime();
        let authority = AuthorityLayout::initialize(&runtime).unwrap();
        let mut store = AuthorityStore::open(authority.authority_db_path()).unwrap();
        store
            .create_session(CreateSession {
                session_id: SessionId(1),
                event_id: EventId(2),
                owner_principal: "owner",
                actor_principal: "owner",
                recorded_at: "2026-08-25T11:10:00Z",
                payload: b"session",
                security_critical: true,
            })
            .unwrap();
        drop(store);
        let connection = Connection::open(authority.authority_db_path()).unwrap();
        connection
            .execute(
                "UPDATE session_events SET event_hash = ?1 WHERE event_id = ?2",
                params![vec![0xCC_u8; 32], 2_u128.to_be_bytes().to_vec()],
            )
            .unwrap();
        drop(connection);

        assert!(matches!(
            KernelApi::open(&runtime, DenyByDefault),
            Err(KernelError::RecoveryRequired(ref report))
                if report.mode == RecoveryMode::Quarantine
        ));
        let startup = start_kernel(&runtime, DenyByDefault).unwrap();
        assert!(matches!(startup, KernelStartup::Quarantined(_)));
        assert_eq!(startup.report().mode, RecoveryMode::Quarantine);
        drop(startup);
        fs::remove_dir_all(runtime.root).unwrap();
    }
}
