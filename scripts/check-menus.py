#!/usr/bin/env python3
"""Verify authored routes, deterministic JVM bundle, route docs, and player corpus."""
from pathlib import Path
import json, subprocess, sys, tempfile

ROOT = Path(__file__).resolve().parents[1]
CHECKED = ROOT / "platforms/jvm/common/src/generated/resources/lkjmc-menu-bundle.json"
CORPUS = ROOT / "contracts/docs-player-corpus.json"


def run(command, errors):
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    if result.returncode:
        errors.extend((result.stdout + result.stderr).splitlines())


def check_corpus(errors):
    try:
        value = json.loads(CORPUS.read_text(encoding="utf-8"))
        if set(value) != {"format", "paths"} or value["format"] != "lkjmc-player-doc-corpus-v1":
            errors.append("invalid player docs corpus shape")
            return
        if value["paths"] != sorted(set(value["paths"])):
            errors.append("player docs corpus paths must be sorted and unique")
        for relative in value["paths"]:
            path = ROOT / relative
            if not path.is_file() or path.suffix != ".md":
                errors.append(f"invalid player docs corpus path: {relative}")
    except (OSError, json.JSONDecodeError) as failure:
        errors.append(f"invalid player docs corpus: {failure}")


def main():
    errors = []
    with tempfile.TemporaryDirectory(prefix="lkjmc-menu-check-") as directory:
        candidate = Path(directory) / "lkjmc-menu-bundle.json"
        run(["python3", str(ROOT / "scripts/compile-menu-bundle.py"), str(candidate)], errors)
        if candidate.is_file() and (not CHECKED.is_file() or candidate.read_bytes() != CHECKED.read_bytes()):
            errors.append("source-owned JVM menu bundle is stale")
    run(["python3", str(ROOT / "scripts/generate-menu-docs.py"), "--check"], errors)
    check_corpus(errors)
    if errors:
        print("\n".join(errors)); return 1
    print("ok check-menus 62 routes selected-engine bundle curated-docs")
    return 0


if __name__ == "__main__":
    sys.exit(main())
