#!/usr/bin/env python3
"""Reject withdrawn Java daemon surfaces from source, assets, docs, and jars."""
from argparse import ArgumentParser
from pathlib import Path
import re
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
JVM = ROOT / "platforms/jvm"
PAPER = JVM / "paper/src/main/java/com/lkjmc/paper"
VELOCITY = JVM / "velocity/src/main/java/com/lkjmc/velocity"
PAPER_ALLOWED = {
    "DocsCommandAdapter.java", "HotbarMenuListener.java", "HotbarMenuTokenService.java",
    "InventorySyncService.java", "LkjmcPaperPlugin.java", "LocalDocsMenu.java",
}
VELOCITY_ALLOWED = {
    "LkjmcVelocityPlugin.java", "VelocityLifecycle.java", "VelocityMotdAdapter.java",
    "VelocityTabListAdapter.java",
}
FORBIDDEN_TEXT = (
    "DaemonClient", "HttpDaemonClient", "DaemonAccess", "DaemonRequest", "DaemonResponse",
    "VelocityCommands", "VelocityLkjmcCommand", "VelocityServerRegistry",
    "VelocityProfileTransferBridge", "VelocityModerationListener",
    "VelocityTemporarySendAdapter", "VelocityWakeJoinAdapter", "LkjmcCommandTree",
    "ClaimCommandAdapter", "EndExpeditionCommandAdapter", "ExchangeCommandAdapter",
    "ShopCommandAdapter", "MenuCommandAdapter", "FoliaSchedulerBridge",
    "UiSessionService", "UiEffectRunner", "UiEntrypoints", "UiUpdate", "UiFrame",
    "UiModel", "MenuDocument", "BindingRegistry", "LKJMC_DAEMON_HTTP_",
    "getCommand(\"lkjmc\")", "getCommandManager", "metaBuilder(",
)
DOC_FORBIDDEN = FORBIDDEN_TEXT[:27] + (
    "com.lkjmc.common.daemon", "com.lkjmc.common.command",
    "com.lkjmc.common.claim", "com.lkjmc.common.permission",
    "com.lkjmc.common.transfer", "com.lkjmc.common.ui",
)
FORBIDDEN_PATHS = (
    "com/lkjmc/common/daemon/", "com/lkjmc/common/command/",
    "com/lkjmc/common/claim/", "com/lkjmc/common/permission/",
    "com/lkjmc/common/transfer/", "com/lkjmc/common/ui/",
)
ACTIVE_TEXT_PATHS = (
    ROOT / "scripts/check-minecraft-smoke.sh",
    ROOT / "scripts/check-minecraft-claim-smoke.sh",
    ROOT / "scripts/check-playable-smoke.sh",
    ROOT / "scripts/verify-live.sh",
    ROOT / "tests/smoke",
    JVM / "paper/src/main/resources",
    JVM / "velocity/src/main/resources",
    JVM / "common/src/main/resources",
)


def fail(errors, message):
    errors.append(message)


def text_files(path):
    if path.is_file():
        return [path]
    return sorted(item for item in path.rglob("*") if item.is_file()) if path.exists() else []


def scan_text(errors, path, tokens, label):
    text = path.read_text(encoding="utf-8", errors="ignore")
    for token in tokens:
        if token in text:
            fail(errors, f"withdrawn {label} token {token}: {path.relative_to(ROOT)}")


def source_names(directory):
    return {path.name for path in directory.glob("*.java")}


def check_docs(errors):
    for path in sorted(DOCS.rglob("*.md")):
        if "archive" not in path.relative_to(DOCS).parts:
            scan_text(errors, path, DOC_FORBIDDEN, "nonarchive documentation")


def check_plugin_metadata(errors, path, label):
    text = path.read_text(encoding="utf-8", errors="ignore")
    commands = re.findall(r"^  ([a-z]+):$", text, re.M)
    if commands != ["menu", "docs"] or "  lkjmc:" in text:
        fail(errors, f"{label} must register only menu and docs")
    scan_text(errors, path, FORBIDDEN_TEXT, label)


def check_sources(errors):
    if source_names(PAPER) != PAPER_ALLOWED:
        fail(errors, "paper source set is not the local-safe allowlist")
    if source_names(VELOCITY) != VELOCITY_ALLOWED:
        fail(errors, "velocity source set is not the local-safe allowlist")
    for path in sorted(JVM.rglob("*.java")):
        scan_text(errors, path, FORBIDDEN_TEXT, "Java source")
    for directory in ("daemon", "command", "claim", "permission", "transfer", "ui"):
        if (JVM / "common/src/main/java/com/lkjmc/common" / directory).exists():
            fail(errors, f"withdrawn common source directory {directory}")
    plugin = JVM / "paper/src/main/resources/plugin.yml"
    if not plugin.is_file():
        fail(errors, "missing paper plugin.yml")
    else:
        check_plugin_metadata(errors, plugin, "paper plugin.yml")
    for root in ACTIVE_TEXT_PATHS:
        for path in text_files(root):
            scan_text(errors, path, FORBIDDEN_TEXT, "active smoke/resource")


def check_jar(path, errors):
    with zipfile.ZipFile(path) as jar:
        names = jar.namelist()
        for forbidden in FORBIDDEN_PATHS:
            if any(name.startswith(forbidden) for name in names):
                fail(errors, f"withdrawn class path {forbidden}: {path.relative_to(ROOT)}")
        for name in names:
            payload = jar.read(name)
            for token in FORBIDDEN_TEXT:
                if token.encode() in name.encode() or token.encode() in payload:
                    fail(errors, f"withdrawn jar token {token}: {path.relative_to(ROOT)}!{name}")
        if path.is_relative_to(JVM / "paper") and "plugin.yml" in names:
            plugin = jar.read("plugin.yml").decode("utf-8", errors="ignore")
            commands = re.findall(r"^  ([a-z]+):$", plugin, re.M)
            if commands != ["menu", "docs"] or "  lkjmc:" in plugin:
                fail(errors, f"paper jar command registration is unsafe: {path.relative_to(ROOT)}")


def check_artifacts(errors):
    jars = sorted(JVM.rglob("build/libs/*.jar"))
    for module in ("paper", "velocity"):
        if not any(path.is_relative_to(JVM / module) for path in jars):
            fail(errors, f"missing built {module} jar")
    for path in jars:
        check_jar(path, errors)


def main():
    parser = ArgumentParser()
    parser.add_argument("--artifacts", action="store_true")
    args = parser.parse_args()
    errors = []
    check_docs(errors)
    check_sources(errors)
    if args.artifacts:
        check_artifacts(errors)
    if errors:
        print("\n".join(errors))
        return 1
    print("ok check-jvm-containment" + (" artifacts" if args.artifacts else " source"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
