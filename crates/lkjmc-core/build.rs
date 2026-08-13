use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const BUILD_NONCE_ENV: &str = "LKJMC_BUILD_NONCE";
const SOURCE_COMMIT_ENV: &str = "LKJMC_SOURCE_COMMIT";

fn main() -> ExitCode {
    match configure() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lkjmc build identity error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn configure() -> Result<(), String> {
    println!("cargo:rerun-if-env-changed={SOURCE_COMMIT_ENV}");
    println!("cargo:rerun-if-env-changed={BUILD_NONCE_ENV}");
    let manifest = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").map_err(|_| "CARGO_MANIFEST_DIR is missing".to_string())?,
    );
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "workspace root is unavailable".to_string())?;
    let has_git = git_output(root, &["rev-parse", "--is-inside-work-tree"])
        .is_some_and(|value| value == "true");
    if has_git {
        watch_git_identity(root);
    }

    let supplied = optional_env(SOURCE_COMMIT_ENV);
    let nonce = optional_env(BUILD_NONCE_ENV);
    if let Some(value) = supplied.as_deref() {
        validate_hex(value, 40, SOURCE_COMMIT_ENV)?;
        if !has_git {
            return Err(format!("{SOURCE_COMMIT_ENV} requires a Git checkout"));
        }
        let nonce = nonce
            .as_deref()
            .ok_or_else(|| format!("{SOURCE_COMMIT_ENV} requires {BUILD_NONCE_ENV}"))?;
        validate_hex(nonce, 32, BUILD_NONCE_ENV)?;
    } else if nonce.is_some() {
        return Err(format!("{BUILD_NONCE_ENV} requires {SOURCE_COMMIT_ENV}"));
    }

    let observed = if has_git {
        Some(
            git_output(root, &["rev-parse", "HEAD"])
                .filter(|value| valid_hex(value, 40))
                .ok_or_else(|| "cannot resolve Git HEAD for build identity".to_string())?,
        )
    } else {
        None
    };
    if let (Some(expected), Some(actual)) = (supplied.as_deref(), observed.as_deref()) {
        if expected != actual {
            return Err(format!("{SOURCE_COMMIT_ENV} differs from Git HEAD"));
        }
    }
    if supplied.is_some() {
        let status = git_output(
            root,
            &["status", "--porcelain=v1", "--untracked-files=normal"],
        )
        .ok_or_else(|| "cannot inspect Git worktree for build identity".to_string())?;
        if !status.is_empty() {
            return Err(format!("{SOURCE_COMMIT_ENV} requires a clean worktree"));
        }
    }

    let commit = supplied
        .or(observed)
        .unwrap_or_else(|| "unknown".to_string());
    let dirty = if nonce.is_some() { "false" } else { "unknown" };
    println!("cargo:rustc-env=LKJMC_BUILD_COMMIT={commit}");
    println!("cargo:rustc-env=LKJMC_BUILD_DIRTY={dirty}");
    Ok(())
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

fn validate_hex(value: &str, length: usize, name: &str) -> Result<(), String> {
    if valid_hex(value, length) {
        Ok(())
    } else {
        Err(format!(
            "{name} must be {length} lowercase hexadecimal characters"
        ))
    }
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
}

fn git_output(root: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
}

fn watch_git_identity(root: &Path) {
    for value in ["HEAD", "index", "packed-refs"] {
        if let Some(path) = git_path(root, value) {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    if let Some(reference) =
        git_output(root, &["symbolic-ref", "-q", "HEAD"]).and_then(|value| git_path(root, &value))
    {
        println!("cargo:rerun-if-changed={}", reference.display());
    }
}

fn git_path(root: &Path, value: &str) -> Option<PathBuf> {
    let path = PathBuf::from(git_output(root, &["rev-parse", "--git-path", value])?);
    Some(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}
