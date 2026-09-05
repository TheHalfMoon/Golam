#![forbid(unsafe_code)]

//! Root-process ownership, bounded parent-side resource enforcement, and terminal reconciliation
//! for the first Spec 005 native profile.
//!
//! This module intentionally does not create or launch a process. T005-078 owns governed launch.
//! T005-072 freezes the supervision semantics that a later launcher must satisfy: one owned root
//! PID, spawn denial proven by the child-side containment receipt, bounded wall time and combined
//! stdout/stderr capture, cancellation as a non-terminal request, and terminal success only after
//! an exact operating-system terminal observation.

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
    pub wall_time_limit_ms: u64,
    pub max_stdout_stderr_bytes: u64,
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
        if self.wall_time_limit_ms == 0 || self.max_stdout_stderr_bytes == 0 {
            return Err(NativeProcessSupervisorError::InvalidContainmentBinding(
                "wall-time and output limits must be finite and nonzero",
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
pub enum RootTerminationRequestKind {
    Cancellation,
    WallTimeLimitExceeded,
    OutputLimitExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CancellationRequestEvidence {
    pub root_pid: u32,
    pub request_dispatched: bool,
    pub terminal_verified: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceLimitKind {
    WallTime,
    CombinedStdoutStderr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceEnforcementEvidence {
    pub root_pid: u32,
    pub limit_kind: ResourceLimitKind,
    pub limit: u64,
    pub observed: u64,
    pub limit_exceeded: bool,
    pub accepted_output_bytes: u64,
    pub termination_request_dispatched: bool,
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
    TerminationRequested(RootTerminationRequestKind),
    TerminalVerified(ProcessTreeTerminalEvidence),
    UnknownOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessTreeReconciliation {
    Unresolved {
        root_pid: u32,
        termination_request: Option<RootTerminationRequestKind>,
    },
    TerminalVerified(ProcessTreeTerminalEvidence),
}

/// Operating-system control is injected by the governed launcher at T005-078.
///
/// T005-072 defines the semantics without introducing a hidden process-launch path. The later
/// launcher must supply exact OS termination/terminal observation and feed monotonic elapsed time
/// plus every stdout/stderr byte through the methods on `RootProcessSupervisor` before retention.
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
    NonMonotonicWallTime { previous: u64, observed: u64 },
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
            Self::NonMonotonicWallTime { previous, observed } => write!(
                f,
                "native process wall-time observation moved backwards: previous {previous} ms, observed {observed} ms"
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
    last_elapsed_ms: u64,
    accepted_output_bytes: u64,
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
            last_elapsed_ms: 0,
            accepted_output_bytes: 0,
        })
    }

    pub fn state(&self) -> &RootProcessState {
        &self.state
    }

    pub const fn root_pid(&self) -> u32 {
        self.binding.root_pid
    }

    pub const fn accepted_output_bytes(&self) -> u64 {
        self.accepted_output_bytes
    }

    /// Dispatch an explicit cancellation request without converting that request into terminal
    /// proof. Repeated termination requests are idempotent while the same root remains unresolved.
    pub fn request_cancel(
        &mut self,
    ) -> Result<CancellationRequestEvidence, NativeProcessSupervisorError> {
        if matches!(self.state, RootProcessState::UnknownOutcome) {
            return Err(NativeProcessSupervisorError::UnknownOutcomeRequiresReconciliation);
        }
        if matches!(self.state, RootProcessState::TerminalVerified(_)) {
            return Ok(CancellationRequestEvidence {
                root_pid: self.binding.root_pid,
                request_dispatched: false,
                terminal_verified: true,
            });
        }

        let request_dispatched =
            self.dispatch_termination_if_running(RootTerminationRequestKind::Cancellation)?;
        Ok(CancellationRequestEvidence {
            root_pid: self.binding.root_pid,
            request_dispatched,
            terminal_verified: false,
        })
    }

    /// Enforce the bound wall clock using a monotonic elapsed-time observation supplied by the
    /// trusted parent launcher. Reaching the configured bound requests termination but is never
    /// itself terminal proof.
    pub fn observe_wall_time_ms(
        &mut self,
        elapsed_ms: u64,
    ) -> Result<ResourceEnforcementEvidence, NativeProcessSupervisorError> {
        if matches!(self.state, RootProcessState::UnknownOutcome) {
            return Err(NativeProcessSupervisorError::UnknownOutcomeRequiresReconciliation);
        }
        if elapsed_ms < self.last_elapsed_ms {
            let previous = self.last_elapsed_ms;
            self.state = RootProcessState::UnknownOutcome;
            return Err(NativeProcessSupervisorError::NonMonotonicWallTime {
                previous,
                observed: elapsed_ms,
            });
        }
        self.last_elapsed_ms = elapsed_ms;

        let limit = self.binding.wall_time_limit_ms;
        let limit_exceeded = elapsed_ms >= limit;
        let terminal_verified = matches!(self.state, RootProcessState::TerminalVerified(_));
        let termination_request_dispatched = if limit_exceeded && !terminal_verified {
            self.dispatch_termination_if_running(RootTerminationRequestKind::WallTimeLimitExceeded)?
        } else {
            false
        };

        Ok(ResourceEnforcementEvidence {
            root_pid: self.binding.root_pid,
            limit_kind: ResourceLimitKind::WallTime,
            limit,
            observed: elapsed_ms,
            limit_exceeded,
            accepted_output_bytes: self.accepted_output_bytes,
            termination_request_dispatched,
            terminal_verified,
        })
    }

    /// Account a stdout/stderr chunk before the caller retains it. A chunk that would exceed the
    /// combined output bound is rejected in full and requests termination. Saturating arithmetic
    /// makes counter overflow conservatively exceed every finite profile bound.
    pub fn account_output_bytes(
        &mut self,
        chunk_bytes: u64,
    ) -> Result<ResourceEnforcementEvidence, NativeProcessSupervisorError> {
        if matches!(self.state, RootProcessState::UnknownOutcome) {
            return Err(NativeProcessSupervisorError::UnknownOutcomeRequiresReconciliation);
        }

        let limit = self.binding.max_stdout_stderr_bytes;
        let prospective = self.accepted_output_bytes.saturating_add(chunk_bytes);
        let limit_exceeded = prospective > limit;
        let terminal_verified = matches!(self.state, RootProcessState::TerminalVerified(_));
        let termination_request_dispatched = if limit_exceeded && !terminal_verified {
            self.dispatch_termination_if_running(RootTerminationRequestKind::OutputLimitExceeded)?
        } else {
            false
        };

        if !limit_exceeded {
            self.accepted_output_bytes = prospective;
        }

        Ok(ResourceEnforcementEvidence {
            root_pid: self.binding.root_pid,
            limit_kind: ResourceLimitKind::CombinedStdoutStderr,
            limit,
            observed: prospective,
            limit_exceeded,
            accepted_output_bytes: self.accepted_output_bytes,
            termination_request_dispatched,
            terminal_verified,
        })
    }

    fn dispatch_termination_if_running(
        &mut self,
        kind: RootTerminationRequestKind,
    ) -> Result<bool, NativeProcessSupervisorError> {
        match self.state {
            RootProcessState::UnknownOutcome => {
                return Err(NativeProcessSupervisorError::UnknownOutcomeRequiresReconciliation);
            }
            RootProcessState::TerminalVerified(_) | RootProcessState::TerminationRequested(_) => {
                return Ok(false);
            }
            RootProcessState::Running => {}
        }

        if let Err(error) = self.control.request_termination(self.binding.root_pid) {
            self.state = RootProcessState::UnknownOutcome;
            return Err(NativeProcessSupervisorError::Control(error));
        }
        self.state = RootProcessState::TerminationRequested(kind);
        Ok(true)
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
            let termination_request = match self.state {
                RootProcessState::TerminationRequested(kind) => Some(kind),
                _ => None,
            };
            return Ok(ProcessTreeReconciliation::Unresolved {
                root_pid: self.binding.root_pid,
                termination_request,
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
        termination_calls: usize,
        terminal: Option<RootTerminalObservation>,
        termination_error: Option<String>,
        observe_error: Option<String>,
    }

    impl RootProcessControl for FakeControl {
        fn request_termination(&mut self, _root_pid: u32) -> Result<(), String> {
            self.termination_calls += 1;
            match self.termination_error.take() {
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
            wall_time_limit_ms: 100,
            max_stdout_stderr_bytes: 128,
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
                termination_request: Some(RootTerminationRequestKind::Cancellation),
            }
        );
    }

    #[test]
    fn repeated_cancel_is_idempotent_and_does_not_redispatch() {
        let mut supervisor = RootProcessSupervisor::new(binding(), FakeControl::default()).unwrap();
        assert!(supervisor.request_cancel().unwrap().request_dispatched);
        assert!(!supervisor.request_cancel().unwrap().request_dispatched);
        assert_eq!(supervisor.into_control().termination_calls, 1);
    }

    #[test]
    fn wall_time_bound_dispatches_termination_without_terminal_claim() {
        let mut supervisor = RootProcessSupervisor::new(binding(), FakeControl::default()).unwrap();
        let before = supervisor.observe_wall_time_ms(99).unwrap();
        assert!(!before.limit_exceeded);
        assert!(!before.termination_request_dispatched);

        let at_limit = supervisor.observe_wall_time_ms(100).unwrap();
        assert!(at_limit.limit_exceeded);
        assert!(at_limit.termination_request_dispatched);
        assert!(!at_limit.terminal_verified);
        assert_eq!(
            supervisor.state(),
            &RootProcessState::TerminationRequested(
                RootTerminationRequestKind::WallTimeLimitExceeded
            )
        );
        assert_eq!(supervisor.into_control().termination_calls, 1);
    }

    #[test]
    fn non_monotonic_wall_time_becomes_unknown_outcome() {
        let mut supervisor = RootProcessSupervisor::new(binding(), FakeControl::default()).unwrap();
        supervisor.observe_wall_time_ms(50).unwrap();
        assert!(matches!(
            supervisor.observe_wall_time_ms(49),
            Err(NativeProcessSupervisorError::NonMonotonicWallTime { .. })
        ));
        assert_eq!(supervisor.state(), &RootProcessState::UnknownOutcome);
    }

    #[test]
    fn output_budget_accepts_exact_cap_and_rejects_overflow_chunk() {
        let mut supervisor = RootProcessSupervisor::new(binding(), FakeControl::default()).unwrap();
        let first = supervisor.account_output_bytes(64).unwrap();
        assert!(!first.limit_exceeded);
        assert_eq!(first.accepted_output_bytes, 64);

        let exact = supervisor.account_output_bytes(64).unwrap();
        assert!(!exact.limit_exceeded);
        assert_eq!(exact.accepted_output_bytes, 128);

        let overflow = supervisor.account_output_bytes(1).unwrap();
        assert!(overflow.limit_exceeded);
        assert!(overflow.termination_request_dispatched);
        assert_eq!(overflow.accepted_output_bytes, 128);
        assert_eq!(supervisor.accepted_output_bytes(), 128);
        assert_eq!(
            supervisor.state(),
            &RootProcessState::TerminationRequested(
                RootTerminationRequestKind::OutputLimitExceeded
            )
        );
        assert_eq!(supervisor.into_control().termination_calls, 1);
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
            termination_error: Some("kill boundary ambiguous".to_owned()),
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
    fn incomplete_containment_receipt_or_resource_binding_cannot_mint_supervision_claim() {
        let mut invalid = binding();
        invalid.seccomp_tsync_installed = false;
        assert!(matches!(
            RootProcessSupervisor::new(invalid, FakeControl::default()),
            Err(NativeProcessSupervisorError::InvalidContainmentBinding(_))
        ));

        let mut invalid = binding();
        invalid.max_stdout_stderr_bytes = 0;
        assert!(matches!(
            RootProcessSupervisor::new(invalid, FakeControl::default()),
            Err(NativeProcessSupervisorError::InvalidContainmentBinding(_))
        ));
    }
}
