#!/usr/bin/env python3
"""Run bounded, real safety probes for operations effects."""
import argparse
import io
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROBES = (
    "docker-secret-context-clean",
    "playable-default-secure",
    "full-skip-summary-truthful",
    "deterministic-smokes-run",
    "real-config-parser",
    "atomic-download-faults",
    "partial-final-files-zero",
    "migration-lock-checksum",
    "database-deadlines",
)


def read(name: str) -> str:
    return (ROOT / name).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def run(*command: str) -> None:
    subprocess.run(command, cwd=ROOT, check=True)


def output(*command: str) -> bytes:
    result = subprocess.run(command, cwd=ROOT, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"command failed: {' '.join(command[:2])}")
    return result.stdout


def remove_fixture(path: Path) -> None:
    path.unlink(missing_ok=True)
    for parent in path.parents:
        if parent == ROOT:
            return
        try:
            parent.rmdir()
        except OSError:
            return


def docker_context() -> str | None:
    docker, git = read(".dockerignore"), read(".gitignore")
    for rule in ("*.secret", "**/*.secret", ".env", "**/.env", ".env.*", "**/.env.*"):
        require(rule in docker, f"missing Docker secret ignore {rule}")
    for rule in ("*.secret", ".env", ".env.*"):
        require(rule in git, f"missing Git secret ignore {rule}")
    if not shutil.which("docker"):
        return "docker"
    if subprocess.run(("docker", "version", "--format", "{{.Server.Version}}"),
                      cwd=ROOT, capture_output=True).returncode:
        return "docker"
    nested = ROOT / "nested"
    require(not nested.exists(), "refusing existing Docker fixture directory")
    dockerfile = tempfile.NamedTemporaryFile("w", delete=False, encoding="utf-8")
    image = container = ""
    try:
        nested.mkdir()
        (nested / "keep.txt").write_text("fixture\n", encoding="utf-8")
        for name in ("review.secret", ".env", ".env.review"):
            (nested / name).write_text("fixture\n", encoding="utf-8")
        dockerfile.write("FROM scratch\nCOPY nested/ /fixture/\n")
        dockerfile.close()
        image = output("docker", "build", "--quiet", "--no-cache", "-f", dockerfile.name, ".").decode().strip()
        require(image, "Docker context fixture produced no image")
        container = output("docker", "create", image, "true").decode().strip()
        names = set(tarfile.open(fileobj=io.BytesIO(output("docker", "export", container))).getnames())
        require("fixture/keep.txt" in names, "Docker context fixture lost its sentinel")
        blocked = {"fixture/review.secret", "fixture/.env", "fixture/.env.review"}
        require(not names & blocked, "Docker context included a secret-shaped fixture")
    finally:
        if container:
            subprocess.run(("docker", "rm", "-f", container), cwd=ROOT, capture_output=True)
        if image:
            subprocess.run(("docker", "image", "rm", "-f", image), cwd=ROOT, capture_output=True)
        Path(dockerfile.name).unlink(missing_ok=True)
        for path in (nested / "review.secret", nested / ".env", nested / ".env.review", nested / "keep.txt"):
            remove_fixture(path)
    return None


def source_probe(name: str) -> None:
    compose = read("docker-compose.yml")
    entrypoint = read("scripts/compose-playable-entrypoint.sh")
    defaults = read("crates/lkjmc-core/src/config/defaults.rs")
    full = read("scripts/verify-full.sh")
    if name == "playable-default-secure":
        require("LKJMC_PLAYABLE_ONLINE_MODE:-true" in compose, "playable auth defaults offline")
        require("LKJMC_PLAYABLE_JAVA_BIND_HOST:-127.0.0.1" in compose, "playable Java binds publicly")
        require("LKJMC_PLAYABLE_JAVA_HOST_BIND:-127.0.0.1" in compose, "published Java port is public")
        require("ONLINE_MODE=${LKJMC_PLAYABLE_ONLINE_MODE:-true}" in entrypoint, "entrypoint auth default differs")
        require('bind_host: "127.0.0.1".to_string()' in defaults, "Rust Java default is public")
        require('host: "127.0.0.1".to_string()' in defaults, "Rust Bedrock default is public")
    elif name == "full-skip-summary-truthful":
        require("ran=%s skipped=%s" in full, "full summary omits outcomes")
        require("run_safe_ops" in full, "full summary hides safe-operation skips")
        require("skips=live-smokes" not in full, "full summary is collapsed")
    elif name == "deterministic-smokes-run":
        require('LKJMC_CLAIM_SMOKE: "1"' in compose, "Compose does not run claim smoke")
        require('LKJMC_WEB_SMOKE: "1"' in compose, "Compose does not run web smoke")
        require("run_when_one claim" in full and "run_when_one web" in full, "full omits deterministic smoke")


def run_exact_cargo_test(command: tuple[str, ...], name: str) -> None:
    listed = subprocess.run((*command, "--", "--list"), cwd=ROOT, capture_output=True, text=True)
    require(listed.returncode == 0, f"cannot list Cargo tests for {name}")
    available = {line.removesuffix(": test") for line in listed.stdout.splitlines() if line.endswith(": test")}
    require(name in available, f"missing Cargo test: {name}")
    run(*command, name, "--", "--exact")


def run_store_safety_test(name: str) -> None:
    run_exact_cargo_test(("cargo", "test", "-p", "lkjmc-store", "--test", "safety"), name)


def database_probe(name: str) -> str | None:
    if not os.environ.get("LKJMC_STORE_TEST_DATABASE_URL"):
        return "LKJMC_STORE_TEST_DATABASE_URL"
    tests = (
        "migration_checksum_rejects_tampering",
        "migration_checksum_backfills_once_then_rejects_null",
        "concurrent_migrations_serialize_to_one_writer",
    ) if name == "migration-lock-checksum" else (
        "deadline_connection_uses_its_supplied_budget",
    )
    for test in tests:
        run_store_safety_test(test)
    return None


def probe(name: str) -> str | None:
    if name == "docker-secret-context-clean":
        return docker_context()
    if name in PROBES[1:4]:
        source_probe(name)
    elif name == "real-config-parser":
        run("./scripts/check-config-examples.py")
    elif name == "atomic-download-faults":
        run_exact_cargo_test(
            ("cargo", "test", "-p", "lkjmc-daemon", "--bin", "lkjmc-daemon"),
            "assets::download_io_tests::truncated_download_leaves_no_final_or_partial_file",
        )
    elif name == "partial-final-files-zero":
        run_exact_cargo_test(
            ("cargo", "test", "-p", "lkjmc-daemon", "--bin", "lkjmc-daemon"),
            "assets::download_io_tests::concurrent_downloads_publish_one_complete_final_file",
        )
    else:
        return database_probe(name)
    return None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--probe", choices=PROBES)
    parser.add_argument("--all", action="store_true")
    args = parser.parse_args()
    if not args.probe and not args.all:
        parser.error("choose --probe or --all")
    ran, skipped = [], []
    try:
        for name in PROBES if args.all else (args.probe,):
            reason = probe(name)
            (skipped if reason else ran).append(f"{name}:{reason}" if reason else name)
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"safe operations probe failed: {error}", file=sys.stderr)
        return 1
    print(f"ok check-safe-ops ran={','.join(ran) or 'none'} skipped={','.join(skipped) or 'none'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
