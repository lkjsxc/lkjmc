import os
import re
import subprocess
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
PROBES = [
    "all-snapshots-revisioned",
    "freshness-bound-pass",
    "reconnect-storm-pass",
    "request-budget-pass",
    "auth-invalidation-pass",
    "shutdown-clean",
    "duplicate-pollers-absent",
]
HARNESS_PROBES = set(PROBES[1:6])
SYNC_MAIN = ROOT / "platforms/jvm/common/src/main/java/com/lkjmc/common/sync"
SYNC_CLASSES = {
    "ReconnectBackoff.java", "SyncBootstrap.java", "SyncCache.java", "SyncConfig.java",
    "RetryGate.java", "SyncCoordinator.java", "SyncHttpClient.java", "SyncKey.java",
    "SyncPayloadValidator.java", "SyncSnapshot.java",
}
TRIGGERS = {
    "sync_admin_grants", "sync_admin_roles", "sync_player_claims", "sync_claim_chunks",
    "sync_claim_trusts", "sync_profiles", "sync_presence", "sync_settings",
    "sync_routing_instances", "sync_routing_observations", "sync_routing_ports",
    "sync_menus_shop", "sync_menus_kits", "sync_menus_votes", "sync_menus_plugins",
}


def text(path, override=None):
    return override(path) if override else path.read_text(encoding="utf-8")


def source_errors(root=ROOT, override=None):
    errors = []
    migration_path = root / "migrations/047-revisioned-sync.sql"
    migration = text(migration_path, override)
    found = set(re.findall(r"create trigger (sync_[a-z_]+)", migration))
    if found != TRIGGERS:
        errors.append(f"sync trigger coverage changed: {sorted(found ^ TRIGGERS)}")
    for token in ("writer_xid xid8", "pg_current_xact_id()", "unique (writer_xid, domain, key)"):
        if token not in migration:
            errors.append(f"transaction touch de-duplication absent: {token}")
    for token in ("octet_length(key) between 1 and 256", "octet_length(sync_key) not between 1 and 256"):
        if token not in migration:
            errors.append(f"UTF-8 key byte bound absent: {token}")
    if re.search(r"create trigger\s+\w+.*?\bon sync_(domain_revisions|change_feed)", migration, re.S):
        errors.append("sync metadata trigger recursion is forbidden")
    presence = re.search(r"create function sync_touch_presence\(\).*?end \$\$;", migration, re.S)
    if not presence or "sync_touch('presence'" not in presence.group() or "sync_touch('routing', 'network')" not in presence.group():
        errors.append("presence-to-routing transactional dependency is absent")
    store = text(root / "crates/lkjmc-store/src/sync.rs", override)
    feed = text(root / "crates/lkjmc-store/src/sync/feed.rs", override)
    if "IsolationLevel::RepeatableRead" not in store or ".read_only(true)" not in store:
        errors.append("snapshot revision and payload are not one read-only repeatable-read view")
    if "IsolationLevel::RepeatableRead" not in feed or ".read_only(true)" not in feed:
        errors.append("feed cursor and rows are not transactionally coherent")
    transport = text(root / "crates/lkjmc-daemon/src/transport/sync.rs", override)
    for token in ('"revision": value.revision', "credentialRevision", "run_blocking"):
        if token not in transport:
            errors.append(f"revisioned daemon snapshot boundary absent: {token}")
    authz = text(root / "crates/lkjmc-daemon/src/authz.rs", override)
    if '"paper" | "velocity"' not in authz or 'value == "lkjmc.sync.read"' not in authz:
        errors.append("sync subject surface/scope policy is open")
    for command in ("claim_read.rs", "player_actionbar.rs"):
        body = text(root / "crates/lkjmc-daemon/src/commands" / command, override)
        if "lkjmc_store::sync::snapshot" not in body or '"revision"' not in body:
            errors.append(f"legacy snapshot is not durably revisioned: {command}")
        if re.search(r"timestamp|count.*hash|generated_at.*revision", body, re.I):
            errors.append(f"legacy synthetic revision remains: {command}")
    coordinator = text(SYNC_MAIN / "SyncCoordinator.java", override)
    config = text(SYNC_MAIN / "SyncConfig.java", override)
    cache = text(SYNC_MAIN / "SyncCache.java", override)
    http = text(SYNC_MAIN / "SyncHttpClient.java", override)
    bounds = ("maxSubscriptions", "maxInflight", "maxEntries", "maxCacheBytes", "maxResponseBytes")
    if any(bound not in config for bound in bounds) or "while (entries.size() > maxEntries || bytes > maxBytes)" not in cache:
        errors.append("sync cache/request bounds are incomplete")
    if "readNBytes(config.maxResponseBytes() + 1)" not in http:
        errors.append("HTTP response read is unbounded")
    if coordinator.count("scheduleWithFixedDelay") != 1:
        errors.append("coordinator must own exactly one feed poller")
    if "checkpoint()" not in coordinator or "awaitClosed" not in coordinator:
        errors.append("cursor reload or bounded shutdown proof hook absent")
    validator = "SyncPayloadValidator.valid"
    if validator not in coordinator or coordinator.index(validator) > coordinator.index("cache.put"):
        errors.append("typed domain payload validation before cache update is absent")
    if "feedRetry.failed()" not in coordinator or "retry.failed()" not in coordinator:
        errors.append("snapshot/feed failure backoff is absent")
    maintenance = text(root / "crates/lkjmc-daemon/src/maintenance.rs", override)
    server = text(root / "crates/lkjmc-daemon/src/transport/server.rs", override)
    if "sync::run_retention" not in maintenance or server.count("state.start_maintenance()") != 1:
        errors.append("exactly one production sync retention worker is absent")
    actual = {path.name for path in SYNC_MAIN.glob("*.java")}
    if actual != SYNC_CLASSES:
        errors.append(f"reviewed read-only sync class allowlist changed: {sorted(actual ^ SYNC_CLASSES)}")
    for module, file in (("paper", "LkjmcPaperPlugin.java"), ("velocity", "VelocityLifecycle.java")):
        body = text(root / f"platforms/jvm/{module}/src/main/java/com/lkjmc/{module}/{file}", override)
        if body.count("Optional<SyncCoordinator>") != 1 or body.count("SyncBootstrap.fromEnvironment") != 1:
            errors.append(f"{module} does not own exactly one shared coordinator")
        if any(token in body for token in ("HttpClient", "ScheduledExecutor", "readString", "readAllBytes")):
            errors.append(f"{module} lifecycle performs scheduler-thread I/O")
    joined = "\n".join(text(path, override) for path in SYNC_MAIN.glob("*.java"))
    if any(token in joined for token in ('"/command"', '"/web/', "TransferBridge", "MutationBridge")):
        errors.append("sync classes contain a mutation or transfer bridge")
    if re.search(r"(print|log)[A-Za-z]*\s*\([^\n]*(credential|token)", joined, re.I):
        errors.append("sync credential may be printed")
    return errors


