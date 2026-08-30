#!/usr/bin/env python3
"""Construct and observe one source-free disposable canonical lkjmc host.

This is test-owned fixture code. It is not an installer, updater, or recovery
authority for a supported host. Release installation is delegated to the exact
packaged artifact installer and all later transitions use packaged product
commands.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import pwd
import grp
import re
import secrets
import shutil
import socket
import stat
import struct
import subprocess
import sys
import time
from typing import Any, Mapping


PROJECT_PATTERN = re.compile(r"lkjmcdrr-[a-z0-9][a-z0-9-]{7,47}")
HEX40 = re.compile(r"[0-9a-f]{40}")
HEX64 = re.compile(r"[0-9a-f]{64}")
INPUT_ROOT = Path("/var/lib/private/lkjmc-drr-input")
RELEASES = Path("/opt/lkjmc/releases")
CURRENT = RELEASES / "current"
CONFIG_ROOT = Path("/etc/lkjmc")
DATA_ROOT = Path("/var/lib/lkjmc")
INSTANCES = DATA_ROOT / "instances"
LOG_ROOT = Path("/var/log/lkjmc")
RUNTIME_ASSETS = Path("/opt/lkjmc/runtime-assets")
SERVICE = "lkjmc-daemon.service"
UNIT = Path("/etc/systemd/system/lkjmc-daemon.service")
CLI_LINK = Path("/usr/local/bin/lkjmc")
MAX_OUTPUT = 4 * 1024 * 1024
SAFE_ENV = {
    "LANG": "C",
    "LC_ALL": "C",
    "PATH": "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
}
IMAGE_MARKER = b"schemaVersion=1\npurpose=disposable-docker-release-recovery-fixture\n"


class FixtureError(RuntimeError):
    pass


class FixtureBlocked(FixtureError):
    pass


def write_all(descriptor: int, value: bytes) -> None:
    view = memoryview(value)
    while view:
        written = os.write(descriptor, view)
        if written <= 0:
            raise FixtureError("fixture file write made no progress")
        view = view[written:]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def run(
    argv: list[str | Path],
    *,
    timeout: int = 120,
    check: bool = True,
    env: Mapping[str, str] | None = None,
    stdin: str | None = None,
) -> subprocess.CompletedProcess[str]:
    command = [str(value) for value in argv]
    try:
        result = subprocess.run(
            command,
            check=False,
            env=dict(SAFE_ENV if env is None else env),
            input=stdin,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=timeout,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise FixtureError(f"command did not execute: {Path(command[0]).name}") from error
    if len(result.stdout.encode()) + len(result.stderr.encode()) > MAX_OUTPUT:
        raise FixtureError(f"command output exceeded bound: {Path(command[0]).name}")
    if check and result.returncode:
        raise FixtureError(f"command failed: {Path(command[0]).name}")
    return result


def regular(path: Path, label: str) -> os.stat_result:
    try:
        value = path.lstat()
    except FileNotFoundError as error:
        raise FixtureError(f"missing {label}") from error
    if not stat.S_ISREG(value.st_mode) or path.is_symlink():
        raise FixtureError(f"{label} is not a regular file")
    return value


def directory(path: Path, label: str) -> os.stat_result:
    try:
        value = path.lstat()
    except FileNotFoundError as error:
        raise FixtureError(f"missing {label}") from error
    if not stat.S_ISDIR(value.st_mode) or path.is_symlink():
        raise FixtureError(f"{label} is not a directory")
    return value


def exact_closure(root: Path, names: set[str], label: str) -> None:
    directory(root, label)
    observed = {path.name for path in root.iterdir()}
    if observed != names:
        raise FixtureError(f"{label} closure differs")


def image_and_project_boundary() -> str:
    project = os.environ.get("LKJMC_DRR_PROJECT", "")
    if not PROJECT_PATTERN.fullmatch(project) or project.endswith("-") or "--" in project:
        raise FixtureBlocked("disposable project identity is absent or unsafe")
    marker = Path("/usr/share/lkjmc-drr-image")
    value = regular(marker, "fixture image marker")
    if value.st_uid != 0 or stat.S_IMODE(value.st_mode) != 0o444 or marker.read_bytes() != IMAGE_MARKER:
        raise FixtureBlocked("fixture image identity differs")
    if Path("/proc/1/comm").read_text().strip() != "systemd" \
            or Path("/proc/1/exe").resolve() != Path("/usr/lib/systemd/systemd"):
        raise FixtureBlocked("real systemd is not PID 1")
    root = directory(INPUT_ROOT, "fixture input root")
    if root.st_uid != 0 or stat.S_IMODE(root.st_mode) != 0o700:
        raise FixtureBlocked("fixture input root ownership or mode differs")
    return project


def validate_manifest(root: Path, expected_commit: str, expected_digest: str) -> dict[str, Any]:
    exact_closure(root, {"artifact-manifest.json", "artifact-manifest.json.sha256", "source"}, "release input")
    manifest = root / "artifact-manifest.json"
    sidecar = root / "artifact-manifest.json.sha256"
    regular(manifest, "release manifest")
    regular(sidecar, "release manifest sidecar")
    directory(root / "source", "release source")
    if sha256(manifest) != expected_digest \
            or sidecar.read_text(encoding="ascii") != f"{expected_digest}  artifact-manifest.json\n":
        raise FixtureError("release manifest digest differs")
    value = json.loads(manifest.read_text())
    if value.get("schemaVersion") != 1 or value.get("commit") != expected_commit:
        raise FixtureError("release manifest identity differs")
    artifacts = value.get("artifacts")
    if not isinstance(artifacts, list) or len(artifacts) != 14:
        raise FixtureError("release manifest artifact closure differs")
    by_name = {item.get("path"): item for item in artifacts if isinstance(item, dict)}
    if len(by_name) != 14 or {path.name for path in (root / "source").iterdir()} != set(by_name):
        raise FixtureError("release source closure differs")
    for name, item in by_name.items():
        if not isinstance(name, str) or Path(name).name != name \
                or item.get("kind") not in {"binary", "config", "jar"}:
            raise FixtureError("release artifact declaration differs")
        path = root / "source" / name
        metadata = regular(path, "release artifact")
        if metadata.st_uid != 0 or stat.S_IMODE(metadata.st_mode) & 0o022 \
                or metadata.st_size != item.get("size") or sha256(path) != item.get("sha256"):
            raise FixtureError(f"release artifact differs: {name}")
    return value


def load_inputs(expected_commit: str, expected_digest: str) -> dict[str, Any]:
    exact_closure(INPUT_ROOT, {"assets", "baseline", "input.json"}, "fixture input root")
    descriptor_path = INPUT_ROOT / "input.json"
    metadata = regular(descriptor_path, "container input descriptor")
    if metadata.st_uid != 0 or stat.S_IMODE(metadata.st_mode) != 0o600 or metadata.st_size > 64 * 1024:
        raise FixtureError("container input descriptor ownership, mode, or size differs")
    value = json.loads(descriptor_path.read_text())
    fields = {"assets", "baseline", "minecraftEulaAccepted", "project", "schemaVersion"}
    if not isinstance(value, dict) or set(value) != fields or value.get("schemaVersion") != 1 \
            or value.get("project") != os.environ.get("LKJMC_DRR_PROJECT") \
            or type(value.get("minecraftEulaAccepted")) is not bool:
        raise FixtureError("container input descriptor schema differs")
    baseline = value.get("baseline")
    if not isinstance(baseline, dict) or set(baseline) != {"commit", "manifestSha256"} \
            or baseline != {"commit": expected_commit, "manifestSha256": expected_digest}:
        raise FixtureError("container baseline identity differs")
    validate_manifest(INPUT_ROOT / "baseline", expected_commit, expected_digest)
    assets = value.get("assets")
    if not isinstance(assets, list) or len(assets) != 2:
        raise FixtureError("container asset closure differs")
    exact_closure(INPUT_ROOT / "assets", {"folia.jar", "velocity.jar"}, "asset input")
    observed = {}
    for item in assets:
        fields = {"id", "name", "project", "sha256", "size"}
        if not isinstance(item, dict) or set(item) != fields or item.get("project") not in {"folia", "velocity"}:
            raise FixtureError("container asset descriptor differs")
        expected_name = f"{item['project']}.jar"
        path = INPUT_ROOT / "assets" / expected_name
        asset = regular(path, "immutable server asset")
        if item.get("name") != expected_name or item.get("id") != f"{item['project']}-server" \
                or not HEX64.fullmatch(str(item.get("sha256", ""))) \
                or type(item.get("size")) is not int or item["size"] <= 0 \
                or asset.st_uid != 0 or stat.S_IMODE(asset.st_mode) & 0o022 \
                or asset.st_size != item["size"] or sha256(path) != item["sha256"]:
            raise FixtureError(f"immutable server asset differs: {item.get('project')}")
        observed[item["project"]] = item
    if set(observed) != {"folia", "velocity"}:
        raise FixtureError("container server asset projects differ")
    return value


def require_fresh_host() -> None:
    paths = (
        CONFIG_ROOT,
        DATA_ROOT,
        LOG_ROOT,
        Path("/opt/lkjmc"),
        Path("/var/backups/lkjmc"),
        Path("/var/lib/private/lkjmc-deployments"),
        UNIT,
    )
    if any(os.path.lexists(path) for path in paths):
        raise FixtureBlocked("fixture refuses a nonempty canonical product root")
    if CLI_LINK.exists() or CLI_LINK.is_symlink():
        raise FixtureBlocked("fixture refuses an existing canonical CLI pointer")
    try:
        pwd.getpwnam("lkjmc")
    except KeyError:
        pass
    else:
        raise FixtureBlocked("fixture refuses an existing lkjmc service identity")
    if run(["pg_lsclusters", "--no-header"]).stdout.strip():
        raise FixtureBlocked("fixture refuses a preexisting PostgreSQL cluster")
    for path in (Path("/etc/lkjmc/deployment-fence.json"), Path("/run/lkjmc-deploy-start-permit")):
        if path.exists() or path.is_symlink():
            raise FixtureBlocked("fixture refuses preexisting deployment control state")


def mkdir(path: Path, mode: int, uid: int, gid: int) -> None:
    path.mkdir(parents=True, exist_ok=False)
    os.chown(path, uid, gid)
    os.chmod(path, mode)


def write(path: Path, payload: bytes, mode: int, uid: int, gid: int) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, mode)
    try:
        write_all(descriptor, payload)
        os.fchmod(descriptor, mode)
        os.fchown(descriptor, uid, gid)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def copy(source: Path, destination: Path, mode: int, uid: int, gid: int) -> None:
    regular(source, "copy source")
    with source.open("rb") as incoming:
        descriptor = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, mode)
        try:
            for block in iter(lambda: incoming.read(1024 * 1024), b""):
                write_all(descriptor, block)
            os.fchmod(descriptor, mode)
            os.fchown(descriptor, uid, gid)
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
    if sha256(source) != sha256(destination):
        raise FixtureError("fixture copy verification failed")


def config_value(database_password_file: Path, assets: Mapping[str, Mapping[str, Any]]) -> dict[str, Any]:
    return {
        "installRoot": "/opt/lkjmc",
        "configRoot": "/etc/lkjmc",
        "dataRoot": "/var/lib/lkjmc",
        "logRoot": "/var/log/lkjmc",
        "socketPath": "/run/lkjmc/daemon.sock",
        "database": {
            "host": "127.0.0.1",
            "port": 5432,
            "database": "lkjmc",
            "user": "lkjmc",
            "secretFile": str(database_password_file),
            "poolSize": 8,
        },
        "network": {
            "revision": 1,
            "instances": [
                {"id": "hub", "owner": "lkjmc-daemon", "kind": "folia", "desiredState": "running", "listener": "hub-java", "memoryMb": 1536, "assetIds": ["folia-server"]},
                {"id": "proxy", "owner": "lkjmc-daemon", "kind": "velocity", "desiredState": "running", "listener": "proxy-java", "memoryMb": 512, "assetIds": ["velocity-server"]},
                {"id": "survival", "owner": "lkjmc-daemon", "kind": "folia", "desiredState": "running", "listener": "survival-java", "memoryMb": 1536, "assetIds": ["folia-server"]},
            ],
            "routes": [{"id": "default", "listener": "proxy-java", "target": "hub", "fallbacks": ["survival"]}],
            "listeners": [
                {"id": "hub-java", "protocol": "java-tcp", "bindHost": "127.0.0.1", "port": 25566, "publicHosts": []},
                {"id": "proxy-java", "protocol": "java-tcp", "bindHost": "0.0.0.0", "port": 25591, "publicHosts": []},
                {"id": "survival-java", "protocol": "java-tcp", "bindHost": "127.0.0.1", "port": 25567, "publicHosts": []},
            ],
            "auth": {"onlineMode": True},
            "forwarding": {"mode": "modern", "secretFile": "/etc/lkjmc/forwarding.secret"},
            "assets": [
                {"id": "folia-server", "kind": "server", "path": "/opt/lkjmc/runtime-assets/folia.jar", "sha256": assets["folia"]["sha256"], "required": True},
                {"id": "velocity-server", "kind": "server", "path": "/opt/lkjmc/runtime-assets/velocity.jar", "sha256": assets["velocity"]["sha256"], "required": True},
            ],
            "capabilities": {"runtime": "local-process", "mountedConfig": True, "mountedSecrets": True, "mountedAssets": True},
        },
        "jars": {"root": "/opt/lkjmc/jars", "defaultChannel": "stable", "userAgent": "lkjmc-docker-release-recovery-lab"},
        "daemonHttp": {"enabled": True, "address": "127.0.0.1:8765", "tokenFile": "/etc/lkjmc/daemon-http.token"},
        "assets": {"root": "/opt/lkjmc/assets", "serverChannel": "stable", "pluginChannel": "stable", "userAgent": "lkjmc-docker-release-recovery-lab", "downloadTimeoutSeconds": 120},
        "plugins": {
            "lkjmc": {"enabled": True},
            "viaversion": {"mode": "disabled", "installOn": "backend"},
            "viabackwards": {"mode": "disabled", "installOn": "backend"},
            "geyser": {"mode": "disabled", "installOn": "proxy"},
            "floodgate": {"mode": "disabled", "installOn": "proxy", "backendApi": False},
        },
        "runtime": {"adapter": "local-process", "defaultJavaMemoryMb": 1536, "proxyJavaMemoryMb": 512, "stopTimeoutSeconds": 60, "portRangeStart": 25566, "portRangeEnd": 25665},
    }


def database_setup(password: str) -> None:
    run(["systemctl", "enable", "postgresql.service"])
    run(["pg_createcluster", "16", "main", "--start"], timeout=300)
    run(["systemctl", "enable", "postgresql@16-main.service"])
    sql = f"CREATE ROLE lkjmc LOGIN PASSWORD '{password}';\n"
    run(["runuser", "-u", "postgres", "--", "psql", "-X", "--quiet", "--set", "ON_ERROR_STOP=1", "--dbname", "postgres"], stdin=sql)
    run(["runuser", "-u", "postgres", "--", "createdb", "--owner=lkjmc", "lkjmc"])


def wait_path(path: Path, timeout: int) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.exists() and not path.is_symlink():
            return
        time.sleep(0.25)
    raise FixtureError(f"timed out waiting for {path.name}")


def provision_credentials(release: Path) -> dict[str, str]:
    unit = "lkjmc-drr-credential-provision.service"
    daemon = release / "bin/lkjmc-daemon"
    cli = release / "bin/lkjmc"
    run([
        "systemd-run", "--unit", unit, "--collect", "--property=Type=exec", "--property=User=lkjmc",
        "--property=Group=lkjmc", "--property=WorkingDirectory=/var/lib/lkjmc", str(daemon),
        "--config", "/etc/lkjmc/lkjmc.json", "--http-token-file", "/etc/lkjmc/daemon-http.token",
    ])
    try:
        wait_path(Path("/run/lkjmc/daemon.sock"), 30)
        for instance, surface in (("proxy", "velocity"), ("hub", "paper"), ("survival", "paper")):
            run([
                "runuser", "-u", "lkjmc", "--", str(cli), "--socket", "/run/lkjmc/daemon.sock",
                "security", "token", "create", "--surface", surface, "--principal-kind", "instance",
                "--principal-id", instance, "--output-file",
                f"/var/lib/lkjmc/private/plugin-credentials/{instance}.secret",
                "--expires-in-seconds", "604800", "--scope", "lkjmc.instance.heartbeat",
            ])
    finally:
        run(["systemctl", "stop", unit], check=False, timeout=60)
    deadline = time.monotonic() + 30
    uid = pwd.getpwnam("lkjmc").pw_uid
    while time.monotonic() < deadline:
        pids = run(["pgrep", "-u", str(uid)], check=False).returncode
        if pids == 1:
            break
        time.sleep(0.25)
    else:
        raise FixtureError("credential provisioner left a service-user process")
    observations = {}
    for instance in ("proxy", "hub", "survival"):
        path = DATA_ROOT / "private/plugin-credentials" / f"{instance}.secret"
        value = regular(path, "plugin credential")
        if (value.st_uid, value.st_gid) != (uid, grp.getgrnam("lkjmc").gr_gid) \
                or stat.S_IMODE(value.st_mode) & 0o077 or value.st_size == 0:
            raise FixtureError("plugin credential ownership or mode differs")
        observations[instance] = sha256(path)
    return observations


def migration_marker() -> list[dict[str, Any]]:
    query = (
        "select coalesce(jsonb_agg(jsonb_build_object('version',version,'name',name,"
        "'checksum',coalesce(checksum,'')) order by version),'[]'::jsonb)::text from schema_migrations"
    )
    output = run(["runuser", "-u", "postgres", "--", "psql", "-d", "lkjmc", "-X", "--quiet", "--no-align", "--tuples-only", "-v", "ON_ERROR_STOP=1", "-c", query]).stdout.strip()
    value = json.loads(output)
    if not isinstance(value, list):
        raise FixtureError("migration marker is not an array")
    return value


def database_fingerprint() -> str:
    query = """
