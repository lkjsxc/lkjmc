#!/usr/bin/env python3
"""Migration-aware immutable release update for the supported single-host deployment."""
import argparse
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
import pwd
import grp
import re
import shutil
import socket
import stat
import struct
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from urllib.parse import urlsplit

RELEASES = Path("/opt/lkjmc/releases")
CURRENT = RELEASES / "current"
SERVICE = "lkjmc-daemon.service"
UNIT = Path("/etc/systemd/system/lkjmc-daemon.service")
CLI_LINK = Path("/usr/local/bin/lkjmc")
CONFIG = Path("/etc/lkjmc/lkjmc.json")
ENV_FILE = Path("/etc/lkjmc/daemon.env")
DATA = Path("/var/lib/lkjmc")
INSTANCES = DATA / "instances"
CREDENTIALS = DATA / "private/plugin-credentials"
DEPLOYMENTS = Path("/var/lib/private/lkjmc-deployments")
BACKUP_ROOT = Path("/var/backups/lkjmc")
FENCE = Path("/etc/lkjmc/deployment-fence.json")
FENCE_DROPIN = Path("/etc/systemd/system/lkjmc-daemon.service.d/10-deployment-fence.conf")
START_PERMIT = Path("/run/lkjmc-deploy-start-permit")
LOCK = Path("/run/lkjmc-deploy.lock")
SYSTEMCTL = Path("/usr/bin/systemctl")
RUNUSER = Path("/usr/sbin/runuser")
PSQL = Path("/usr/bin/psql")
PGRESTORE = Path("/usr/bin/pg_restore")
PGREP = Path("/usr/bin/pgrep")
PYTHON = Path("/usr/bin/python3")
TRUSTED_COMMAND_TARGET_ROOTS = {
    SYSTEMCTL: (Path("/usr/bin"),),
    RUNUSER: (Path("/usr/sbin"),),
    PSQL: (Path("/usr/bin"), Path("/usr/share/postgresql-common")),
    PGRESTORE: (Path("/usr/bin"), Path("/usr/share/postgresql-common")),
    PGREP: (Path("/usr/bin"),),
    PYTHON: (Path("/usr/bin"),),
}
MAX_COMMAND_SYMLINKS = 8
HEX40 = re.compile(r"[0-9a-f]{40}")
HEX64 = re.compile(r"[0-9a-f]{64}")
SAFE_LABEL = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,127}")
ARTIFACT_FIELDS = {"component", "kind", "path", "provenance", "sha256", "size", "source"}
SAFE_ENV = {
    "PATH": "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
    "LANG": "C",
    "LC_ALL": "C",
}
EXPECTED_ARTIFACTS = {
    "lkjmc": "binary",
    "lkjmc-daemon": "binary",
    "lkjmc-discord": "binary",
    "lkjmc-deploy-release": "binary",
    "lkjmc-install-artifacts": "binary",
    "lkjmc-backup-postgres": "binary",
    "lkjmc-restore-postgres": "binary",
    "lkjmc-bootstrap-after-start": "binary",
    "lkjmc-deployment-fence-check": "binary",
    "lkjmc-daemon.service": "config",
    "lkjmc-deployment-fence.conf": "config",
    "lkjmc-common.jar": "jar",
    "lkjmc-paper.jar": "jar",
    "lkjmc-velocity.jar": "jar",
}


class DeployError(RuntimeError):
    pass


