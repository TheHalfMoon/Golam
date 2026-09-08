#![forbid(unsafe_code)]

use std::future::Future;
use std::time::Duration;

pub const MAX_BOUNDED_ASYNC_MILLIS: u64 = 30_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedAsyncError {
    InvalidTimeout,
    RuntimeUnavailable,
    TimedOut,
}

pub fn run_bounded<F, T>(timeout: Duration, future: F) -> Result<T, BoundedAsyncError>
where
    F: Future<Output = T>,
{
    let timeout_millis = u64::try_from(timeout.as_millis())
        .map_err(|_| BoundedAsyncError::InvalidTimeout)?;
    if timeout_millis == 0 || timeout_millis > MAX_BOUNDED_ASYNC_MILLIS {
        return Err(BoundedAsyncError::InvalidTimeout);
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|_| BoundedAsyncError::RuntimeUnavailable)?;
    runtime.block_on(async move {
        tokio::time::timeout(timeout, future)
            .await
            .map_err(|_| BoundedAsyncError::TimedOut)
    })
}
