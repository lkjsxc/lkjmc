#!/usr/bin/env python3
"""Validate all authored menu routes and emit the deterministic JVM bundle."""
from pathlib import Path
import argparse, json, re, sys

ROOT = Path(__file__).resolve().parents[1]
MENU = ROOT / "contracts/menus"
FIELDS = {"id", "kind", "title", "theme", "size", "params", "parent",
          "dependencies", "chrome", "slots", "dynamic", "confirmation"}
KINDS = {"STATIC", "LIST", "DETAIL", "CONFIRM", "CUSTOM"}
DOMAINS = {"LOCAL_DOCS", "MENUS", "PERMISSIONS", "CLAIMS", "SETTINGS",
           "PROFILES", "ROUTING", "PRESENCE"}
SCOPES = {"LOCAL", "GLOBAL", "PLAYER", "NETWORK", "SERVER"}
ACTIONS = {"NAVIGATE", "BACK", "CLOSE", "REFRESH", "NONE", "MUTATION"}
ROLES = {"INFO", "ACTION", "NAVIGATION", "DECORATION", "DISABLED", "SUCCESS", "DANGER"}
CHROME = {"info", "back", "refresh", "close", "mainMenu"}
BORDER = set(range(9)) | set(range(45, 54)) | {9, 17, 18, 26, 27, 35, 36, 44}
CHROME_SLOTS = {45, 46, 47, 48, 49, 50, 53}
TAG = re.compile(r"<[^>]+>")
PLACEHOLDER = re.compile(r"\{([a-zA-Z0-9]+)\}")


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def expect(condition, message, errors):
    if not condition:
        errors.append(message)


def locale_refs(route):
    values = [route["title"], route["chrome"]["info"]]
    dynamic = route["dynamic"] or {}
    values += [dynamic.get("emptyName"), *dynamic.get("emptyLore", [])]
    for slot in route["slots"]:
        values += [slot["name"], *slot["lore"]]
    return [value for value in values if value and not value.startswith("literal:")]


def check_action(route, slot, routes, errors):
    action = slot["action"]
    kind = action.get("type")
    expect(kind in ACTIONS, f"{route['id']}: unknown action {kind}", errors)
    allowed = {"type"}
    if kind == "NAVIGATE":
        allowed |= {"route", "params"}
        target = routes.get(action.get("route"))
        expect(target is not None, f"{route['id']}: unknown target {action.get('route')}", errors)
        if target:
            passed = set(action.get("params", {}))
            required = {item["name"] for item in target["params"] if item["required"]}
            expect(required <= passed, f"{route['id']}: missing target params for {target['id']}", errors)
    if kind == "MUTATION":
        allowed |= {"operation", "capability"}
        expect(bool(action.get("operation")), f"{route['id']}: mutation operation required", errors)
        expect(str(action.get("capability", "")).startswith("menu.action."),
               f"{route['id']}: mutation capability required", errors)
    expect(set(action) <= allowed, f"{route['id']}: action has generic or unknown members", errors)


def check_route(route, routes, locales, errors):
    rid = route.get("id", "?")
    expect(set(route) == FIELDS, f"{rid}: route members must be exact", errors)
    expect(route.get("kind") in KINDS, f"{rid}: invalid kind", errors)
    expect(route.get("size") in {27, 54}, f"{rid}: invalid size", errors)
    expect(set(route.get("chrome", {})) == CHROME, f"{rid}: chrome members must be exact", errors)
    params = route.get("params", [])
    names = [item.get("name") for item in params]
    expect(len(names) == len(set(names)) and all(names), f"{rid}: invalid params", errors)
    domains = []
    for dependency in route.get("dependencies", []):
        expect(set(dependency) == {"domain", "scope"}, f"{rid}: dependency members", errors)
        expect(dependency.get("domain") in DOMAINS, f"{rid}: dependency domain", errors)
        expect(dependency.get("scope") in SCOPES, f"{rid}: dependency scope", errors)
        domains.append(dependency.get("domain"))
    expect(len(domains) == len(set(domains)), f"{rid}: duplicate dependency", errors)
    occupied = set()
    for slot in route.get("slots", []):
        number = slot.get("slot")
        expect(set(slot) == {"slot", "material", "name", "lore", "role", "action"},
               f"{rid}: slot members must be exact", errors)
        expect(isinstance(number, int) and 0 <= number < route["size"], f"{rid}: slot bounds", errors)
        expect(number not in occupied, f"{rid}: duplicate slot {number}", errors)
        occupied.add(number)
        expect(slot.get("role") in ROLES, f"{rid}: invalid role", errors)
        if route["size"] == 54:
            expect(number not in BORDER and number not in CHROME_SLOTS, f"{rid}: chrome collision {number}", errors)
        check_action(route, slot, routes, errors)
    for key in locale_refs(route):
        for name, catalog in locales.items():
            expect(key in catalog, f"{rid}: missing {name} locale key {key}", errors)
            if key in catalog:
                expect(bool(TAG.sub("", catalog[key]).strip()), f"{rid}: color-only label {key}", errors)


def check_graph(routes, errors):
    expect(len(routes) == 62, f"catalog must contain 62 routes, got {len(routes)}", errors)
    for rid, route in routes.items():
        parent = route["parent"]
        expect((rid == "root") == (parent is None), f"{rid}: invalid root parent", errors)
        if parent is not None:
            expect(parent in routes, f"{rid}: unknown parent {parent}", errors)
        seen = set()
        current = rid
        while current != "root" and current in routes:
            expect(current not in seen, f"{rid}: parent cycle", errors)
            if current in seen:
                break
            seen.add(current); current = routes[current]["parent"]


def compile_bundle(output):
    errors = []
    index = load(MENU / "README.json")["menus"]
    files = sorted(path.stem for path in MENU.glob("*.json") if path.name != "README.json")
    expect(index == files, "README menu index must be sorted and complete", errors)
    routes = {rid: load(MENU / f"{rid}.json") for rid in index}
    locales = {name: load(ROOT / f"config/locales/{name}.json") for name in ("en", "ja")}
    expect(locales["en"].keys() == locales["ja"].keys(), "locale key sets differ", errors)
    for key in locales["en"]:
        expect(set(PLACEHOLDER.findall(locales["en"][key])) == set(PLACEHOLDER.findall(locales["ja"][key])),
               f"locale placeholders differ: {key}", errors)
    check_graph(routes, errors)
    for route in routes.values():
        check_route(route, routes, locales, errors)
    text = json.dumps({"format": "lkjmc-menu-bundle-v1", "routes": list(routes.values())},
                      ensure_ascii=False, separators=(",", ":"), sort_keys=True) + "\n"
    if errors:
        raise ValueError("\n".join(errors))
    output.parent.mkdir(parents=True, exist_ok=True); output.write_text(text, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("output", type=Path)
    args = parser.parse_args()
    try:
        compile_bundle(args.output); print(f"ok menu bundle {args.output}"); return 0
    except (OSError, ValueError, json.JSONDecodeError) as failure:
        print(failure, file=sys.stderr); return 1


if __name__ == "__main__":
    raise SystemExit(main())
