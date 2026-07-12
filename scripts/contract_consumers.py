"""Consumer and dispatch-boundary checks for sharded command contracts."""
import json
import re
from pathlib import Path


def check_consumers(root: Path, commands, errors):
    try:
        rows = json.loads((root / "contracts/consumers.json").read_text(encoding="utf-8"))["consumers"]
    except (OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        errors.append(f"contracts/consumers.json: invalid JSON: {error}")
        return
    expected = {"cli": "checked", "web": "checked", "paper": "withdrawn", "velocity": "withdrawn", "discord": "withdrawn"}
    found = {row.get("name"): row.get("status") for row in rows if isinstance(row, dict)}
    if found != expected or len(found) != len(expected):
        errors.append("contracts/consumers.json: compatibility results mismatch")
    for row in rows:
        if not isinstance(row, dict) or not isinstance(row.get("source"), str) or not (root / row["source"]).exists():
            errors.append("contracts/consumers.json: consumer source missing")
    names = {command["name"] for command in commands}
    cli = literals(root / "crates/lkjmc-cli/src", names)
    web = literals(root / "crates/lkjmc-daemon/src/web", names)
    for command in commands:
        surfaces = [item for item, found in (("cli", cli), ("web", web)) if command["name"] in found] or ["internal"]
        if command["surfaces"] != surfaces:
            errors.append(f"{command['name']}: consumer surface mismatch")
    cli_client = (root / "crates/lkjmc-cli/src/client.rs").read_text(encoding="utf-8")
    web_api = (root / "crates/lkjmc-daemon/src/web/api.rs").read_text(encoding="utf-8")
    if "command_registry::validate_body(command, &body)" not in cli_client:
        errors.append("cli payloads do not validate before transport")
    if "crate::dispatch::dispatch_as(" not in web_api:
        errors.append("web payloads do not dispatch through validation")


def check_closed_dispatch(root: Path, handlers, errors):
    source_root = root / "crates/lkjmc-daemon/src"
    for path in source_root.rglob("*.rs"):
        if path.name == "command_registrations.rs" or "tests" in path.parts:
            continue
        text = path.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
        for handler in handlers:
            if f"{handler}(" in text:
                errors.append(f"{path.relative_to(root)}: registered handler bypasses dispatch: {handler}")


def literals(directory: Path, names):
    values = set()
    for path in directory.rglob("*.rs"):
        values.update(re.findall(r'"([a-z][a-z0-9.-]+)"', path.read_text(encoding="utf-8")))
    return values & names
