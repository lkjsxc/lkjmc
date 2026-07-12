"""Sharded command-contract checks used by the truth-probe harness."""
import json
import re
from pathlib import Path

from truth_payloads import consumer_issues

FIELD_TYPES = {
    "array", "boolean", "empty-object", "integer", "number", "rcon-config",
    "shop-metadata", "string", "world-location",
}
EMPTY_BODY_COMMANDS = {
    "admin.role.list", "adventure.catalog.list", "asset.plugin.list", "config.reload",
    "doctor", "economy.catalog.seed-defaults", "instance.list", "instance.wake.cleanup",
    "jar.list", "player.exchange.rates", "player.kit.list", "player.shop.list",
    "player.vote.list", "player.warp.list", "security.daemon-token.plan",
    "security.daemon-token.rotate", "security.daemon-token.status",
    "security.daemon-token.verify", "status",
}
ASSET_COMMANDS = {"asset.plugin.sync"}


def issues(root: Path):
    commands, errors = load_commands(root)
    errors.extend(validate_commands(commands))
    errors.extend(validate_asset_shard(commands))
    errors.extend(consumer_issues(root, commands))
    return errors


def load_commands(root: Path):
    directory = root / "contracts/commands"
    try:
        index = json.loads((directory / "README.json").read_text(encoding="utf-8"))
        shards = index["shards"]
    except (OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        return [], [("generic-schema-rejected", f"sharded command registry is unreadable: {error}")]
    errors = []
    files = sorted(path.name for path in directory.glob("*.json") if path.name != "README.json")
    if (
        index.get("format") != "lkjmc-command-shards-v1"
        or not isinstance(shards, list)
        or not shards
        or any(not isinstance(name, str) or "/" in name or not name.endswith(".json") for name in shards)
        or shards != sorted(shards)
        or len(shards) != len(set(shards))
        or shards != files
    ):
        errors.append(("generic-schema-rejected", "command shard manifest is incomplete"))
    includes = generated_includes(root)
    if shards != includes:
        errors.append(("generic-schema-rejected", "command shard manifest and generated includes differ"))
    commands = []
    for name in shards if isinstance(shards, list) else []:
        try:
            shard = json.loads((directory / name).read_text(encoding="utf-8"))
            entries = shard["commands"]
        except (OSError, KeyError, TypeError, json.JSONDecodeError) as error:
            errors.append(("generic-schema-rejected", f"{name}: unreadable shard: {error}"))
            continue
        if not isinstance(entries, list) or not entries:
            errors.append(("generic-schema-rejected", f"{name}: commands must be nonempty"))
            continue
        commands.extend(entry for entry in entries if isinstance(entry, dict))
    return commands, errors


def generated_includes(root: Path):
    source = root / "crates/lkjmc-core/src/command_shards.rs"
    if not source.is_file():
        return []
    return re.findall(r'contracts/commands/([^"/]+\.json)', source.read_text(encoding="utf-8"))


def validate_commands(commands):
    errors = []
    names = set()
    for command in commands:
        name = command.get("name", "?")
        if not isinstance(name, str) or not name or name in names:
            errors.append(("generic-schema-rejected", f"{name}: command name is invalid"))
            continue
        names.add(name)
        request = command.get("request")
        if request == {"body": "handler-defined"}:
            if name not in EMPTY_BODY_COMMANDS:
                errors.append(("generic-schema-rejected", f"{name}: generic request remains"))
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
    return errors


def validate_asset_shard(commands):
    by_name = {command.get("name"): command for command in commands}
    errors = []
    if not ASSET_COMMANDS <= set(by_name):
        errors.append(("generic-schema-rejected", "asset-01.json contract commands are missing"))
        return errors
    sync = by_name["asset.plugin.sync"].get("request")
    expected = {"fields": {"plugin": {"required": True, "type": "string"}}}
    if sync != expected:
        errors.append(("generic-schema-rejected", "asset.plugin.sync is not a closed plugin request"))
    return errors
