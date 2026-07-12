#!/usr/bin/env python3
"""Run the isolated E-CONTRACT source comparison; no product files are changed."""
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SOURCE = Path(__file__).with_name("e-contract-20260711-source.json")
RUST = Path(__file__).with_name("e-contract-20260711-rust.rs")
NAMES = ("status", "instance.start", "player.transfer.saved", "player.shop.purchase")
HAND = {"status": {}, "instance.start": {"id": "string"},
        "player.transfer.saved": {"playerUuid": "uuid"},
        "player.shop.purchase": {"playerUuid": "uuid", "name": "string",
        "itemId": "string", "correlationId": "uuid"}}
VALID = {"status": {}, "instance.start": {"id": "alpha"},
         "player.transfer.saved": {"playerUuid": "00000000-0000-4000-8000-000000000001"},
         "player.shop.purchase": {"playerUuid": "00000000-0000-4000-8000-000000000001",
         "name": "test", "itemId": "item", "correlationId": "00000000-0000-4000-8000-000000000002"}}


def dump(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_digest(directory):
    parts = [(path.relative_to(directory).as_posix(), digest(path))
             for path in sorted(directory.rglob("*")) if path.is_file()]
    return hashlib.sha256(json.dumps(parts).encode()).hexdigest(), len(parts)


def run(result, output, label, command):
    try:
        completed = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
        stdout, stderr, exit_code = completed.stdout, completed.stderr, completed.returncode
    except FileNotFoundError as error:
        stdout, stderr, exit_code = "", str(error), 127
    log = output / f"{label}.log"
    log.write_text(stdout + stderr, encoding="utf-8")
    result["tools"].append({"label": label, "command": command, "exit": exit_code,
                            "log": log.name, "lines": len(log.read_text().splitlines())})
    return exit_code == 0


def schema(source):
    return {item["name"]: {field["name"]: field["type"] for field in item["fields"]}
            for item in source["commands"]}


def valid(shape, body):
    if set(shape) != set(body):
        return False
    for name, kind in shape.items():
        value = body[name]
        if kind == "string" and not isinstance(value, str):
            return False
        if kind == "uuid":
            if not isinstance(value, str):
                return False
            try:
                uuid.UUID(value)
            except ValueError:
                return False
    return True


def checked(shape):
    for name, body in VALID.items():
        unknown = dict(body, unexpected=True)
        wrong = {"wrong": 7} if not shape[name] else dict(body)
        if shape[name]:
            wrong[next(iter(shape[name]))] = 7
        if not valid(shape[name], body) or valid(shape[name], unknown) or valid(shape[name], wrong):
            return False
    return True


def shard(output):
    catalog = json.loads((ROOT / "contracts/commands.json").read_text())
    selected = {item["name"]: item for item in catalog["commands"] if item["name"] in NAMES}
    if tuple(sorted(selected)) != tuple(sorted(NAMES)):
        raise RuntimeError("selected commands missing from current registry")
    for item in selected.values():
        dump(output / "commands" / f"{item['family']}.json", [item])
    compiled = [selected[name] for name in sorted(selected)]
    dump(output / "commands.compiled.json", compiled)
    locales = {}
    for language in ("en", "ja"):
        data = json.loads((ROOT / "config/locales" / f"{language}.json").read_text())
        groups = {key: {} for key in ("core", "instance", "player", "shop", "other")}
        for key, value in data.items():
            group = next((prefix for prefix in groups if prefix != "other" and key.startswith(prefix + ".")), "other")
            groups[group][key] = value
        for group, values in groups.items():
            dump(output / "locales" / language / f"{group}.json", values)
        locales[language] = groups
    dump(output / "locales.compiled.json", locales)
    return tree_digest(output)


def generated(source, output):
    output.mkdir(parents=True, exist_ok=True)
    names = [item["name"] for item in source["commands"]]
    rust = "const COMMANDS: &[&str] = &[" + ",".join(json.dumps(name) for name in names) + "];\n"
    rust += "fn main() { println!(\"{}\", COMMANDS.len()); }\n#[cfg(test)] mod t { use super::*; #[test] fn names() { assert_eq!(COMMANDS.len(), 4); } }\n"
    java = "public final class EContractBindings {\n  static final String[] COMMANDS = {" + ",".join(json.dumps(name) for name in names) + "};\n  private EContractBindings() {}\n}\n"
    (output / "EContractBindings.rs").write_text(rust, encoding="utf-8")
    (output / "EContractBindings.java").write_text(java, encoding="utf-8")


def source_text(path):
    return path.read_text(encoding="utf-8")


def coverage(result):
    registry = json.loads((ROOT / "contracts/commands.json").read_text())["commands"]
    names = {item["name"] for item in registry}
    text = source_text(ROOT / "crates/lkjmc-daemon/src/commands/command_registrations.rs")
    registered = {line.split('name: "', 1)[1].split('"', 1)[0] for line in text.splitlines() if 'Registration { name: "' in line}
    result["handlerCoverage"] = {"registry": len(names), "registered": len(registered), "exact": names == registered}
    cli = source_text(ROOT / "crates/lkjmc-cli/src/commands.rs") + source_text(ROOT / "crates/lkjmc-cli/src/commands_status.rs") + source_text(ROOT / "crates/lkjmc-cli/src/commands_shop.rs")
    web = source_text(ROOT / "crates/lkjmc-daemon/src/web/api.rs")
    result["surfaces"] = {name: {"cli": f'"{name}"' in cli, "web": f'"{name}"' in web} for name in NAMES}
    result["withdrawals"] = {"javaDaemon": "CommandEnvelope" not in "".join(path.read_text(errors="ignore") for path in (ROOT / "platforms/jvm").rglob("*.java")),
                             "discordCommands": "json!([])" in source_text(ROOT / "crates/lkjmc-discord/src/commands.rs")}


def menus(result):
    files = sorted(path.stem for path in (ROOT / "contracts/menus").glob("*.json") if path.name != "README.json")
    local = source_text(ROOT / "platforms/jvm/paper/src/main/java/com/lkjmc/paper/LocalDocsMenu.java")
    result["menus"] = {"routes": files, "staticCatalogOnly": "contracts/menus" not in local and "lkjmc-docs-bundle.json" in local}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    output = args.output or Path(tempfile.mkdtemp(prefix="lkjmc-e-contract-"))
    if output.exists() and any(output.iterdir()):
        raise SystemExit("output must be a new or empty directory")
    output.mkdir(parents=True, exist_ok=True)
    result = {"base": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "seed": 20260711, "tools": [], "probes": {}}
    source = json.loads(SOURCE.read_text())
    neutral = schema(source)
    generic = json.loads((ROOT / "contracts/schemas/command-request.schema.json").read_text())
    result["baselineGenericAccepts"] = {"unknown": generic.get("type") == "object" and "additionalProperties" not in generic,
                                      "wrongType": generic.get("type") == "object" and "properties" not in generic}
    rust_test = run(result, output, "rust-descriptor-test", ["rustc", "--test", str(RUST), "-o", str(output / "rust-test")])
    rust_test = run(result, output, "rust-descriptor-execute", [str(output / "rust-test")]) and rust_test
    rust_emit = run(result, output, "rust-descriptor-build", ["rustc", str(RUST), "-o", str(output / "rust-emitter")])
    rust_shape = json.loads(subprocess.check_output([str(output / "rust-emitter")], text=True)) if rust_emit else {}
    rust_shape = {name: dict(fields) for name, fields in rust_shape}
    result["probes"]["three-contract-candidates-run"] = rust_test and rust_emit and HAND == neutral == rust_shape and checked(HAND) and checked(neutral) and checked(rust_shape)
    result["probes"]["payload-drift-rejected"] = checked(HAND) and checked(neutral) and checked(rust_shape)
    repeats = []
    for index in range(3):
        directory = output / f"shard-{index}"
        repeats.append(shard(directory))
    result["shards"] = {"digests": [value[0] for value in repeats], "files": repeats[0][1]}
    result["probes"]["shard-build-repeatable"] = len(set(value[0] for value in repeats)) == 1
    generated(source, output / "generated-a"); generated(source, output / "generated-b")
    same = tree_digest(output / "generated-a")[0] == tree_digest(output / "generated-b")[0]
    stale = output / "generated-b/EContractBindings.java"; stale.write_text(stale.read_text() + "// stale\n")
    stale_detected = tree_digest(output / "generated-a")[0] != tree_digest(output / "generated-b")[0]
    generated(source, output / "generated")
    rust_generated = run(result, output, "generated-rust-test", ["rustc", "--test", str(output / "generated/EContractBindings.rs"), "-o", str(output / "generated-rust-test")])
    rust_generated = run(result, output, "generated-rust-execute", [str(output / "generated-rust-test")]) and rust_generated
    java_generated = run(result, output, "generated-java", ["javac", "--release", "21", "-d", str(output / "classes"), str(output / "generated/EContractBindings.java")])
    result["generation"] = {"repeatable": same, "staleDetected": stale_detected, "rust": rust_generated, "java": java_generated}
    coverage(result); menus(result)
    handlers = run(result, output, "handler-coverage", ["cargo", "test", "-p", "lkjmc-daemon", "daemon_registrations_match_command_contract"])
    menu_check = run(result, output, "menu-schema", ["./scripts/check-menus.py"])
    jvm_check = run(result, output, "jvm-containment", ["./scripts/check-jvm-containment.py"])
    discord_check = run(result, output, "discord-withdrawal", ["cargo", "test", "-p", "lkjmc-discord"])
    result["withdrawalChecks"] = {"jvm": jvm_check, "discord": discord_check}
    result["probes"]["handler-coverage-executable"] = handlers and result["handlerCoverage"]["exact"]
    result["probes"]["menu-schema-executable"] = menu_check and result["menus"]["staticCatalogOnly"]
    result["probes"]["all-surface-slice"] = "blocked: absent CLI/web mappings for profile transfer and shop purchase; Java and Discord withdrawn"
    result["probes"]["contract-combinations-run"] = all((result["probes"]["three-contract-candidates-run"], result["probes"]["shard-build-repeatable"], same, stale_detected, rust_generated, java_generated, menu_check))
    dump(output / "result.json", result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
