pub mod bootstrap;
pub mod database;
pub mod deploy;
pub mod error;
pub mod eula;
pub mod fence;
pub mod fleet;
pub mod host_deploy;
pub mod host_install;
pub mod install;
pub mod journal;
pub mod manifest;
pub mod process;
mod secure_fs;

pub use error::{OpsError, Result};

pub fn require_root() -> Result<()> {
    secure_fs::require_root()
}
