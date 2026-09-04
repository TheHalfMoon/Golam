#![forbid(unsafe_code)]

pub mod errno {
    pub use golam_core::unix_fs::Errno;
}

pub mod fcntl {
    pub use golam_core::unix_fs::{AtFlags, OFlag, open, openat, renameat};
}

pub mod sys {
    pub mod stat {
        pub use golam_core::unix_fs::Mode;
    }
}

pub mod unistd {
    pub use golam_core::unix_fs::{UnlinkatFlags, linkat, unlinkat};
}
