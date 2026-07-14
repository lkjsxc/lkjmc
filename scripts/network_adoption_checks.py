#!/usr/bin/env python3
import json
import re
from pathlib import Path

PROBES = [
    "network-path-single",
    "inspect-apply-pass",
    "reapply-pass",
    "partial-failure-pass",
    "local-kube-capabilities",
    "config-example-pass",
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
    surface_paths = (
        root / "config/defaults/daemon.json.example",
        root / "scripts/install.sh",
        root / "scripts/compose-playable-entrypoint.sh",
        root / "docker-compose.yml",
    )
    for path in surface_paths:
        current = read(path)
        for legacy in ("velocityForwarding", "fallbackServer", "javaEntry", "forwardingSecretFile"):
            if legacy in current:
                errors.append(f"legacy network path remains in {path.name}: {legacy}")
        if re.search(r'(?i)(example\.invalid|placeholder|fake[-_ ]?(sha|digest|url|artifact))', current):
            errors.append(f"placeholder input remains in {path.name}")
        for digest in re.findall(r'(?i)(?<![0-9a-f])[0-9a-f]{64}(?![0-9a-f])', current):
            lowered = digest.lower()
            repeated = any(
                all(part == lowered[:width] for part in
                    (lowered[index:index + width] for index in range(0, 64, width)))
                for width in (1, 2, 4, 8, 16, 32)
            )
            if repeated or lowered.startswith("deadbeef"):
                errors.append(f"placeholder or repeated digest remains in {path.name}")
    example = json.loads(read(root / "config/defaults/daemon.json.example"))
    if example["network"]["assets"] or example["network"]["capabilities"]["mountedAssets"]:
        errors.append("example must deny its unavailable artifact capability")
    compose = read(root / "docker-compose.yml")
    if not re.search(r'image: postgres:\S+@sha256:[0-9a-f]{64}', compose):
        errors.append("Compose PostgreSQL image is not immutable")
    registrations = read(root / "crates/lkjmc-daemon/src/commands/command_registrations.rs")
    for command in ("bootstrap.plan", "bootstrap.apply"):
        if registrations.count(f'name: "{command}"') != 1:
            errors.append(f"command registration is not singular: {command}")
    product = "\n".join(read(path) for path in (root / "crates").rglob("*.rs"))
    if "e-network" in product.lower() or "declarative compiler" in product.lower():
        errors.append("research compiler is referenced by product Rust")
    required = {
        "migrations/048-network-intent.sql": "network_apply_attempts",
        "crates/lkjmc-store/src/network_intent.rs": "record_desired",
        "crates/lkjmc-core/src/network_intent.rs": "pub fn inspect",
        "crates/lkjmc-daemon/src/templates/render.rs": "replace_private",
        "crates/lkjmc-daemon/src/commands/bootstrap_api/apply/lock.rs": "try_lock_exclusive",
    }
    for relative, marker in required.items():
        if marker not in read(root / relative):
            errors.append(f"network ownership marker missing: {relative}:{marker}")
    return errors
