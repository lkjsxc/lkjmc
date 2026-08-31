#!/usr/bin/env python3
import json
import re
from pathlib import Path

PROBES = [
    "network-path-single", "inspect-apply-pass", "reapply-pass",
    "partial-failure-pass", "local-kube-capabilities", "config-example-pass",
]
DB_PROBES = {"inspect-apply-pass", "reapply-pass", "partial-failure-pass"}


def source_errors(root: Path, reader=None) -> list[str]:
    read = reader or (lambda path: path.read_text(encoding="utf-8"))
    errors: list[str] = []
    intent = root / "crates/lkjmc-core/src/config/network_intent.rs"
    text = read(intent)
    for member in ("instances", "routes", "listeners", "auth", "forwarding", "assets", "capabilities"):
        if f"pub {member}:" not in text:
            errors.append(f"network intent member missing: {member}")
    check_surfaces(root, read, errors)
    check_entrypoints(root, read, errors)
    check_process_inventory(root, read, errors)
    check_ownership(root, read, errors)
    return errors


def check_surfaces(root, read, errors):
    paths = [root / value for value in (
        "config/defaults/daemon.json.example", "docker-compose.yml",
    )]
    for path in paths:
        current = read(path)
        for superseded in ("velocityForwarding", "fallbackServer", "javaEntry", "forwardingSecretFile"):
            if superseded in current:
                errors.append(f"superseded network path remains in {path.name}: {superseded}")
        if re.search(r"(?i)(example\.invalid|placeholder|fake[-_ ]?(sha|digest|url|artifact))", current):
            errors.append(f"placeholder input remains in {path.name}")
        for digest in re.findall(r"(?i)(?<![0-9a-f])[0-9a-f]{64}(?![0-9a-f])", current):
            repeated = any(digest.lower() == digest[:width].lower() * (64 // width)
                           for width in (1, 2, 4, 8, 16, 32))
            if repeated or digest.lower().startswith("deadbeef"):
                errors.append(f"placeholder or repeated digest remains in {path.name}")
    example = json.loads(read(root / "config/defaults/daemon.json.example"))
    if example["network"]["assets"] or example["network"]["capabilities"]["mountedAssets"]:
        errors.append("example must deny its unavailable artifact capability")
    compose = read(root / "docker-compose.yml")
    if not re.search(r"image: postgres:\S+@sha256:[0-9a-f]{64}", compose):
        errors.append("Compose PostgreSQL image is not immutable")


def check_entrypoints(root, read, errors):
    registrations = read(root / "crates/lkjmc-daemon/src/commands/command_registrations.rs")
    for command in ("bootstrap.plan", "bootstrap.apply"):
        if registrations.count(f'name: "{command}"') != 1:
            errors.append(f"command registration is not singular: {command}")
    exact = {
        "crates/lkjmc-core/src/network_intent.rs": [("pub fn inspect(", 1), ("NetworkInspection {", 4)],
        "crates/lkjmc-daemon/src/commands/bootstrap_api.rs": [("fn plan(", 1)],
        "crates/lkjmc-daemon/src/commands/bootstrap_api/apply.rs": [("pub fn apply(", 1)],
        "crates/lkjmc-daemon/src/commands/bootstrap_api/network_state.rs": [("network_intent::inspect(", 1)],
        "crates/lkjmc-daemon/src/commands/bootstrap_api/apply/network_plan.rs": [("pub(super) fn effects(", 1)],
    }
    for relative, markers in exact.items():
        current = read(root / relative)
        for marker, count in markers:
            if current.count(marker) != count:
                errors.append(f"network entrypoint inventory changed: {relative}:{marker}")
    rust = rust_sources(root, read)
    product = "\n".join(rust.values())
    for removed in ("DesiredNetwork", "BootstrapPlan", "plan_bootstrap"):
        if removed in product:
            errors.append(f"superseded network compiler export remains: {removed}")
    compiler = re.compile(r"fn\s+\w+[^;{]*NetworkConfig[^;{]*(?:NetworkInspection|NetworkChange)", re.S)
    sites = [(path, len(compiler.findall(text))) for path, text in rust.items() if compiler.search(text)]
    if sites != [("crates/lkjmc-core/src/network_intent.rs", 1)]:
        errors.append(f"network compiler inventory changed: {sites}")


def check_process_inventory(root, read, errors):
    rust = rust_sources(root, read)
    expected = {
        "crates/lkjmc-core/build.rs": (1, 0),
        "crates/lkjmc-cli/src/commands.rs": (1, 0),
        "crates/lkjmc-xtask/src/main.rs": (1, 0),
        "crates/lkjmc-daemon/src/runtime/local_start.rs": (1, 1),
        "crates/lkjmc-daemon/src/runtime/kubernetes_command.rs": (1, 1),
        "crates/lkjmc-daemon/src/runtime/process.rs": (1, 0),
        "crates/lkjmc-discord/src/diagnostics.rs": (0, 1),
        "crates/lkjmc-ops/src/process.rs": (1, 1),
    }
    actual = {}
    for path, text in rust.items():
        commands = len(re.findall(r"(?:\bCommand::new|process::Command::new)", text))
        spawns = len(re.findall(r"\.spawn\s*\(", text))
        if commands or spawns:
            actual[path] = (commands, spawns)
        libc_process = (
            r"libc::(?:fork|vfork|clone3?|posix_spawnp?|exec(?:l|le|lp|v|ve|veat|vp|vpe)|system)"
        )
        if re.search(rf"\b(?:fork|posix_spawn|{libc_process}|tokio::process)\b", text):
            errors.append(f"alternate Rust process path: {path}")
    if actual != expected:
        errors.append(f"Rust process entrypoint inventory changed: {actual}")
    launch = {
        path: text.count("verified_launch(")
        for path, text in rust.items() if "verified_launch(" in text
    }
    expected_launch = {
        "crates/lkjmc-daemon/src/commands/jars.rs": 1,
        "crates/lkjmc-daemon/src/runtime/instance_launch.rs": 1,
    }
    if launch != expected_launch:
        errors.append(f"Java launch-plan inventory changed: {launch}")
    java_root = root / "platforms/jvm"
    for path in java_root.rglob("*.java"):
        if "/src/main/" not in path.as_posix() or "/generated/" in path.as_posix():
            continue
        if re.search(r"ProcessBuilder|Runtime\s*\.\s*getRuntime|\.exec\s*\(", read(path)):
            errors.append(f"Java process launch path is forbidden: {path.relative_to(root)}")
    shell_launch = re.compile(r"(?im)^\s*(?:exec\s+|nohup\s+)?(?:\"?\$JAVA_HOME/[^ ]*java\"?|java)\s|\s-jar\s")
    for path in (root / "scripts").glob("*.sh"):
        if shell_launch.search(read(path)):
            errors.append(f"shell Java launch path is forbidden: {path.relative_to(root)}")


def rust_sources(root, read):
    values = {}
    for path in (root / "crates").rglob("*.rs"):
        relative = path.relative_to(root).as_posix()
        if any(part == "tests" or part.endswith("_tests") for part in path.parts):
            continue
        if path.name.endswith("_tests.rs"):
            continue
        values[relative] = read(path).split("\n#[cfg(test)]", 1)[0]
    return values


def check_ownership(root, read, errors):
    required = {
        "migrations/048-network-intent.sql": "network_apply_attempts",
        "crates/lkjmc-store/src/network_intent.rs": "record_desired_with_attempt",
        "crates/lkjmc-core/src/network_intent.rs": "pub fn inspect",
        "crates/lkjmc-daemon/src/templates/render.rs": "replace_private",
        "crates/lkjmc-daemon/src/commands/bootstrap_api/apply/lock.rs": "try_lock_exclusive",
        "crates/lkjmc-daemon/src/runtime/reconcile.rs": "RuntimeGoal::Observe",
    }
    for relative, marker in required.items():
        if marker not in read(root / relative):
            errors.append(f"network ownership marker missing: {relative}:{marker}")
