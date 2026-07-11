#!/usr/bin/env python3
"""Validate the intentionally local-only documentation menu catalog."""
from pathlib import Path
import json
import subprocess
import sys

MENU = Path("contracts/menus")
LOCALES = (Path("config/locales/en.json"), Path("config/locales/ja.json"))
EXPECTED = {"docs-directory", "docs-file", "docs-links", "docs-search"}
FIELDS = {"id", "kind", "title", "theme", "size", "params", "parent", "data",
          "chrome", "static", "confirmation"}
FORBIDDEN = ('"daemon"', '"command"', '"transfer"', '"refresh"', '"action"')


def load_json(path, errors):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        errors.append(f"{path}: invalid JSON: {error}")
        return None


def locale_keys(errors):
    catalogs = []
    for path in LOCALES:
        catalog = load_json(path, errors)
        if not isinstance(catalog, dict):
            errors.append(f"{path}: locale catalog must be an object")
            continue
        catalogs.append(set(catalog))
    return set.intersection(*catalogs) if catalogs else set()


def check_params(path, params, errors):
    if not isinstance(params, list):
        errors.append(f"{path}: params must be an array")
        return
    names = set()
    for param in params:
        if not isinstance(param, dict) or set(param) != {"name", "required"}:
            errors.append(f"{path}: params must contain name and required")
            continue
        name = param["name"]
        if not isinstance(name, str) or not name or name in names:
            errors.append(f"{path}: param names must be unique nonempty strings")
        if param["required"] is not True:
            errors.append(f"{path}: local params must be required")
        names.add(name)


def check_route(path, name, route, keys, errors):
    if not isinstance(route, dict) or set(route) != FIELDS:
        errors.append(f"{path}: route fields must be exactly {sorted(FIELDS)}")
        return None
    if route["id"] != name:
        errors.append(f"{path}: route id must match filename")
    if route["kind"] not in {"list", "detail"}:
        errors.append(f"{path}: route kind must be list or detail")
    if route["theme"] != "docs" or route["size"] != 54:
        errors.append(f"{path}: route must use the 54-slot docs theme")
    if not isinstance(route["title"], str) or route["title"] not in keys:
        errors.append(f"{path}: title must be a locale key")
    check_params(path, route["params"], errors)
    data = route["data"]
    if data != {"binding": name, "source": "local"}:
        errors.append(f"{path}: route data must be this local binding")
    chrome = route["chrome"]
    if not isinstance(chrome, dict) or set(chrome) != {"back", "close"}:
        errors.append(f"{path}: chrome must define back and close")
    elif not all(isinstance(value, bool) for value in chrome.values()):
        errors.append(f"{path}: chrome values must be booleans")
    if route["static"] != []:
        errors.append(f"{path}: static slots are withdrawn")
    if route["confirmation"] is not None:
        errors.append(f"{path}: confirmations are withdrawn")
    text = json.dumps(route, sort_keys=True)
    if any(token in text for token in FORBIDDEN):
        errors.append(f"{path}: withdrawn daemon-shaped data")
    return route.get("parent")


def check_parents(parents, errors):
    for name, parent in parents.items():
        if parent is not None and parent not in parents:
            errors.append(f"contracts/menus/{name}.json: parent must name a local route")
            continue
        visited = set()
        current = name
        while parents.get(current) is not None:
            if current in visited:
                errors.append(f"contracts/menus/{name}.json: parent chain must reach a root")
                break
            visited.add(current)
            current = parents[current]


def check_generated(errors):
    generated = subprocess.run(["./scripts/generate-menu-docs.py", "--check"], text=True,
                               capture_output=True, check=False)
    if generated.returncode:
        errors.extend(line for line in (generated.stdout + generated.stderr).splitlines() if line)


def main():
    errors = []
    files = {path.stem for path in MENU.glob("*.json") if path.name != "README.json"}
    if files != EXPECTED:
        errors.append("menu catalog must contain only local documentation routes")
    keys = locale_keys(errors)
    parents = {}
    for name in sorted(files):
        path = MENU / f"{name}.json"
        route = load_json(path, errors)
        parent = check_route(path, name, route, keys, errors)
        if route is not None:
            parents[name] = parent
    check_parents(parents, errors)
    index = load_json(MENU / "README.json", errors)
    if not isinstance(index, dict) or index.get("menus") != sorted(EXPECTED):
        errors.append("contracts/menus/README.json: local route index mismatch")
    check_generated(errors)
    if errors:
        print("\n".join(errors))
        return 1
    print("ok check-menus")
    return 0


if __name__ == "__main__":
    sys.exit(main())
