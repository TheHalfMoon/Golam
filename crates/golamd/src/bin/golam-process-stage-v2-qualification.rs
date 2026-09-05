#![forbid(unsafe_code)]

#[cfg(unix)]
extern crate self as nix;

#[cfg(unix)]
mod errno {
    pub use golam_core::unix_fs::Errno;
}
#[cfg(unix)]
mod fcntl {
    pub use golam_core::unix_fs::{OFlag, openat};
}
#[cfg(unix)]
mod sys {
    pub mod stat {
        pub use golam_core::unix_fs::Mode;
    }
}

#[path = "../local_fs.rs"]
mod local_fs;
#[path = "../process_execution_v2.rs"]
mod process_execution_v2;
#[path = "../static_elf_v2.rs"]
mod static_elf_v2;

fn main() {
    // Compile/test carrier only. Product daemon wiring follows exact-head qualification.
}
