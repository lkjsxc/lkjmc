use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::parse;

#[test]
fn http_api_token_file_trailing_newline_is_trimmed() -> Result<(), String> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("lkjmc-http-token-{suffix}"));
    fs::write(&path, "AbCdEFghIJ09+/==\n").map_err(|error| error.to_string())?;
    let args = parse(vec![
        "--http-token-file".into(),
        path.to_string_lossy().into_owned(),
    ])?;
    fs::remove_file(path).ok();
    assert_eq!(args.http_token.as_deref(), Some("AbCdEFghIJ09+/=="));
    Ok(())
}

#[test]
fn cli_http_override_cannot_bypass_listener_validation() {
    for value in [
        "localhost:8765",
        "0.0.0.0:8765",
        "[::]:8765",
        "[::ffff:127.0.0.1]:8765",
    ] {
        assert!(
            parse(vec!["--http".into(), value.into()]).is_err(),
            "accepted {value}"
        );
    }
    assert!(parse(vec!["--http".into(), "[::1]:8765".into()]).is_ok());
}
