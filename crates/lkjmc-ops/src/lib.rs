pub mod bootstrap;
pub mod database;
pub mod error;
pub mod eula;
pub mod fence;
pub mod fleet;
pub mod install;
pub mod journal;
pub mod manifest;
pub mod process;
mod secure_fs;

pub use error::{OpsError, Result};

pub fn require_root() -> Result<()> {
    secure_fs::require_root()
}
