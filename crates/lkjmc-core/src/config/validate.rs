use crate::error::ConfigError;
use crate::validation::{is_absolute_path, is_kebab_id, is_non_empty, is_valid_port};

pub(super) fn require_path(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if is_absolute_path(value) {
        Ok(())
    } else {
        Err(ConfigError::invalid(field, "must be an absolute path"))
    }
}

pub(super) fn require_kebab(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if is_kebab_id(value) {
        Ok(())
    } else {
        Err(ConfigError::invalid(field, "must be lowercase kebab-case"))
    }
}

pub(super) fn require_non_empty(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if is_non_empty(value) {
        Ok(())
    } else {
        Err(ConfigError::invalid(field, "must not be empty"))
    }
}

pub(super) fn require_loopback_address(field: &'static str, value: &str) -> Result<(), ConfigError> {
    let address: std::net::SocketAddr = value.parse().map_err(|_| ConfigError::invalid(field, "must be a literal loopback socket address"))?;
    if address.ip().is_loopback() { Ok(()) } else { Err(ConfigError::invalid(field, "must be a literal loopback socket address")) }
}

pub(super) fn require_port(field: &'static str, port: u16) -> Result<(), ConfigError> {
    if is_valid_port(port) {
        Ok(())
    } else {
        Err(ConfigError::invalid(field, "must be 1..65535"))
    }
}

pub(super) fn require_positive(field: &'static str, value: u32) -> Result<(), ConfigError> {
    if value > 0 {
        Ok(())
    } else {
        Err(ConfigError::invalid(field, "must be positive"))
    }
}

pub(super) fn require_range(
    field: &'static str,
    value: u32,
    min: u32,
    max: u32,
) -> Result<(), ConfigError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(ConfigError::invalid(
            field,
            format!("must be {min}..={max}"),
        ))
    }
}

pub(super) fn require_user_agent(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if value.contains("lkjmc") && (value.contains("http") || value.contains('@')) {
        Ok(())
    } else {
        Err(ConfigError::invalid(
            field,
            "must identify lkjmc and include contact information",
        ))
    }
}
