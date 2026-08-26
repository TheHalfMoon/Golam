#![forbid(unsafe_code)]

use std::io::{self, Read, Write};
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(2);

/// Bounds a nonblocking local IPC stream by one absolute connection deadline.
///
/// Spec 002's CLI protocol is deliberately short-lived request/response IPC.
/// Bounding the whole connection is stronger than a handshake-only deadline:
/// a client cannot hold the single synchronous daemon connection slot forever
/// either before authentication or after READY.
pub struct DeadlineIo<S> {
    inner: S,
    deadline: Instant,
}

impl<S> DeadlineIo<S> {
    pub fn new(inner: S, lifetime: Duration) -> Self {
        Self {
            inner,
            deadline: Instant::now() + lifetime,
        }
    }

    fn remaining(&self) -> io::Result<Duration> {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Golam local IPC connection deadline exceeded",
                )
            })
    }

    fn wait_for_progress(&self) -> io::Result<()> {
        let remaining = self.remaining()?;
        thread::sleep(remaining.min(POLL_INTERVAL));
        Ok(())
    }
}

impl<S: Read> Read for DeadlineIo<S> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        loop {
            match self.inner.read(buffer) {
                Ok(read) => return Ok(read),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    self.wait_for_progress()?;
                }
                Err(error) => return Err(error),
            }
        }
    }
}

impl<S: Write> Write for DeadlineIo<S> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        loop {
            match self.inner.write(buffer) {
                Ok(written) => return Ok(written),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    self.wait_for_progress()?;
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        // UnixStream and Windows named-pipe writes already hand bytes to the
        // kernel. A synchronous named-pipe FlushFileBuffers-style wait can be
        // held forever by a peer that stops reading, defeating this deadline.
        // The bounded writer therefore treats protocol flush as a logical
        // no-op; subsequent reads/replies provide protocol-level settlement.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct WouldBlockForever;

    impl Read for WouldBlockForever {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::WouldBlock))
        }
    }

    impl Write for WouldBlockForever {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::WouldBlock))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn silent_peer_cannot_hold_connection_forever() {
        let mut io = DeadlineIo::new(WouldBlockForever, Duration::from_millis(10));
        let started = Instant::now();
        let error = io.read(&mut [0_u8; 1]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn blocked_writer_is_bounded_by_same_deadline() {
        let mut io = DeadlineIo::new(WouldBlockForever, Duration::from_millis(10));
        let error = io.write(b"challenge").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }
}