select jsonb_build_object(
  'migrations', (select coalesce(jsonb_agg(to_jsonb(t) order by version), '[]'::jsonb) from schema_migrations t),
  'instances', (select coalesce(jsonb_agg(to_jsonb(t) order by id), '[]'::jsonb) from instances t),
  'ports', (select coalesce(jsonb_agg(to_jsonb(t) order by port), '[]'::jsonb) from instance_ports t),
  'assets', (select coalesce(jsonb_agg(to_jsonb(t) order by id), '[]'::jsonb) from jar_assets t),
  'intent', (select coalesce(jsonb_agg(to_jsonb(t) order by revision), '[]'::jsonb) from network_intents t),
  'tokens', (select coalesce(jsonb_agg(
    jsonb_build_object(
      'credentialId', credential_id,
      'tokenHash', token_hash,
      'surface', surface,
      'principalKind', principal_kind,
      'principalId', principal_id,
      'scopes', scopes,
      'expiresAt', expires_at,
      'revokedAt', revoked_at
    ) order by credential_id
  ), '[]'::jsonb) from daemon_tokens)
)::text
"""
    raw = run([
        "runuser", "-u", "postgres", "--", "psql", "-d", "lkjmc", "-X", "--quiet",
        "--no-align", "--tuples-only", "-v", "ON_ERROR_STOP=1", "-c", query,
    ]).stdout.strip()
    value = json.loads(raw)
    canonical = json.dumps(value, separators=(",", ":"), sort_keys=True).encode()
    return hashlib.sha256(canonical).hexdigest()


def file_record(path: Path, label: str) -> dict[str, Any]:
    value = path.lstat()
    relative = str(path)
    if stat.S_ISLNK(value.st_mode):
        return {
            "gid": value.st_gid,
            "mode": stat.S_IMODE(value.st_mode),
            "path": relative,
            "target": os.readlink(path),
            "type": "symlink",
            "uid": value.st_uid,
        }
    if not stat.S_ISREG(value.st_mode):
        raise FixtureError(f"{label} contains a non-regular entry")
    if value.st_size > 1024 * 1024 * 1024:
        raise FixtureError(f"{label} file exceeds fingerprint bound")
    return {
        "gid": value.st_gid,
        "mode": stat.S_IMODE(value.st_mode),
        "path": relative,
        "sha256": sha256(path),
        "size": value.st_size,
        "type": "regular",
        "uid": value.st_uid,
    }


def append_tree(records: list[dict[str, Any]], root: Path, label: str) -> None:
    if not os.path.lexists(root):
        return
    if root.is_symlink() or root.is_file():
        records.append(file_record(root, label))
        return
    directory(root, label)
    paths = sorted(root.rglob("*"), key=lambda path: str(path).encode())
    for path in paths:
        if path.is_dir() and not path.is_symlink():
            continue
        records.append(file_record(path, label))
        if len(records) > 512:
            raise FixtureError("fingerprint file closure exceeds bound")


def fingerprint(expected_commit: str, expected_digest: str) -> dict[str, Any]:
    image_and_project_boundary()
    if not CURRENT.is_symlink() or CURRENT.resolve(strict=True) != RELEASES / expected_commit \
            or sha256(CURRENT.resolve(strict=True) / "meta/artifact-manifest.json") != expected_digest:
        raise FixtureError("fingerprint selected release differs")
    records: list[dict[str, Any]] = []
    for root, label in (
        (RELEASES, "release tree"),
        (CONFIG_ROOT, "configuration tree"),
        (RUNTIME_ASSETS, "runtime asset tree"),
        (UNIT, "systemd unit"),
        (Path("/etc/systemd/system/lkjmc-daemon.service.d"), "systemd drop-in tree"),
        (Path("/var/backups/lkjmc"), "backup tree"),
        (Path("/var/lib/private/lkjmc-deployments"), "deployment state tree"),
        (Path("/etc/lkjmc/deployment-fence.json"), "deployment fence"),
        (Path("/run/lkjmc-deploy-start-permit"), "deployment start permit"),
    ):
        append_tree(records, root, label)
    append_tree(records, CLI_LINK, "CLI pointer")
    for instance in ("proxy", "hub", "survival"):
        root = INSTANCES / instance
        for relative in (
            ".lkjmc-runtime-identity.json",
            "eula.txt",
            "forwarding.secret",
            "server.properties",
            "spigot.yml",
            "velocity.toml",
        ):
            append_tree(records, root / relative, "rendered instance state")
        append_tree(records, root / "config", "rendered instance configuration")
        append_tree(records, root / "plugins", "installed plugin tree")
    records.sort(key=lambda item: item["path"].encode())
    raw = json.dumps(records, separators=(",", ":"), sort_keys=True).encode()
    service = pwd.getpwnam("lkjmc")
    show_raw = run([
        "systemctl", "show", SERVICE,
        "-p", "ActiveState", "-p", "SubState", "-p", "Result", "-p", "NRestarts",
        "-p", "MainPID", "-p", "ControlGroup", "-p", "ExecMainStartTimestampMonotonic",
        "-p", "ExecStartPre",
    ]).stdout
    show = dict(line.split("=", 1) for line in show_raw.splitlines() if "=" in line)
    return {
        "databaseSha256": database_fingerprint(),
        "fileIndex": records,
        "fileIndexSha256": hashlib.sha256(raw).hexdigest(),
        "migrationMarker": migration_marker(),
        "processes": process_observations(service.pw_uid),
        "selectedCommit": expected_commit,
        "selectedManifestSha256": expected_digest,
        "systemd": show,
    }


def verify_backup(path: Path, expected_commit: str) -> dict[str, Any]:
    if not path.is_absolute() or path.suffix != ".dump":
        raise FixtureError("backup path is not an absolute .dump path")
    try:
        path.relative_to("/var/backups/lkjmc")
    except ValueError as error:
        raise FixtureError("backup path escapes the lab backup root") from error
    members = [path, Path(f"{path}.manifest"), Path(f"{path}.metadata.json"), Path(f"{path}.sha256")]
    if {item.name for item in path.parent.iterdir()} != {item.name for item in members}:
        raise FixtureError("backup directory closure differs")
    observations = []
    for member in members:
        value = regular(member, "backup member")
        if value.st_uid != 0 or stat.S_IMODE(value.st_mode) != 0o600 or value.st_size <= 0:
            raise FixtureError("backup member ownership, mode, or size differs")
        observations.append({"name": member.name, "sha256": sha256(member), "size": value.st_size})
    checksums = {}
    for line in members[-1].read_text(encoding="ascii").splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  ([^/]+)", line)
        if not match or match.group(2) in checksums:
            raise FixtureError("backup checksum sidecar is malformed")
        checksums[match.group(2)] = match.group(1)
    for member in members[:-1]:
        if checksums.get(member.name) != sha256(member):
            raise FixtureError("backup member checksum differs")
    if set(checksums) != {member.name for member in members[:-1]}:
        raise FixtureError("backup checksum closure differs")
    metadata = json.loads(members[2].read_text())
    fields = {
        "dumpSha256", "lsn", "lsnSha256", "manifestSha256", "migrationMarker",
        "migrationSha256", "postgresServerVersion", "schemaSha256", "schemaVersion", "sourceCommit",
    }
    if not isinstance(metadata, dict) or set(metadata) != fields or metadata.get("schemaVersion") != 1 \
            or metadata.get("sourceCommit") != expected_commit \
            or metadata.get("dumpSha256") != sha256(path) \
            or metadata.get("manifestSha256") != sha256(members[1]):
        raise FixtureError("backup metadata identity differs")
    marker = json.loads(metadata.get("migrationMarker", "null"))
    canonical = json.dumps(marker, separators=(",", ":"), sort_keys=True)
    if not isinstance(marker, list) or marker != migration_marker() \
            or hashlib.sha256(canonical.encode()).hexdigest() != metadata.get("migrationSha256"):
        raise FixtureError("backup migration marker differs")
    listed = run(["pg_restore", "--list", path], timeout=120).stdout.encode()
    if listed != members[1].read_bytes():
        raise FixtureError("backup manifest differs from independent pg_restore listing")
    server = int(run([
        "runuser", "-u", "postgres", "--", "psql", "-d", "lkjmc", "-X", "--quiet",
        "--no-align", "--tuples-only", "-v", "ON_ERROR_STOP=1", "-c",
        "select current_setting('server_version_num')",
    ]).stdout.strip())
    if metadata.get("postgresServerVersion") != server or server // 10000 != 16:
        raise FixtureError("backup PostgreSQL compatibility differs")
    return {
        "members": observations,
        "migrationMarker": marker,
        "postgresServerVersion": server,
        "sourceCommit": expected_commit,
    }


def prepare_restore(
    expected_commit: str,
    expected_digest: str,
    baseline_commit: str,
    baseline_digest: str,
    backup: Path,
) -> dict[str, Any]:
    project = image_and_project_boundary()
    inputs = load_inputs(baseline_commit, baseline_digest)
    require_fresh_host()
    target_input = Path("/var/lib/private/lkjmc-drr-target")
    validate_manifest(target_input, expected_commit, expected_digest)
    if not backup.is_absolute() or backup.suffix != ".dump" \
            or backup.parent != Path("/var/lib/private/lkjmc-drr-restore-input"):
        raise FixtureError("restore input path differs")
    exact_closure(
        backup.parent,
        {backup.name, f"{backup.name}.manifest", f"{backup.name}.metadata.json", f"{backup.name}.sha256"},
        "restore backup input",
    )
    metadata = json.loads(Path(f"{backup}.metadata.json").read_text())
    if metadata.get("sourceCommit") != baseline_commit:
        raise FixtureError("restore backup source commit differs")

    run(["groupadd", "--system", "lkjmc"])
    run(["useradd", "--system", "--gid", "lkjmc", "--home-dir", "/var/lib/lkjmc", "--shell", "/usr/sbin/nologin", "lkjmc"])
    service = pwd.getpwnam("lkjmc")
    uid, gid = service.pw_uid, service.pw_gid
    password = secrets.token_hex(24)
    daemon_token = secrets.token_hex(32)
    forwarding_secret = secrets.token_hex(32)
    database_setup(password)

    mkdir(Path("/opt/lkjmc"), 0o755, 0, 0)
    mkdir(RELEASES, 0o755, 0, 0)
    release = RELEASES / expected_commit
    run([
        "python3", target_input / "source/lkjmc-install-artifacts", "--scope", "system",
        "--manifest", target_input / "artifact-manifest.json", "--manifest-sha256", expected_digest,
        "--source", target_input / "source", "--root", release,
        "--service-uid", str(uid), "--service-gid", str(gid),
    ], timeout=300)
    os.symlink(expected_commit, CURRENT)
    os.symlink("/opt/lkjmc/releases/current/bin/lkjmc", CLI_LINK)

    assets = {item["project"]: item for item in inputs["assets"]}
    mkdir(RUNTIME_ASSETS, 0o750, 0, gid)
    for name in ("folia", "velocity"):
        copy(INPUT_ROOT / "assets" / f"{name}.jar", RUNTIME_ASSETS / f"{name}.jar", 0o640, 0, gid)
    mkdir(CONFIG_ROOT, 0o750, 0, gid)
    mkdir(DATA_ROOT, 0o750, uid, gid)
    mkdir(INSTANCES, 0o750, uid, gid)
    mkdir(LOG_ROOT, 0o750, uid, gid)
    mkdir(LOG_ROOT / "instances", 0o750, uid, gid)
    mkdir(Path("/opt/lkjmc/jars"), 0o750, uid, gid)
    mkdir(Path("/opt/lkjmc/assets"), 0o750, uid, gid)
    database_secret = CONFIG_ROOT / "database.secret"
    write(database_secret, f"{password}\n".encode(), 0o600, uid, gid)
    write(CONFIG_ROOT / "daemon-http.token", f"{daemon_token}\n".encode(), 0o600, uid, gid)
    write(CONFIG_ROOT / "forwarding.secret", f"{forwarding_secret}\n".encode(), 0o600, uid, gid)
    database_url = f"postgres://lkjmc:{password}@127.0.0.1:5432/lkjmc"
    write(CONFIG_ROOT / "daemon.env", f"LKJMC_DATABASE_URL={database_url}\n".encode("ascii"), 0o600, uid, gid)
    write(
        CONFIG_ROOT / "lkjmc.json",
        (json.dumps(config_value(database_secret, assets), indent=2, sort_keys=True) + "\n").encode(),
        0o640,
        0,
        gid,
    )
    restore_env = dict(SAFE_ENV)
    restore_env.update({"LKJMC_CLI": str(release / "bin/lkjmc"), "LKJMC_DATABASE_URL": database_url})
    run([target_input / "source/lkjmc-restore-postgres", backup], env=restore_env, timeout=900)
    restored_marker = migration_marker()
    unit = "lkjmc-drr-restore-daemon.service"
    run([
        "systemd-run", "--unit", unit, "--collect", "--property=Type=exec", "--property=User=lkjmc",
        "--property=Group=lkjmc", "--property=WorkingDirectory=/var/lib/lkjmc",
        str(release / "bin/lkjmc-daemon"), "--config", "/etc/lkjmc/lkjmc.json",
        "--http-token-file", "/etc/lkjmc/daemon-http.token",
    ])
    wait_path(Path("/run/lkjmc/daemon.sock"), 30)
    status = json.loads(run([
        "runuser", "-u", "lkjmc", "--", release / "bin/lkjmc", "--json", "status",
    ], timeout=30).stdout)
    if status.get("build", {}).get("commit") != expected_commit or status.get("build", {}).get("dirty") is not False \
            or status.get("daemon") != "running" or status.get("database", {}).get("connected") is not True:
        raise FixtureError("restored application boundary differs")
    counts_query = """
