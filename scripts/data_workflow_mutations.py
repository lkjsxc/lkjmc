#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def old_path_errors():
    errors = []
    java = source_text(ROOT / "platforms", ".java")
    for marker in ("ObjectOutputStream", "ObjectInputStream", "BukkitObject"):
        if marker in java: errors.append(f"old Java serialization remains: {marker}")
    daemon_player = read("crates/lkjmc-daemon/src/commands/player.rs")
    active_commands = daemon_player + read(
        "crates/lkjmc-daemon/src/commands/command_registrations.rs"
    ) + source_text(ROOT / "contracts/commands", ".json")
    if "player.transfer.saved" in active_commands:
        errors.append("audit-only transfer command remains registered")
    temporary = read("crates/lkjmc-daemon/src/commands/temporary_api/transfer.rs")
    if "command.denied_unproved" not in temporary or "create_intent" in temporary:
        errors.append("removed temporary transfer command does not fail closed")
    active_profile = daemon_player + source_text(ROOT / "contracts/commands", ".json")
    for marker in ("payloadBase64", "paper-bukkit-object-stream", "decode_payload"):
        if marker in active_profile: errors.append(f"opaque daemon profile path remains: {marker}")
    cli_player = read("crates/lkjmc-cli/src/commands_player.rs")
    for marker in ("payloadBase64", "Sha256::digest", "payload_base64"):
        if marker in cli_player: errors.append(f"caller profile integrity path remains: {marker}")
    store = source_text(ROOT / "crates/lkjmc-store/src", ".rs")
    for marker in ("temporary_transfer_intents", "payload_format", "NewTransferIntent"):
        if marker in store: errors.append(f"superseded store path remains: {marker}")
    migration = read("migrations/045-durable-data-workflows.sql")
    for marker in ("drop table temporary_transfer_intents", "untyped-profile", "lkjmc-profile-one"):
        if marker not in migration: errors.append(f"cutover marker missing: {marker}")
    profile = read("crates/lkjmc-core/src/profile_envelope.rs")
    required = ("inventory", "armor", "offhand", "selected_hotbar_slot", "ender_chest",
                "experience", "vitals", "potion_effects", "game_mode", "plugin_data",
                "homes", "warps", "points", "achievements", "settings", "language")
    for field in required:
        if f"pub {field}:" not in profile: errors.append(f"profile field missing: {field}")
    return errors


def source_text(root, suffix):
    return "\n".join(path.read_text(encoding="utf-8") for path in root.rglob(f"*{suffix}")
                      if "build" not in path.parts)


def read(path):
    return (ROOT / path).read_text(encoding="utf-8")
