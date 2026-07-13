#!/usr/bin/env python3
"""Check bounded daemon command, consumer, menu, and config contracts."""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

from contract_consumers import check_closed_dispatch, check_consumers

ROOT = Path(__file__).resolve().parents[1]
COMMANDS = ROOT / "contracts/commands"
PROBES = (
    "all-command-payloads-specific", "all-config-fields-owned",
    "all-consumers-checked", "generation-repeatable",
    "handler-coverage-complete", "menu-schema-all-documents",
    "old-generic-files-absent", "shards-bounded", "unknown-fields-rejected",
    "validated-dispatch-closed",
)
COMMAND_KEYS = {
    "authorization", "deadline", "doc", "effect", "errors", "handler",
    "idempotency", "identity", "name", "request", "response", "summary", "surfaces",
}
FIELD_KEYS = {"required", "type"}
FIELD_TYPES = {"array", "boolean", "empty-object", "integer", "number", "rcon-config", "shop-metadata", "string", "world-location"}
EFFECT_METADATA = {
    "denied-unproved": ("not-run", "not-run"),
    "local-observation": ("8-seconds", "no-mutation"),
    "postgresql-desired-set": ("8-seconds", "desired-state-repeat-safe"),
    "postgresql-read": ("8-seconds", "no-mutation"),
    "restart-required": ("not-run", "not-run"),
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
            if request != {"body": "handler-defined"}:
                keys = {"fields", "requiredAnyOf"}
                if not isinstance(request, dict) or set(request) not in ({"fields"}, keys):
                    errors.append(f"{command['name']}: request must be typed or handler-defined")
                    continue
                fields = request.get("fields")
                if not isinstance(fields, dict) or not fields:
                    errors.append(f"{command['name']}: typed request needs fields")
                    continue
                if list(fields) != sorted(fields):
                    errors.append(f"{command['name']}: fields must be sorted")
                for field, shape in fields.items():
                    if not isinstance(field, str) or not field or not isinstance(shape, dict):
                        errors.append(f"{command['name']}: invalid request field")
                        continue
                    if set(shape) != FIELD_KEYS or not isinstance(shape["required"], bool):
                        errors.append(f"{command['name']}: invalid field shape for {field}")
                    if shape.get("type") not in FIELD_TYPES:
                        errors.append(f"{command['name']}: invalid field type for {field}")
                groups = request.get("requiredAnyOf", [])
                if not isinstance(groups, list) or any(
                    not isinstance(group, list) or not group or any(item not in fields for item in group)
                    for group in groups
                ):
                    errors.append(f"{command['name']}: invalid requiredAnyOf")
            if command["response"] != {"body": "handler-defined", "envelope": "command-response-v1"}:
                errors.append(f"{command['name']}: response boundary mismatch")
            expected = EFFECT_METADATA.get(command["effect"])
            if expected != (command["deadline"], command["idempotency"]):
                errors.append(f"{command['name']}: invalid effect lifecycle metadata")
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


def config_owners(errors):
    config_doc = (ROOT / "docs/contracts/config-schema.md").read_text(encoding="utf-8")
    if "contracts/config/README.json" not in config_doc or "owners.json" in config_doc:
        errors.append("config owner documentation does not name the shard inventory")
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
        check_consumers(ROOT, commands, errors)
    if probe in (None, "validated-dispatch-closed"):
        check_closed_dispatch(ROOT, set(registered().values()), errors)
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
