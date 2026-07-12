"""Sharded command-contract checks used by the truth-probe harness."""
import json
import re
from pathlib import Path

FIELD_TYPES = {"array", "boolean", "integer", "number", "object", "string", "value"}


def issues(root: Path):
    commands, error = load_commands(root)
    if error:
        return [("generic-schema-rejected", error)]
    errors = []
    for command in commands:
        name = command.get("name", "?") if isinstance(command, dict) else "?"
        request = command.get("request") if isinstance(command, dict) else None
        if request == {"body": "handler-defined"}:
            continue
        fields = request.get("fields") if isinstance(request, dict) else None
        if not isinstance(fields, dict) or not fields:
            errors.append(("generic-schema-rejected", f"{name}: generic request remains"))
            continue
        for field, shape in fields.items():
            if not isinstance(shape, dict) or set(shape) != {"required", "type"}:
                errors.append(("generic-schema-rejected", f"{name}.{field}: field is untyped"))
            elif not isinstance(shape["required"], bool) or shape["type"] not in FIELD_TYPES:
                errors.append(("generic-schema-rejected", f"{name}.{field}: field shape is invalid"))
    errors.extend(consumer_issues(root, {item["name"] for item in commands if isinstance(item, dict)}))
    return errors


def load_commands(root: Path):
    directory = root / "contracts/commands"
    try:
        index = json.loads((directory / "README.json").read_text(encoding="utf-8"))
        shards = index["shards"]
        if index.get("format") != "lkjmc-command-shards-v1" or not isinstance(shards, list):
            raise ValueError("invalid shard index")
        commands = []
        for name in shards:
            shard = json.loads((directory / name).read_text(encoding="utf-8"))
            commands.extend(shard["commands"])
        return commands, None
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        return [], f"sharded command registry is unreadable: {error}"


def consumer_issues(root: Path, names):
    cli = root / "crates/lkjmc-cli/src/client.rs"
    web = root / "crates/lkjmc-daemon/src/web/api.rs"
    cli_text = read(cli)
    web_text = read(web)
    errors = []
    if "command_registry::validate_body(command, &body)" not in cli_text:
        errors.append(("payload-consumers-required", "CLI bodies bypass contract validation"))
    if "crate::dispatch::dispatch_as(" not in web_text:
        errors.append(("payload-consumers-required", "web bodies bypass validated dispatch"))
    literals = literals_in(root / "crates/lkjmc-cli/src") | literals_in(root / "crates/lkjmc-daemon/src/web")
    if not literals & names:
        errors.append(("payload-consumers-required", "no command literal is bound to a checked schema"))
    return errors


def read(path: Path):
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def literals_in(directory: Path):
    values = set()
    for path in directory.rglob("*.rs"):
        values.update(re.findall(r'"([a-z][a-z0-9.-]+)"', read(path)))
    return values
