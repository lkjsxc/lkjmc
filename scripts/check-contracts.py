#!/usr/bin/env python3
"""Check bounded daemon command, consumer, menu, and config contracts."""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COMMANDS = ROOT / "contracts/commands"
PROBES = (
    "all-command-payloads-specific", "all-config-fields-owned",
    "all-consumers-checked", "generation-repeatable",
    "handler-coverage-complete", "menu-schema-all-documents",
    "old-generic-files-absent", "shards-bounded", "unknown-fields-rejected",
)
COMMAND_KEYS = {
    "authorization", "deadline", "doc", "effect", "errors", "handler",
    "idempotency", "identity", "name", "request", "response", "summary", "surfaces",
}


def load(path, errors):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        errors.append(f"{path.relative_to(ROOT)}: invalid JSON: {error}")
        return None


def command_data(errors):
    index = load(COMMANDS / "README.json", errors)
    if not isinstance(index, dict) or set(index) != {"format", "shards"}:
        errors.append("contracts/commands/README.json: invalid index")
        return []
    shards = index["shards"]
    files = sorted(path.name for path in COMMANDS.glob("*.json") if path.name != "README.json")
    if index["format"] != "lkjmc-command-shards-v1" or shards != files:
        errors.append("contracts/commands/README.json: shard list mismatch")
    commands = []
    for name in shards if isinstance(shards, list) else []:
        shard = load(COMMANDS / name, errors)
        if not isinstance(shard, dict) or set(shard) != {"commands", "domain"}:
            errors.append(f"contracts/commands/{name}: invalid shard")
            continue
        if not isinstance(shard["commands"], list) or not shard["commands"]:
            errors.append(f"contracts/commands/{name}: commands must be nonempty")
            continue
        for command in shard["commands"]:
            if not isinstance(command, dict) or set(command) != COMMAND_KEYS:
                errors.append(f"contracts/commands/{name}: invalid command shape")
                continue
            request = command["request"]
            if not isinstance(request, dict) or set(request) != {"optional", "required"}:
                errors.append(f"{command['name']}: request must be closed")
                continue
            values = request["required"] + request["optional"]
            if not all(isinstance(value, str) and value for value in values):
                errors.append(f"{command['name']}: request members must be strings")
            if len(values) != len(set(values)) or values != sorted(values):
                errors.append(f"{command['name']}: request members must be sorted and unique")
            if command["response"] != {"body": "handler-defined", "envelope": "command-response-v1"}:
                errors.append(f"{command['name']}: response boundary mismatch")
            if command["identity"] != "transport-subject":
                errors.append(f"{command['name']}: identity boundary mismatch")
            if not Path(command["doc"]).is_file():
                errors.append(f"{command['name']}: missing owner doc")
            commands.append(command)
    names = [command["name"] for command in commands]
    if len(names) != len(set(names)):
        errors.append("command shards: names must be globally unique")
    return commands


def registered():
    text = (ROOT / "crates/lkjmc-daemon/src/commands/command_registrations.rs").read_text(encoding="utf-8")
    return dict(re.findall(r'Registration \{ name: "([^"]+)", handler: ([^ }]+)', text))


def literals(directory, names):
    values = set()
    for path in directory.rglob("*.rs"):
        values.update(re.findall(r'"([a-z][a-z0-9.-]+)"', path.read_text(encoding="utf-8")))
    return values & names


def consumers(commands, errors):
    data = load(ROOT / "contracts/consumers.json", errors)
    rows = data.get("consumers") if isinstance(data, dict) else None
    expected = {"cli": "checked", "web": "checked", "paper": "withdrawn", "velocity": "withdrawn", "discord": "withdrawn"}
    found = {row.get("name"): row.get("status") for row in rows if isinstance(row, dict)} if isinstance(rows, list) else {}
    if found != expected or len(found) != len(expected):
        errors.append("contracts/consumers.json: compatibility results mismatch")
    for row in rows if isinstance(rows, list) else []:
        if not isinstance(row.get("source"), str) or not (ROOT / row["source"]).exists():
            errors.append("contracts/consumers.json: consumer source missing")
    names = {command["name"] for command in commands}
    cli = literals(ROOT / "crates/lkjmc-cli/src", names)
    web = literals(ROOT / "crates/lkjmc-daemon/src/web", names)
    for command in commands:
        surfaces = ([item for item, found in (("cli", cli), ("web", web)) if command["name"] in found] or ["internal"])
        if command["surfaces"] != surfaces:
            errors.append(f"{command['name']}: consumer surface mismatch")


