#!/usr/bin/env python3
"""Deterministically compile canonical command and JVM sync JSON into Java 21."""
import argparse
import hashlib
import json
from pathlib import Path

PACKAGE = "com.lkjmc.bindings"
JAVA_TYPES = {"string": "String", "integer": "long", "long": "long", "int": "int",
              "number": "double", "boolean": "boolean", "array": "List<Object>"}
IMPORTS = {"Instant": "java.time.Instant", "UUID": "java.util.UUID",
           "List": "java.util.List", "Set": "java.util.Set"}

def load(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"malformed binding contract {path}: {error}")

def require(condition, message):
    if not condition:
        raise SystemExit(f"malformed binding contract: {message}")

def write(output, name, body):
    path = output / PACKAGE.replace(".", "/") / f"{name}.java"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(f"package {PACKAGE};\n\n{body.rstrip()}\n", encoding="utf-8")

def record_source(name, fields):
    require(fields and all(isinstance(k, str) and isinstance(v, str) for k, v in fields.items()), name)
    known = set(IMPORTS) | {"String", "long", "int", "boolean"}
    record_names = set(RECORDS)
    imports = set()
    for value in fields.values():
        tokens = value.replace("<", " ").replace(">", " ").replace(",", " ").split()
        require(all(token in known or token in record_names for token in tokens), f"unknown Java type in {name}")
        imports.update(IMPORTS[token] for token in tokens if token in IMPORTS)
    lines = [f"import {item};" for item in sorted(imports)]
    lines += ["", f"public record {name}("]
    items = list(fields.items())
    lines += [f"        {kind} {field}{',' if index + 1 < len(items) else ''}"
              for index, (field, kind) in enumerate(items)]
    lines += [") {", f"    public {name} {{"]
    for field, kind in items:
        if kind.startswith("List<"):
            lines.append(f"        {field} = List.copyOf({field});")
        elif kind.startswith("Set<"):
            lines.append(f"        {field} = Set.copyOf({field});")
        elif kind not in {"long", "int", "boolean"}:
            lines.append(f"        java.util.Objects.requireNonNull({field}, \"{field}\");")
        if kind == "String":
            lines.append(f"        if ({field}.isBlank()) throw new IllegalArgumentException(\"{field}\");")
        if kind == "long" and ("revision" in field.lower() or field == "fence"):
            lines.append(f"        if ({field} <= 0) throw new IllegalArgumentException(\"{field}\");")
    lines += ["    }", "}"]
    return "\n".join(lines)

