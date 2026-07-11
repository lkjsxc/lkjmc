#!/usr/bin/env python3
"""Containment probes for the JVM paths that have had unsafe implementations."""
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
JAVA = ROOT / "platforms/jvm"


def text(path):
    return path.read_text(encoding="utf-8")


def probe(name, condition):
    if not condition:
        raise SystemExit(f"failed {name}")
    print(f"ok {name}")


def main():
    http = text(JAVA / "common/src/main/java/com/lkjmc/common/daemon/HttpDaemonClient.java")
    token = re.search(r"Optional<String> currentToken\(\) \{(.*?)\n    \}", http, re.S)
    probe("scheduler-filesystem-zero", token is not None and "Files." not in token.group(1))

    session = text(JAVA / "paper/src/main/java/com/lkjmc/paper/ui/UiSessionService.java")
    runner = text(JAVA / "paper/src/main/java/com/lkjmc/paper/ui/UiEffectRunner.java")
    update = text(JAVA / "common/src/main/java/com/lkjmc/common/ui/kernel/UiUpdate.java")
    view = text(JAVA / "common/src/main/java/com/lkjmc/common/ui/kernel/UiView.java")
    dispatch = text(JAVA / "common/src/main/java/com/lkjmc/common/ui/kernel/UiActionDispatch.java")
    request = text(JAVA / "common/src/main/java/com/lkjmc/common/ui/kernel/UiRequest.java")
    probe("menu-old-response-dropped", all(value in session + runner + update for value in
        ["ConcurrentHashMap", "request.matches(model)", "sessions.accepts(player, request)"]))
    probe("request-correlation-complete", all(value in request for value in
        ["playerId.equals(model.playerId())", "issuedRequests().get(requestId)", "actionKey.equals"]))
    probe("stale-actions-disabled", "staleMutation" in view and "menu.stale.action-disabled" in view)
    probe("duplicate-click-contained", "model.pending(actionKey)" in dispatch and "completeMutation" in session)

    profiles = text(JAVA / "paper/src/main/java/com/lkjmc/paper/PlayerProfileAdapter.java")
    forbidden = ["ObjectInputStream", "ObjectOutputStream", "readObject", "BukkitObject"]
    probe("unsafe-deserialization-absent", all(value not in profiles for value in forbidden))

    schema = ROOT / "contracts/menus.schema.json"
    menus = subprocess.run([sys.executable, str(ROOT / "scripts/check-menus.py")], cwd=ROOT,
                           capture_output=True, text=True)
    probe("menu-schema-real", schema.is_file() and menus.returncode == 0 and "ok check-menus" in menus.stdout)

    adventure = text(JAVA / "common/src/main/java/com/lkjmc/common/ui/binding/AdventureBinding.java")
    confirm = text(ROOT / "contracts/menus/adventures-end-confirm.json")
    direct = "".join(text(path) for path in [
        JAVA / "common/src/main/java/com/lkjmc/common/ui/binding/PlanBodies.java",
        JAVA / "paper/src/main/java/com/lkjmc/paper/EndExpeditionCommandAdapter.java",
        JAVA / "paper/src/main/java/com/lkjmc/paper/PaperAdminCommandAdapter.java",
        JAVA / "velocity/src/main/java/com/lkjmc/velocity/VelocityLkjmcCommand.java"])
    probe("adventure-confirmation-real", "Views.open(\"adventures-end-confirm\")" in adventure
        and "Views.daemon(\"adventure.purchase\"" not in adventure
        and '"eulaAcceptance": true' in confirm)
    probe("direct-eula-acceptance-withdrawn", "acceptMinecraftEula" not in direct)
    confirmation = text(ROOT / "crates/lkjmc-daemon/src/commands/adventure_confirmation.rs")
    purchase = text(ROOT / "crates/lkjmc-daemon/src/commands/adventure_api/purchase.rs")
    temporary = text(ROOT / "crates/lkjmc-daemon/src/commands/temporary_api/create.rs")
    delivery = text(ROOT / "crates/lkjmc-daemon/src/commands/player_shop_delivery.rs")
    instances = text(ROOT / "crates/lkjmc-daemon/src/commands/instance_api.rs")
    bootstrap = text(ROOT / "crates/lkjmc-daemon/src/commands/bootstrap_api.rs")
    apply = text(ROOT / "crates/lkjmc-daemon/src/commands/bootstrap_api/apply.rs")
    cli = "".join(text(path) for path in [
        ROOT / "crates/lkjmc-cli/src/args_bootstrap.rs",
        ROOT / "crates/lkjmc-cli/src/args_instance.rs",
        ROOT / "crates/lkjmc-cli/src/commands_bootstrap.rs",
        ROOT / "crates/lkjmc-cli/src/commands_instance.rs"])
    locales = "".join(text(path) for path in [
        ROOT / "config/locales/en.json", ROOT / "config/locales/ja.json"])
    protected = [purchase, temporary, delivery, instances, bootstrap, apply]
    probe("eula-confirmation-contract-unified",
        'const CODE: &str = "adventure.confirmation_required"' in confirmation
        and all("adventure_confirmation::required" in source for source in protected)
        and "instance.error" not in purchase and "bootstrap.eula_required" not in bootstrap)
    probe("eula-confirmation-before-effects",
        purchase.index("if !adventure_confirmation::accepted") < purchase.index("with_connection(state")
        and temporary.index("if !adventure_confirmation::accepted") < temporary.index("with_connection(state")
        and "request::require_eula" not in temporary
        and '"bootstrap.status" => status' in bootstrap
        and "json!({\"acceptMinecraftEula\": true})" not in bootstrap)
    probe("eula-consent-origin-constrained",
        "acceptMinecraftEula" not in cli
        and "Value::Bool(true)" not in cli + delivery
        and "acceptMinecraftEula" not in direct
        and "adventure.end.eula.required" not in direct + locales)

    paper = text(JAVA / "paper/src/main/java/com/lkjmc/paper/LkjmcPaperPlugin.java")
    tree = text(JAVA / "common/src/main/java/com/lkjmc/common/command/LkjmcCommandTree.java")
    probe("registered-handlers-complete", "getCommand(\"party\")).setExecutor(commands)" in paper
        and "daemon-token create" not in tree and "daemon-token revoke" not in tree)

    folia = text(JAVA / "paper/src/main/java/com/lkjmc/paper/FoliaSchedulerBridge.java")
    heartbeat = text(JAVA / "paper/src/main/java/com/lkjmc/paper/ServerHeartbeat.java")
    teleports = text(JAVA / "paper/src/main/java/com/lkjmc/paper/TeleportCommandAdapter.java")
    routes = ["ProfileTransferListener.java", "TeleportArrivalListener.java", "HomeCommandAdapter.java",
              "WarpCommandAdapter.java", "EndExpeditionReturnService.java"]
    probe("folia-affinity-suite", "getGlobalRegionScheduler" in folia and all(
        "runGlobal" in text(JAVA / "paper/src/main/java/com/lkjmc/paper" / route) for route in routes))
    probe("heartbeat-http-off-scheduler", "scheduler.runGlobal(this::capture)" in heartbeat
        and "scheduler.runAsync(() -> send(snapshot))" in heartbeat)
    probe("teleport-feedback-completed", "teleportAsync(targetLocation).whenComplete" in teleports
        and "completeAccept(source, target" in teleports)

    velocity = "".join(text(path) for path in [
        JAVA / "velocity/src/main/java/com/lkjmc/velocity/VelocityTransferCoordinator.java",
        JAVA / "velocity/src/main/java/com/lkjmc/velocity/VelocityHubCommand.java",
        JAVA / "velocity/src/main/java/com/lkjmc/velocity/VelocitySendAdapter.java",
        JAVA / "velocity/src/main/java/com/lkjmc/velocity/VelocityProfileTransferBridge.java"])
    probe("velocity-transfer-result-real", "fireAndForget" not in velocity
        and "result.isSuccessful()" in velocity and "transfer failed" in velocity)


if __name__ == "__main__":
    main()
