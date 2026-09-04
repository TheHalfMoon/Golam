#![forbid(unsafe_code)]

#[cfg(unix)]
pub use nix::errno::Errno;
#[cfg(unix)]
pub use nix::fcntl::{AtFlags, OFlag, open, openat, renameat};
#[cfg(unix)]
pub use nix::sys::stat::Mode;
#[cfg(unix)]
pub use nix::unistd::{UnlinkatFlags, linkat, unlinkat};
