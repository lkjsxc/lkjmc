#!/usr/bin/env python3
"""Prove shipped Java plugins contain only local-safe presentation surfaces."""
from argparse import ArgumentParser
from pathlib import Path
import re
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
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
FORBIDDEN = (
    "DaemonClient", "HttpDaemonClient", "DaemonAccess", "DaemonRequest", "DaemonResponse",
    "VelocityCommands", "VelocityLkjmcCommand", "VelocityServerRegistry",
    "VelocityProfileTransferBridge", "VelocityModerationListener",
    "VelocityTemporarySendAdapter", "VelocityWakeJoinAdapter", "LkjmcCommandTree",
    "getCommand(\"lkjmc\")", "getCommandManager", "metaBuilder(",
)
FORBIDDEN_PATHS = (
    "com/lkjmc/common/daemon/", "com/lkjmc/common/command/",
    "com/lkjmc/common/claim/", "com/lkjmc/common/permission/",
    "com/lkjmc/common/transfer/", "com/lkjmc/common/ui/",
)


def fail(errors, message):
    errors.append(message)


def source_names(directory):
    return {path.name for path in directory.glob("*.java")}


def check_sources(errors):
    if source_names(PAPER) != PAPER_ALLOWED:
        fail(errors, "paper source set is not the local-safe allowlist")
    if source_names(VELOCITY) != VELOCITY_ALLOWED:
        fail(errors, "velocity source set is not the local-safe allowlist")
    for path in JVM.rglob("*.java"):
        text = path.read_text(encoding="utf-8")
        for token in FORBIDDEN:
            if token in text:
                fail(errors, f"withdrawn source token {token}: {path.relative_to(ROOT)}")
    plugin = (JVM / "paper/src/main/resources/plugin.yml").read_text(encoding="utf-8")
    commands = re.findall(r"^  ([a-z]+):$", plugin, re.M)
    if commands != ["menu", "docs"] or "  lkjmc:" in plugin:
        fail(errors, "plugin.yml must register only menu and docs")
    for directory in ("daemon", "command", "claim", "permission", "transfer", "ui"):
        if (JVM / "common/src/main/java/com/lkjmc/common" / directory).exists():
            fail(errors, f"withdrawn common source directory {directory}")


def check_jar(path, errors):
    with zipfile.ZipFile(path) as jar:
        names = jar.namelist()
        for forbidden in FORBIDDEN_PATHS:
            if any(name.startswith(forbidden) for name in names):
                fail(errors, f"withdrawn class path {forbidden}: {path.name}")
        for name in names:
            if any(token in name for token in FORBIDDEN[:11]):
                fail(errors, f"withdrawn class {name}: {path.name}")
        plugin = jar.read("plugin.yml").decode("utf-8") if "plugin.yml" in names else ""
        if path.parent.parent.name == "paper" and ("  lkjmc:" in plugin or "  menu:" not in plugin or "  docs:" not in plugin):
            fail(errors, f"paper jar command registration is unsafe: {path.name}")


def check_artifacts(errors):
    jars = []
    for module in ("paper", "velocity"):
        found = sorted((JVM / module / "build/libs").glob("*-all.jar"))
        if not found:
            fail(errors, f"missing built {module} shadow jar")
        jars.extend(found)
    for path in jars:
        check_jar(path, errors)


def main():
    parser = ArgumentParser()
    parser.add_argument("--artifacts", action="store_true")
    args = parser.parse_args()
    errors = []
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
