#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::time::{Duration, Instant};

pub const DECOMPRESSION_INPUT_QUANTUM_BYTES: usize = 64 * 1024;
pub const DECOMPRESSION_OUTPUT_QUANTUM_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitOperationDeadline {
    started: Instant,
    max_duration: Duration,
}

impl GitOperationDeadline {
    pub fn start(max_duration: Duration) -> Result<Self, GitOperationBudgetError> {
        if max_duration.is_zero() {
            return Err(GitOperationBudgetError::InvalidDuration);
        }
        Ok(Self {
            started: Instant::now(),
            max_duration,
        })
    }

    pub fn require_active(&self) -> Result<(), GitOperationBudgetError> {
        if self.started.elapsed() >= self.max_duration {
            return Err(GitOperationBudgetError::DeadlineExceeded);
        }
        Ok(())
    }

    pub fn remaining(&self) -> Result<Duration, GitOperationBudgetError> {
        self.max_duration
            .checked_sub(self.started.elapsed())
            .filter(|remaining| !remaining.is_zero())
            .ok_or(GitOperationBudgetError::DeadlineExceeded)
    }

    /// Run one synchronous, non-preemptive Git operation step under the same
    /// absolute operation deadline. The result is discarded if the step
    /// returns after the shared deadline has expired.
    pub fn run_step<T>(
        &self,
        step: impl FnOnce() -> T,
    ) -> Result<T, GitOperationBudgetError> {
        self.require_active()?;
        let result = step();
        self.require_active()?;
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitOperationBudgetError {
    InvalidDuration,
    DeadlineExceeded,
}

impl fmt::Display for GitOperationBudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDuration => f.write_str("Git operation duration must be positive"),
            Self::DeadlineExceeded => f.write_str("Git operation deadline exceeded"),
        }
    }
}

impl Error for GitOperationBudgetError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecompressionDeadline {
    started: Instant,
    max_duration: Duration,
}

impl DecompressionDeadline {
    pub fn start(max_duration: Duration) -> Result<Self, DecompressionBudgetError> {
        if max_duration.is_zero() {
            return Err(DecompressionBudgetError::InvalidDuration);
        }
        Ok(Self {
            started: Instant::now(),
            max_duration,
        })
    }

    pub fn from_operation(operation: GitOperationDeadline) -> Self {
        Self {
            started: operation.started,
            max_duration: operation.max_duration,
        }
    }

    pub fn run_quantum<T>(
        &self,
        input: &[u8],
        output: &mut [u8],
        inflate_call: impl FnOnce(&[u8], &mut [u8]) -> T,
    ) -> Result<T, DecompressionBudgetError> {
        if input.len() > DECOMPRESSION_INPUT_QUANTUM_BYTES
            || output.len() > DECOMPRESSION_OUTPUT_QUANTUM_BYTES
        {
            return Err(DecompressionBudgetError::QuantumExceeded);
        }

        self.require_before_call()?;
        let result = inflate_call(input, output);
        self.require_after_call()?;
        Ok(result)
    }

    fn require_before_call(&self) -> Result<(), DecompressionBudgetError> {
        if self.started.elapsed() >= self.max_duration {
            return Err(DecompressionBudgetError::DeadlineExceededBeforeCall);
        }
        Ok(())
    }

    fn require_after_call(&self) -> Result<(), DecompressionBudgetError> {
        if self.started.elapsed() >= self.max_duration {
            return Err(DecompressionBudgetError::DeadlineExceededAfterCall);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecompressionBudgetError {
    InvalidDuration,
    QuantumExceeded,
    DeadlineExceededBeforeCall,
    DeadlineExceededAfterCall,
}

impl fmt::Display for DecompressionBudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDuration => f.write_str("decompression duration must be positive"),
            Self::QuantumExceeded => f.write_str("decompression synchronous work quantum exceeded"),
            Self::DeadlineExceededBeforeCall => {
                f.write_str("decompression deadline expired before synchronous call")
            }
            Self::DeadlineExceededAfterCall => {
                f.write_str("decompression deadline expired during non-preemptive synchronous call")
            }
        }
    }
}

