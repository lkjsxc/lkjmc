use std::collections::HashMap;

use ring::signature::{UnparsedPublicKey, ED25519};

use crate::config::Config;

pub fn verify(
    config: &Config,
    headers: &HashMap<String, String>,
    body: &str,
) -> Result<(), String> {
    let public_key = parse_hex(config.public_key.as_deref().ok_or("missing public key")?)?;
    let signature = parse_hex(
        headers
            .get("x-signature-ed25519")
            .ok_or("missing signature")?,
    )?;
    let timestamp = headers
        .get("x-signature-timestamp")
        .ok_or("missing timestamp")?;
    let message = [timestamp.as_bytes(), body.as_bytes()].concat();
    UnparsedPublicKey::new(&ED25519, public_key)
        .verify(&message, &signature)
        .map_err(|_| "invalid signature".into())
}

fn parse_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 {
        return Err("invalid hex length".into());
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16).map_err(|_| "invalid hex".into())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_hex;

    #[test]
    fn rejects_odd_hex() {
        assert!(parse_hex("abc").is_err());
    }
}
