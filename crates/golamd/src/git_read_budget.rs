#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::time::{Duration, Instant};

pub const DECOMPRESSION_INPUT_QUANTUM_BYTES: usize = 64 * 1024;
pub const DECOMPRESSION_OUTPUT_QUANTUM_BYTES: usize = 64 * 1024;

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