impl Error for DecompressionBudgetError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    #[test]
    fn operation_deadline_rejects_zero_duration_and_expires_monotonically() {
        assert_eq!(
            GitOperationDeadline::start(Duration::ZERO),
            Err(GitOperationBudgetError::InvalidDuration)
        );

        let deadline = GitOperationDeadline::start(Duration::from_millis(20)).unwrap();
        assert!(deadline.remaining().is_ok());
        thread::sleep(Duration::from_millis(30));
        assert_eq!(
            deadline.require_active(),
            Err(GitOperationBudgetError::DeadlineExceeded)
        );
        assert_eq!(
            deadline.remaining(),
            Err(GitOperationBudgetError::DeadlineExceeded)
        );
    }

    #[test]
    fn operation_step_rejects_expiry_before_invocation() {
        let deadline = GitOperationDeadline::start(Duration::from_millis(20)).unwrap();
        thread::sleep(Duration::from_millis(30));
        let invoked = AtomicBool::new(false);

        let result = deadline.run_step(|| {
            invoked.store(true, Ordering::Relaxed);
        });

        assert_eq!(result, Err(GitOperationBudgetError::DeadlineExceeded));
        assert!(!invoked.load(Ordering::Relaxed));
    }

    #[test]
    fn operation_step_discards_nonpreemptive_overrun() {
        let deadline = GitOperationDeadline::start(Duration::from_millis(20)).unwrap();

        let result = deadline.run_step(|| {
            thread::sleep(Duration::from_millis(30));
            7_u8
        });

        assert_eq!(result, Err(GitOperationBudgetError::DeadlineExceeded));
    }

    #[test]
    fn decompression_deadline_can_share_the_operation_absolute_budget() {
        let operation = GitOperationDeadline::start(Duration::from_millis(20)).unwrap();
        thread::sleep(Duration::from_millis(30));
        let deadline = DecompressionDeadline::from_operation(operation);
        let invoked = AtomicBool::new(false);
        let mut output = [0_u8; 1];

        let result = deadline.run_quantum(b"a", &mut output, |_, _| {
            invoked.store(true, Ordering::Relaxed);
        });

        assert_eq!(
            result,
            Err(DecompressionBudgetError::DeadlineExceededBeforeCall)
        );
        assert!(!invoked.load(Ordering::Relaxed));
    }

    #[test]
    fn oversized_synchronous_quantum_is_rejected_before_call() {
        let deadline = DecompressionDeadline::start(Duration::from_secs(1)).unwrap();
        let invoked = AtomicBool::new(false);
        let input = vec![0_u8; DECOMPRESSION_INPUT_QUANTUM_BYTES + 1];
        let mut output = [0_u8; 1];

        let result = deadline.run_quantum(&input, &mut output, |_, _| {
            invoked.store(true, Ordering::Relaxed);
        });

        assert_eq!(result, Err(DecompressionBudgetError::QuantumExceeded));
        assert!(!invoked.load(Ordering::Relaxed));
    }

    #[test]
    fn deadline_expiry_between_chunks_rejects_next_call() {
        let deadline = DecompressionDeadline::start(Duration::from_millis(20)).unwrap();
        let mut output = [0_u8; 1];
        deadline.run_quantum(b"a", &mut output, |_, _| ()).unwrap();
        thread::sleep(Duration::from_millis(30));

        let invoked = AtomicBool::new(false);
        let result = deadline.run_quantum(b"b", &mut output, |_, _| {
            invoked.store(true, Ordering::Relaxed);
        });

        assert_eq!(
            result,
            Err(DecompressionBudgetError::DeadlineExceededBeforeCall)
        );
        assert!(!invoked.load(Ordering::Relaxed));
    }

    #[test]
    fn in_flight_overrun_is_detected_after_non_preemptive_call() {
        let deadline = DecompressionDeadline::start(Duration::from_millis(20)).unwrap();
        let mut output = [0_u8; 1];

        let result = deadline.run_quantum(b"a", &mut output, |_, _| {
            thread::sleep(Duration::from_millis(30));
        });

        assert_eq!(
            result,
            Err(DecompressionBudgetError::DeadlineExceededAfterCall)
        );
    }
}
