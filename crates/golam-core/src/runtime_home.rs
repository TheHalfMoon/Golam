#![forbid(unsafe_code)]

use std::env;
use std::io;
use std::path::PathBuf;

const GOLAM_ROOT_ENV: &str = "GOLAM_ROOT";

pub fn default_runtime_root() -> io::Result<PathBuf> {
    if let Some(root) = env_path(GOLAM_ROOT_ENV) {
        return Ok(root);
    }

    #[cfg(windows)]
    {
        if let Some(local_app_data) = env_path("LOCALAPPDATA") {
            return Ok(local_app_data.join("Golam"));
        }
        if let Some(profile) = env_path("USERPROFILE") {
            return Ok(profile.join("AppData").join("Local").join("Golam"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = env_path("HOME") {
            return Ok(home
                .join("Library")
                .join("Application Support")
                .join("Golam"));
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(state_home) = env_path("XDG_STATE_HOME") {
            return Ok(state_home.join("golam"));
        }
        if let Some(home) = env_path("HOME") {
            return Ok(home.join(".local").join("state").join("golam"));
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        if let Some(home) = env_path("HOME") {
            return Ok(home.join(".golam"));
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "cannot resolve Golam runtime root; set GOLAM_ROOT explicitly",
    ))
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
