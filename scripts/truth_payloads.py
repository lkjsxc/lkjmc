"""Parse literal Rust command bodies and check them against command contracts."""
import json
import re
from pathlib import Path

LITERAL = re.compile(r'"([a-z][a-z0-9.-]+)"\s*,\s*json!\(')
SOURCES = ("crates/lkjmc-cli/src", "crates/lkjmc-daemon/src/web")

def consumer_issues(root: Path, commands):
    contracts = {command.get("name"): command.get("request") for command in commands}
    cli = read(root / "crates/lkjmc-cli/src/client.rs")
    web = read(root / "crates/lkjmc-daemon/src/web/api.rs")
    errors = []
    if "command_registry::validate_body(command, &body)" not in cli:
        errors.append(("payload-consumers-required", "CLI bodies bypass contract validation"))
    if "crate::dispatch::dispatch_as(" not in web:
        errors.append(("payload-consumers-required", "web bodies bypass validated dispatch"))
    bodies, parse_errors = literal_bodies(root)
    errors.extend(("payload-consumers-required", error) for error in parse_errors)
    if not bodies:
        errors.append(("payload-consumers-required", "no literal CLI or web bodies were found"))
    for path, command, body in bodies:
        request = contracts.get(command)
        if request is None:
            errors.append(("payload-consumers-required", f"{path}: {command} has no contract"))
            continue
        error = validate_body(request, body)
        if error:
            errors.append(("payload-consumers-required", f"{path}: {command}: {error}"))
    return errors

def literal_bodies(root: Path):
    bodies, errors = [], []
    for relative in SOURCES:
        for path in (root / relative).rglob("*.rs"):
            source = read(path)
            for match in LITERAL.finditer(source):
                try:
                    macro, _ = balanced(source, match.end() - 1)
                    body = parse_object(macro)
                except ValueError as error:
                    errors.append(f"{path.relative_to(root)}: {match.group(1)} body: {error}")
                    continue
                bodies.append((path.relative_to(root), match.group(1), body))
    return bodies, errors

def balanced(source, opening):
    depth, quote, escaped = 0, None, False
    for position in range(opening, len(source)):
        char = source[position]
        if quote:
            escaped = char == "\\" and not escaped
            if char == quote and not escaped:
                quote = None
            elif char != "\\":
                escaped = False
            continue
        if char in ('"', "'"):
            quote = char
        elif char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
            if depth == 0:
                return source[opening + 1:position], position + 1
    raise ValueError("unclosed json! macro")

def parse_object(source):
    source = source.strip()
    if not source.startswith("{") or not source.endswith("}"):
        raise ValueError("json! body is not an object")
    fields = {}
    for entry in split_top(source[1:-1]):
        if not entry.strip():
            continue
        entry = entry.lstrip()
        key, end = json.JSONDecoder().raw_decode(entry)
        rest = entry[end:].lstrip()
        if not isinstance(key, str) or not rest.startswith(":"):
            raise ValueError("object member is invalid")
        if key in fields:
            raise ValueError(f"duplicate object member: {key}")
        fields[key] = parse_value(rest[1:].strip())
    return fields

def split_top(source):
    entries, start, depth, quote, escaped = [], 0, 0, None, False
    for position, char in enumerate(source):
        if quote:
            escaped = char == "\\" and not escaped
            if char == quote and not escaped:
                quote = None
            elif char != "\\":
                escaped = False
            continue
        if char in ('"', "'"):
            quote = char
        elif char in "{[(":
            depth += 1
        elif char in "})]":
            depth -= 1
        elif char == "," and depth == 0:
            entries.append(source[start:position])
            start = position + 1
    entries.append(source[start:])
    return entries


def parse_value(source):
    if not source:
        raise ValueError("object member has no value")
    if source.startswith("{") and source.endswith("}"):
        return "object", parse_object(source)
    if source.startswith("[") and source.endswith("]"):
        return "array", None
    if source in ("true", "false"):
        return "boolean", None
    if source == "null":
        return "null", None
    if re.fullmatch(r"-?\d+", source):
        return "integer", None
    if re.fullmatch(r"-?(?:\d+\.\d*|\d*\.\d+)(?:[eE][+-]?\d+)?", source):
        return "number", None
    if source.startswith('"'):
        value, end = json.JSONDecoder().raw_decode(source)
        if isinstance(value, str) and not source[end:].strip():
            return "string", value
    return "dynamic", None


def validate_body(request, body):
    if request == {"body": "handler-defined"}:
        return None if not body else "body has members but command accepts none"
    fields = request.get("fields") if isinstance(request, dict) else None
    if not isinstance(fields, dict):
        return "contract request is unreadable"
    for name, shape in fields.items():
        if shape.get("required") and name not in body:
            return f"missing required member: {name}"
    for group in request.get("requiredAnyOf", []):
        if not any(name in body for name in group):
            return "missing one required alternative"
    for name, value in body.items():
        shape = fields.get(name)
        if shape is None:
            return f"unknown member: {name}"
        if not valid_type(value, shape.get("type")):
            return f"wrong type for member: {name}"
    return None


def valid_type(value, expected):
    actual, _ = value
    if actual == "dynamic":
        return True
    primitive = {"array": "array", "boolean": "boolean", "integer": "integer", "number": "number", "string": "string"}
    if expected in primitive:
        return actual == primitive[expected] or (expected == "number" and actual == "integer")
    if expected == "empty-object":
        return actual == "object" and not value[1]
    if actual != "object":
        return False
    if expected == "rcon-config":
        return closed(value[1], {"host", "password", "port"}) and typed(value[1], "password", "string") and typed(value[1], "port", "integer") and optional(value[1], "host", "string")
    if expected == "world-location":
        return closed(value[1], {"world", "x", "y", "z"}) and all(typed(value[1], name, kind) for name, kind in (("world", "string"), ("x", "number"), ("y", "number"), ("z", "number")))
    if expected == "shop-metadata":
        return shop_metadata(value[1])
    return False


def shop_metadata(fields):
    if not closed(fields, {"category", "delivery"}) or not optional(fields, "category", "string"):
        return False
    delivery = fields.get("delivery")
    if delivery is None:
        return True
    if delivery[0] != "object":
        return delivery[0] == "dynamic"
    delivery = delivery[1]
    executor = delivery.get("executor")
    if executor == ("string", "minecraft-item"):
        return closed(delivery, {"executor", "material", "amount"}) and typed(delivery, "material", "string") and typed(delivery, "amount", "integer")
    return fields == {"delivery": ("object", {"executor": ("string", "adventure"), "adventureId": ("string", "end-expedition")})}


def closed(fields, allowed):
    return set(fields) <= allowed


def typed(fields, name, expected):
    return name in fields and valid_type(fields[name], expected)


def optional(fields, name, expected):
    return name not in fields or valid_type(fields[name], expected)


def read(path):
    return path.read_text(encoding="utf-8") if path.is_file() else ""
