#!/usr/bin/env python3
"""Enforce the reviewed typed JVM adapter and single-menu-engine surface."""
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
    "ActionbarSnapshotAdapter.java", "DocsCommandAdapter.java", "FreshAuthorityAdapter.java",
    "HotbarMenuListener.java", "HotbarMenuTokenService.java", "InventorySyncService.java",
    "LkjmcPaperPlugin.java", "MenuResponseOwnership.java", "PaperEffectRouter.java",
    "PaperMenuAdapter.java", "PaperMenuProtocolAdapter.java", "PaperMenuSnapshots.java",
    "PaperSchedulerBridge.java", "ProfileApplicationAdapter.java",
}
VELOCITY_ALLOWED = {
    "LkjmcVelocityCommand.java", "LkjmcVelocityPlugin.java", "RoutingPlatform.java",
    "RoutingTarget.java", "VelocityLifecycle.java", "VelocityMotdAdapter.java",
    "VelocityProxyPlatform.java", "VelocityRoutingAdapter.java",
    "VelocitySchedulerBridge.java", "VelocityTabListAdapter.java", "VelocityTransferAdapter.java",
}
MENU_ALLOWED = {
    "DocsRouteRenderer.java", "MenuAction.java", "MenuBundle.java", "MenuController.java",
    "MenuFrame.java", "MenuRenderer.java", "MenuResult.java", "MenuRoute.java",
    "MenuSession.java", "MenuSnapshotView.java", "MenuTypes.java",
}
SYNC_ALLOWED = {
    "ClosedSyncDecoder.java", "ReconnectBackoff.java", "RetryGate.java", "StrictRecordReader.java",
    "SyncBootstrap.java", "SyncCache.java", "SyncConfig.java", "SyncCoordinator.java",
    "SyncHttpClient.java", "SyncKey.java", "SyncSnapshot.java",
}
SYNC_FORBIDDEN = (
    '"/command"', '"/web/', "MutationBridge", "TransferBridge", "ProfileApply",
    "PlayerTransfer", "sendPlayer", "saveProfile", "loadProfile",
)
FORBIDDEN_TEXT = (
    "DaemonClient", "HttpDaemonClient", "DaemonAccess", "DaemonRequest", "DaemonResponse",
    "VelocityCommands", "VelocityLkjmcCommand", "VelocityServerRegistry",
    "VelocityProfileTransferBridge", "VelocityModerationListener",
    "VelocityTemporarySendAdapter", "VelocityWakeJoinAdapter", "LkjmcCommandTree",
    "ClaimCommandAdapter", "EndExpeditionCommandAdapter", "ExchangeCommandAdapter",
    "ShopCommandAdapter", "MenuCommandAdapter", "FoliaSchedulerBridge",
    "UiSessionService", "UiEffectRunner", "UiEntrypoints", "UiUpdate", "UiFrame",
    "UiModel", "MenuDocument", "BindingRegistry", "LocalDocsMenu", "LKJMC_DAEMON_HTTP_",
    "getCommand(\"lkjmc\")",
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
)


def workspace_package_value(name):
    in_package = False
    for raw in (ROOT / "Cargo.toml").read_text().splitlines():
        line = raw.strip()
        if line.startswith("[") and line.endswith("]"):
            in_package = line == "[workspace.package]"
        elif in_package:
            match = re.fullmatch(rf"{re.escape(name)}\s*=\s*\"([^\"]+)\"", line)
            if match:
                return match.group(1)
    raise RuntimeError(f"missing workspace.package {name}")


VERSION = workspace_package_value("version")
LICENSE = workspace_package_value("license")