def fail(message):
    raise DeployError(message)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def fsync_dir(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def fsync_regular(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            fail(f"fsync target is not regular: {path}")
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def regular(path, label, private=False):
    try:
        value = path.lstat()
    except FileNotFoundError:
        fail(f"missing {label}: {path}")
    if not stat.S_ISREG(value.st_mode) or path.is_symlink():
        fail(f"{label} is not a regular file: {path}")
    if private and stat.S_IMODE(value.st_mode) & 0o077:
        fail(f"{label} is not private: {path}")
    return value


def trusted_command(path, label="required command"):
    allowed_roots = TRUSTED_COMMAND_TARGET_ROOTS.get(path)
    if allowed_roots is None:
        fail(f"unexpected {label}: {path}")
    if not path.is_absolute():
        fail(f"{label} is not absolute: {path}")
    resolved = path
    seen = set()
    for _ in range(MAX_COMMAND_SYMLINKS + 1):
        try:
            metadata = resolved.lstat()
        except (FileNotFoundError, OSError) as error:
            fail(f"missing or unreadable {label}: {resolved}: {error}")
        require_root_ancestry(resolved.parent, f"{label} parent")
        if not stat.S_ISLNK(metadata.st_mode):
            break
        if metadata.st_uid != 0:
            fail(f"{label} symlink is not root-owned: {resolved}")
        identity = (metadata.st_dev, metadata.st_ino)
        if identity in seen:
            fail(f"{label} symlink chain contains a loop: {resolved}")
        seen.add(identity)
        try:
            target = Path(os.readlink(resolved))
        except OSError as error:
            fail(f"cannot read {label} symlink: {resolved}: {error}")
        try:
            confirmed = resolved.lstat()
        except OSError as error:
            fail(f"cannot re-observe {label} symlink: {resolved}: {error}")
        if (confirmed.st_dev, confirmed.st_ino, confirmed.st_mode, confirmed.st_uid) != \
                (metadata.st_dev, metadata.st_ino, metadata.st_mode, metadata.st_uid):
            fail(f"{label} symlink identity changed during validation: {resolved}")
        if not target.is_absolute():
            target = resolved.parent / target
        resolved = Path(os.path.abspath(target))
    else:
        fail(f"{label} exceeds {MAX_COMMAND_SYMLINKS} symlinks")
    if not any(root in resolved.parents for root in allowed_roots):
        fail(f"{label} resolves outside its allowed target roots: {resolved}")
    require_root_ancestry(resolved, label)
    confirmed = regular(resolved, label)
    if (confirmed.st_dev, confirmed.st_ino) != (metadata.st_dev, metadata.st_ino):
        fail(f"{label} identity changed during validation: {resolved}")
    if not root_owned_safe(confirmed) or not stat.S_IMODE(confirmed.st_mode) & 0o111:
        fail(f"{label} ownership or executable mode is unsafe: {resolved}")
    return resolved


def read_json(path, label):
    regular(path, label)
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, TypeError, ValueError) as error:
        fail(f"invalid {label}: {error}")


def root_owned_safe(metadata):
    return metadata.st_uid == 0 and not (stat.S_IMODE(metadata.st_mode) & 0o022)


def require_root_directory(path, label, mode=None):
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        fail(f"missing {label}: {path}")
    if not stat.S_ISDIR(metadata.st_mode) or path.is_symlink() or not root_owned_safe(metadata):
        fail(f"{label} is not a root-owned directory: {path}")
    permissions = stat.S_IMODE(metadata.st_mode)
    if mode is not None and permissions != mode:
        fail(f"{label} mode is unsafe: {path}")
    return metadata


def require_root_ancestry(path, label):
    resolved = path.resolve(strict=True)
    if resolved != path:
        fail(f"{label} contains a symlinked path component")
    current = Path("/")
    for part in path.parts[1:]:
        current /= part
        metadata = current.lstat()
        if current == path and stat.S_ISREG(metadata.st_mode):
            if metadata.st_uid != 0 or stat.S_IMODE(metadata.st_mode) & 0o022:
                fail(f"{label} file ownership or mode is unsafe: {current}")
        elif not stat.S_ISDIR(metadata.st_mode) or metadata.st_uid != 0 \
                or stat.S_IMODE(metadata.st_mode) & 0o022:
            fail(f"{label} directory ownership or mode is unsafe: {current}")


@contextmanager
def deployment_lock():
    descriptor = os.open(LOCK, os.O_RDWR | os.O_CREAT | os.O_CLOEXEC | os.O_NOFOLLOW, 0o600)
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_uid != os.geteuid() \
                or stat.S_IMODE(metadata.st_mode) != 0o600:
            fail("deployment lock ownership or mode differs")
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            fail("another lkjmc deployment operation holds the global lock")
        yield
    finally:
        os.close(descriptor)


def database_url_from_environment(path=ENV_FILE):
    metadata = regular(path, "daemon environment", private=True)
    if metadata.st_size < 1 or metadata.st_size > 4096:
        fail("daemon environment size differs")
    raw = path.read_bytes()
    if raw.count(b"\n") != 1 or not raw.endswith(b"\n") or b"\x00" in raw:
        fail("daemon environment must contain one newline-terminated assignment")
    try:
        line = raw[:-1].decode("ascii")
    except UnicodeError:
        fail("daemon environment is not ASCII")
    prefix = "LKJMC_DATABASE_URL="
    if not line.startswith(prefix) or any(ord(character) < 0x20 or character.isspace() for character in line):
        fail("daemon environment must contain only LKJMC_DATABASE_URL")
    value = line[len(prefix):]
    parsed = urlsplit(value)
    try:
        port = parsed.port
        username = parsed.username
        password = parsed.password
    except ValueError:
        fail("daemon database URL is malformed")
    if parsed.scheme not in ("postgres", "postgresql") or username != "lkjmc" or not password \
            or parsed.hostname not in ("127.0.0.1", "localhost") or port != 5432 \
            or parsed.path != "/lkjmc" or parsed.query or parsed.fragment:
        fail("daemon database URL differs from the private local database")
    return value, metadata


def artifact_location(kind, name):
    if kind == "jar":
        return Path("jars") / name
    if kind == "config":
        return Path("share") / name
    return Path("bin") / name


def parse_manifest(path, expected_digest=None, strict_artifacts=False, require_source=False):
    regular(path, "artifact manifest")
    sidecar = path.with_suffix(path.suffix + ".sha256")
    regular(sidecar, "artifact manifest sidecar")
    raw = path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if expected_digest is not None and digest != expected_digest:
        fail("artifact manifest differs from --manifest-sha256")
    try:
        sidecar_text = sidecar.read_text(encoding="ascii")
    except (OSError, UnicodeError) as error:
        fail(f"invalid artifact manifest sidecar: {error}")
    if sidecar_text != f"{digest}  artifact-manifest.json\n":
        fail("artifact manifest checksum sidecar differs")
    try:
        data = json.loads(raw)
    except (TypeError, ValueError) as error:
        fail(f"invalid artifact manifest: {error}")
    if not isinstance(data, dict) or data.get("schemaVersion") != 1:
        fail("unsupported artifact manifest schema")
    commit = data.get("commit", "")
    if not HEX40.fullmatch(commit):
        fail("invalid artifact manifest commit")
    artifacts = data.get("artifacts")
    if not isinstance(artifacts, list) or not artifacts:
        fail("artifact manifest has no artifacts")
    parsed = {}
    for item in artifacts:
        if not isinstance(item, dict) or set(item) != ARTIFACT_FIELDS:
            fail("artifact manifest fields differ")
        name = item.get("path")
        kind = item.get("kind")
        size = item.get("size")
        if not isinstance(name, str) or Path(name).name != name or name in ("", ".", ".."):
            fail("unsafe artifact path")
        if name in parsed:
            fail("duplicate artifact path")
        if kind not in ("binary", "jar", "config"):
            fail("unsupported artifact kind")
        if (name.endswith(".jar")) != (kind == "jar"):
            fail("artifact kind differs from path")
        if not HEX64.fullmatch(item.get("sha256", "")):
            fail("invalid artifact SHA-256")
        if isinstance(size, bool) or not isinstance(size, int) or size < 0:
            fail("invalid artifact size")
        parsed[name] = item
    if strict_artifacts and {name: item["kind"] for name, item in parsed.items()} != EXPECTED_ARTIFACTS:
        fail("release artifact set differs from the deployer contract")
    if require_source:
        source = path.parent / "source"
        if source.is_symlink() or not source.is_dir():
            fail("release source directory is unsafe")
        actual = {entry.name for entry in source.iterdir() if entry.is_file() and not entry.is_symlink()}
        if actual != set(parsed) or any(entry.is_symlink() or not entry.is_file() for entry in source.iterdir()):
            fail("release source closure differs from manifest")
        for name, item in parsed.items():
            artifact = source / name
            value = regular(artifact, "release artifact")
            if value.st_size != item["size"] or sha256(artifact) != item["sha256"]:
                fail(f"release artifact differs: {name}")
            if stat.S_IMODE(value.st_mode) & 0o022:
                fail(f"release artifact is group/other writable: {name}")
    return {"commit": commit, "digest": digest, "data": data, "artifacts": parsed, "path": path}


def load_anchored_release(release_root, manifest_digest):
    if not release_root.is_absolute() or release_root.is_symlink() or not release_root.is_dir():
        fail("--release-root must be an absolute, non-symlink directory")
    if not HEX64.fullmatch(manifest_digest):
        fail("--manifest-sha256 must be one lowercase SHA-256 digest")
    return parse_manifest(
        release_root / "artifact-manifest.json",
        expected_digest=manifest_digest,
        strict_artifacts=True,
        require_source=True,
    )


def verify_running_deployer(release):
    running = Path(__file__).resolve()
    regular(running, "running release deployer")
    expected = release["artifacts"]["lkjmc-deploy-release"]
    if running != release["path"].parent / "source/lkjmc-deploy-release" \
            or running.stat().st_size != expected["size"] or sha256(running) != expected["sha256"]:
        fail("invoke the exact deployer from the anchored release source directory")


def secure_release_for_root(release):
    root = release["path"].parent
    require_root_ancestry(root, "release root")
    require_root_ancestry(root / "source", "release source")
    for path in (release["path"], release["path"].with_suffix(release["path"].suffix + ".sha256")):
        require_root_ancestry(path, "release metadata")
    for name in release["artifacts"]:
        require_root_ancestry(root / "source" / name, "release artifact")


def installed_manifest(target):
    candidates = (target / "meta/artifact-manifest.json", target / "artifact-manifest.json")
    for candidate in candidates:
        if candidate.exists() and not candidate.is_symlink():
            return candidate
    fail(f"installed release manifest missing: {target}")


def validate_installed_release(target, expected_commit=None, expected_manifest_digest=None):
    if target.is_symlink() or not target.is_dir():
        fail(f"installed release target is unsafe: {target}")
    require_root_ancestry(target, "installed release target")
    manifest_path = installed_manifest(target)
    require_root_ancestry(manifest_path, "installed release manifest")
    require_root_ancestry(
        manifest_path.with_suffix(manifest_path.suffix + ".sha256"),
        "installed release manifest sidecar",
    )
    manifest = parse_manifest(manifest_path)
    if expected_commit is not None and manifest["commit"] != expected_commit:
        fail("installed release commit differs")
    if expected_manifest_digest is not None and manifest["digest"] != expected_manifest_digest:
        fail("installed release manifest differs")
    for name, item in manifest["artifacts"].items():
        path = target / artifact_location(item["kind"], name)
        require_root_ancestry(path.parent, "installed artifact directory")
        value = regular(path, "installed artifact")
        if not root_owned_safe(value):
            fail(f"installed artifact ownership or mode is unsafe: {name}")
        if value.st_size != item["size"] or sha256(path) != item["sha256"]:
            fail(f"installed artifact differs: {name}")
        if stat.S_IMODE(value.st_mode) & 0o022:
            fail(f"installed artifact is group/other writable: {name}")
    return manifest


def resolve_current(expected_commit):
    releases = RELEASES.lstat() if RELEASES.exists() else None
    if releases is None or not stat.S_ISDIR(releases.st_mode) or RELEASES.is_symlink() \
            or releases.st_uid != 0 or stat.S_IMODE(releases.st_mode) & 0o022:
        fail("release root ownership or mode is unsafe")
    if not CURRENT.is_symlink():
        fail("current release pointer is not a symlink")
    if not CLI_LINK.is_symlink() or os.readlink(CLI_LINK) != "/opt/lkjmc/releases/current/bin/lkjmc":
        fail("operator CLI pointer differs from the canonical current path")
    target = CURRENT.resolve(strict=True)
    try:
        target.relative_to(RELEASES)
    except ValueError:
        fail("current release pointer escapes the release root")
    if target.parent != RELEASES or target.name != expected_commit:
        fail("current release pointer differs from --from-commit")
    target_metadata = target.stat()
    if target_metadata.st_uid != 0 or stat.S_IMODE(target_metadata.st_mode) & 0o022:
        fail("current release target ownership or mode is unsafe")
    manifest = validate_installed_release(target, expected_commit)
    return target, manifest


def run(arguments, *, timeout=120, check=True, env=None):
    command = tuple(str(value) for value in arguments)
    try:
        result = subprocess.run(
            command,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
            env=SAFE_ENV if env is None else env,
        )
    except (OSError, subprocess.SubprocessError) as error:
        fail(f"command failed to execute: {Path(command[0]).name}: {error}")
    if check and result.returncode:
        detail = (result.stderr or result.stdout).strip().splitlines()
        suffix = f": {detail[-1]}" if detail else ""
        fail(f"command failed: {Path(command[0]).name}{suffix}")
    return result


def run_as(user, arguments, *, timeout=120):
    return run((RUNUSER, "-u", user, "--", *arguments), timeout=timeout)


def json_command(user, arguments, label, *, timeout=120):
    result = run_as(user, arguments, timeout=timeout)
    try:
        return json.loads(result.stdout)
    except (TypeError, ValueError) as error:
        fail(f"invalid {label} JSON: {error}")


def migration_marker():
    query = (
        "select coalesce(jsonb_agg(jsonb_build_object('version',version,'name',name,"
        "'checksum',coalesce(checksum,'')) order by version),'[]'::jsonb)::text "
        "from schema_migrations"
    )
    result = run_as(
        "postgres",
        (PSQL, "-d", "lkjmc", "-X", "--quiet", "--no-align", "--tuples-only", "-v", "ON_ERROR_STOP=1", "-c", query),
    )
    try:
        rows = json.loads(result.stdout)
    except (TypeError, ValueError) as error:
        fail(f"invalid migration ledger: {error}")
    if not isinstance(rows, list):
        fail("migration ledger is not an array")
    return rows


def validate_network_topology(network):
    instances = network.get("instances")
    expected_instances = {
        "hub": ("folia", "hub-java"),
        "proxy": ("velocity", "proxy-java"),
        "survival": ("folia", "survival-java"),
    }
    if not isinstance(instances, list) or len(instances) != 3 or not all(isinstance(item, dict) for item in instances):
        fail("operator configuration must contain exactly proxy, hub, and survival")
    instance_ids = [item.get("id") for item in instances]
    if len(set(instance_ids)) != 3:
        fail("operator configuration has duplicate instance IDs")
    observed_instances = {
        item["id"]: (item.get("kind"), item.get("listener")) for item in instances
    }
    if observed_instances != expected_instances \
            or any(item.get("desiredState") != "running" or item.get("owner") != "lkjmc-daemon" for item in instances):
        fail("operator instance topology differs from the supported running network")
    if any(not isinstance(item.get("assetIds"), list) or len(item["assetIds"]) != 1 for item in instances):
        fail("each supported instance must reference one immutable server asset")
    by_instance = {item["id"]: item for item in instances}
    if by_instance["hub"]["assetIds"] != by_instance["survival"]["assetIds"] \
            or by_instance["proxy"]["assetIds"] == by_instance["hub"]["assetIds"]:
        fail("backend and proxy server asset assignments differ from the supported topology")
    listeners = network.get("listeners")
    expected_listeners = {
        "hub-java": ("127.0.0.1", 25566),
        "proxy-java": ("0.0.0.0", 25591),
        "survival-java": ("127.0.0.1", 25567),
    }
    if not isinstance(listeners, list) or len(listeners) != 3 or not all(isinstance(item, dict) for item in listeners):
        fail("operator configuration listeners are missing or duplicated")
    listener_ids = [item.get("id") for item in listeners]
    if len(set(listener_ids)) != 3:
        fail("operator configuration has duplicate listener IDs")
    observed_listeners = {
        item["id"]: (item.get("bindHost"), item.get("port")) for item in listeners
    }
    if observed_listeners != expected_listeners or any(item.get("protocol") != "java-tcp" for item in listeners):
        fail("operator configuration listeners differ from the supported private topology")
    for item in listeners:
        public = item.get("publicHosts", [])
        if not isinstance(public, list) or any(not isinstance(host, str) or not host for host in public):
            fail("listener public hosts are invalid")
        if item.get("id") != "proxy-java" and public:
            fail("a backend listener is public")
    routes = network.get("routes")
    if not isinstance(routes, list) or len(routes) != 1 or not isinstance(routes[0], dict) \
            or routes[0].get("id") != "default" or routes[0].get("listener") != "proxy-java" \
            or routes[0].get("target") != "hub" or routes[0].get("fallbacks") != ["survival"]:
        fail("operator configuration route differs from hub with survival fallback")
    if network.get("capabilities", {}) != {
        "runtime": "local-process",
        "mountedConfig": True,
        "mountedSecrets": True,
        "mountedAssets": True,
    }:
        fail("operator configuration capabilities differ from the local mounted deployment")
    if network.get("auth", {}).get("onlineMode") is not True:
        fail("supported update requires online-mode proxy authentication")
    if network.get("forwarding", {}).get("mode") != "modern":
        fail("supported update requires modern forwarding")
    return instances


def validate_config_and_assets(service_uid, service_gid):
    value = regular(CONFIG, "operator configuration")
    if stat.S_IMODE(value.st_mode) & 0o022:
        fail("operator configuration is group/other writable")
    config = read_json(CONFIG, "operator configuration")
    if config.get("socketPath") != "/run/lkjmc/daemon.sock":
        fail("operator configuration uses an unsupported daemon socket")
    database = config.get("database", {})
    if database.get("host") not in ("127.0.0.1", "localhost") or database.get("database") != "lkjmc":
        fail("operator configuration does not use the private local database")
    daemon_http = config.get("daemonHttp", {})
    if daemon_http.get("enabled") is not True or daemon_http.get("address") != "127.0.0.1:8765":
        fail("operator configuration does not use the private daemon listener")
    network = config.get("network", {})
    instances = validate_network_topology(network)
    secret_paths = (
        database.get("secretFile"),
        daemon_http.get("tokenFile"),
        network.get("forwarding", {}).get("secretFile"),
    )
    if secret_paths != (
        "/etc/lkjmc/database.secret",
        "/etc/lkjmc/daemon-http.token",
        "/etc/lkjmc/forwarding.secret",
    ):
        fail("operator configuration secret paths differ from the canonical private files")
    for path_text in secret_paths:
        metadata = regular(Path(path_text), "deployment secret", private=True)
        if (metadata.st_uid, metadata.st_gid) != (service_uid, service_gid) or metadata.st_size == 0:
            fail(f"deployment secret ownership differs: {path_text}")
    if config.get("plugins", {}).get("lkjmc", {}).get("enabled") is not True:
        fail("lkjmc plugins are not enabled")
    assets = network.get("assets")
    if not isinstance(assets, list) or not assets:
        fail("operator configuration has no immutable server assets")
    asset_ids = set()
    for asset in assets:
        if not isinstance(asset, dict) or asset.get("kind") != "server" or asset.get("required") is not True:
            fail("unsupported network asset declaration")
        asset_id = asset.get("id")
        path_text = asset.get("path")
        digest = asset.get("sha256")
        if not isinstance(asset_id, str) or not asset_id or asset_id in asset_ids:
            fail("invalid or duplicate network asset ID")
        asset_ids.add(asset_id)
        if not isinstance(path_text, str) or not Path(path_text).is_absolute() or not HEX64.fullmatch(digest or ""):
            fail(f"invalid immutable network asset: {asset_id}")
        path = Path(path_text)
        metadata = regular(path, "immutable network asset")
        if metadata.st_size <= 0 or sha256(path) != digest:
            fail(f"immutable network asset differs: {asset_id}")
        if stat.S_IMODE(metadata.st_mode) & 0o022:
            fail(f"immutable network asset is group/other writable: {asset_id}")
    referenced = {asset_id for item in instances for asset_id in item.get("assetIds", [])}
    if referenced != asset_ids:
        fail("configured instances and immutable network assets differ")
    directory = CREDENTIALS.lstat() if CREDENTIALS.exists() else None
    if directory is None or not stat.S_ISDIR(directory.st_mode) or CREDENTIALS.is_symlink():
        fail("plugin credential directory is missing or unsafe")
    if stat.S_IMODE(directory.st_mode) != 0o700 or (directory.st_uid, directory.st_gid) != (service_uid, service_gid):
        fail("plugin credential directory ownership or mode differs")
    for instance_id in ("proxy", "hub", "survival"):
        credential = CREDENTIALS / f"{instance_id}.secret"
        metadata = regular(credential, "plugin heartbeat credential", private=True)
        if (metadata.st_uid, metadata.st_gid) != (service_uid, service_gid) or metadata.st_size == 0:
            fail(f"plugin heartbeat credential ownership differs: {instance_id}")
    if not existing_eula_acceptance(service_gid):
        fail("network update blocked: no existing Minecraft EULA acceptance record")


def effective_eula(raw):
    effective = None
    for source in raw.splitlines():
        line = source.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        name, value = (part.strip() for part in line.split("=", 1))
        if name == "eula":
            effective = value
    return effective


def existing_eula_acceptance(service_gid):
    marker = Path("/etc/lkjmc/minecraft-eula.accepted")
    if marker.exists() and not marker.is_symlink() and marker.is_file():
        metadata = marker.stat()
        if metadata.st_uid == 0 and metadata.st_gid == service_gid \
                and stat.S_IMODE(metadata.st_mode) in (0o440, 0o640) \
                and marker.read_bytes() == b"schemaVersion=1\naccepted=true\n":
            return True
    for instance_id in ("hub", "survival"):
        path = INSTANCES / instance_id / "eula.txt"
        if not path.exists() or path.is_symlink() or not path.is_file():
            return False
        if effective_eula(path.read_text(encoding="utf-8", errors="strict")) != "true":
            return False
    return True


def read_tail(path, maximum=4 * 1024 * 1024):
    size = path.stat().st_size
    with path.open("rb") as stream:
        if size > maximum:
            stream.seek(size - maximum)
        return stream.read(maximum).decode("utf-8", errors="replace")


def encode_varint(value):
    value &= 0xFFFFFFFF
    output = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            byte |= 0x80
        output.append(byte)
        if not value:
            return bytes(output)


def read_varint(stream):
    value = 0
    for shift in range(0, 35, 7):
        raw = stream.recv(1)
        if not raw:
            fail("proxy status ping ended before a VarInt")
        value |= (raw[0] & 0x7F) << shift
        if raw[0] & 0x80 == 0:
            return value
    fail("proxy status ping VarInt is too long")


def proxy_status_ping():
    host = "127.0.0.1"
    port = 25591
    address = host.encode()
    handshake = (
        encode_varint(0) + encode_varint(-1) + encode_varint(len(address)) + address
        + struct.pack(">H", port) + encode_varint(1)
    )
    request = encode_varint(len(handshake)) + handshake + b"\x01\x00"
    try:
        with socket.create_connection((host, port), timeout=5) as stream:
            stream.settimeout(5)
            stream.sendall(request)
            packet_length = read_varint(stream)
            if packet_length < 3 or packet_length > 1024 * 1024:
                fail("proxy status ping packet length is invalid")
            packet_id = read_varint(stream)
            text_length = read_varint(stream)
            if packet_id != 0 or text_length < 2 or text_length > packet_length or text_length > 1024 * 1024:
                fail("proxy status ping response header differs")
            chunks = []
            remaining = text_length
            while remaining:
                chunk = stream.recv(min(remaining, 65536))
                if not chunk:
                    fail("proxy status ping response is truncated")
                chunks.append(chunk)
                remaining -= len(chunk)
    except (OSError, ValueError) as error:
        fail(f"proxy status ping failed: {error.__class__.__name__}")
    try:
        payload = json.loads(b"".join(chunks))
    except (TypeError, ValueError) as error:
        fail(f"proxy status ping JSON is invalid: {error}")
    if not isinstance(payload, dict) or not isinstance(payload.get("version"), dict) \
            or not isinstance(payload.get("players"), dict):
        fail("proxy status ping payload differs")
    return payload


def validate_status(cli, commit, require_startup_evidence=False):
    status = json_command("lkjmc", (cli, "--json", "status"), "status", timeout=30)
    build = status.get("build", {})
    if build.get("commit") != commit or build.get("dirty") is not False:
        fail("daemon status build identity differs")
    if status.get("daemon") != "running" or status.get("database", {}).get("connected") is not True:
        fail("daemon or database is not healthy")
    instances = status.get("instances")
    if not isinstance(instances, list) or len(instances) != 3 or not all(isinstance(item, dict) for item in instances):
        fail("status instance set differs")
    ids = [item.get("id") for item in instances]
    if set(ids) != {"proxy", "hub", "survival"} or len(set(ids)) != 3:
        fail("status instance set is missing or duplicated")
    by_id = {item["id"]: item for item in instances}
    for instance_id in ("hub", "survival"):
        item = by_id[instance_id]
        if item.get("kind") != "folia" or item.get("processHealthy") is not True or item.get("ready") is not True or item.get("joinable") is not True:
            fail(f"backend is not ready and joinable: {instance_id}")
        if not isinstance(item.get("readinessAgeSeconds"), int) or item["readinessAgeSeconds"] > 30:
            fail(f"backend heartbeat is stale: {instance_id}")
        if not isinstance(item.get("proxyRegistrationAgeSeconds"), int) or item["proxyRegistrationAgeSeconds"] > 30:
            fail(f"proxy registration is stale: {instance_id}")
    proxy = by_id["proxy"]
    if proxy.get("kind") != "velocity" or proxy.get("processHealthy") is not True or proxy.get("joinDisabledReason") != "not-a-backend":
        fail("proxy status differs")
    plan = json_command(
        "lkjmc",
        (cli, "--json", "bootstrap", "plan", "--profile", "playable", "--bedrock", "disabled"),
        "bootstrap plan",
        timeout=30,
    )
    if plan.get("outcome") != "no-op" or plan.get("changes") != [] or plan.get("unsupported") != []:
        fail("bootstrap plan is not a supported no-op")
    for instance_id in ("proxy", "hub", "survival"):
        log = INSTANCES / instance_id / "logs/latest.log"
        regular(log, "instance log")
        text = read_tail(log)
        if require_startup_evidence and f"commit={commit} dirty=false" not in text:
            fail(f"instance plugin identity is absent after start: {instance_id}")
        if require_startup_evidence and f"lkjmc heartbeat active instance={instance_id} detail=accepted" not in text:
            fail(f"instance heartbeat acceptance is absent after start: {instance_id}")
        if "Bearer " in text:
            fail(f"authorization material found in instance log: {instance_id}")
    proxy_status_ping()
    return status


def validate_plugins(release_target):
    expected = {
        INSTANCES / "proxy/plugins/lkjmc-velocity.jar": release_target / "jars/lkjmc-velocity.jar",
        INSTANCES / "hub/plugins/lkjmc-paper.jar": release_target / "jars/lkjmc-paper.jar",
        INSTANCES / "survival/plugins/lkjmc-paper.jar": release_target / "jars/lkjmc-paper.jar",
    }
    for target, source in expected.items():
        regular(target, "installed plugin")
        regular(source, "release plugin")
        if sha256(target) != sha256(source):
            fail(f"installed plugin differs from current release: {target}")


def normalized_schema_hash(path):
    lines = path.read_bytes().splitlines(keepends=True)
    raw = b"".join(line for line in lines if not line.startswith((b"\\restrict ", b"\\unrestrict ")))
    return hashlib.sha256(raw).hexdigest()


def verify_backup(path, expected_commit, expected_marker, max_age_seconds):
    if not path.is_absolute() or path.suffix != ".dump":
        fail("--backup must be an absolute .dump path")
    try:
        path.relative_to(BACKUP_ROOT)
    except ValueError:
        fail("--backup must be under /var/backups/lkjmc")
    require_root_directory(path.parent, "database backup directory", mode=0o700)
    members = [path, Path(str(path) + ".manifest"), Path(str(path) + ".metadata.json"), Path(str(path) + ".sha256")]
    now = time.time()
    for member in members:
        require_root_ancestry(member, "database backup member")
        metadata = regular(member, "database backup member", private=True)
        if metadata.st_uid != 0 or stat.S_IMODE(metadata.st_mode) != 0o600:
            fail("database backup member ownership or mode differs")
        if now - metadata.st_mtime > max_age_seconds or metadata.st_mtime > now + 5:
            fail("database backup is not fresh")
    checksum_file = members[-1]
    expected_names = {member.name for member in members[:-1]}
    observed_names = set()
    for line in checksum_file.read_text(encoding="ascii").splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  ([^/]+)", line)
        if not match or match.group(2) in observed_names:
            fail("database backup checksum file is malformed")
        observed_names.add(match.group(2))
        member = path.parent / match.group(2)
        if member.name not in expected_names or sha256(member) != match.group(1):
            fail("database backup member checksum differs")
    if observed_names != expected_names:
        fail("database backup checksum closure differs")
    metadata = read_json(members[2], "database backup metadata")
    fields = {
        "schemaVersion", "sourceCommit", "postgresServerVersion", "lsn", "lsnSha256",
        "schemaSha256", "migrationMarker", "migrationSha256", "dumpSha256", "manifestSha256",
    }
    if not isinstance(metadata, dict) or set(metadata) != fields or metadata.get("schemaVersion") != 1 \
            or metadata.get("sourceCommit") != expected_commit:
        fail("database backup metadata fields or source release differ")
    if isinstance(metadata.get("postgresServerVersion"), bool) \
            or not isinstance(metadata.get("postgresServerVersion"), int):
        fail("database backup PostgreSQL version is invalid")
    for name in ("lsnSha256", "schemaSha256", "migrationSha256", "dumpSha256", "manifestSha256"):
        if not HEX64.fullmatch(metadata.get(name, "")):
            fail(f"database backup {name} is invalid")
    lsn = metadata.get("lsn")
    if not isinstance(lsn, str) or not re.fullmatch(r"[0-9A-F]+/[0-9A-F]+", lsn) \
            or hashlib.sha256(lsn.encode()).hexdigest() != metadata["lsnSha256"]:
        fail("database backup WAL marker differs")
    marker_text = metadata.get("migrationMarker")
    try:
        marker = json.loads(marker_text)
    except (TypeError, ValueError) as error:
        fail(f"database backup migration marker is invalid: {error}")
    canonical = json.dumps(marker, separators=(",", ":"), sort_keys=True, ensure_ascii=False)
    if canonical != marker_text or hashlib.sha256(canonical.encode()).hexdigest() != metadata["migrationSha256"] \
            or marker != expected_marker:
        fail("database backup migration marker differs from the live database")
    if metadata["dumpSha256"] != sha256(path) or metadata["manifestSha256"] != sha256(members[1]):
        fail("database backup metadata checksums differ")
    with tempfile.TemporaryDirectory(prefix=".verify-", dir=path.parent) as raw:
        temporary = Path(raw)
        os.chmod(temporary, 0o700)
        listed = run((PGRESTORE, "--list", path), timeout=120).stdout.encode()
        if listed != members[1].read_bytes():
            fail("pg_restore listing differs from the backup manifest")
        schema = temporary / "schema.sql"
        run((PGRESTORE, "--schema-only", f"--file={schema}", path), timeout=120)
        if normalized_schema_hash(schema) != metadata["schemaSha256"]:
            fail("pg_restore schema differs from backup metadata")
    return metadata


def create_and_verify_backup(path, tool, commit, marker, database_url, max_age_seconds):
    members = [path, Path(str(path) + ".manifest"), Path(str(path) + ".metadata.json"), Path(str(path) + ".sha256")]
    if any(member.exists() or member.is_symlink() for member in members) or path.parent.exists() or path.parent.is_symlink():
        fail("fresh database backup path already exists")
    BACKUP_ROOT.mkdir(parents=True, mode=0o700, exist_ok=True)
    require_root_directory(BACKUP_ROOT, "database backup root")
    fsync_dir(BACKUP_ROOT.parent)
    path.parent.mkdir(mode=0o700)
    os.chown(path.parent, 0, 0)
    os.chmod(path.parent, 0o700)
    environment = SAFE_ENV | {"LKJMC_DATABASE_URL": database_url, "LKJMC_SOURCE_COMMIT": commit}
    run((tool, path), timeout=900, env=environment)
    for member in members:
        regular(member, "database backup member")
        os.chown(member, 0, 0)
        os.chmod(member, 0o600)
        fsync_regular(member)
    fsync_dir(path.parent)
    fsync_dir(BACKUP_ROOT)
    verify_backup(path, commit, marker, max_age_seconds)


def validate_plugin_directories(service_uid, service_gid):
    directories = [INSTANCES]
    for instance_id in ("proxy", "hub", "survival"):
        directories.extend((INSTANCES / instance_id, INSTANCES / instance_id / "plugins"))
    for directory in directories:
        try:
            metadata = directory.lstat()
        except FileNotFoundError:
            fail(f"managed plugin directory is missing: {directory}")
        if not stat.S_ISDIR(metadata.st_mode) or directory.is_symlink() \
                or (metadata.st_uid, metadata.st_gid) != (service_uid, service_gid) \
                or stat.S_IMODE(metadata.st_mode) != 0o750:
            fail(f"managed plugin directory ownership or mode differs: {directory}")


def atomic_copy_in_directory(source, directory, name, mode, uid, gid, expected_directory_owner):
    regular(source, "publication source")
    if Path(name).name != name:
        fail("unsafe publication destination name")
    descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    temporary = f".{name}.{os.getpid()}.{os.urandom(8).hex()}"
    output_descriptor = None
    try:
        directory_metadata = os.fstat(descriptor)
        if not stat.S_ISDIR(directory_metadata.st_mode) \
                or (directory_metadata.st_uid, directory_metadata.st_gid) != expected_directory_owner \
                or stat.S_IMODE(directory_metadata.st_mode) != 0o750:
            fail(f"publication directory changed: {directory}")
        output_descriptor = os.open(
            temporary,
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC,
            mode,
            dir_fd=descriptor,
        )
        digest = hashlib.sha256()
        with source.open("rb") as incoming, os.fdopen(output_descriptor, "wb") as outgoing:
            output_descriptor = None
            for block in iter(lambda: incoming.read(1024 * 1024), b""):
                digest.update(block)
                outgoing.write(block)
            outgoing.flush()
            os.fchmod(outgoing.fileno(), mode)
            os.fchown(outgoing.fileno(), uid, gid)
            os.fsync(outgoing.fileno())
        if digest.hexdigest() != sha256(source):
            fail("publication copy checksum differs")
        os.replace(temporary, name, src_dir_fd=descriptor, dst_dir_fd=descriptor)
        os.fsync(descriptor)
    except Exception:
        if output_descriptor is not None:
            os.close(output_descriptor)
        try:
            os.unlink(temporary, dir_fd=descriptor)
        except FileNotFoundError:
            pass
        raise
    finally:
        os.close(descriptor)


def atomic_write_bytes(destination, payload, mode, uid, gid):
    destination.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{destination.name}.", dir=destination.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(payload)
            output.flush()
            os.fchmod(output.fileno(), mode)
            os.fchown(output.fileno(), uid, gid)
            os.fsync(output.fileno())
        os.replace(temporary, destination)
        fsync_dir(destination.parent)
    except Exception:
        try:
            os.close(descriptor)
        except OSError:
            pass
        temporary.unlink(missing_ok=True)
        raise


def atomic_copy(source, destination, mode, uid, gid):
    regular(source, "publication source")
    atomic_write_bytes(destination, source.read_bytes(), mode, uid, gid)


def atomic_symlink(target, destination):
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.parent / f".{destination.name}.{os.getpid()}"
    temporary.unlink(missing_ok=True)
    os.symlink(target, temporary)
    os.replace(temporary, destination)
    fsync_dir(destination.parent)


def publish_runtime_files(release_target, service_uid, service_gid):
    owner = (service_uid, service_gid)
    atomic_copy_in_directory(
        release_target / "jars/lkjmc-velocity.jar",
        INSTANCES / "proxy/plugins",
        "lkjmc-velocity.jar",
        0o640,
        0,
        service_gid,
        owner,
    )
    for instance_id in ("hub", "survival"):
        atomic_copy_in_directory(
            release_target / "jars/lkjmc-paper.jar",
            INSTANCES / instance_id / "plugins",
            "lkjmc-paper.jar",
            0o640,
            0,
            service_gid,
            owner,
        )
    atomic_copy(release_target / "share/lkjmc-daemon.service", UNIT, 0o644, 0, 0)
    atomic_symlink(release_target.name, CURRENT)
    atomic_symlink("/opt/lkjmc/releases/current/bin/lkjmc", CLI_LINK)
    run((SYSTEMCTL, "daemon-reload"))


def prepare_state_directory(commit):
    require_root_directory(Path("/var/lib/private"), "private state parent", mode=0o700)
    if not DEPLOYMENTS.exists():
        DEPLOYMENTS.mkdir(mode=0o700)
        os.chown(DEPLOYMENTS, 0, 0)
        os.chmod(DEPLOYMENTS, 0o700)
        fsync_dir(DEPLOYMENTS.parent)
    require_root_directory(DEPLOYMENTS, "deployment state root", mode=0o700)
    directory = DEPLOYMENTS / commit
    if directory.exists() or directory.is_symlink():
        fail(f"deployment state already exists: {directory}")
    directory.mkdir(mode=0o700)
    os.chown(directory, 0, 0)
    fsync_dir(DEPLOYMENTS)
    return directory


def write_state(directory, result, values):
    require_root_directory(directory, "deployment state directory", mode=0o700)
    payload = dict(values)
    payload.update({"schemaVersion": 1, "result": result})
    destination = directory / "deployment.json"
    descriptor, name = tempfile.mkstemp(prefix=".deployment.", dir=directory)
    temporary = Path(name)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as output:
            json.dump(payload, output, indent=2, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        os.chmod(temporary, 0o600)
        os.chown(temporary, 0, 0)
        os.replace(temporary, destination)
        fsync_dir(directory)
    except Exception:
        temporary.unlink(missing_ok=True)
        raise


def atomic_json(destination, payload):
    require_root_directory(destination.parent, "deployment control directory")
    descriptor, name = tempfile.mkstemp(prefix=f".{destination.name}.", dir=destination.parent)
    temporary = Path(name)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as output:
            json.dump(payload, output, indent=2, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        os.chmod(temporary, 0o600)
        os.chown(temporary, 0, 0)
        os.replace(temporary, destination)
        fsync_dir(destination.parent)
    except Exception:
        temporary.unlink(missing_ok=True)
        raise


def write_fence(values):
    payload = {
        "schemaVersion": 1,
        "fromCommit": values["fromCommit"],
        "toCommit": values["toCommit"],
        "stateDirectory": str(DEPLOYMENTS / values["toCommit"]),
        "backup": values["backup"],
        "rollbackSnapshot": values["rollbackSnapshot"],
    }
    atomic_json(FENCE, payload)


def remove_fence():
    if FENCE.is_symlink():
        fail("deployment fence is a symlink")
    if FENCE.exists():
        metadata = regular(FENCE, "deployment fence", private=True)
        if metadata.st_uid != 0 or stat.S_IMODE(metadata.st_mode) != 0o600:
            fail("deployment fence ownership or mode differs")
        FENCE.unlink()
        fsync_dir(FENCE.parent)


def write_start_permit():
    descriptor = os.open(
        START_PERMIT,
        os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC,
        0o400,
    )
    try:
        os.fchmod(descriptor, 0o400)
        os.fchown(descriptor, 0, 0)
        os.write(descriptor, b"lkjmc-deploy-start-permit\n")
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    fsync_dir(START_PERMIT.parent)


def remove_start_permit():
    if START_PERMIT.is_symlink():
        fail("deployment start permit is a symlink")
    if START_PERMIT.exists():
        metadata = regular(START_PERMIT, "deployment start permit", private=True)
        if metadata.st_uid != 0 or stat.S_IMODE(metadata.st_mode) != 0o400 \
                or START_PERMIT.read_bytes() != b"lkjmc-deploy-start-permit\n":
            fail("deployment start permit ownership, mode, or contents differ")
        START_PERMIT.unlink()
        fsync_dir(START_PERMIT.parent)


def install_fence_guard(release_target):
    parent = FENCE_DROPIN.parent
    if not parent.exists():
        require_root_directory(parent.parent, "systemd unit directory")
        parent.mkdir(mode=0o755)
        os.chown(parent, 0, 0)
        fsync_dir(parent.parent)
    require_root_directory(parent, "systemd fence drop-in directory")
    template_path = release_target / "share/lkjmc-deployment-fence.conf"
    template = template_path.read_text(encoding="ascii")
    placeholder = "@LKJMC_FENCE_CHECKER@"
    checker = release_target / "bin/lkjmc-deployment-fence-check"
    if template.count(placeholder) != 1:
        fail("deployment fence drop-in template differs")
    atomic_write_bytes(
        FENCE_DROPIN,
        template.replace(placeholder, str(checker)).encode("ascii"),
        0o644,
        0,
        0,
    )
    validate_systemd_files()
    run((SYSTEMCTL, "daemon-reload"))
    effective = run((SYSTEMCTL, "show", SERVICE, "-p", "ExecStartPre")).stdout
    if effective_exec_start_pre_paths(effective) != [str(checker)]:
        fail("effective systemd unit does not enforce the single privileged deployment fence check")


def effective_exec_start_pre_paths(value):
    """Return systemd's effective ExecStartPre executable paths, not argv repetitions."""
    return re.findall(r"(?:^|\{)\s*path=([^;\s{}]+)\s*;", value)


def service_user_processes_absent(service_uid):
    result = run((PGREP, "-u", str(service_uid)), check=False)
    if result.returncode == 1:
        return True
    if result.returncode == 0:
        return False
    fail("could not verify service-user process exit")


def stop_service(service_uid):
    run((SYSTEMCTL, "stop", SERVICE), timeout=180)
    if not service_user_processes_absent(service_uid):
        fail("a process owned by the dedicated lkjmc service user survived service stop")


def start_service(expected_commit):
    regular(FENCE, "deployment fence", private=True)
    write_start_permit()
    try:
        run((SYSTEMCTL, "reset-failed", SERVICE))
        run((SYSTEMCTL, "start", SERVICE), timeout=1600)
        if START_PERMIT.exists() or START_PERMIT.is_symlink():
            fail("privileged systemd fence check did not consume the one-start permit")
        run((SYSTEMCTL, "is-active", "--quiet", SERVICE))
        show = run((SYSTEMCTL, "show", SERVICE, "-p", "ActiveState", "-p", "SubState", "-p", "NRestarts", "-p", "Result"))
        fields = dict(line.split("=", 1) for line in show.stdout.splitlines() if "=" in line)
        if fields.get("ActiveState") != "active" or fields.get("SubState") != "running" \
                or fields.get("NRestarts") != "0" or fields.get("Result") != "success":
            fail("systemd service state differs after update")
        cli = CURRENT / "bin/lkjmc"
        validate_status(cli, expected_commit, require_startup_evidence=True)
        validate_plugins(CURRENT.resolve(strict=True))
    except Exception:
        remove_start_permit()
        raise


def binary_rollback_allowed(before, observed):
    return observed is not None and observed == before


def run_migrations(cli, database_url):
    environment = SAFE_ENV | {"LKJMC_DATABASE_URL": database_url}
    run((RUNUSER, "-u", "lkjmc", "--", cli, "db", "migrate"), timeout=900, env=environment)


def install_release(release, target, service_uid, service_gid):
    if target.exists() or target.is_symlink():
        validate_installed_release(target, release["commit"], release["digest"])
        return
    installer = release["path"].parent / "source/lkjmc-install-artifacts"
    run(
        (
            PYTHON,
            installer,
            "--scope",
            "system",
            "--manifest",
            release["path"],
            "--manifest-sha256",
            release["digest"],
            "--source",
            release["path"].parent / "source",
            "--root",
            target,
            "--service-uid",
            str(service_uid),
            "--service-gid",
            str(service_gid),
        ),
        timeout=300,
    )
    validate_installed_release(target, release["commit"], release["digest"])


def identity_check(target, expected_commit):
    result = run((target / "bin/lkjmc", "--json", "version"), timeout=30)
    try:
        value = json.loads(result.stdout)
    except (TypeError, ValueError) as error:
        fail(f"invalid installed CLI identity: {error}")
    if value.get("commit") != expected_commit or value.get("dirty") is not False:
        fail("installed CLI identity differs")
    for name in ("lkjmc-daemon", "lkjmc-discord"):
        output = run((target / "bin" / name, "--version"), timeout=30).stdout.strip()
        if expected_commit not in output or "dirty=false" not in output:
            fail(f"installed executable identity differs: {name}")


def restore_previous_files(
        old_target, old_unit, old_unit_mode, old_unit_uid, old_unit_gid,
        service_uid, service_gid, old_commit):
    require_root_ancestry(old_unit, "saved previous systemd unit")
    old_unit_metadata = regular(old_unit, "saved previous systemd unit", private=True)
    if old_unit_metadata.st_uid != 0 or stat.S_IMODE(old_unit_metadata.st_mode) != 0o600:
        fail("saved previous systemd unit ownership or mode differs")
    validate_plugin_directories(service_uid, service_gid)
    owner = (service_uid, service_gid)
    atomic_copy_in_directory(
        old_target / "jars/lkjmc-velocity.jar",
        INSTANCES / "proxy/plugins",
        "lkjmc-velocity.jar",
        0o640,
        0,
        service_gid,
        owner,
    )
    for instance_id in ("hub", "survival"):
        atomic_copy_in_directory(
            old_target / "jars/lkjmc-paper.jar",
            INSTANCES / instance_id / "plugins",
            "lkjmc-paper.jar",
            0o640,
            0,
            service_gid,
            owner,
        )
    atomic_copy(old_unit, UNIT, old_unit_mode, old_unit_uid, old_unit_gid)
    atomic_symlink(old_commit, CURRENT)
    atomic_symlink("/opt/lkjmc/releases/current/bin/lkjmc", CLI_LINK)
    run((SYSTEMCTL, "daemon-reload"))


def validate_systemd_files():
    require_root_ancestry(UNIT, "systemd service unit")
    metadata = regular(UNIT, "systemd service unit")
    if not root_owned_safe(metadata):
        fail("systemd service unit ownership or mode is unsafe")
    parent = FENCE_DROPIN.parent
    if parent.exists() or parent.is_symlink():
        require_root_ancestry(parent, "systemd service drop-in directory")
        for path in parent.iterdir():
            if path.suffix != ".conf":
                fail(f"unexpected systemd service drop-in: {path}")
            require_root_ancestry(path, "systemd service drop-in")
            value = regular(path, "systemd service drop-in")
            if not root_owned_safe(value):
                fail(f"systemd service drop-in ownership or mode is unsafe: {path}")
    return metadata


def service_identity():
    service = pwd.getpwnam("lkjmc")
    group = grp.getgrnam("lkjmc")
    if service.pw_gid != group.gr_gid or service.pw_shell not in ("/usr/sbin/nologin", "/sbin/nologin"):
        fail("lkjmc service account identity or login shell differs")
    properties = run((SYSTEMCTL, "show", SERVICE, "-p", "User", "-p", "Group", "-p", "KillMode"))
    fields = dict(line.split("=", 1) for line in properties.stdout.splitlines() if "=" in line)
    if fields != {"User": "lkjmc", "Group": "lkjmc", "KillMode": "mixed"}:
        fail("systemd service identity or process-group policy differs")
    return service


def ensure_update_fence(values):
    if START_PERMIT.exists() or START_PERMIT.is_symlink():
        remove_start_permit()
    write_fence(values)


def finish_fenced_start():
    if START_PERMIT.exists() or START_PERMIT.is_symlink():
        fail("deployment start permit was not consumed")
    remove_fence()


def abort_before_mutation(error, state, values, old_commit, service_uid):
    values["failure"] = str(error)
    run((SYSTEMCTL, "stop", SERVICE), check=False, timeout=180)
    if not service_user_processes_absent(service_uid):
        write_state(state, "stop-failed", values)
        raise DeployError(
            "update stopped before publication but a dedicated service-user process survived; "
            "the deployment fence remains and operator recovery is required"
        ) from error
    try:
        start_service(old_commit)
        write_state(state, "aborted-verified", values)
        finish_fenced_start()
    except Exception as restart_error:
        write_state(state, "restart-failed", values | {"restartFailure": str(restart_error)})
        raise DeployError(
            f"update failed before publication and the fenced old-service restart failed: {restart_error}"
        ) from error
    raise DeployError(f"update failed before publication; previous release verified: {error}") from error


def rollback_before_migration(
        error, state, values, old_target, old_unit, old_unit_mode, old_unit_uid,
        old_unit_gid, service_uid, service_gid, old_commit):
    ensure_update_fence(values)
    run((SYSTEMCTL, "stop", SERVICE), check=False, timeout=180)
    if not service_user_processes_absent(service_uid):
        write_state(state, "rollback-blocked", values | {"failure": str(error)})
        raise DeployError("automatic rollback blocked by a surviving service-user process") from error
    try:
        restore_previous_files(
            old_target,
            old_unit,
            old_unit_mode,
            old_unit_uid,
            old_unit_gid,
            service_uid,
            service_gid,
            old_commit,
        )
        start_service(old_commit)
        write_state(state, "rolled-back-verified", values | {"failure": str(error)})
        finish_fenced_start()
    except Exception as rollback_error:
        write_state(
            state,
            "rollback-failed",
            values | {"failure": str(error), "rollbackFailure": str(rollback_error)},
        )
        raise DeployError(f"update failed and automatic pre-migration rollback failed: {rollback_error}") from error
    raise DeployError(f"update failed; previous release restored and verified: {error}") from error


def update(args):
    if os.geteuid() != 0:
        fail("release update requires root")
    for command in (SYSTEMCTL, RUNUSER, PSQL, PGRESTORE, PGREP, PYTHON):
        trusted_command(command)
    if FENCE.exists() or FENCE.is_symlink() or START_PERMIT.exists() or START_PERMIT.is_symlink():
        fail("an incomplete deployment fence exists; run the anchored recover command")
    if not HEX40.fullmatch(args.from_commit):
        fail("--from-commit must be one lowercase 40-character Git commit")
    release_root = Path(args.release_root)
    release = load_anchored_release(release_root, args.manifest_sha256)
    verify_running_deployer(release)
    secure_release_for_root(release)
    old_unit_metadata = validate_systemd_files()
    service = service_identity()
    database_url, environment_metadata = database_url_from_environment()
    if (environment_metadata.st_uid, environment_metadata.st_gid) != (service.pw_uid, service.pw_gid):
        fail("daemon environment ownership differs")
    old_target, _ = resolve_current(args.from_commit)
    validate_config_and_assets(service.pw_uid, service.pw_gid)
    validate_plugin_directories(service.pw_uid, service.pw_gid)
    run((SYSTEMCTL, "is-active", "--quiet", SERVICE))
    identity_check(old_target, args.from_commit)
    validate_plugins(old_target)
    validate_status(old_target / "bin/lkjmc", args.from_commit)
    if release["commit"] == args.from_commit:
        if release["digest"] != parse_manifest(installed_manifest(old_target))["digest"]:
            fail("same-commit release manifest differs from the installed release")
        print(json.dumps({"schemaVersion": 1, "result": "no-op", "commit": args.from_commit}, sort_keys=True))
        return
    if not args.backup or not args.rollback_snapshot:
        fail("changed update requires --backup and --rollback-snapshot")
    if not SAFE_LABEL.fullmatch(args.rollback_snapshot):
        fail("--rollback-snapshot is not a safe snapshot label")
    before = migration_marker()
    backup = Path(args.backup)
    create_and_verify_backup(
        backup,
        release_root / "source/lkjmc-backup-postgres",
        args.from_commit,
        before,
        database_url,
        args.backup_max_age_seconds,
    )
    target = RELEASES / release["commit"]
    install_release(release, target, service.pw_uid, service.pw_gid)
    identity_check(target, release["commit"])
    state = prepare_state_directory(release["commit"])
    old_unit = state / "previous-lkjmc-daemon.service"
    old_unit_mode = stat.S_IMODE(old_unit_metadata.st_mode)
    old_unit_uid = old_unit_metadata.st_uid
    old_unit_gid = old_unit_metadata.st_gid
    atomic_copy(UNIT, old_unit, 0o600, 0, 0)
    values = {
        "fromCommit": args.from_commit,
        "toCommit": release["commit"],
        "manifestSha256": release["digest"],
        "backup": str(backup),
        "rollbackSnapshot": args.rollback_snapshot,
        "migrationBefore": before,
        "oldUnitMode": old_unit_mode,
        "oldUnitUid": old_unit_uid,
        "oldUnitGid": old_unit_gid,
    }
    install_fence_guard(target)
    write_state(state, "prepared", values)
    write_fence(values)
    mutating = False
    completed = False
    try:
        stop_service(service.pw_uid)
        validate_plugin_directories(service.pw_uid, service.pw_gid)
        mutating = True
        publish_runtime_files(target, service.pw_uid, service.pw_gid)
        run_migrations(target / "bin/lkjmc", database_url)
        after = migration_marker()
        values["migrationAfter"] = after
        write_state(state, "migrated", values)
        start_service(release["commit"])
        write_state(state, "validated", values)
        finish_fenced_start()
        completed = True
        write_state(state, "updated", values)
    except Exception as error:
        if completed:
            raise DeployError(f"updated release is active but final receipt publication failed: {error}") from error
        if not mutating:
            abort_before_mutation(error, state, values, args.from_commit, service.pw_uid)
        ensure_update_fence(values)
        try:
            observed = migration_marker()
        except Exception:
            observed = None
        values["migrationAfterFailure"] = observed
        if binary_rollback_allowed(before, observed):
            rollback_before_migration(
                error,
                state,
                values,
                old_target,
                old_unit,
                old_unit_mode,
                old_unit_uid,
                old_unit_gid,
                service.pw_uid,
                service.pw_gid,
                args.from_commit,
            )
        run((SYSTEMCTL, "stop", SERVICE), check=False, timeout=180)
        write_state(state, "restore-required", values | {"failure": str(error)})
        raise DeployError(
            "update failed after the migration ledger changed or became unreadable; service restart is fenced and "
            f"binary-only rollback is forbidden; restore backup {backup} with snapshot {args.rollback_snapshot}: {error}"
        ) from error
    receipt = {
        "schemaVersion": 1,
        "result": "updated",
        "fromCommit": args.from_commit,
        "toCommit": release["commit"],
        "manifestSha256": release["digest"],
        "backup": str(backup),
        "rollbackSnapshot": args.rollback_snapshot,
        "migrationChanged": values["migrationAfter"] != before,
    }
    print(json.dumps(receipt, sort_keys=True))


def recover(args):
    if os.geteuid() != 0:
        fail("deployment recovery requires root")
    for command in (SYSTEMCTL, RUNUSER, PSQL, PGREP, PYTHON):
        trusted_command(command)
    release = load_anchored_release(Path(args.release_root), args.manifest_sha256)
    verify_running_deployer(release)
    secure_release_for_root(release)
    if release["commit"] != args.to_commit or not HEX40.fullmatch(args.to_commit):
        fail("recovery target differs from the anchored release")
    if not FENCE.exists() or FENCE.is_symlink():
        fail("no regular deployment fence exists")
    fence_metadata = regular(FENCE, "deployment fence", private=True)
    if fence_metadata.st_uid != 0 or stat.S_IMODE(fence_metadata.st_mode) != 0o600:
        fail("deployment fence ownership or mode differs")
    fence = read_json(FENCE, "deployment fence")
    expected_fence_fields = {
        "schemaVersion", "fromCommit", "toCommit", "stateDirectory", "backup", "rollbackSnapshot",
    }
    if not isinstance(fence, dict) or set(fence) != expected_fence_fields \
            or fence.get("schemaVersion") != 1 or fence.get("toCommit") != args.to_commit \
            or fence.get("stateDirectory") != str(DEPLOYMENTS / args.to_commit):
        fail("deployment fence fields differ")
    state = DEPLOYMENTS / args.to_commit
    require_root_directory(state, "deployment state directory", mode=0o700)
    state_file = state / "deployment.json"
    state_metadata = regular(state_file, "deployment state", private=True)
    if state_metadata.st_uid != 0 or stat.S_IMODE(state_metadata.st_mode) != 0o600:
        fail("deployment state ownership or mode differs")
    values = read_json(state_file, "deployment state")
    required = {
        "schemaVersion", "result", "fromCommit", "toCommit", "manifestSha256", "backup",
        "rollbackSnapshot", "migrationBefore", "oldUnitMode", "oldUnitUid", "oldUnitGid",
    }
    if not isinstance(values, dict) or not required.issubset(values) \
            or values.get("fromCommit") != fence.get("fromCommit") \
            or values.get("toCommit") != args.to_commit \
            or values.get("manifestSha256") != release["digest"] \
            or values.get("backup") != fence.get("backup") \
            or values.get("rollbackSnapshot") != fence.get("rollbackSnapshot") \
            or not HEX40.fullmatch(values.get("fromCommit", "")) \
            or any(isinstance(values.get(name), bool) or not isinstance(values.get(name), int)
                   for name in ("oldUnitMode", "oldUnitUid", "oldUnitGid")) \
            or values.get("oldUnitMode", -1) < 0 or values.get("oldUnitMode", 0) > 0o7777 \
            or values.get("oldUnitUid", -1) < 0 or values.get("oldUnitGid", -1) < 0:
        fail("deployment state differs from the fence or anchored release")
    validate_systemd_files()
    service = service_identity()
    if START_PERMIT.exists() or START_PERMIT.is_symlink():
        remove_start_permit()
    run((SYSTEMCTL, "stop", SERVICE), check=False, timeout=180)
    if not service_user_processes_absent(service.pw_uid):
        fail("recovery blocked by a surviving service-user process")
    try:
        observed = migration_marker()
    except Exception:
        observed = None
    before = values["migrationBefore"]
    if not binary_rollback_allowed(before, observed):
        write_state(state, "restore-required", values | {"migrationAfterRecovery": observed})
        fail(
            "migration ledger changed or is unreadable; service restart remains fenced; "
            f"restore {values['backup']} with snapshot {values['rollbackSnapshot']}"
        )
    old_commit = values["fromCommit"]
    old_target = RELEASES / old_commit
    validate_installed_release(old_target, old_commit)
    restore_previous_files(
        old_target,
        state / "previous-lkjmc-daemon.service",
        values["oldUnitMode"],
        values["oldUnitUid"],
        values["oldUnitGid"],
        service.pw_uid,
        service.pw_gid,
        old_commit,
    )
    start_service(old_commit)
    write_state(state, "recovered-verified", values | {"migrationAfterRecovery": observed})
    finish_fenced_start()
    print(json.dumps({"schemaVersion": 1, "result": "recovered", "commit": old_commit}, sort_keys=True))


def add_release_arguments(command):
    command.add_argument("--release-root", required=True)
    command.add_argument("--manifest-sha256", required=True)


def main():
    parser = argparse.ArgumentParser(
        description="Update or recover an existing lkjmc system deployment from an anchored immutable release."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)
    command = subparsers.add_parser("update")
    add_release_arguments(command)
    command.add_argument("--from-commit", required=True)
    command.add_argument("--backup")
    command.add_argument("--rollback-snapshot")
    command.add_argument("--backup-max-age-seconds", type=int, default=3600)
    recovery = subparsers.add_parser("recover")
    add_release_arguments(recovery)
    recovery.add_argument("--to-commit", required=True)
    args = parser.parse_args()
    if os.geteuid() != 0:
        fail("release deployment operations require root")
    with deployment_lock():
        if args.command == "update":
            if args.backup_max_age_seconds < 60 or args.backup_max_age_seconds > 86400:
                fail("--backup-max-age-seconds must be between 60 and 86400")
            update(args)
        else:
            recover(args)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"release deploy failed: {error}", file=sys.stderr)
        sys.exit(1)
