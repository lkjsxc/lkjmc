"""Generate the closed command inventory consumed by JVM surfaces."""
import hashlib
from pathlib import Path
from binding_codegen import enum_source, require, write


def generate(root, output, contract, consumed, manifest, load):
    require(set(consumed) == {"commands", "format", "reason"}, "consumption keys")
    require(set(manifest) == {"format", "shards"}, "command manifest keys")
    require(consumed["format"] == "lkjmc-jvm-command-consumption-v1", "consumption format")
    require(manifest["format"] == "lkjmc-command-shards-v1", "command manifest format")
    shards = manifest["shards"]
    require(isinstance(shards, list) and len(shards) == len(set(shards)), "command shard list")
    directory = root / "contracts/commands"
    present = {path.name for path in directory.glob("*.json") if path.name != "README.json"}
    require(set(shards) == present, "listed command shards")
    commands, sources = [], [root / "platforms/jvm/contracts/sync.json",
                              root / "platforms/jvm/contracts/consumption.json",
                              directory / "README.json"]
    for shard_name in shards:
        require(Path(shard_name).name == shard_name and shard_name.endswith(".json"), "shard path")
        path = directory / shard_name
        shard = load(path)
        sources.append(path)
        require(set(shard) == {"commands", "domain"}, f"shard keys {shard_name}")
        for command in shard["commands"]:
            fields = {"authorization", "deadline", "doc", "effect", "errors", "handler",
                      "idempotency", "identity", "name", "request", "response", "summary", "surfaces"}
            require(set(command) == fields, f"command keys {shard_name}")
            commands.append(command)
    names = [item["name"] for item in commands]
    require(len(names) == len(set(names)), "duplicate command")
    selected = consumed["commands"]
    require(isinstance(selected, list) and set(selected) <= set(names), "consumed command set")
    canonical = {item["name"] for item in commands if set(item["surfaces"]) & {"paper", "velocity"}}
    require(set(selected) == canonical, "JVM consumer coverage")
    effects = sorted({item["effect"] for item in commands})
    effect_names = {value: value.upper().replace("-", "_") for value in effects}
    write(output, "CommandEffect", enum_source("CommandEffect", list(effect_names.values())))
    write(output, "CommandErrorBoundary", "public enum CommandErrorBoundary { HANDLER_DEFINED }")
    write(output, "CommandRequest", "public sealed interface CommandRequest permits UnavailableCommandRequest {}")
    write(output, "UnavailableCommandRequest", "public enum UnavailableCommandRequest implements CommandRequest { INSTANCE }")
    write(output, "CommandResponse", "public sealed interface CommandResponse permits UnavailableCommandResponse {}")
    write(output, "UnavailableCommandResponse", "public enum UnavailableCommandResponse implements CommandResponse { INSTANCE }")
    write(output, "CommandBinding", "public record CommandBinding(String name, CommandEffect effect, String response, CommandErrorBoundary errors) {}")
    rows = [f'        new CommandBinding("{item["name"]}", CommandEffect.{effect_names[item["effect"]]}, '
            f'"{item["response"]["envelope"]}", CommandErrorBoundary.HANDLER_DEFINED)'
            for item in sorted(commands, key=lambda value: value["name"])]
    digest = hashlib.sha256(b"".join(path.read_bytes() for path in sources)).hexdigest()
    body = "import java.util.List;\n\npublic final class CommandCatalog {\n"
    body += f'    public static final String SOURCE_SHA256 = "{digest}";\n'
    body += "    public static final List<CommandBinding> ALL = List.of(\n" + ",\n".join(rows) + "\n    );\n"
    body += f"    public static final int JVM_CONSUMED = {len(selected)};\n    private CommandCatalog() {{}}\n}}"
    write(output, "CommandCatalog", body)
    write(output, "SyncError", enum_source("SyncError", contract["errors"]))
    write(output, "EffectClass", enum_source("EffectClass", contract["effects"]))