def config_owners(errors):
    directory = ROOT / "contracts/config"
    index = load(directory / "README.json", errors)
    shards = index.get("shards") if isinstance(index, dict) else None
    files = sorted(path.name for path in directory.glob("*.json") if path.name != "README.json")
    owners = []
    if not isinstance(index, dict) or index.get("format") != "lkjmc-config-owners-v1" or shards != files:
        errors.append("contracts/config/README.json: shard list mismatch")
    for name in shards if isinstance(shards, list) else []:
        shard = load(directory / name, errors)
        for owner in shard.get("owners", []) if isinstance(shard, dict) else []:
            if set(owner) != {"member", "path", "source"}:
                errors.append(f"contracts/config/{name}: invalid owner")
                continue
            source = ROOT / owner["source"]
            if not source.is_file() or f"pub {owner['member']}:" not in source.read_text(encoding="utf-8"):
                errors.append(f"{owner['path']}: owner source does not declare member")
            owners.append(owner["path"])
    if len(owners) != len(set(owners)):
        errors.append("config owners: duplicate path")
    sources = [ROOT / "crates/lkjmc-core/src/config/types.rs", ROOT / "crates/lkjmc-core/src/config/runtime_types.rs"]
    declared = set().union(*(set(re.findall(r"pub ([a-z_]+):", source.read_text(encoding="utf-8"))) for source in sources))
    owned = {owner["member"] for name in shards or [] for owner in (load(directory / name, errors) or {}).get("owners", [])}
    if declared - owned:
        errors.append("config owners: accepted member lacks owner")
    example = load(ROOT / "config/defaults/daemon.json.example", errors)
    def leaves(value, prefix=""):
        if isinstance(value, dict):
            return sum((leaves(item, f"{prefix}.{key}" if prefix else key) for key, item in value.items()), [])
        return [prefix]
    if isinstance(example, dict) and not set(leaves(example)) <= set(owners):
        errors.append("config owners: example field lacks owner")


def command(args):
    return subprocess.run(args, cwd=ROOT, text=True, capture_output=True, check=False)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--probe", choices=PROBES)
    args = parser.parse_args()
    errors = []
    commands = command_data(errors)
    probe = args.probe
    if probe in (None, "handler-coverage-complete"):
        actual = registered()
        expected = {item["name"]: item["handler"] for item in commands}
        if actual != expected:
            errors.append("handler registrations do not match command shards")
    if probe in (None, "all-consumers-checked"):
        consumers(commands, errors)
    if probe in (None, "all-config-fields-owned"):
        config_owners(errors)
        result = command(["./scripts/check-config-examples.py"])
        if result.returncode:
            errors.append("Rust config parser rejected an example")
    if probe in (None, "menu-schema-all-documents") and command(["./scripts/check-menus.py"]).returncode:
        errors.append("menu schema check failed")
    if probe in (None, "unknown-fields-rejected") and command(["cargo", "test", "-p", "lkjmc-core", "every_contract_rejects_unknown_body_members", "--quiet"]).returncode:
        errors.append("unknown-member rejection test failed")
    if probe in (None, "generation-repeatable"):
        for tool in ("./scripts/generate-contracts.py", "./scripts/generate-command-catalog.py"):
            if command([tool, "--check"]).returncode:
                errors.append(f"generation drift: {tool}")
    if probe in (None, "shards-bounded"):
        for path in (ROOT / "contracts").rglob("*"):
            if path.is_file() and len(path.read_text(encoding="utf-8").splitlines()) > 200:
                errors.append(f"{path.relative_to(ROOT)}: exceeds 200 lines")
    if probe in (None, "old-generic-files-absent"):
        for path in ("contracts/commands.json", "contracts/commands.schema.json", "contracts/schemas/command-request.schema.json", "contracts/schemas/command-response.schema.json"):
            if (ROOT / path).exists(): errors.append(f"old generic source remains: {path}")
    if errors:
        print("\n".join(sorted(set(errors))))
        return 1
    print(f"ok check-contracts probe={probe or 'all'} commands={len(commands)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
