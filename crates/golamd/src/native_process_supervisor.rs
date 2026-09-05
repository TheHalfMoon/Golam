#![forbid(unsafe_code)]

//! Root-process ownership and terminal reconciliation for the first Spec 005 native profile.
//!
//! This module intentionally does not create or launch a process. T005-078 owns governed launch.
//! T005-072 freezes the supervision semantics that a later launcher must satisfy: one owned root
//! PID, spawn denial proven by the child-side containment receipt, cancellation as a non-terminal
//! request, and terminal success only after an exact operating-system terminal observation.

use std::error::Error;
use std::fmt;

pub const LINUX_PROFILE_TOKEN: &str = "platform:linux-x86_64-landlock-v4-seccomp-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootContainmentBinding {
    pub profile_token: String,
    pub root_pid: u32,
    pub landlock_ruleset_fully_enforced: bool,
    pub no_new_privs: bool,
    pub seccomp_tsync_installed: bool,
    pub spawn_denied: bool,
    pub strict_local: bool,
}

impl RootContainmentBinding {
    pub fn validate(&self) -> Result<(), NativeProcessSupervisorError> {
        if self.profile_token != LINUX_PROFILE_TOKEN {
            return Err(NativeProcessSupervisorError::InvalidContainmentBinding(
                "profile token mismatch",
            ));
        }
        if self.root_pid == 0 {
            return Err(NativeProcessSupervisorError::InvalidContainmentBinding(
                "root pid must be nonzero",
            ));
        }
        if !self.landlock_ruleset_fully_enforced
            || !self.no_new_privs
            || !self.seccomp_tsync_installed
            || !self.spawn_denied
            || !self.strict_local
        {
            return Err(NativeProcessSupervisorError::InvalidContainmentBinding(
                "root process lacks the exact required containment receipt",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootTerminationKind {
    Exited(i32),
    Signaled(i32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RootTerminalObservation {
    pub root_pid: u32,
    pub termination: RootTerminationKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CancellationRequestEvidence {
    pub root_pid: u32,
    pub request_dispatched: bool,
    pub terminal_verified: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessTreeTerminalEvidence {
    pub root_pid: u32,
    pub termination: RootTerminationKind,
    pub observed_descendant_count: u32,
    pub spawn_denial_bound: bool,
    pub terminal_verified: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RootProcessState {
    Running,
    CancellationRequested,
    TerminalVerified(ProcessTreeTerminalEvidence),
    UnknownOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessTreeReconciliation {
    Unresolved {
        root_pid: u32,
        cancellation_requested: bool,
    },
    TerminalVerified(ProcessTreeTerminalEvidence),
}

/// Operating-system control is injected by the governed launcher at T005-078.
///
/// T005-072 defines the semantics without introducing a hidden process-launch path.
pub trait RootProcessControl {
    fn request_termination(&mut self, root_pid: u32) -> Result<(), String>;

    fn observe_terminal(
        &mut self,
        root_pid: u32,
    ) -> Result<Option<RootTerminalObservation>, String>;
}

#[derive(Debug)]
pub enum NativeProcessSupervisorError {
    InvalidContainmentBinding(&'static str),
    Control(String),
    TerminalPidMismatch { expected: u32, observed: u32 },
    UnknownOutcomeRequiresReconciliation,
}

impl fmt::Display for NativeProcessSupervisorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContainmentBinding(reason) => {
                write!(f, "native process containment binding is invalid: {reason}")
            }
            Self::Control(error) => write!(f, "native process control failed: {error}"),
            Self::TerminalPidMismatch { expected, observed } => write!(
                f,
                "native process terminal observation pid mismatch: expected {expected}, observed {observed}"
            ),
            Self::UnknownOutcomeRequiresReconciliation => f.write_str(
                "native process supervision is in UNKNOWN_OUTCOME and requires reconciliation",
            ),
        }
    }
}

impl Error for NativeProcessSupervisorError {}

pub struct RootProcessSupervisor<C> {
    binding: RootContainmentBinding,
    control: C,
    state: RootProcessState,
}

impl<C: RootProcessControl> RootProcessSupervisor<C> {
    pub fn new(
        binding: RootContainmentBinding,
        control: C,
    ) -> Result<Self, NativeProcessSupervisorError> {
        binding.validate()?;
        Ok(Self {
            binding,
            control,
            state: RootProcessState::Running,
        })
    }

    pub fn state(&self) -> &RootProcessState {
        &self.state
    }

    pub const fn root_pid(&self) -> u32 {
        self.binding.root_pid
    }

    /// Dispatch a cancellation request without converting that request into terminal proof.
    ///
    /// Repeated cancellation is idempotent while the same root remains unresolved.
    pub fn request_cancel(
        &mut self,
    ) -> Result<CancellationRequestEvidence, NativeProcessSupervisorError> {
        match self.state {
            RootProcessState::UnknownOutcome => {
                return Err(NativeProcessSupervisorError::UnknownOutcomeRequiresReconciliation);
            }
            RootProcessState::TerminalVerified(_) => {
                return Ok(CancellationRequestEvidence {
                    root_pid: self.binding.root_pid,
                    request_dispatched: false,
                    terminal_verified: true,
                });
            }
            RootProcessState::CancellationRequested => {
                return Ok(CancellationRequestEvidence {
                    root_pid: self.binding.root_pid,
                    request_dispatched: false,
                    terminal_verified: false,
                });
            }
            RootProcessState::Running => {}
        }

        if let Err(error) = self.control.request_termination(self.binding.root_pid) {
            self.state = RootProcessState::UnknownOutcome;
            return Err(NativeProcessSupervisorError::Control(error));
        }
        self.state = RootProcessState::CancellationRequested;
        Ok(CancellationRequestEvidence {
            root_pid: self.binding.root_pid,
            request_dispatched: true,
            terminal_verified: false,
        })
    }

    /// Reconcile the exact owned root against operating-system terminal evidence.
    ///
    /// The first profile denies process creation before untrusted execution. Therefore an exact
    /// terminal observation for the owned root, combined with the bound spawn-denial receipt,
    /// proves the profile's process tree terminal with an observed descendant count of zero.
    pub fn reconcile_terminal(
        &mut self,
    ) -> Result<ProcessTreeReconciliation, NativeProcessSupervisorError> {
        if let RootProcessState::TerminalVerified(evidence) = self.state {
            return Ok(ProcessTreeReconciliation::TerminalVerified(evidence));
        }

        let observation = match self.control.observe_terminal(self.binding.root_pid) {
            Ok(observation) => observation,
            Err(error) => {
                self.state = RootProcessState::UnknownOutcome;
                return Err(NativeProcessSupervisorError::Control(error));
            }
        };

        let Some(observation) = observation else {
            return Ok(ProcessTreeReconciliation::Unresolved {
                root_pid: self.binding.root_pid,
                cancellation_requested: matches!(
                    self.state,
                    RootProcessState::CancellationRequested
                ),
            });
        };

        if observation.root_pid != self.binding.root_pid {
            self.state = RootProcessState::UnknownOutcome;
            return Err(NativeProcessSupervisorError::TerminalPidMismatch {
                expected: self.binding.root_pid,
                observed: observation.root_pid,
            });
        }

        let evidence = ProcessTreeTerminalEvidence {
            root_pid: observation.root_pid,
            termination: observation.termination,
            observed_descendant_count: 0,
            spawn_denial_bound: self.binding.spawn_denied && self.binding.seccomp_tsync_installed,
            terminal_verified: true,
        };
        self.state = RootProcessState::TerminalVerified(evidence);
        Ok(ProcessTreeReconciliation::TerminalVerified(evidence))
    }

    pub fn into_control(self) -> C {
        self.control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeControl {
        cancel_calls: usize,
        terminal: Option<RootTerminalObservation>,
        cancel_error: Option<String>,
        observe_error: Option<String>,
    }

    impl RootProcessControl for FakeControl {
        fn request_termination(&mut self, _root_pid: u32) -> Result<(), String> {
            self.cancel_calls += 1;
            match self.cancel_error.take() {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }

        fn observe_terminal(
            &mut self,
            _root_pid: u32,
        ) -> Result<Option<RootTerminalObservation>, String> {
            match self.observe_error.take() {
                Some(error) => Err(error),
                None => Ok(self.terminal),
            }
        }
    }

    fn binding() -> RootContainmentBinding {
        RootContainmentBinding {
            profile_token: LINUX_PROFILE_TOKEN.to_owned(),
            root_pid: 4242,
            landlock_ruleset_fully_enforced: true,
            no_new_privs: true,
            seccomp_tsync_installed: true,
            spawn_denied: true,
            strict_local: true,
        }
    }

    #[test]
    fn cancellation_request_is_not_terminal_proof() {
        let mut supervisor = RootProcessSupervisor::new(binding(), FakeControl::default()).unwrap();
        let cancellation = supervisor.request_cancel().unwrap();
        assert!(cancellation.request_dispatched);
        assert!(!cancellation.terminal_verified);
        assert_eq!(
            supervisor.reconcile_terminal().unwrap(),
            ProcessTreeReconciliation::Unresolved {
                root_pid: 4242,
                cancellation_requested: true,
            }
        );
    }

    #[test]
    fn repeated_cancel_is_idempotent_and_does_not_redispatch() {
        let mut supervisor = RootProcessSupervisor::new(binding(), FakeControl::default()).unwrap();
        assert!(supervisor.request_cancel().unwrap().request_dispatched);
        assert!(!supervisor.request_cancel().unwrap().request_dispatched);
        assert_eq!(supervisor.into_control().cancel_calls, 1);
    }

    #[test]
    fn exact_root_terminal_observation_proves_spawn_denied_tree_terminal() {
        let control = FakeControl {
            terminal: Some(RootTerminalObservation {
                root_pid: 4242,
                termination: RootTerminationKind::Exited(0),
            }),
            ..FakeControl::default()
        };
        let mut supervisor = RootProcessSupervisor::new(binding(), control).unwrap();
        let result = supervisor.reconcile_terminal().unwrap();
        assert_eq!(
            result,
            ProcessTreeReconciliation::TerminalVerified(ProcessTreeTerminalEvidence {
                root_pid: 4242,
                termination: RootTerminationKind::Exited(0),
                observed_descendant_count: 0,
                spawn_denial_bound: true,
                terminal_verified: true,
            })
        );
    }

    #[test]
    fn mismatched_terminal_pid_becomes_unknown_outcome() {
        let control = FakeControl {
            terminal: Some(RootTerminalObservation {
                root_pid: 9999,
                termination: RootTerminationKind::Exited(0),
            }),
            ..FakeControl::default()
        };
        let mut supervisor = RootProcessSupervisor::new(binding(), control).unwrap();
        assert!(matches!(
            supervisor.reconcile_terminal(),
            Err(NativeProcessSupervisorError::TerminalPidMismatch { .. })
        ));
        assert_eq!(supervisor.state(), &RootProcessState::UnknownOutcome);
    }

    #[test]
    fn control_failure_never_becomes_success() {
        let control = FakeControl {
            cancel_error: Some("kill boundary ambiguous".to_owned()),
            ..FakeControl::default()
        };
        let mut supervisor = RootProcessSupervisor::new(binding(), control).unwrap();
        assert!(matches!(
            supervisor.request_cancel(),
            Err(NativeProcessSupervisorError::Control(_))
        ));
        assert_eq!(supervisor.state(), &RootProcessState::UnknownOutcome);
    }

    #[test]
    fn incomplete_containment_receipt_cannot_mint_supervision_claim() {
        let mut invalid = binding();
        invalid.seccomp_tsync_installed = false;
        assert!(matches!(
            RootProcessSupervisor::new(invalid, FakeControl::default()),
            Err(NativeProcessSupervisorError::InvalidContainmentBinding(_))
        ));
    }
}