def compile_contracts(root, output):
    sync_path = root / "platforms/jvm/contracts/sync.json"
    consume_path = root / "platforms/jvm/contracts/consumption.json"
    manifest_path = root / "contracts/commands/README.json"
    sync, consumed, manifest = load(sync_path), load(consume_path), load(manifest_path)
    require(set(sync) == {"domains", "effects", "errors", "format", "records", "requests", "responses"},
            "sync keys")
    require(set(consumed) == {"commands", "format", "reason"}, "consumption keys")
    require(set(manifest) == {"format", "shards"}, "command manifest keys")
    require(sync.get("format") == "lkjmc-jvm-sync-v1", "sync format")
    require(consumed.get("format") == "lkjmc-jvm-command-consumption-v1", "consumption format")
    require(manifest.get("format") == "lkjmc-command-shards-v1", "command manifest format")
    shards = manifest.get("shards")
    require(isinstance(shards, list) and len(shards) == len(set(shards)), "command shard list")
    command_dir = manifest_path.parent
    present = {path.name for path in command_dir.glob("*.json") if path.name != "README.json"}
    require(set(shards) == present, "listed command shards")
    commands, sources = [], [sync_path, consume_path, manifest_path]
    for shard_name in shards:
        require(Path(shard_name).name == shard_name and shard_name.endswith(".json"), "shard path")
        shard_path = root / "contracts/commands" / shard_name
        shard = load(shard_path); sources.append(shard_path)
        require(set(shard) == {"commands", "domain"}, f"shard keys {shard_name}")
        for command in shard["commands"]:
            required = {"authorization", "deadline", "doc", "effect", "errors", "handler",
                        "idempotency", "identity", "name", "request", "response", "summary", "surfaces"}
            require(set(command) == required, f"command keys {shard_name}")
            commands.append(command)
    names = [item["name"] for item in commands]
    require(len(names) == len(set(names)), "duplicate command")
    selected = consumed.get("commands")
    require(isinstance(selected, list) and set(selected) <= set(names), "consumed command set")
    canonical_jvm = {item["name"] for item in commands if set(item["surfaces"]) & {"paper", "velocity"}}
    require(set(selected) == canonical_jvm, "JVM consumer coverage")
    global RECORDS
    RECORDS = sync.get("records", {})
    require(isinstance(RECORDS, dict), "records")
    domains = sync.get("domains", [])
    require(isinstance(domains, list) and all(isinstance(item, dict)
            and set(item) == {"name", "payload"} for item in domains), "sync domains")
    domain_names = [item.get("name") for item in domains]
    require(set(domain_names) == {"permissions", "claims", "menus", "profiles", "presence", "routing", "settings"}
            and len(domain_names) == 7, "closed sync domains")
    require(all(item.get("payload") in RECORDS for item in domains), "sync payload record")
    for name, fields in RECORDS.items():
        write(output, name, record_source(name, fields))
    write(output, "SyncError", enum_source("SyncError", sync.get("errors")))
    write(output, "EffectClass", enum_source("EffectClass", sync.get("effects")))
    domain_rows = [f'    {item["name"].upper()}({item["payload"]}.class)' for item in domains]
    write(output, "SyncDomain", "public enum SyncDomain {\n" + ",\n".join(domain_rows)
          + ";\n    private final Class<?> payloadType;\n    SyncDomain(Class<?> type) { payloadType = type; }\n"
          + "    public Class<?> payloadType() { return payloadType; }\n}")
    request_interface, request_records = sealed_requests(sync.get("requests"))
    write(output, "SyncRequest", request_interface)
    for request_name, request_body in request_records.items():
        write(output, request_name, request_body)
    write(output, "SyncResponse", sealed_responses(sync.get("responses")))
    write(output, "TypedSnapshot", """import java.time.Instant;
public record TypedSnapshot(String domain, String key, long revision, Instant generatedAt,
                            long credentialRevision, Object payload) implements SyncResponse {}""")
    write(output, "FeedResponse", """import java.util.List;
public record FeedResponse(long cursor, long activeFloor, long credentialRevision,
                           List<FeedChange> changes) implements SyncResponse {
    public FeedResponse { changes = List.copyOf(changes); }
}""")
    write(output, "ReloadRequired", "public record ReloadRequired(long cursor, long activeFloor, long credentialRevision) implements SyncResponse {}")
    effects = sorted({c["effect"] for c in commands})
    effect_names = {value: value.upper().replace("-", "_") for value in effects}
    write(output, "CommandEffect", enum_source("CommandEffect", list(effect_names.values())))
    write(output, "CommandErrorBoundary", "public enum CommandErrorBoundary { HANDLER_DEFINED }")
    write(output, "CommandRequest", "public sealed interface CommandRequest permits UnavailableCommandRequest {}")
    write(output, "UnavailableCommandRequest", "public enum UnavailableCommandRequest implements CommandRequest { INSTANCE }")
    write(output, "CommandResponse", "public sealed interface CommandResponse permits UnavailableCommandResponse {}")
    write(output, "UnavailableCommandResponse", "public enum UnavailableCommandResponse implements CommandResponse { INSTANCE }")
    write(output, "CommandBinding", "public record CommandBinding(String name, CommandEffect effect, String response, CommandErrorBoundary errors) {}")
    rows = [f'        new CommandBinding("{c["name"]}", CommandEffect.{effect_names[c["effect"]]}, '
            f'"{c["response"]["envelope"]}", CommandErrorBoundary.HANDLER_DEFINED)' for c in sorted(commands, key=lambda x: x["name"])]
    digest = hashlib.sha256(b"".join(path.read_bytes() for path in sources)).hexdigest()
    body = "import java.util.List;\n\npublic final class CommandCatalog {\n"
    body += f'    public static final String SOURCE_SHA256 = "{digest}";\n'
    body += "    public static final List<CommandBinding> ALL = List.of(\n" + ",\n".join(rows) + "\n    );\n"
    body += f"    public static final int JVM_CONSUMED = {len(selected)};\n    private CommandCatalog() {{}}\n}}"
    write(output, "CommandCatalog", body)

def enum_source(name, values):
    require(isinstance(values, list) and values and len(values) == len(set(values)), name)
    require(all(isinstance(value, str) and value.isupper() for value in values), name)
    return f"public enum {name} {{\n    " + ",\n    ".join(values) + "\n}"

def sealed_requests(requests):
    require(isinstance(requests, list) and requests, "requests")
    permits = ", ".join(item["name"] for item in requests)
    records = {}
    for item in requests:
        require(isinstance(item, dict) and set(item) == {"fields", "name"}
                and isinstance(item["name"], str), "request keys")
        fields = item.get("fields", {})
        require(len(fields) and all(kind in JAVA_TYPES for kind in fields.values()), item.get("name", "request"))
        args = ", ".join(f"{JAVA_TYPES[kind]} {name}" for name, kind in fields.items())
        lines = ["import java.util.Objects;", "", f"public record {item['name']}({args}) implements SyncRequest {{",
                 f"    public {item['name']} {{"]
        for field, kind in fields.items():
            if kind == "string": lines.append(f"        Objects.requireNonNull({field}, \"{field}\");")
        lines += ["    }", "}"]
        records[item["name"]] = "\n".join(lines)
    return f"public sealed interface SyncRequest permits {permits} {{}}", records

def sealed_responses(responses):
    require(responses == ["TypedSnapshot", "FeedResponse", "ReloadRequired"], "responses")
    return "public sealed interface SyncResponse permits TypedSnapshot, FeedResponse, ReloadRequired {}"

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    compile_contracts(args.root.resolve(), args.output.resolve())