def jvm_resource_paths():
    return sorted(JVM.glob("*/src/*/resources"))


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
        fail(errors, "paper source set is not the reviewed attestation-gated allowlist")
    if source_names(VELOCITY) != VELOCITY_ALLOWED:
        fail(errors, "velocity source set is not the reviewed attestation-gated allowlist")
    menu = JVM / "common/src/main/java/com/lkjmc/common/menu"
    if source_names(menu) != MENU_ALLOWED:
        fail(errors, "common menu source set is not the reviewed selected-engine allowlist")
    sync = JVM / "common/src/main/java/com/lkjmc/common/sync"
    if source_names(sync) != SYNC_ALLOWED:
        fail(errors, "common sync source set is not the reviewed read-only allowlist")
    for path in sorted(sync.glob("*.java")):
        scan_text(errors, path, SYNC_FORBIDDEN, "read-only sync")
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
        if "version: '${version}'" not in plugin.read_text():
            fail(errors, "paper plugin.yml must expand the canonical build version")
    velocity_plugin = VELOCITY / "LkjmcVelocityPlugin.java"
    if "version = LkjmcBuildInfo.VERSION" not in velocity_plugin.read_text():
        fail(errors, "Velocity descriptor must use the canonical build version")
    for root in (*ACTIVE_TEXT_PATHS, *jvm_resource_paths()):
        for path in text_files(root):
            scan_text(errors, path, FORBIDDEN_TEXT, "active smoke/resource")


def manifest_value(manifest, name):
    manifest = manifest.replace("\r\n", "\n")
    match = re.search(rf"^{re.escape(name)}: ([^\n]+)$", manifest, re.M)
    return match.group(1) if match else None


def check_jar(path, errors):
    with zipfile.ZipFile(path) as jar:
        names = jar.namelist()
        if "META-INF/MANIFEST.MF" not in names:
            fail(errors, f"missing jar manifest: {path.relative_to(ROOT)}")
        else:
            manifest = jar.read("META-INF/MANIFEST.MF").decode("utf-8", errors="ignore")
            if manifest_value(manifest, "Implementation-Version") != VERSION:
                fail(errors, f"wrong jar version: {path.relative_to(ROOT)}")
            if manifest_value(manifest, "Bundle-License") != LICENSE:
                fail(errors, f"wrong jar license: {path.relative_to(ROOT)}")
            commit = manifest_value(manifest, "LKJMC-Build-Commit")
            if commit != "unknown" and not re.fullmatch(r"[0-9a-f]{40}", commit or ""):
                fail(errors, f"invalid jar build commit: {path.relative_to(ROOT)}")
            if manifest_value(manifest, "LKJMC-Build-Dirty") not in {"false", "unknown"}:
                fail(errors, f"invalid jar dirty state: {path.relative_to(ROOT)}")
        for forbidden in FORBIDDEN_PATHS:
            if any(name.startswith(forbidden) for name in names):
                fail(errors, f"withdrawn class path {forbidden}: {path.relative_to(ROOT)}")
        prefix = "com/lkjmc/common/sync/"
        for name in (item for item in names if item.startswith(prefix) and item.endswith(".class")):
            stem = name.removeprefix(prefix).removesuffix(".class").split("$", 1)[0] + ".java"
            if stem not in SYNC_ALLOWED:
                fail(errors, f"unreviewed sync class: {path.relative_to(ROOT)}!{name}")
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
            if f"version: '{VERSION}'" not in plugin or "0.0.0" in plugin:
                fail(errors, f"paper jar version is wrong: {path.relative_to(ROOT)}")
        if path.is_relative_to(JVM / "velocity") and "velocity-plugin.json" in names:
            descriptor = jar.read("velocity-plugin.json").decode("utf-8", errors="ignore")
            if f'"version":"{VERSION}"' not in descriptor or "0.0.0" in descriptor:
                fail(errors, f"Velocity jar version is wrong: {path.relative_to(ROOT)}")


def check_artifacts(errors):
    jars = sorted(JVM.rglob("build/libs/*.jar"))
    expected = {JVM / module / "build/libs" / f"{module}-all.jar"
                for module in ("common", "paper", "velocity")}
    for path in sorted(expected):
        if path not in jars:
            fail(errors, f"missing built shaded jar: {path.relative_to(ROOT)}")
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