select jsonb_build_object(
  'instances', (select count(*) from instances),
  'networkIntents', (select count(*) from network_intents),
  'jarAssets', (select count(*) from jar_assets),
  'daemonTokens', (select count(*) from daemon_tokens)
)::text
"""
    counts = json.loads(run([
        "runuser", "-u", "postgres", "--", "psql", "-d", "lkjmc", "-X", "--quiet",
        "--no-align", "--tuples-only", "-v", "ON_ERROR_STOP=1", "-c", counts_query,
    ]).stdout.strip())
    if counts.get("instances") != 3 or counts.get("networkIntents", 0) < 1 \
            or counts.get("jarAssets") != 2 or counts.get("daemonTokens", 0) < 3:
        raise FixtureError("restored retained product data differs")
    if (CONFIG_ROOT / "minecraft-eula.accepted").exists():
        raise FixtureError("restore fixture unexpectedly created EULA acceptance")
    return {
        "applicationStatus": status,
        "backupSha256": sha256(backup),
        "counts": counts,
        "daemonProcesses": process_observations(uid),
        "migrationMarker": restored_marker,
        "project": project,
        "systemdUnit": unit,
    }


def deployment_state(expected_commit: str, expected_digest: str) -> dict[str, Any]:
    image_and_project_boundary()
    target = Path("/var/lib/private/lkjmc-drr-target")
    validate_manifest(target, expected_commit, expected_digest)
    fence_path = CONFIG_ROOT / "deployment-fence.json"
    fence_metadata = regular(fence_path, "deployment fence")
    if fence_metadata.st_uid != 0 or stat.S_IMODE(fence_metadata.st_mode) != 0o600:
        raise FixtureError("deployment fence ownership or mode differs")
    fence = json.loads(fence_path.read_text())
    state_path = Path(str(fence.get("stateDirectory", ""))) / "deployment.json"
    state_metadata = regular(state_path, "deployment state")
    if state_metadata.st_uid != 0 or stat.S_IMODE(state_metadata.st_mode) != 0o600:
        raise FixtureError("deployment state ownership or mode differs")
    state = json.loads(state_path.read_text())
    if fence.get("toCommit") != expected_commit or state.get("toCommit") != expected_commit \
            or state.get("manifestSha256") != expected_digest \
            or state.get("fromCommit") != fence.get("fromCommit") \
            or state.get("backup") != fence.get("backup") \
            or state.get("rollbackSnapshot") != fence.get("rollbackSnapshot"):
        raise FixtureError("deployment fence and state identities differ")
    from_commit = state.get("fromCommit", "")
    if not HEX40.fullmatch(from_commit) or not (RELEASES / from_commit).is_dir() \
            or not (RELEASES / expected_commit).is_dir():
        raise FixtureError("deployment retained release closure differs")
    selected = os.readlink(CURRENT) if CURRENT.is_symlink() else None
    service = pwd.getpwnam("lkjmc")
    show_raw = run([
        "systemctl", "show", SERVICE, "-p", "ActiveState", "-p", "SubState", "-p", "Result",
        "-p", "MainPID", "-p", "ExecStartPre",
    ], check=False).stdout
    show = dict(line.split("=", 1) for line in show_raw.splitlines() if "=" in line)
    backup = Path(str(state.get("backup", "")))
    backup_present = all(Path(f"{backup}{suffix}").is_file() for suffix in ("", ".manifest", ".metadata.json", ".sha256"))
    dropin = Path("/etc/systemd/system/lkjmc-daemon.service.d/10-deployment-fence.conf")
    if not dropin.is_file() or dropin.is_symlink() or not backup_present:
        raise FixtureError("fenced deployment retained state differs")
    return {
        "backupPresent": backup_present,
        "dropinSha256": sha256(dropin),
        "fence": fence,
        "migrationMarker": migration_marker(),
        "processes": process_observations(service.pw_uid),
        "selectedCommit": selected,
        "state": state,
        "systemd": show,
    }


def prepare(expected_commit: str, expected_digest: str, accept_eula: bool) -> dict[str, Any]:
    project = image_and_project_boundary()
    inputs = load_inputs(expected_commit, expected_digest)
    if not accept_eula or inputs["minecraftEulaAccepted"] is not True:
        raise FixtureBlocked("explicit Minecraft EULA acceptance is absent")
    require_fresh_host()
    run(["groupadd", "--system", "lkjmc"])
    run(["useradd", "--system", "--gid", "lkjmc", "--home-dir", "/var/lib/lkjmc", "--shell", "/usr/sbin/nologin", "lkjmc"])
    service = pwd.getpwnam("lkjmc")
    group = grp.getgrnam("lkjmc")
    if service.pw_gid != group.gr_gid:
        raise FixtureError("service user/group identity differs")
    uid, gid = service.pw_uid, service.pw_gid
    database_password = secrets.token_hex(24)
    daemon_token = secrets.token_hex(32)
    forwarding_secret = secrets.token_hex(32)
    database_setup(database_password)

    mkdir(Path("/opt/lkjmc"), 0o755, 0, 0)
    mkdir(RELEASES, 0o755, 0, 0)
    release = RELEASES / expected_commit
    installer = INPUT_ROOT / "baseline/source/lkjmc-install-artifacts"
    run([
        "python3", installer, "--scope", "system", "--manifest", INPUT_ROOT / "baseline/artifact-manifest.json",
        "--manifest-sha256", expected_digest, "--source", INPUT_ROOT / "baseline/source", "--root", release,
        "--service-uid", str(uid), "--service-gid", str(gid),
    ], timeout=300)
    os.symlink(expected_commit, CURRENT)
    os.symlink("/opt/lkjmc/releases/current/bin/lkjmc", CLI_LINK)

    mkdir(RUNTIME_ASSETS, 0o750, 0, gid)
    assets = {item["project"]: item for item in inputs["assets"]}
    for name in ("folia", "velocity"):
        copy(INPUT_ROOT / "assets" / f"{name}.jar", RUNTIME_ASSETS / f"{name}.jar", 0o640, 0, gid)

    mkdir(CONFIG_ROOT, 0o750, 0, gid)
    mkdir(DATA_ROOT, 0o750, uid, gid)
    mkdir(DATA_ROOT / "private", 0o700, uid, gid)
    mkdir(DATA_ROOT / "private/plugin-credentials", 0o700, uid, gid)
    mkdir(INSTANCES, 0o750, uid, gid)
    for instance in ("proxy", "hub", "survival"):
        mkdir(INSTANCES / instance, 0o750, uid, gid)
        mkdir(INSTANCES / instance / "plugins", 0o750, uid, gid)
    mkdir(LOG_ROOT, 0o750, uid, gid)
    mkdir(LOG_ROOT / "instances", 0o750, uid, gid)
    mkdir(Path("/opt/lkjmc/jars"), 0o750, uid, gid)
    mkdir(Path("/opt/lkjmc/assets"), 0o750, uid, gid)

    database_secret = CONFIG_ROOT / "database.secret"
    write(database_secret, f"{database_password}\n".encode(), 0o600, uid, gid)
    write(CONFIG_ROOT / "daemon-http.token", f"{daemon_token}\n".encode(), 0o600, uid, gid)
    write(CONFIG_ROOT / "forwarding.secret", f"{forwarding_secret}\n".encode(), 0o600, uid, gid)
    database_url = f"postgres://lkjmc:{database_password}@127.0.0.1:5432/lkjmc"
    write(CONFIG_ROOT / "daemon.env", f"LKJMC_DATABASE_URL={database_url}\n".encode("ascii"), 0o600, uid, gid)
    config = config_value(database_secret, assets)
    write(CONFIG_ROOT / "lkjmc.json", (json.dumps(config, indent=2, sort_keys=True) + "\n").encode(), 0o640, 0, gid)
    write(CONFIG_ROOT / "minecraft-eula.accepted", b"schemaVersion=1\naccepted=true\n", 0o440, 0, gid)

    migrate_env = dict(SAFE_ENV)
    migrate_env["LKJMC_DATABASE_URL"] = database_url
    run(["runuser", "-u", "lkjmc", "--", release / "bin/lkjmc", "db", "migrate"], env=migrate_env, timeout=900)
    credentials = provision_credentials(release)
    copy(release / "jars/lkjmc-velocity.jar", INSTANCES / "proxy/plugins/lkjmc-velocity.jar", 0o640, 0, gid)
    for instance in ("hub", "survival"):
        copy(release / "jars/lkjmc-paper.jar", INSTANCES / instance / "plugins/lkjmc-paper.jar", 0o640, 0, gid)
    copy(release / "share/lkjmc-daemon.service", UNIT, 0o644, 0, 0)
    run(["systemctl", "daemon-reload"])
    run(["systemctl", "enable", SERVICE])
    run(["systemctl", "start", SERVICE], timeout=1600)
    observation = observe(expected_commit, expected_digest, require_startup_evidence=True)
    return {
        "credentialDigests": credentials,
        "databaseSecretSha256": hashlib.sha256(f"{database_password}\n".encode()).hexdigest(),
        "daemonTokenSha256": hashlib.sha256(f"{daemon_token}\n".encode()).hexdigest(),
        "forwardingSecretSha256": hashlib.sha256(f"{forwarding_secret}\n".encode()).hexdigest(),
        "observation": observation,
        "project": project,
    }


def encode_varint(value: int) -> bytes:
    value &= 0xFFFFFFFF
    output = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        output.append(byte | (0x80 if value else 0))
        if not value:
            return bytes(output)


def read_varint(stream: socket.socket) -> int:
    value = 0
    for shift in range(0, 35, 7):
        raw = stream.recv(1)
        if not raw:
            raise FixtureError("proxy status response ended early")
        value |= (raw[0] & 0x7F) << shift
        if raw[0] & 0x80 == 0:
            return value
    raise FixtureError("proxy status VarInt is oversized")


def proxy_status() -> dict[str, Any]:
    address = b"127.0.0.1"
    handshake = encode_varint(0) + encode_varint(-1) + encode_varint(len(address)) + address \
        + struct.pack(">H", 25591) + encode_varint(1)
    request = encode_varint(len(handshake)) + handshake + b"\x01\x00"
    try:
        with socket.create_connection(("127.0.0.1", 25591), timeout=5) as stream:
            stream.settimeout(5)
            stream.sendall(request)
            length = read_varint(stream)
            packet_id = read_varint(stream)
            text_length = read_varint(stream)
            if length < 3 or length > 1024 * 1024 or packet_id != 0 or text_length > 1024 * 1024:
                raise FixtureError("proxy status response header differs")
            payload = b""
            while len(payload) < text_length:
                chunk = stream.recv(text_length - len(payload))
                if not chunk:
                    raise FixtureError("proxy status response is truncated")
                payload += chunk
    except OSError as error:
        raise FixtureError("proxy status connection failed") from error
    value = json.loads(payload)
    if not isinstance(value, dict) or not isinstance(value.get("version"), dict) \
            or not isinstance(value.get("players"), dict):
        raise FixtureError("proxy status payload differs")
    return {"descriptionPresent": "description" in value, "players": value["players"], "version": value["version"]}


def process_observations(uid: int) -> list[dict[str, Any]]:
    result = run(["pgrep", "-u", str(uid)], check=False)
    if result.returncode not in {0, 1}:
        raise FixtureError("service process enumeration failed")
    values = []
    for raw in result.stdout.split():
        if not raw.isdigit():
            raise FixtureError("service process identity is invalid")
        pid = int(raw)
        proc = Path("/proc") / raw
        try:
            executable = str((proc / "exe").resolve(strict=True))
            stat_fields = (proc / "stat").read_text().split()
            cgroup = (proc / "cgroup").read_text().strip()
            comm = (proc / "comm").read_text().strip()
        except (FileNotFoundError, ProcessLookupError):
            continue
        if len(stat_fields) < 22 or not cgroup:
            raise FixtureError("service process start or cgroup identity is unavailable")
        values.append({"cgroup": cgroup, "comm": comm, "executable": executable, "pid": pid, "startTicks": int(stat_fields[21])})
    values.sort(key=lambda item: item["pid"])
    return values


def observe(expected_commit: str, expected_digest: str, *, require_startup_evidence: bool) -> dict[str, Any]:
    image_and_project_boundary()
    if not CURRENT.is_symlink() or os.readlink(CURRENT) != expected_commit \
            or CURRENT.resolve(strict=True) != RELEASES / expected_commit:
        raise FixtureError("current release pointer differs")
    release = CURRENT.resolve(strict=True)
    manifest = release / "meta/artifact-manifest.json"
    if sha256(manifest) != expected_digest:
        raise FixtureError("installed release manifest differs")
    version = json.loads(run([release / "bin/lkjmc", "--json", "version"]).stdout)
    if version.get("commit") != expected_commit or version.get("dirty") is not False:
        raise FixtureError("installed CLI identity differs")
    active = run(["systemctl", "is-active", SERVICE]).stdout.strip()
    show_raw = run(["systemctl", "show", SERVICE, "-p", "ActiveState", "-p", "SubState", "-p", "Result", "-p", "NRestarts", "-p", "MainPID", "-p", "ControlGroup", "-p", "User", "-p", "Group", "-p", "KillMode", "-p", "ExecStartPre"]).stdout
    show = dict(line.split("=", 1) for line in show_raw.splitlines() if "=" in line)
    if active != "active" or show.get("ActiveState") != "active" or show.get("SubState") != "running" \
            or show.get("Result") != "success" or show.get("User") != "lkjmc" \
            or show.get("Group") != "lkjmc" or show.get("KillMode") != "mixed":
        raise FixtureError("systemd service state or ownership differs")
    cli = release / "bin/lkjmc"
    status = json.loads(run(["runuser", "-u", "lkjmc", "--", cli, "--json", "status"], timeout=30).stdout)
    instances = status.get("instances")
    if status.get("build", {}).get("commit") != expected_commit or status.get("build", {}).get("dirty") is not False \
            or status.get("daemon") != "running" or status.get("database", {}).get("connected") is not True \
            or not isinstance(instances, list) or {item.get("id") for item in instances if isinstance(item, dict)} != {"proxy", "hub", "survival"}:
        raise FixtureError("application status differs")
    by_id = {item["id"]: item for item in instances}
    for instance in ("hub", "survival"):
        item = by_id[instance]
        if item.get("kind") != "folia" or item.get("processHealthy") is not True \
                or item.get("ready") is not True or item.get("joinable") is not True \
                or not isinstance(item.get("readinessAgeSeconds"), int) or item["readinessAgeSeconds"] > 30 \
                or not isinstance(item.get("proxyRegistrationAgeSeconds"), int) or item["proxyRegistrationAgeSeconds"] > 30:
            raise FixtureError(f"backend is not freshly ready: {instance}")
    if by_id["proxy"].get("kind") != "velocity" or by_id["proxy"].get("processHealthy") is not True:
        raise FixtureError("proxy process is not healthy")
    plan = json.loads(run(["runuser", "-u", "lkjmc", "--", cli, "--json", "bootstrap", "plan", "--profile", "playable", "--bedrock", "disabled"], timeout=30).stdout)
    if plan.get("outcome") != "no-op" or plan.get("changes") != [] or plan.get("unsupported") != []:
        raise FixtureError("network reconciliation is not a no-op")
    for instance in ("proxy", "hub", "survival"):
        log = INSTANCES / instance / "logs/latest.log"
        text = regular(log, "instance log") and log.read_text(encoding="utf-8", errors="replace")[-4 * 1024 * 1024:]
        if "Bearer " in text:
            raise FixtureError("instance log contains authorization material")
        if require_startup_evidence and (f"commit={expected_commit} dirty=false" not in text \
                or f"lkjmc heartbeat active instance={instance} detail=accepted" not in text):
            raise FixtureError(f"plugin startup evidence differs: {instance}")
    service = pwd.getpwnam("lkjmc")
    listeners = run(["ss", "-H", "-lntup"]).stdout.splitlines()
    expected_ports = {5432, 8765, 25566, 25567, 25591}
    observed_ports = set()
    for line in listeners:
        match = re.search(r"(?:127\.0\.0\.1|0\.0\.0\.0|\[::\]):([0-9]+)\b", line)
        if match:
            observed_ports.add(int(match.group(1)))
    if not expected_ports.issubset(observed_ports):
        raise FixtureError("required private listener set differs")
    controls = {
        "fence": Path("/etc/lkjmc/deployment-fence.json").exists(),
        "permit": Path("/run/lkjmc-deploy-start-permit").exists(),
    }
    if any(controls.values()):
        raise FixtureError("unexpected deployment fence or permit remains")
    plugin_digests = {}
    for instance, name in (("proxy", "lkjmc-velocity.jar"), ("hub", "lkjmc-paper.jar"), ("survival", "lkjmc-paper.jar")):
        installed = INSTANCES / instance / "plugins" / name
        expected = release / "jars" / name
        if sha256(installed) != sha256(expected):
            raise FixtureError(f"installed plugin differs: {instance}")
        plugin_digests[instance] = sha256(installed)
    return {
        "controls": controls,
        "listenerPorts": sorted(observed_ports),
        "manifestSha256": expected_digest,
        "migrationMarker": migration_marker(),
        "pluginDigests": plugin_digests,
        "processes": process_observations(service.pw_uid),
        "proxyStatus": proxy_status(),
        "releaseCommit": expected_commit,
        "status": status,
        "systemd": show,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "mode",
        choices=(
            "deployment-state",
            "fingerprint",
            "prepare",
            "observe",
            "prepare-restore",
            "verify-backup",
        ),
    )
    parser.add_argument("--expected-commit", required=True)
    parser.add_argument("--manifest-sha256", required=True)
    parser.add_argument("--accept-minecraft-eula", action="store_true")
    parser.add_argument("--require-startup-evidence", action="store_true")
    parser.add_argument("--backup", type=Path)
    parser.add_argument("--baseline-commit")
    parser.add_argument("--baseline-manifest-sha256")
    args = parser.parse_args()
    if not HEX40.fullmatch(args.expected_commit) or not HEX64.fullmatch(args.manifest_sha256):
        parser.error("expected release identity is invalid")
    result: dict[str, Any] = {"schemaVersion": 1, "mode": args.mode, "status": "FAILED"}
    code = 1
    try:
        if args.mode not in {"prepare-restore", "verify-backup"} and args.backup is not None:
            raise FixtureError("backup path is only valid for verify-backup")
        if args.mode == "prepare":
            result["observation"] = prepare(args.expected_commit, args.manifest_sha256, args.accept_minecraft_eula)
        elif args.mode == "deployment-state":
            if args.accept_minecraft_eula or args.require_startup_evidence:
                raise FixtureError("deployment-state accepts no mutable intent")
            result["observation"] = deployment_state(args.expected_commit, args.manifest_sha256)
        elif args.mode == "fingerprint":
            if args.accept_minecraft_eula or args.require_startup_evidence:
                raise FixtureError("fingerprint accepts no mutable intent")
            result["observation"] = fingerprint(args.expected_commit, args.manifest_sha256)
        elif args.mode == "verify-backup":
            if args.backup is None or args.accept_minecraft_eula or args.require_startup_evidence:
                raise FixtureError("verify-backup requires only one backup path")
            result["observation"] = verify_backup(args.backup, args.expected_commit)
        elif args.mode == "prepare-restore":
            if args.backup is None or not HEX40.fullmatch(args.baseline_commit or "") \
                    or not HEX64.fullmatch(args.baseline_manifest_sha256 or "") \
                    or args.accept_minecraft_eula or args.require_startup_evidence:
                raise FixtureError("prepare-restore input identity differs")
            result["observation"] = prepare_restore(
                args.expected_commit,
                args.manifest_sha256,
                args.baseline_commit,
                args.baseline_manifest_sha256,
                args.backup,
            )
        else:
            if args.accept_minecraft_eula:
                raise FixtureError("observe does not accept EULA intent")
            result["observation"] = observe(args.expected_commit, args.manifest_sha256, require_startup_evidence=args.require_startup_evidence)
        result["status"] = "PASS"
        code = 0
    except FixtureBlocked as error:
        result["status"] = "BLOCKED"
        result["error"] = str(error)
        code = 2
    except (FixtureError, OSError, ValueError, json.JSONDecodeError) as error:
        result["error"] = str(error)
    print(json.dumps(result, sort_keys=True))
    return code


if __name__ == "__main__":
    raise SystemExit(main())
