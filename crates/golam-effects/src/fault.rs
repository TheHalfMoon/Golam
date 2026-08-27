#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use crate::{EffectHandler, EffectStatus, HandlerIntent, HandlerOutcome};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionBoundary {
    BeforeCommit,
    AfterCommit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SimulatedRemoteBoundary {
    BeforeAccept,
    AfterAccept,
    BeforeAck,
    AfterAck,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaultPoint {
    DurableTransition {
        from: Option<EffectStatus>,
        to: EffectStatus,
        boundary: TransitionBoundary,
    },
    SimulatedRemote(SimulatedRemoteBoundary),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InjectedCrash {
    pub point: FaultPoint,
}

impl fmt::Display for InjectedCrash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "injected crash at {:?}", self.point)
    }
}

impl Error for InjectedCrash {}

pub trait FaultInjector {
    fn check(&mut self, point: FaultPoint) -> Result<(), InjectedCrash>;
}

#[derive(Default)]
pub struct NoFaults;

impl FaultInjector for NoFaults {
    fn check(&mut self, _point: FaultPoint) -> Result<(), InjectedCrash> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CrashOnce {
    target: FaultPoint,
    fired: bool,
}

impl CrashOnce {
    pub const fn new(target: FaultPoint) -> Self {
        Self {
            target,
            fired: false,
        }
    }

    pub const fn fired(&self) -> bool {
        self.fired
    }
}

impl FaultInjector for CrashOnce {
    fn check(&mut self, point: FaultPoint) -> Result<(), InjectedCrash> {
        if !self.fired && point == self.target {
            self.fired = true;
            Err(InjectedCrash { point })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum FaultedOperation<E> {
    Injected(InjectedCrash),
    Operation(E),
}

impl<E: fmt::Display> fmt::Display for FaultedOperation<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Injected(crash) => crash.fmt(f),
            Self::Operation(error) => write!(f, "fault-wrapped operation failed: {error}"),
        }
    }
}

impl<E: Error + 'static> Error for FaultedOperation<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Injected(crash) => Some(crash),
            Self::Operation(error) => Some(error),
        }
    }
}

pub trait FaultInjectableEffectHandler: EffectHandler {
    fn execute_with_faults<I: FaultInjector>(
        &mut self,
        intent: &HandlerIntent<'_>,
        injector: &mut I,
    ) -> Result<HandlerOutcome, InjectedCrash>;
}

pub fn run_durable_transition<I, F, T, E>(
    injector: &mut I,
    from: Option<EffectStatus>,
    to: EffectStatus,
    commit: F,
) -> Result<T, FaultedOperation<E>>
where
    I: FaultInjector,
    F: FnOnce() -> Result<T, E>,
{
    injector
        .check(FaultPoint::DurableTransition {
            from,
            to,
            boundary: TransitionBoundary::BeforeCommit,
        })
        .map_err(FaultedOperation::Injected)?;
    let value = commit().map_err(FaultedOperation::Operation)?;
    injector
        .check(FaultPoint::DurableTransition {
            from,
            to,
            boundary: TransitionBoundary::AfterCommit,
        })
        .map_err(FaultedOperation::Injected)?;
    Ok(value)
}

pub const PLANNED_DURABLE_TRANSITIONS: &[(Option<EffectStatus>, EffectStatus)] = &[
    (None, EffectStatus::Proposed),
    (Some(EffectStatus::Proposed), EffectStatus::Denied),
    (Some(EffectStatus::Proposed), EffectStatus::Authorized),
    (
        Some(EffectStatus::Authorized),
        EffectStatus::ApprovalRequired,
    ),
    (Some(EffectStatus::Authorized), EffectStatus::Executing),
    (
        Some(EffectStatus::ApprovalRequired),
        EffectStatus::Authorized,
    ),
    (Some(EffectStatus::ApprovalRequired), EffectStatus::Denied),
    (Some(EffectStatus::Executing), EffectStatus::Succeeded),
    (Some(EffectStatus::Executing), EffectStatus::Failed),
    (Some(EffectStatus::Executing), EffectStatus::UnknownOutcome),
    (
        Some(EffectStatus::UnknownOutcome),
        EffectStatus::Reconciling,
    ),
    (
        Some(EffectStatus::UnknownOutcome),
        EffectStatus::ManualReview,
    ),
    (Some(EffectStatus::Reconciling), EffectStatus::Succeeded),
    (Some(EffectStatus::Reconciling), EffectStatus::Failed),
    (Some(EffectStatus::Reconciling), EffectStatus::ManualReview),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transition_allowed;

    #[test]
    fn every_planned_durable_transition_can_crash_before_and_after_commit() {
        for &(from, to) in PLANNED_DURABLE_TRANSITIONS {
            if let Some(from) = from {
                assert!(transition_allowed(from, to));
            } else {
                assert_eq!(to, EffectStatus::Proposed);
            }

            for boundary in [
                TransitionBoundary::BeforeCommit,
                TransitionBoundary::AfterCommit,
            ] {
                let target = FaultPoint::DurableTransition { from, to, boundary };
                let mut injector = CrashOnce::new(target);
                let mut committed = false;
                let result = run_durable_transition(&mut injector, from, to, || {
                    committed = true;
                    Ok::<_, ()>(())
                });
                assert!(matches!(
                    result,
                    Err(FaultedOperation::Injected(InjectedCrash { point })) if point == target
                ));
                assert!(injector.fired());
                assert_eq!(committed, boundary == TransitionBoundary::AfterCommit);
            }
        }
    }

    #[test]
    fn crash_once_only_fires_at_its_exact_target() {
        let target = FaultPoint::SimulatedRemote(SimulatedRemoteBoundary::AfterAccept);
        let mut injector = CrashOnce::new(target);
        assert!(
            injector
                .check(FaultPoint::SimulatedRemote(
                    SimulatedRemoteBoundary::BeforeAccept
                ))
                .is_ok()
        );
        assert!(injector.check(target).is_err());
        assert!(injector.check(target).is_ok());
    }
}
