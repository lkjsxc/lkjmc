#!/usr/bin/env python3
"""Deterministically compile canonical daemon sync and command contracts."""
import argparse
import json
from pathlib import Path
from binding_codegen import require
import command_bindings
import sync_bindings


def load(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"malformed binding contract {path}: {error}")


def compile_contracts(root, output):
    sync = load(root / "platforms/jvm/contracts/sync.json")
    require(set(sync) == {"domains", "effects", "errors", "format", "requests", "results"}, "sync keys")
    require(sync["format"] == "lkjmc-jvm-sync-v2", "sync format")
    require(sync["domains"] == ["claims", "menus", "permissions", "presence", "profiles",
                                "routing", "settings"], "sync domain projection")
    require(sync["results"] == ["snapshot", "unavailable-snapshot", "changes",
                                "reload-required", "unavailable-error"], "sync result projection")
    sync_bindings.generate(root, output, sync)
    command_bindings.generate(root, output, sync,
        load(root / "platforms/jvm/contracts/consumption.json"),
        load(root / "contracts/commands/README.json"), load)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    compile_contracts(args.root.resolve(), args.output.resolve())