def prerequisites():
    errors = []
    try:
        parsed = urlparse(os.environ.get("LKJMC_STORE_TEST_DATABASE_URL", ""))
        if parsed.scheme not in {"postgres", "postgresql"} or not parsed.hostname or not parsed.path.strip("/"):
            errors.append("valid LKJMC_STORE_TEST_DATABASE_URL is required")
    except ValueError:
        errors.append("valid LKJMC_STORE_TEST_DATABASE_URL is required")
    try:
        version = subprocess.run(["java", "-version"], capture_output=True, text=True, timeout=10)
        output = version.stderr + version.stdout
        if version.returncode or not re.search(r'version "21(?:\.|\")', output):
            errors.append("Java 21 is required")
    except (OSError, subprocess.SubprocessError):
        errors.append("Java 21 is required")
    if not (ROOT / "gradlew").is_file():
        errors.append("Gradle wrapper is required")
    return errors


def command(values):
    result = subprocess.run(values, cwd=ROOT, capture_output=True, text=True, check=False)
    if result.returncode:
        print(result.stdout, end="")
        print(result.stderr, end="")
    return result.returncode


def run_probe(probe):
    if probe == "duplicate-pollers-absent":
        return 0
    if probe == "all-snapshots-revisioned":
        return command(["cargo", "test", "-p", "lkjmc-store", "--test", "sync", "--test", "sync_coherence", "--", "--nocapture"])
    if command(["cargo", "build", "-p", "lkjmc-daemon"]):
        return 1
    return command(["./gradlew", "--no-daemon", "--no-build-cache",
                    ":platforms:jvm:common:syncHarness", f"-PsyncProbe={probe}"])
