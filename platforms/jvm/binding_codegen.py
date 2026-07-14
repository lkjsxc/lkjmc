"""Small deterministic Java source helpers for JVM bindings."""
import re
from pathlib import Path

PACKAGE = "com.lkjmc.bindings"
IMPORTS = {"Instant": "java.time.Instant", "UUID": "java.util.UUID",
           "List": "java.util.List", "Map": "java.util.Map"}
PRIMITIVES = {"long", "int", "double", "boolean"}


def require(condition, message):
    if not condition:
        raise SystemExit(f"malformed binding contract: {message}")


def write(output, name, body):
    path = output / PACKAGE.replace(".", "/") / f"{name}.java"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(f"package {PACKAGE};\n\n{body.rstrip()}\n", encoding="utf-8")


def tokens(kind):
    return re.findall(r"[A-Za-z][A-Za-z0-9]*", kind.rstrip("?"))


def record_source(name, fields, known, interface=""):
    imports = sorted({IMPORTS[token] for kind in fields.values()
                      for token in tokens(kind) if token in IMPORTS})
    lines = [f"import {item};" for item in imports]
    if imports:
        lines.append("")
    rows = list(fields.items())
    lines.append(f"public record {name}(")
    lines.extend(f"        {kind.rstrip('?')} {field}{',' if index < len(rows) - 1 else ''}"
                 for index, (field, kind) in enumerate(rows))
    lines.append(f"){(' implements ' + interface) if interface else ''} {{")
    lines.append(f"    public {name} {{")
    for field, kind in rows:
        base = kind.rstrip("?")
        require(all(token in known or token in IMPORTS or token in PRIMITIVES
                    or token in {"String", "Integer", "Boolean"} for token in tokens(kind)),
                f"unknown Java type {kind} in {name}")
        if kind.endswith("?"):
            continue
        if base.startswith("List<"):
            lines.append(f"        {field} = List.copyOf({field});")
        elif base.startswith("Map<"):
            lines.append(f"        {field} = Map.copyOf({field});")
        elif base not in PRIMITIVES:
            lines.append(f"        java.util.Objects.requireNonNull({field}, \"{field}\");")
        if base == "String":
            lines.append(f"        if ({field}.isBlank()) throw new IllegalArgumentException(\"{field}\");")
        if base == "long" and ("revision" in field.lower() or field in {"cursor", "activeFloor"}):
            lines.append(f"        if ({field} < 0) throw new IllegalArgumentException(\"{field}\");")
    lines.extend(["    }", "}"])
    return "\n".join(lines)


def enum_source(name, values):
    require(values and len(values) == len(set(values)), name)
    return f"public enum {name} {{\n    " + ",\n    ".join(values) + "\n}"
