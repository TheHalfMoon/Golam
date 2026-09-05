#![forbid(unsafe_code)]
#![allow(dead_code)]
#![allow(clippy::redundant_guards)]

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
extern crate self as nix;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod fcntl {
    pub use golam_core::unix_fs::{OFlag, openat};
}
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
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
