use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: boundary POLICY JSON");
        std::process::exit(64);
    }
    let mut child = match Command::new("python3")
        .arg(&args[1])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            eprintln!("policy spawn failed: {error}");
            std::process::exit(1);
        }
    };
    if let Some(mut input) = child.stdin.take() {
        if writeln!(input, "{}", args[2]).is_err() {
            eprintln!("policy input failed");
            std::process::exit(1);
        }
    }
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(error) => {
            eprintln!("policy wait failed: {error}");
            std::process::exit(1);
        }
    };
    if !output.status.success() {
        eprint!("policy exited {}: ", output.status);
        eprintln!("{}", String::from_utf8_lossy(&output.stderr).trim());
        std::process::exit(1);
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
}
