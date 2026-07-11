#!/usr/bin/env python3
"""Validate the intentionally local-only documentation menu catalog."""
from pathlib import Path
import json
import subprocess
import sys

MENU = Path("contracts/menus")
EXPECTED = {"docs-directory", "docs-file", "docs-links", "docs-search"}


def main():
    errors = []
    files = {path.stem for path in MENU.glob("*.json") if path.name != "README.json"}
    if files != EXPECTED:
        errors.append("menu catalog must contain only local documentation routes")
    for name in sorted(files):
        path = MENU / f"{name}.json"
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            errors.append(f"{path}: invalid JSON: {error}")
            continue
        if data.get("id") != name or data.get("data", {}).get("source") != "local":
            errors.append(f"{path}: route must be named local data")
        text = json.dumps(data, sort_keys=True)
        if any(token in text for token in ('"daemon"', '"command"', '"transfer"', '"refresh"')):
            errors.append(f"{path}: withdrawn daemon action or source")
    try:
        index = json.loads((MENU / "README.json").read_text(encoding="utf-8"))
        if index.get("menus") != sorted(EXPECTED):
            errors.append("contracts/menus/README.json: local route index mismatch")
    except json.JSONDecodeError as error:
        errors.append(f"contracts/menus/README.json: invalid JSON: {error}")
    generated = subprocess.run(["./scripts/generate-menu-docs.py", "--check"], text=True,
                               capture_output=True, check=False)
    if generated.returncode:
        errors.extend(line for line in generated.stdout.splitlines() if line)
    if errors:
        print("\n".join(errors))
        return 1
    print("ok check-menus")
    return 0


if __name__ == "__main__":
    sys.exit(main())
