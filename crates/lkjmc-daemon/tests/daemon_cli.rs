use std::process::Command;

#[test]
fn daemon_cli_rejects_noncanonical_loopback_listener() -> Result<(), String> {
    let output = Command::new(env!("CARGO_BIN_EXE_lkjmc-daemon"))
        .args(["--http", "127.0.0.2:8765"])
        .output()
        .map_err(|error| error.to_string())?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("--http must be a literal loopback socket address"));
    Ok(())
}
