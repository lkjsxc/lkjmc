"""Bounded Docker/systemd acceptance helpers owned only by the test lab."""
from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from typing import Any, Iterable, Mapping
import urllib.error
import urllib.request
import zipfile


SCHEMA_VERSION = 1
INPUT_SCHEMA_VERSION = 1
PROJECT_LABEL = "io.lkjmc.docker-release-recovery.project"
PURPOSE_LABEL = "io.lkjmc.docker-release-recovery.purpose"
PROJECT_PATTERN = re.compile(r"lkjmcdrr-[a-z0-9][a-z0-9-]{7,47}")
MAX_COMMAND_OUTPUT = 4 * 1024 * 1024
MAX_TRANSPORT_BYTES = 512 * 1024 * 1024
MAX_EVIDENCE_BYTES = 16 * 1024 * 1024
MIN_ENGINE_MEMORY = 10 * 1024**3
MIN_ENGINE_CPUS = 8
MIN_WORKSPACE_DISK_AVAILABLE = 5 * 1024**3
MIN_SUBSTRATE_DOCKER_DISK_AVAILABLE = 1 * 1024**3
MIN_MATRIX_DOCKER_DISK_AVAILABLE = 30 * 1024**3
EXPECTED_CAPABILITIES = frozenset(
    {
        "CHOWN",
        "DAC_OVERRIDE",
        "FOWNER",
        "FSETID",
        "KILL",
        "NET_BIND_SERVICE",
        "SETFCAP",
        "SETGID",
        "SETPCAP",
        "SETUID",
        "SYS_ADMIN",
        "SYS_CHROOT",
    }
)
HEX40 = re.compile(r"[0-9a-f]{40}")
HEX64 = re.compile(r"[0-9a-f]{64}")
ROOT = Path(__file__).resolve().parents[2]
SUPPORT = Path(__file__).resolve().parent
COMPOSE = SUPPORT / "compose.yml"
SERVICE = "lkjmc-daemon.service"


class LabError(RuntimeError):
    """A failed acceptance invariant."""


class Blocked(LabError):
    """A missing or unsafe host prerequisite."""


class MissedInterruptWindow(LabError):
    """The unmodified updater passed the accepted external interruption window."""


@dataclass(frozen=True)
class Completed:
    argv: tuple[str, ...]
    returncode: int
    stdout: str
    stderr: str
    seconds: float


def _write_all(descriptor: int, value: bytes) -> None:
    view = memoryview(value)
    while view:
        written = os.write(descriptor, view)
        if written <= 0:
            raise LabError("private file write made no progress")
        view = view[written:]


def new_project() -> str:
    return f"lkjmcdrr-{int(time.time()):x}-{secrets.token_hex(5)}"


def validate_project(value: str) -> str:
    if not PROJECT_PATTERN.fullmatch(value) or value.endswith("-") or "--" in value:
        raise LabError("project identity must be a bounded canonical lkjmcdrr name")
    return value


def endpoint_class(host: str) -> str:
    if host == "unix:///var/run/docker.sock":
        return "local-default-unix"
    if host.startswith("unix://"):
        return "local-other-unix"
    if host.startswith("tcp://127.0.0.1") or host.startswith("tcp://localhost"):
        return "local-loopback-tcp"
    if host.startswith("ssh://"):
        return "ssh-remote"
    return "nonlocal-or-unknown"


def docker_storage_observation(info: Mapping[str, Any]) -> dict[str, Any]:
    raw = info.get("DockerRootDir")
    if not isinstance(raw, str) or not raw or not Path(raw).is_absolute():
        raise Blocked("Docker data-root identity is unavailable")
    try:
        values = os.statvfs(raw)
    except OSError as error:
        raise Blocked("Docker data-root filesystem capacity is unavailable") from error
    block_size = values.f_frsize or values.f_bsize
    return {
        "availableBytes": block_size * values.f_bavail,
        "path": raw,
        "totalBytes": block_size * values.f_blocks,
    }


def _normalize_capability(value: str) -> str:
    value = value.upper()
    return value[4:] if value.startswith("CAP_") else value


def validate_compose_model(model: Mapping[str, Any], project: str) -> dict[str, Any]:
    services = model.get("services")
    if not isinstance(services, dict) or set(services) != {"host"}:
        raise LabError("Compose must contain exactly the systemd host service")
    host = services["host"]
    if not isinstance(host, dict):
        raise LabError("Compose host service is invalid")
    if host.get("privileged") is True:
        raise LabError("privileged Docker is forbidden")
    if host.get("command") != ["/usr/local/libexec/lkjmc-drr-systemd-entrypoint"]:
        raise LabError("the systemd host entrypoint differs")
    if host.get("network_mode") == "host" or host.get("pid") == "host":
        raise LabError("host network and host PID namespaces are forbidden")
    if host.get("ports") or host.get("expose"):
        raise LabError("the Docker lab may not publish or expose a host port")
    volumes = host.get("volumes") or []
    if volumes:
        raise LabError("the substrate probe may not use host or named-volume mounts")
    if host.get("devices"):
        raise LabError("the Docker lab may not pass through devices")
    if host.get("cgroup") != "private":
        raise LabError("the systemd host must use a private cgroup namespace")
    cap_drop = {_normalize_capability(str(item)) for item in host.get("cap_drop") or []}
    cap_add = {_normalize_capability(str(item)) for item in host.get("cap_add") or []}
    if cap_drop != {"ALL"} or cap_add != EXPECTED_CAPABILITIES:
        raise LabError("the Docker capability closure differs from the bounded contract")
    security = {str(item).lower() for item in host.get("security_opt") or []}
    if security != {"apparmor=unconfined", "no-new-privileges:false"}:
        raise LabError("the Docker security options differ from the systemd contract")
    labels = host.get("labels") or {}
    if labels.get(PROJECT_LABEL) != project or labels.get(PURPOSE_LABEL) != "systemd-host":
        raise LabError("the systemd host labels differ")
    networks = model.get("networks")
    if not isinstance(networks, dict) or set(networks) != {"lab"}:
        raise LabError("Compose must declare exactly one private lab network")
    network = networks["lab"]
    if not isinstance(network, dict) or network.get("internal") is not True:
        raise LabError("the lab network must be internal")
    network_labels = network.get("labels") or {}
    if network_labels.get(PROJECT_LABEL) != project:
        raise LabError("the lab network project label differs")
    return {
        "capabilities": sorted(cap_add),
        "cgroupNamespace": host["cgroup"],
        "networkInternal": True,
        "ports": [],
        "privileged": False,
        "volumes": [],
    }


def validate_container_inspect(value: Mapping[str, Any], project: str, container: str) -> dict[str, Any]:
    config = value.get("Config") or {}
    host = value.get("HostConfig") or {}
    network = value.get("NetworkSettings") or {}
    labels = config.get("Labels") or {}
    if value.get("Name") != f"/{container}":
        raise LabError("Docker returned a different container identity")
    if labels.get(PROJECT_LABEL) != project or labels.get(PURPOSE_LABEL) != "systemd-host":
        raise LabError("container ownership labels differ")
    if config.get("Cmd") != ["/usr/local/libexec/lkjmc-drr-systemd-entrypoint"]:
        raise LabError("effective systemd host entrypoint differs")
    if host.get("Privileged") is not False:
        raise LabError("container is privileged")
    if host.get("NetworkMode") == "host" or host.get("PidMode") == "host":
        raise LabError("container shares a host namespace")
    if host.get("CgroupnsMode") != "private":
        raise LabError("container cgroup namespace is not private")
    if host.get("Binds") or value.get("Mounts"):
        raise LabError("container has an unexpected bind or volume mount")
    if host.get("Devices"):
        raise LabError("container has an unexpected device")
    if config.get("ExposedPorts") or host.get("PortBindings") or network.get("Ports"):
        raise LabError("container exposes or publishes a port")
    cap_drop = {_normalize_capability(str(item)) for item in host.get("CapDrop") or []}
    cap_add = {_normalize_capability(str(item)) for item in host.get("CapAdd") or []}
    if cap_drop != {"ALL"} or cap_add != EXPECTED_CAPABILITIES:
        raise LabError("effective container capabilities differ")
    tmpfs = host.get("Tmpfs") or {}
    if set(tmpfs) != {"/run", "/run/lock", "/sys/fs/cgroup", "/tmp"}:
        raise LabError("effective tmpfs closure differs")
    security = {str(item).lower() for item in host.get("SecurityOpt") or []}
    if security != {"apparmor=unconfined", "no-new-privileges:false"}:
        raise LabError(f"effective container security options differ: {sorted(security)}")
    if any("docker.sock" in str(item) for item in (host.get("Binds") or [])):
        raise LabError("Docker socket mount is forbidden")
    return {
        "capabilities": sorted(cap_add),
        "cgroupNamespace": host["CgroupnsMode"],
        "containerId": str(value.get("Id", "")),
        "hostPid": int((value.get("State") or {}).get("Pid", 0)),
        "memoryLimit": int(host.get("Memory", 0)),
        "nanoCpus": int(host.get("NanoCpus", 0)),
        "networkMode": host.get("NetworkMode"),
        "pidLimit": int(host.get("PidsLimit", 0)),
        "ports": [],
        "privileged": False,
        "startedAt": (value.get("State") or {}).get("StartedAt"),
    }


def object_list_commands(project: str) -> dict[str, tuple[str, ...]]:
    validate_project(project)
    selector = f"label={PROJECT_LABEL}={project}"
    return {
        "containers": ("container", "ls", "-aq", "--filter", selector),
        "networks": ("network", "ls", "-q", "--filter", selector),
        "volumes": ("volume", "ls", "-q", "--filter", selector),
        "images": ("image", "ls", "-q", "--filter", selector),
    }


def verify_owned(value: Mapping[str, Any], project: str, kind: str) -> None:
    labels = (value.get("Config") or {}).get("Labels") if kind in {"containers", "images"} else value.get("Labels")
    if not isinstance(labels, dict) or labels.get(PROJECT_LABEL) != project:
        raise LabError(f"refusing cleanup of {kind} object without exact project ownership")


def private_json(path: Path, value: Mapping[str, Any]) -> None:
    if not path.is_absolute():
        raise LabError("evidence output must be an absolute path")
    parent = path.parent
    parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    parent_metadata = parent.lstat()
    if not stat.S_ISDIR(parent_metadata.st_mode) or parent.is_symlink() \
            or parent_metadata.st_uid != os.getuid() or stat.S_IMODE(parent_metadata.st_mode) & 0o077:
        raise LabError("evidence parent must be a private current-user directory")
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW
    descriptor = os.open(path, flags, 0o600)
    try:
        raw = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()
        _write_all(descriptor, raw)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def publish_evidence_packet(path: Path, value: dict[str, Any]) -> dict[str, Any]:
    path = path.absolute()
    index_path = path.with_name(f"{path.name}.index.json")
    if os.path.lexists(path) or os.path.lexists(index_path):
        raise LabError("refusing an existing evidence packet target")
    parent = path.parent
    parent_metadata = parent.lstat()
    if not stat.S_ISDIR(parent_metadata.st_mode) or parent.is_symlink() \
            or parent_metadata.st_uid != os.getuid() or stat.S_IMODE(parent_metadata.st_mode) & 0o077:
        raise LabError("evidence packet parent must be a private current-user directory")
    value["evidence"] = {
        "indexSelfExcluded": f"{index_path.name} is omitted to avoid recursive hashing",
        "retainedFiles": [path.name, index_path.name],
        "secretScan": "PASS",
    }
    stage = Path(tempfile.mkdtemp(prefix=f".{path.name}.partial-", dir=parent))
    os.chmod(stage, 0o700)
    staged_result = stage / path.name
    staged_index = stage / index_path.name
    published = False
    try:
        private_json(staged_result, value)
        result_metadata = staged_result.stat()
        if result_metadata.st_size > MAX_EVIDENCE_BYTES:
            raise LabError("structured evidence exceeds the retained size bound")
        index = {
            "entries": [
                {
                    "maxSize": MAX_EVIDENCE_BYTES,
                    "mode": "0600",
                    "path": path.name,
                    "sha256": _sha256_path(staged_result),
                    "size": result_metadata.st_size,
                }
            ],
            "schemaVersion": 1,
            "selfExcluded": f"{index_path.name} is omitted to avoid recursive hashing",
        }
        private_json(staged_index, index)
        canary = f"lkjmc-drr-scan-{secrets.token_hex(32)}"
        scan = subprocess.run(
            (
                sys.executable,
                str(ROOT / "scripts/scan-secrets.py"),
                "--canary",
                canary,
                "--path",
                str(stage),
            ),
            cwd=ROOT,
            env={
                "LANG": "C",
                "LC_ALL": "C",
                "PATH": "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
            },
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=300,
            check=False,
        )
        if scan.returncode:
            raise LabError("canonical secret scan rejected the private evidence packet")
        os.rename(staged_result, path)
        os.rename(staged_index, index_path)
        directory_descriptor = os.open(parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            os.fsync(directory_descriptor)
        finally:
            os.close(directory_descriptor)
        published = True
        return {
            "index": str(index_path),
            "indexSha256": _sha256_path(index_path),
            "result": str(path),
            "resultSha256": _sha256_path(path),
            "secretScan": "PASS",
        }
    finally:
        if not published:
            path.unlink(missing_ok=True)
            index_path.unlink(missing_ok=True)
        shutil.rmtree(stage, ignore_errors=True)


def _sha256_path(path: Path) -> str:
    import hashlib

    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def _sha256_json(value: Any) -> str:
    import hashlib

    raw = json.dumps(value, separators=(",", ":"), sort_keys=True).encode()
    return hashlib.sha256(raw).hexdigest()


def _sha256_bytes(value: bytes) -> str:
    import hashlib

    return hashlib.sha256(value).hexdigest()


def _runtime_build_context_sha256() -> str:
    names = (
        "Dockerfile",
        "fixture.py",
        "lkjmc-lab-probe.service",
        "runtime-packages.lock",
        "systemd-entrypoint",
    )
    return _sha256_json({name: _sha256_path(SUPPORT / name) for name in names})


def extract_transport_zip(transport: Path, output: Path) -> dict[str, Any]:
    """Extract only the canonical three-file Actions transport into a private new root."""
    transport = transport.absolute()
    output = output.absolute()
    metadata = transport.lstat()
    if not stat.S_ISREG(metadata.st_mode) or transport.is_symlink() \
            or metadata.st_uid != os.getuid() or stat.S_IMODE(metadata.st_mode) & 0o077:
        raise LabError("artifact transport must be a private current-user regular file")
    if metadata.st_size <= 0 or metadata.st_size > MAX_TRANSPORT_BYTES:
        raise LabError("artifact transport size exceeds the bounded contract")
    if os.path.lexists(output):
        raise LabError("refusing an existing artifact output")
    parent = output.parent
    parent_metadata = parent.lstat()
    if not stat.S_ISDIR(parent_metadata.st_mode) or parent.is_symlink() \
            or parent_metadata.st_uid != os.getuid() or stat.S_IMODE(parent_metadata.st_mode) & 0o077:
        raise LabError("artifact output parent must be a private current-user directory")
    stage = parent / f".{output.name}.partial-{secrets.token_hex(8)}"
    stage.mkdir(mode=0o700)
    published = False
    try:
        with zipfile.ZipFile(transport) as archive:
            items = archive.infolist()
            names = [item.filename for item in items]
            if len(items) != 3 or len(names) != len(set(names)):
                raise LabError("artifact transport must contain exactly three unique members")
            if any(
                not name
                or Path(name).name != name
                or name in {".", ".."}
                or "/" in name
                or "\\" in name
                for name in names
            ):
                raise LabError("artifact transport contains an unsafe member name")
            archives = [name for name in names if name.endswith(".tar")]
            if len(archives) != 1 or set(names) != {
                archives[0],
                f"{archives[0]}.sha256",
                "release-handoff.json",
            }:
                raise LabError("artifact transport member closure differs")
            total = 0
            for item in items:
                mode = (item.external_attr >> 16) & 0xFFFF
                if item.is_dir() or stat.S_ISLNK(mode) or (mode and not stat.S_ISREG(mode)):
                    raise LabError("artifact transport contains a link, directory, or special member")
                if item.file_size < 0 or item.file_size > MAX_TRANSPORT_BYTES \
                        or item.compress_size < 0 or item.compress_size > MAX_TRANSPORT_BYTES:
                    raise LabError("artifact transport member exceeds the bounded contract")
                total += item.file_size
            if total > MAX_TRANSPORT_BYTES:
                raise LabError("artifact transport expanded closure exceeds the bounded contract")
            records = []
            for item in sorted(items, key=lambda value: value.filename.encode()):
                destination = stage / item.filename
                descriptor = os.open(
                    destination,
                    os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                    0o600,
                )
                written = 0
                try:
                    with archive.open(item, "r") as source:
                        while block := source.read(1024 * 1024):
                            written += len(block)
                            if written > item.file_size:
                                raise LabError("artifact transport member grew during extraction")
                            _write_all(descriptor, block)
                    if written != item.file_size:
                        raise LabError("artifact transport member was truncated")
                    os.fsync(descriptor)
                finally:
                    os.close(descriptor)
                records.append(
                    {
                        "path": item.filename,
                        "sha256": _sha256_path(destination),
                        "size": written,
                    }
                )
        directory = os.open(stage, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
        os.rename(stage, output)
        published = True
        return {
            "outerFiles": records,
            "outerSha256": _sha256_path(transport),
            "outerSize": metadata.st_size,
        }
    except (OSError, zipfile.BadZipFile) as error:
        raise LabError(f"artifact transport extraction failed: {error.__class__.__name__}") from error
    finally:
        if not published and stage.exists():
            shutil.rmtree(stage)


def _private_regular(path: Path, label: str, mode: int | None = None) -> os.stat_result:
    metadata = path.lstat()
    if not stat.S_ISREG(metadata.st_mode) or path.is_symlink() or metadata.st_uid != os.getuid() \
            or stat.S_IMODE(metadata.st_mode) & 0o077:
        raise LabError(f"{label} must be a private current-user regular file")
    if mode is not None and stat.S_IMODE(metadata.st_mode) != mode:
        raise LabError(f"{label} mode differs")
    return metadata


def _private_directory(path: Path, label: str) -> os.stat_result:
    metadata = path.lstat()
    if not stat.S_ISDIR(metadata.st_mode) or path.is_symlink() or metadata.st_uid != os.getuid() \
            or stat.S_IMODE(metadata.st_mode) != 0o700:
        raise LabError(f"{label} must be a private current-user directory")
    return metadata


def _release_contract() -> dict[str, str]:
    data = json.loads((ROOT / "config" / "release-artifacts.json").read_text())
    if set(data) != {"schemaVersion", "artifacts"} or data["schemaVersion"] != 1:
        raise LabError("release artifact contract schema differs")
    values = {}
    for item in data["artifacts"]:
        name = item.get("destination")
        kind = item.get("kind")
        if not isinstance(name, str) or Path(name).name != name or kind not in {"binary", "config", "jar"}:
            raise LabError("release artifact contract is invalid")
        if name in values:
            raise LabError("release artifact contract is duplicated")
        values[name] = kind
    return values


def _validate_release_input(value: Mapping[str, Any], label: str) -> dict[str, Any]:
    fields = {
        "archiveSha256",
        "archiveSize",
        "artifactDir",
        "artifactId",
        "artifactName",
        "artifactServiceDigest",
        "commit",
        "expiresAt",
        "manifestSha256",
        "metadataPath",
        "outerSha256",
        "outerSize",
        "producerJob",
        "releaseRoot",
        "source",
        "transportPath",
        "workflowEvent",
        "workflowRef",
        "workflowRunAttempt",
        "workflowRunId",
    }
    if set(value) != fields:
        raise LabError(f"{label} release descriptor fields differ")
    commit = value.get("commit")
    digests = (value.get("archiveSha256"), value.get("manifestSha256"), value.get("outerSha256"))
    if not isinstance(commit, str) or not HEX40.fullmatch(commit) \
            or any(not isinstance(item, str) or not HEX64.fullmatch(item) for item in digests):
        raise LabError(f"{label} release identity is invalid")
    if value.get("source") != "github-actions-retained" \
            or value.get("artifactServiceDigest") != f"sha256:{value['outerSha256']}":
        raise LabError(f"{label} release source identity differs")
    for name in ("artifactId", "workflowRunId", "workflowRunAttempt", "outerSize", "archiveSize"):
        if type(value.get(name)) is not int or value[name] <= 0:
            raise LabError(f"{label} release numeric identity is invalid")
    if value.get("workflowEvent") != "push" or value.get("workflowRef") != "refs/heads/main" \
            or value.get("producerJob") != "verify-compose":
        raise LabError(f"{label} workflow identity differs")
    paths = {
        name: Path(str(value[name]))
        for name in ("artifactDir", "metadataPath", "releaseRoot", "transportPath")
    }
    if any(not path.is_absolute() for path in paths.values()):
        raise LabError(f"{label} release paths must be absolute")
    metadata_stat = _private_regular(paths["metadataPath"], f"{label} artifact metadata", 0o600)
    transport_stat = _private_regular(paths["transportPath"], f"{label} artifact transport", 0o600)
    if metadata_stat.st_size > 1024 * 1024 or transport_stat.st_size != value["outerSize"] \
            or _sha256_path(paths["transportPath"]) != value["outerSha256"]:
        raise LabError(f"{label} artifact transport differs")
    metadata = json.loads(paths["metadataPath"].read_text())
    comparisons = {
        "id": value["artifactId"],
        "name": value["artifactName"],
        "digest": value["artifactServiceDigest"],
        "size_in_bytes": value["outerSize"],
        "expires_at": value["expiresAt"],
    }
    if any(metadata.get(name) != expected for name, expected in comparisons.items()) \
            or metadata.get("expired") is not False:
        raise LabError(f"{label} artifact service metadata differs")
    _private_directory(paths["artifactDir"], f"{label} artifact directory")
    outer_files = {path.name: path for path in paths["artifactDir"].iterdir()}
    archives = [name for name in outer_files if name.endswith(".tar")]
    if len(archives) != 1 or set(outer_files) != {
        archives[0],
        f"{archives[0]}.sha256",
        "release-handoff.json",
    }:
        raise LabError(f"{label} artifact directory closure differs")
    for path in outer_files.values():
        _private_regular(path, f"{label} artifact member", 0o600)
    archive = outer_files[archives[0]]
    if archive.stat().st_size != value["archiveSize"] or _sha256_path(archive) != value["archiveSha256"]:
        raise LabError(f"{label} release archive differs")
    sidecar = outer_files[f"{archives[0]}.sha256"].read_text(encoding="ascii")
    if sidecar != f"{value['archiveSha256']}  {archives[0]}\n":
        raise LabError(f"{label} release archive sidecar differs")
    handoff = json.loads(outer_files["release-handoff.json"].read_text())
    handoff_expectations = {
        "archiveFilename": archives[0],
        "archiveSha256": value["archiveSha256"],
        "archiveSize": value["archiveSize"],
        "outerArtifactName": value["artifactName"],
        "producerJob": value["producerJob"],
        "releaseManifestSha256": value["manifestSha256"],
        "repository": "lkjsxc/lkjmc",
        "sourceCommit": commit,
        "workflowEvent": value["workflowEvent"],
        "workflowRef": value["workflowRef"],
        "workflowRunAttempt": value["workflowRunAttempt"],
        "workflowRunId": str(value["workflowRunId"]),
    }
    if any(handoff.get(name) != expected for name, expected in handoff_expectations.items()):
        raise LabError(f"{label} release handoff differs")
    _private_directory(paths["releaseRoot"], f"{label} release root")
    manifest_path = paths["releaseRoot"] / "artifact-manifest.json"
    manifest_sidecar = paths["releaseRoot"] / "artifact-manifest.json.sha256"
    source = paths["releaseRoot"] / "source"
    _private_regular(manifest_path, f"{label} release manifest", 0o600)
    _private_regular(manifest_sidecar, f"{label} release manifest sidecar", 0o600)
    _private_directory(source, f"{label} release source")
    if _sha256_path(manifest_path) != value["manifestSha256"] \
            or manifest_sidecar.read_text(encoding="ascii") != (
                f"{value['manifestSha256']}  artifact-manifest.json\n"
            ):
        raise LabError(f"{label} release manifest digest differs")
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("schemaVersion") != 1 or manifest.get("commit") != commit:
        raise LabError(f"{label} release manifest identity differs")
    contract = _release_contract()
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list) or len(artifacts) != len(contract):
        raise LabError(f"{label} release manifest artifact closure differs")
    by_name = {item.get("path"): item for item in artifacts if isinstance(item, dict)}
    if set(by_name) != set(contract):
        raise LabError(f"{label} release manifest artifact names differ")
    if {path.name for path in source.iterdir()} != set(contract):
        raise LabError(f"{label} extracted release source closure differs")
    for name, kind in contract.items():
        item = by_name[name]
        artifact = source / name
        expected_mode = 0o700 if kind == "binary" else 0o600
        artifact_stat = _private_regular(artifact, f"{label} release artifact", expected_mode)
        if item.get("kind") != kind or item.get("size") != artifact_stat.st_size \
                or item.get("sha256") != _sha256_path(artifact):
            raise LabError(f"{label} release artifact differs: {name}")
    if {path.name for path in paths["releaseRoot"].iterdir()} != {
        "artifact-manifest.json",
        "artifact-manifest.json.sha256",
        "source",
    }:
        raise LabError(f"{label} extracted release root closure differs")
    return {
        "archiveSha256": value["archiveSha256"],
        "artifactId": value["artifactId"],
        "artifactName": value["artifactName"],
        "commit": commit,
        "manifestSha256": value["manifestSha256"],
        "outerSha256": value["outerSha256"],
        "releaseRoot": str(paths["releaseRoot"]),
    }


def _validate_asset(value: Mapping[str, Any]) -> dict[str, Any]:
    fields = {"build", "channel", "id", "kind", "name", "path", "project", "sha256", "size", "source", "version"}
    if set(value) != fields or value.get("kind") != "server" or value.get("channel") != "STABLE":
        raise LabError("immutable server asset fields differ")
    project = value.get("project")
    expected = {
        "folia": ("folia-server", "1.21.11"),
        "velocity": ("velocity-server", "3.4.0-SNAPSHOT"),
    }
    if project not in expected or (value.get("id"), value.get("version")) != expected[project]:
        raise LabError("immutable server asset identity differs")
    digest = value.get("sha256")
    size = value.get("size")
    build = value.get("build")
    if not isinstance(digest, str) or not HEX64.fullmatch(digest) or type(size) is not int or size <= 0 \
            or type(build) is not int or build <= 0:
        raise LabError("immutable server asset digest, size, or build is invalid")
    path = Path(str(value.get("path")))
    if not path.is_absolute() or path.name != value.get("name"):
        raise LabError("immutable server asset path differs")
    metadata = _private_regular(path, "immutable server asset", 0o600)
    if metadata.st_size != size or _sha256_path(path) != digest:
        raise LabError("immutable server asset bytes differ")
    expected_source = f"https://fill-data.papermc.io/v1/objects/{digest}/{value['name']}"
    if value.get("source") != expected_source:
        raise LabError("immutable server asset source differs")
    return {
        "build": build,
        "id": value["id"],
        "name": value["name"],
        "project": project,
        "sha256": digest,
        "size": size,
        "version": value["version"],
    }


def validate_input_descriptor(path: Path, *, require_target: bool = False, require_consent: bool = False) -> dict[str, Any]:
    path = path.absolute()
    _private_regular(path, "lab input descriptor", 0o600)
    if path.stat().st_size > 1024 * 1024:
        raise LabError("lab input descriptor is oversized")
    data = json.loads(path.read_text())
    fields = {"assets", "baseline", "minecraftEulaAccepted", "repository", "runtime", "schemaVersion", "target"}
    if not isinstance(data, dict) or set(data) != fields or data.get("schemaVersion") != INPUT_SCHEMA_VERSION \
            or data.get("repository") != "lkjsxc/lkjmc" or type(data.get("minecraftEulaAccepted")) is not bool:
        raise LabError("lab input descriptor schema differs")
    runtime = data.get("runtime")
    runtime_fields = {
        "architecture",
        "baseImage",
        "buildContextSha256",
        "composeSha256",
        "dockerfileSha256",
        "javaMajor",
        "packagesLockSha256",
        "postgresqlMajor",
        "snapshot",
    }
    if not isinstance(runtime, dict) or set(runtime) != runtime_fields:
        raise LabError("runtime image descriptor fields differ")
    runtime_expected = {
        "architecture": "amd64",
        "baseImage": "ubuntu:24.04@sha256:1e0a86e57d247923571b75e0aaf48a1449cf8c543d51fb3e07a4a7d7bfa79316",
        "buildContextSha256": _runtime_build_context_sha256(),
        "composeSha256": _sha256_path(COMPOSE),
        "dockerfileSha256": _sha256_path(SUPPORT / "Dockerfile"),
        "javaMajor": 21,
        "packagesLockSha256": _sha256_path(SUPPORT / "runtime-packages.lock"),
        "postgresqlMajor": 16,
        "snapshot": "20260830T000000Z",
    }
    if runtime != runtime_expected:
        raise LabError("runtime image descriptor identity differs")
    baseline = data.get("baseline")
    if not isinstance(baseline, dict):
        raise LabError("baseline release descriptor is missing")
    baseline_observation = _validate_release_input(baseline, "baseline")
    target = data.get("target")
    target_observation = None
    if target is not None:
        if not isinstance(target, dict):
            raise LabError("target release descriptor is invalid")
        target_observation = _validate_release_input(target, "target")
    elif require_target:
        raise Blocked("exact target release input is not yet available")
    assets = data.get("assets")
    if not isinstance(assets, list) or len(assets) != 2:
        raise LabError("immutable server asset closure differs")
    asset_observations = [_validate_asset(item) for item in assets if isinstance(item, dict)]
    if len(asset_observations) != 2 or {item["project"] for item in asset_observations} != {"folia", "velocity"}:
        raise LabError("immutable server asset project closure differs")
    if require_consent and data["minecraftEulaAccepted"] is not True:
        raise Blocked("explicit Minecraft EULA acceptance is absent")
    return {
        "assets": sorted(asset_observations, key=lambda item: item["project"]),
        "baseline": baseline_observation,
        "minecraftEulaAccepted": data["minecraftEulaAccepted"],
        "runtime": runtime_expected,
        "target": target_observation,
    }


def _host_command(
    arguments: Iterable[str | Path],
    *,
    timeout: int = 300,
    stdout_file: Path | None = None,
) -> Completed:
    argv = tuple(str(item) for item in arguments)
    started = time.monotonic()
    output_handle = None
    try:
        if stdout_file is not None:
            descriptor = os.open(
                stdout_file,
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                0o600,
            )
            output_handle = os.fdopen(descriptor, "wb")
        environment = {
            "LANG": "C",
            "LC_ALL": "C",
            "PATH": os.environ.get("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"),
            "PYTHONDONTWRITEBYTECODE": "1",
        }
        for name in (
            "GH_CONFIG_DIR",
            "GH_ENTERPRISE_TOKEN",
            "GH_HOST",
            "GH_TOKEN",
            "GITHUB_TOKEN",
            "HOME",
            "XDG_CONFIG_HOME",
        ):
            if name in os.environ:
                environment[name] = os.environ[name]
        result = subprocess.run(
            argv,
            cwd=ROOT,
            env=environment,
            stdout=output_handle if output_handle is not None else subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=output_handle is None,
            timeout=timeout,
            check=False,
        )
        if output_handle is not None:
            output_handle.flush()
            os.fsync(output_handle.fileno())
    except (OSError, subprocess.SubprocessError) as error:
        raise LabError(f"host command did not execute: {Path(argv[0]).name}") from error
    finally:
        if output_handle is not None:
            output_handle.close()
    elapsed = time.monotonic() - started
    stdout = "" if stdout_file is not None else str(result.stdout)
    stderr = result.stderr.decode(errors="replace") if isinstance(result.stderr, bytes) else str(result.stderr)
    if len(stdout.encode()) + len(stderr.encode()) > MAX_COMMAND_OUTPUT:
        raise LabError("host command output exceeded the evidence bound")
    completed = Completed(argv, result.returncode, stdout, stderr, elapsed)
    if result.returncode:
        diagnostic = stderr.strip()[-4096:]
        suffix = f": {diagnostic}" if diagnostic else ""
        raise LabError(
            f"host command failed with exit {result.returncode}: {Path(argv[0]).name}{suffix}"
        )
    return completed


def _gh_json(endpoint: str) -> dict[str, Any]:
    gh = shutil.which("gh")
    if not gh:
        raise Blocked("GitHub CLI is unavailable")
    result = _host_command((gh, "api", endpoint), timeout=120)
    try:
        value = json.loads(result.stdout)
    except (TypeError, ValueError) as error:
        raise LabError("GitHub API returned invalid JSON") from error
    if not isinstance(value, dict):
        raise LabError("GitHub API object shape differs")
    return value


def _release_common(metadata: Mapping[str, Any], run: Mapping[str, Any]) -> tuple[str, ...]:
    return (
        "--repository",
        "lkjsxc/lkjmc",
        "--outer-artifact-name",
        str(metadata["name"]),
        "--workflow-event",
        str(run["event"]),
        "--workflow-ref",
        "refs/heads/main",
        "--workflow-run-id",
        str(run["id"]),
        "--workflow-run-attempt",
        str(run["run_attempt"]),
        "--producer-job",
        "verify-compose",
    )


def _prepare_release_input(
    root: Path,
    label: str,
    artifact_id: int,
    expected_commit: str,
) -> dict[str, Any]:
    if label not in {"baseline", "target"} or artifact_id <= 0 or not HEX40.fullmatch(expected_commit):
        raise LabError("release preparation identity is invalid")
    release_dir = root / f"{label}-release"
    release_dir.mkdir(mode=0o700)
    metadata = _gh_json(f"repos/lkjsxc/lkjmc/actions/artifacts/{artifact_id}")
    workflow = metadata.get("workflow_run") or {}
    run_id = workflow.get("id")
    if metadata.get("id") != artifact_id or metadata.get("expired") is not False \
            or workflow.get("head_sha") != expected_commit or not isinstance(run_id, int):
        raise Blocked(f"{label} retained artifact identity or availability differs")
    digest_text = metadata.get("digest")
    if not isinstance(digest_text, str) or not digest_text.startswith("sha256:") \
            or not HEX64.fullmatch(digest_text.removeprefix("sha256:")):
        raise LabError(f"{label} artifact service digest differs")
    run = _gh_json(f"repos/lkjsxc/lkjmc/actions/runs/{run_id}")
    if run.get("id") != run_id or run.get("head_sha") != expected_commit \
            or run.get("head_branch") != "main" or run.get("event") != "push" \
            or run.get("status") != "completed" or run.get("conclusion") != "success" \
            or not isinstance(run.get("run_attempt"), int) or run["run_attempt"] <= 0:
        raise Blocked(f"{label} producer workflow is not the exact successful main push")
    expected_name = f"lkjmc-release-{expected_commit}-run-{run_id}-attempt-{run['run_attempt']}"
    if metadata.get("name") != expected_name:
        raise LabError(f"{label} artifact name differs from workflow identity")
    jobs = _gh_json(f"repos/lkjsxc/lkjmc/actions/runs/{run_id}/jobs?per_page=100").get("jobs")
    if not isinstance(jobs, list):
        raise LabError(f"{label} workflow job closure is unavailable")
    by_name = {item.get("name"): item for item in jobs if isinstance(item, dict)}
    required = {"docs-contracts", "verify-compose", "verify-release-artifact"}
    if not required.issubset(by_name) or any(by_name[name].get("conclusion") != "success" for name in required):
        raise Blocked(f"{label} required workflow jobs are not successful")
    metadata_path = release_dir / "artifact-metadata.json"
    private_json(metadata_path, metadata)
    transport = release_dir / "artifact-transport.zip"
    gh = shutil.which("gh")
    if not gh:
        raise Blocked("GitHub CLI is unavailable")
    _host_command(
        (gh, "api", f"repos/lkjsxc/lkjmc/actions/artifacts/{artifact_id}/zip"),
        timeout=900,
        stdout_file=transport,
    )
    transport_metadata = transport.stat()
    outer_digest = _sha256_path(transport)
    if outer_digest != digest_text.removeprefix("sha256:") \
            or transport_metadata.st_size != metadata.get("size_in_bytes"):
        raise LabError(f"{label} downloaded artifact transport differs")
    artifact_dir = release_dir / "outer"
    extract_transport_zip(transport, artifact_dir)
    common = _release_common(metadata, run)
    git = shutil.which("git")
    if not git:
        raise Blocked("Git is unavailable for exact release consumption")
    consumer_checkout = root / f".{label}-consumer-{secrets.token_hex(8)}"
    checkout_added = False
    primary_error: BaseException | None = None
    try:
        _host_command((git, "worktree", "add", "--detach", consumer_checkout, expected_commit))
        checkout_added = True
        checkout_head = _host_command((git, "-C", consumer_checkout, "rev-parse", "HEAD")).stdout.strip()
        checkout_state = _host_command(
            (git, "-C", consumer_checkout, "status", "--porcelain=v1", "--untracked-files=normal")
        ).stdout
        if checkout_head != expected_commit or checkout_state:
            raise LabError(f"{label} exact consumer checkout identity differs")
        archive_script = consumer_checkout / "scripts/release_archive.py"
        _host_command(
            (sys.executable, archive_script, "verify", *common, "--artifact-dir", artifact_dir)
        )
        receipt = release_dir / "consumer-receipt.json"
        _host_command(
            (
                sys.executable,
                archive_script,
                "consume",
                *common,
                "--artifact-dir",
                artifact_dir,
                "--work-parent",
                release_dir,
                "--receipt",
                receipt,
                "--artifact-id",
                str(artifact_id),
                "--artifact-digest",
                digest_text,
            ),
            timeout=900,
        )
        release_root = release_dir / "release-root"
        _host_command(
            (
                sys.executable,
                archive_script,
                "extract",
                *common,
                "--artifact-dir",
                artifact_dir,
                "--output",
                release_root,
            ),
            timeout=900,
        )
    except BaseException as error:
        primary_error = error
        raise
    finally:
        if checkout_added:
            try:
                _host_command((git, "worktree", "remove", consumer_checkout), timeout=300)
            except BaseException as cleanup_error:
                if primary_error is None:
                    raise
                raise LabError(
                    f"{primary_error}; exact consumer worktree cleanup failed: {cleanup_error}"
                ) from primary_error
    archives = [path for path in artifact_dir.iterdir() if path.name.endswith(".tar")]
    if len(archives) != 1:
        raise LabError(f"{label} artifact archive closure differs")
    archive = archives[0]
    manifest_digest = _sha256_path(release_root / "artifact-manifest.json")
    descriptor = {
        "archiveSha256": _sha256_path(archive),
        "archiveSize": archive.stat().st_size,
        "artifactDir": str(artifact_dir),
        "artifactId": artifact_id,
        "artifactName": metadata["name"],
        "artifactServiceDigest": digest_text,
        "commit": expected_commit,
        "expiresAt": metadata["expires_at"],
        "manifestSha256": manifest_digest,
        "metadataPath": str(metadata_path),
        "outerSha256": outer_digest,
        "outerSize": transport_metadata.st_size,
        "producerJob": "verify-compose",
        "releaseRoot": str(release_root),
        "source": "github-actions-retained",
        "transportPath": str(transport),
        "workflowEvent": "push",
        "workflowRef": "refs/heads/main",
        "workflowRunAttempt": run["run_attempt"],
        "workflowRunId": run_id,
    }
    _validate_release_input(descriptor, label)
    return descriptor


def _paper_api(project: str, version: str) -> list[dict[str, Any]]:
    url = f"https://fill.papermc.io/v3/projects/{project}/versions/{version}/builds"
    request = urllib.request.Request(url, headers={"User-Agent": "lkjmc-docker-release-recovery-lab/1"})
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            raw = response.read(4 * 1024 * 1024 + 1)
    except (OSError, urllib.error.URLError) as error:
        raise Blocked(f"PaperMC {project} resolver is unavailable") from error
    if len(raw) > 4 * 1024 * 1024:
        raise LabError(f"PaperMC {project} resolver response is oversized")
    try:
        value = json.loads(raw)
    except (TypeError, ValueError) as error:
        raise LabError(f"PaperMC {project} resolver returned invalid JSON") from error
    if not isinstance(value, list) or not all(isinstance(item, dict) for item in value):
        raise LabError(f"PaperMC {project} resolver shape differs")
    return value


def _prepare_server_asset(root: Path, project: str, version: str) -> dict[str, Any]:
    builds = _paper_api(project, version)
    stable = [item for item in builds if item.get("channel") == "STABLE" and type(item.get("id")) is int]
    if not stable:
        raise Blocked(f"PaperMC {project} has no stable build for {version}")
    selected = max(stable, key=lambda item: item["id"])
    download = (selected.get("downloads") or {}).get("server:default")
    if not isinstance(download, dict):
        raise LabError(f"PaperMC {project} default server download differs")
    name = download.get("name")
    digest = (download.get("checksums") or {}).get("sha256")
    size = download.get("size")
    source = download.get("url")
    if not isinstance(name, str) or Path(name).name != name or not HEX64.fullmatch(str(digest or "")) \
            or type(size) is not int or size <= 0 or source != (
                f"https://fill-data.papermc.io/v1/objects/{digest}/{name}"
            ):
        raise LabError(f"PaperMC {project} immutable download identity differs")
    path = root / name
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    written = 0
    calculated = hashlib.sha256()
    try:
        request = urllib.request.Request(source, headers={"User-Agent": "lkjmc-docker-release-recovery-lab/1"})
        with urllib.request.urlopen(request, timeout=300) as response:
            while block := response.read(1024 * 1024):
                written += len(block)
                if written > size:
                    raise LabError(f"PaperMC {project} download exceeded declared size")
                calculated.update(block)
                _write_all(descriptor, block)
        os.fsync(descriptor)
    except (OSError, urllib.error.URLError):
        raise Blocked(f"PaperMC {project} immutable download is unavailable")
    finally:
        os.close(descriptor)
    if written != size or calculated.hexdigest() != digest:
        raise LabError(f"PaperMC {project} downloaded bytes differ")
    return {
        "build": selected["id"],
        "channel": "STABLE",
        "id": f"{project}-server",
        "kind": "server",
        "name": name,
        "path": str(path),
        "project": project,
        "sha256": digest,
        "size": size,
        "source": source,
        "version": version,
    }


def prepare_input_descriptor(
    root: Path,
    *,
    baseline_artifact_id: int,
    baseline_commit: str,
    target_artifact_id: int | None,
    target_commit: str | None,
    accept_minecraft_eula: bool,
) -> dict[str, Any]:
    root = root.absolute()
    if os.path.lexists(root):
        raise LabError("refusing an existing lab input root")
    parent = root.parent.lstat()
    if not stat.S_ISDIR(parent.st_mode) or root.parent.is_symlink() \
            or parent.st_uid != os.getuid() or stat.S_IMODE(parent.st_mode) & 0o077:
        raise LabError("lab input parent must be a private current-user directory")
    root.mkdir(mode=0o700)
    completed = False
    try:
        assets_root = root / "assets"
        assets_root.mkdir(mode=0o700)
        baseline = _prepare_release_input(root, "baseline", baseline_artifact_id, baseline_commit)
        target = None
        if target_artifact_id is not None or target_commit is not None:
            if target_artifact_id is None or target_commit is None:
                raise LabError("target artifact ID and commit must be supplied together")
            target = _prepare_release_input(root, "target", target_artifact_id, target_commit)
        assets = [
            _prepare_server_asset(assets_root, "folia", "1.21.11"),
            _prepare_server_asset(assets_root, "velocity", "3.4.0-SNAPSHOT"),
        ]
        value = {
            "assets": assets,
            "baseline": baseline,
            "minecraftEulaAccepted": accept_minecraft_eula,
            "repository": "lkjsxc/lkjmc",
            "runtime": {
                "architecture": "amd64",
                "baseImage": "ubuntu:24.04@sha256:1e0a86e57d247923571b75e0aaf48a1449cf8c543d51fb3e07a4a7d7bfa79316",
                "buildContextSha256": _runtime_build_context_sha256(),
                "composeSha256": _sha256_path(COMPOSE),
                "dockerfileSha256": _sha256_path(SUPPORT / "Dockerfile"),
                "javaMajor": 21,
                "packagesLockSha256": _sha256_path(SUPPORT / "runtime-packages.lock"),
                "postgresqlMajor": 16,
                "snapshot": "20260830T000000Z",
            },
            "schemaVersion": INPUT_SCHEMA_VERSION,
            "target": target,
        }
        descriptor = root / "lab-input-v1.json"
        private_json(descriptor, value)
        observation = validate_input_descriptor(descriptor)
        completed = True
        return {
            "descriptor": str(descriptor),
            "descriptorSha256": _sha256_path(descriptor),
            "observation": observation,
            "root": str(root),
        }
    finally:
        if not completed and root.exists():
            shutil.rmtree(root)


class DockerLab:
    def __init__(self, project: str, *, docker: str | None = None, matrix_resources: bool = False) -> None:
        self.project = validate_project(project)
        self.container = f"{project}-host"
        self.image = f"{project}-systemd-host:runtime"
        self.docker = docker or shutil.which("docker") or ""
        if not self.docker:
            raise Blocked("Docker client is unavailable")
        self.matrix_resources = matrix_resources
        self.commands: list[dict[str, Any]] = []
        self.environment = os.environ.copy()
        self.environment.update(
            {
                "LKJMC_DRR_CONTAINER": self.container,
                "LKJMC_DRR_CPU_LIMIT": "8.0" if matrix_resources else "2.0",
                "LKJMC_DRR_IMAGE": self.image,
                "LKJMC_DRR_MEMORY_LIMIT": "6g" if matrix_resources else "768m",
                "LKJMC_DRR_PIDS_LIMIT": "4096" if matrix_resources else "512",
                "LKJMC_DRR_PROJECT": self.project,
            }
        )

    def run(
        self,
        arguments: Iterable[str],
        *,
        timeout: int = 60,
        check: bool = True,
        record: bool = True,
    ) -> Completed:
        argv = (self.docker, *tuple(str(item) for item in arguments))
        started = time.monotonic()
        try:
            result = subprocess.run(
                argv,
                cwd=ROOT,
                env=self.environment,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=timeout,
                check=False,
            )
        except (OSError, subprocess.SubprocessError) as error:
            raise Blocked(f"Docker command could not execute: {error.__class__.__name__}") from error
        elapsed = time.monotonic() - started
        if len(result.stdout.encode()) + len(result.stderr.encode()) > MAX_COMMAND_OUTPUT:
            raise LabError("Docker command output exceeded the evidence bound")
        completed = Completed(argv, result.returncode, result.stdout, result.stderr, elapsed)
        if record:
            self.commands.append(
                {
                    "argv": list(argv),
                    "exit": result.returncode,
                    "seconds": round(elapsed, 3),
                }
            )
        if check and result.returncode:
            detail = (result.stderr or result.stdout).strip().splitlines()
            suffix = f": {detail[-1][:512]}" if detail else ""
            raise LabError(f"Docker command failed ({Path(argv[1]).name}){suffix}")
        return completed

    def json(self, arguments: Iterable[str], *, timeout: int = 60) -> Any:
        result = self.run(arguments, timeout=timeout)
        try:
            return json.loads(result.stdout)
        except (TypeError, ValueError) as error:
            raise LabError("Docker returned invalid JSON") from error

    def compose(self, arguments: Iterable[str], *, timeout: int = 120, check: bool = True) -> Completed:
        return self.run(
            (
                "compose",
                "--project-name",
                self.project,
                "--file",
                str(COMPOSE),
                *tuple(arguments),
            ),
            timeout=timeout,
            check=check,
        )

    def owned_objects(self) -> dict[str, list[str]]:
        found: dict[str, list[str]] = {}
        for kind, arguments in object_list_commands(self.project).items():
            output = self.run(arguments, record=False).stdout
            found[kind] = sorted(set(line for line in output.splitlines() if line))
        return found

    def require_empty(self) -> None:
        found = {kind: values for kind, values in self.owned_objects().items() if values}
        if found:
            raise Blocked(f"project identity collides with existing labeled Docker objects: {sorted(found)}")

    def preflight(self) -> dict[str, Any]:
        self.run(("version", "--format", "{{json .}}"))
        context_name = self.run(("context", "show")).stdout.strip()
        contexts = self.json(("context", "inspect", context_name))
        if not isinstance(contexts, list) or len(contexts) != 1:
            raise Blocked("Docker context inspection differed")
        context = contexts[0]
        endpoint = ((context.get("Endpoints") or {}).get("docker") or {}).get("Host", "")
        endpoint_kind = endpoint_class(endpoint)
        if endpoint_kind not in {"local-default-unix", "local-other-unix", "local-loopback-tcp"}:
            raise Blocked(f"Docker context is not an accepted local development endpoint: {endpoint_kind}")
        info = self.json(("info", "--format", "{{json .}}"))
        security = [str(item) for item in info.get("SecurityOptions") or []]
        rootless = any("rootless" in item.lower() for item in security)
        cgroup_version = str(info.get("CgroupVersion", ""))
        cgroup_driver = str(info.get("CgroupDriver", ""))
        if cgroup_version != "2" or cgroup_driver != "systemd":
            raise Blocked("Docker must use cgroup v2 with the systemd cgroup driver")
        cpus = int(info.get("NCPU", 0))
        memory = int(info.get("MemTotal", 0))
        workspace = os.statvfs(ROOT)
        workspace_available = workspace.f_frsize * workspace.f_bavail
        docker_storage = docker_storage_observation(info)
        docker_required = (
            MIN_MATRIX_DOCKER_DISK_AVAILABLE
            if self.matrix_resources
            else MIN_SUBSTRATE_DOCKER_DISK_AVAILABLE
        )
        if cpus < MIN_ENGINE_CPUS or memory < MIN_ENGINE_MEMORY \
                or workspace_available < MIN_WORKSPACE_DISK_AVAILABLE:
            raise Blocked("Docker host CPU, memory, or workspace capacity is below the bounded minimum")
        if docker_storage["availableBytes"] < docker_required:
            raise Blocked(
                "Docker data-root capacity is below the bounded "
                f"{'full-matrix' if self.matrix_resources else 'substrate'} minimum: "
                f"available={docker_storage['availableBytes']} required={docker_required}"
            )
        self.require_empty()
        return {
            "architecture": info.get("Architecture"),
            "cgroupDriver": cgroup_driver,
            "cgroupVersion": cgroup_version,
            "context": context_name,
            "cpus": cpus,
            "dockerStorage": docker_storage,
            "dockerStorageRequiredBytes": docker_required,
            "endpointClass": endpoint_kind,
            "engineVersion": info.get("ServerVersion"),
            "memoryBytes": memory,
            "rootless": rootless,
            "securityOptions": security,
            "storageDriver": info.get("Driver"),
            "workspaceAvailableBytes": workspace_available,
        }

    def _container_inspect(self) -> dict[str, Any]:
        values = self.json(("container", "inspect", self.container))
        if not isinstance(values, list) or len(values) != 1 or not isinstance(values[0], dict):
            raise LabError("container inspection closure differs")
        return values[0]

    def _exec(
        self,
        arguments: Iterable[str],
        *,
        timeout: int = 60,
        check: bool = True,
        record: bool = True,
    ) -> Completed:
        return self.run(
            ("exec", self.container, *tuple(arguments)),
            timeout=timeout,
            check=check,
            record=record,
        )

    def _wait_systemd(self, timeout: int = 60) -> str:
        deadline = time.monotonic() + timeout
        last = "unknown"
        while time.monotonic() < deadline:
            inspect = self._container_inspect()
            state = inspect.get("State") or {}
            if state.get("Running") is not True:
                logs = self.run(("logs", "--tail", "200", self.container), check=False)
                detail = (logs.stderr or logs.stdout).strip().splitlines()
                suffix = " | ".join(line[:512] for line in detail[-20:]) if detail else "no container log"
                raise LabError(
                    f"systemd container exited early: exit={state.get('ExitCode')} error={state.get('Error') or '-'}: {suffix}"
                )
            result = self._exec(
                ("/usr/bin/systemctl", "show", "--property=SystemState", "--value"),
                timeout=10,
                check=False,
            )
            last = result.stdout.strip()
            if result.returncode == 0 and last in {"running", "degraded"}:
                return last
            time.sleep(1)
        raise LabError(f"systemd did not reach a bounded terminal boot state: {last}")

    def _process_observation(self) -> dict[str, Any]:
        comm = self._exec(("/usr/bin/cat", "/proc/1/comm")).stdout.strip()
        executable = self._exec(("/usr/bin/readlink", "-f", "/proc/1/exe")).stdout.strip()
        if comm != "systemd" or executable != "/usr/lib/systemd/systemd":
            raise LabError("PID 1 is not the real packaged systemd executable")
        active = self._exec(("/usr/bin/systemctl", "is-active", "lkjmc-lab-probe.service")).stdout.strip()
        if active != "active":
            raise LabError("the real systemd probe unit is not active")
        main_pid_text = self._exec(
            ("/usr/bin/systemctl", "show", "lkjmc-lab-probe.service", "--property=MainPID", "--value")
        ).stdout.strip()
        if not main_pid_text.isdigit() or int(main_pid_text) <= 1:
            raise LabError("the systemd probe MainPID is invalid")
        main_pid = int(main_pid_text)
        unit_cgroup = self._exec(
            ("/usr/bin/systemctl", "show", "lkjmc-lab-probe.service", "--property=ControlGroup", "--value")
        ).stdout.strip()
        pid_cgroup = self._exec(("/usr/bin/cat", f"/proc/{main_pid}/cgroup")).stdout.strip()
        pid_stat = self._exec(("/usr/bin/cat", f"/proc/{main_pid}/stat")).stdout.strip().split()
        init_stat = self._exec(("/usr/bin/cat", "/proc/1/stat")).stdout.strip().split()
        if len(pid_stat) < 22 or len(init_stat) < 22:
            raise LabError("process start identity is unavailable")
        if unit_cgroup not in pid_cgroup:
            raise LabError("systemd unit cgroup and process cgroup differ")
        return {
            "initExecutable": executable,
            "initStartTicks": int(init_stat[21]),
            "mainPid": main_pid,
            "mainStartTicks": int(pid_stat[21]),
            "pid1Comm": comm,
            "processCgroup": pid_cgroup,
            "unitActive": active,
            "unitCgroup": unit_cgroup,
        }

    def _network_observation(self) -> dict[str, Any]:
        objects = self.owned_objects()
        if len(objects["networks"]) != 1:
            raise LabError("the lab network closure differs")
        networks = self.json(("network", "inspect", objects["networks"][0]))
        if not isinstance(networks, list) or len(networks) != 1:
            raise LabError("lab network inspection differs")
        network = networks[0]
        verify_owned(network, self.project, "networks")
        if network.get("Internal") is not True:
            raise LabError("Docker lab network is not internal")
        return {"id": network.get("Id"), "internal": True, "name": network.get("Name")}

    def _build_and_boot(self) -> dict[str, Any]:
        preflight = self.preflight()
        model = json.loads(self.compose(("config", "--format", "json")).stdout)
        compose_boundary = validate_compose_model(model, self.project)
        self.compose(("build", "--pull", "host"), timeout=1800)
        images = self.owned_objects()["images"]
        if len(images) != 1:
            raise LabError("the project-owned runtime image closure differs")
        image_values = self.json(("image", "inspect", images[0]))
        if not isinstance(image_values, list) or len(image_values) != 1:
            raise LabError("runtime image inspection differs")
        image_value = image_values[0]
        verify_owned(image_value, self.project, "images")
        if (image_value.get("Config") or {}).get("ExposedPorts"):
            raise LabError("runtime image unexpectedly exposes a port")
        self.compose(("up", "--detach", "--no-build", "host"), timeout=180)
        system_state = self._wait_systemd()
        inspected = validate_container_inspect(self._container_inspect(), self.project, self.container)
        audit = self._exec(
            (
                "/bin/sh",
                "-ec",
                "for name in cargo rustc gradle git; do ! command -v \"$name\"; done; "
                "command -v java >/dev/null; command -v pg_dump >/dev/null; "
                "test ! -e /workspace; test ! -e /src; "
                "test -f /usr/share/lkjmc-docker-release-recovery-packages.txt; "
                "test -f /usr/share/lkjmc-drr-image",
            )
        )
        return {
            "composeBoundary": compose_boundary,
            "container": inspected,
            "image": {
                "id": image_value.get("Id"),
                "labels": (image_value.get("Config") or {}).get("Labels"),
                "repoDigests": image_value.get("RepoDigests") or [],
                "size": image_value.get("Size"),
            },
            "imageAuditExit": audit.returncode,
            "network": self._network_observation(),
            "preflight": preflight,
            "systemState": system_state,
        }

    def _copy_to_container(self, source: Path, destination: str, *, contents: bool = False) -> None:
        source = source.absolute()
        if not source.exists() or source.is_symlink():
            raise LabError("fixture copy source is absent or unsafe")
        source_text = f"{source}/." if contents else str(source)
        self.run(("container", "cp", source_text, f"{self.container}:{destination}"), timeout=300)

    def stage_baseline_inputs(self, descriptor: Path, *, accept_minecraft_eula: bool) -> dict[str, Any]:
        observation = validate_input_descriptor(descriptor)
        value = json.loads(descriptor.read_text())
        baseline = value["baseline"]
        release_root = Path(baseline["releaseRoot"])
        assets = {item["project"]: item for item in value["assets"]}
        self._exec(
            (
                "/usr/bin/install",
                "-d",
                "-m",
                "0700",
                "/var/lib/private/lkjmc-drr-input",
                "/var/lib/private/lkjmc-drr-input/assets",
                "/var/lib/private/lkjmc-drr-input/baseline",
            )
        )
        self._copy_to_container(
            release_root,
            "/var/lib/private/lkjmc-drr-input/baseline",
            contents=True,
        )
        for project in ("folia", "velocity"):
            self._copy_to_container(
                Path(assets[project]["path"]),
                f"/var/lib/private/lkjmc-drr-input/assets/{project}.jar",
            )
        container_input = {
            "assets": [
                {
                    "id": assets[project]["id"],
                    "name": f"{project}.jar",
                    "project": project,
                    "sha256": assets[project]["sha256"],
                    "size": assets[project]["size"],
                }
                for project in ("folia", "velocity")
            ],
            "baseline": {
                "commit": baseline["commit"],
                "manifestSha256": baseline["manifestSha256"],
            },
            "minecraftEulaAccepted": bool(value["minecraftEulaAccepted"] or accept_minecraft_eula),
            "project": self.project,
            "schemaVersion": 1,
        }
        with tempfile.TemporaryDirectory(prefix="lkjmc-drr-container-input-") as raw:
            temporary = Path(raw)
            os.chmod(temporary, 0o700)
            path = temporary / "input.json"
            private_json(path, container_input)
            self._copy_to_container(path, "/var/lib/private/lkjmc-drr-input/input.json")
        self._exec(
            (
                "/bin/sh",
                "-ec",
                "chown -R 0:0 /var/lib/private/lkjmc-drr-input; "
                "find /var/lib/private/lkjmc-drr-input -type d -exec chmod 0700 {} +; "
                "find /var/lib/private/lkjmc-drr-input -type f -exec chmod go-rwx {} +; "
                "chmod 0600 /var/lib/private/lkjmc-drr-input/input.json "
                "/var/lib/private/lkjmc-drr-input/assets/folia.jar "
                "/var/lib/private/lkjmc-drr-input/assets/velocity.jar",
            ),
            timeout=300,
        )
        return {
            "baseline": observation["baseline"],
            "containerEulaAccepted": container_input["minecraftEulaAccepted"],
            "serverAssets": observation["assets"],
        }

    def fixture(
        self,
        mode: str,
        commit: str,
        manifest_sha256: str,
        *,
        accept_minecraft_eula: bool = False,
        backup: str | None = None,
        baseline_commit: str | None = None,
        baseline_manifest_sha256: str | None = None,
        require_startup_evidence: bool = False,
        timeout: int = 1800,
        check: bool = True,
    ) -> tuple[Completed, dict[str, Any]]:
        arguments = [
            "/usr/local/libexec/lkjmc-drr-fixture",
            mode,
            "--expected-commit",
            commit,
            "--manifest-sha256",
            manifest_sha256,
        ]
        if accept_minecraft_eula:
            arguments.append("--accept-minecraft-eula")
        if backup is not None:
            arguments.extend(("--backup", backup))
        if baseline_commit is not None:
            arguments.extend(("--baseline-commit", baseline_commit))
        if baseline_manifest_sha256 is not None:
            arguments.extend(("--baseline-manifest-sha256", baseline_manifest_sha256))
        if require_startup_evidence:
            arguments.append("--require-startup-evidence")
        completed = self._exec(arguments, timeout=timeout, check=False)
        try:
            value = json.loads(completed.stdout)
        except (TypeError, ValueError) as error:
            raise LabError("fixture returned invalid JSON") from error
        if not isinstance(value, dict) or value.get("schemaVersion") != 1 or value.get("mode") != mode:
            raise LabError("fixture result schema differs")
        if check and completed.returncode:
            raise LabError(f"fixture {mode} did not pass: {value.get('status')}")
        return completed, value

    def consent_gate(self, descriptor: Path) -> dict[str, Any]:
        boundary = self._build_and_boot()
        staged = self.stage_baseline_inputs(descriptor, accept_minecraft_eula=False)
        baseline = staged["baseline"]
        completed, result = self.fixture(
            "prepare",
            baseline["commit"],
            baseline["manifestSha256"],
            check=False,
        )
        if completed.returncode != 2 or result.get("status") != "BLOCKED" \
                or result.get("error") != "explicit Minecraft EULA acceptance is absent":
            raise LabError("missing-consent fixture gate did not block exactly")
        no_effect = self._exec(
            (
                "/bin/sh",
                "-ec",
                "test ! -e /etc/lkjmc; test ! -e /opt/lkjmc; test ! -e /var/lib/lkjmc; "
                "test ! -e /var/log/lkjmc; ! getent passwd lkjmc >/dev/null; "
                "test -z \"$(pg_lsclusters --no-header)\"; "
                "test ! -e /etc/lkjmc/minecraft-eula.accepted",
            )
        )
        return {
            "boundary": boundary,
            "fixture": result,
            "noProductEffectExit": no_effect.returncode,
            "stagedInputs": staged,
        }

    def stage_target_input(self, descriptor: Path) -> dict[str, Any]:
        observation = validate_input_descriptor(descriptor, require_target=True)
        target = observation["target"]
        if target is None:
            raise LabError("validated target input is absent")
        destination = "/var/lib/private/lkjmc-drr-target"
        self._exec(("/usr/bin/install", "-d", "-m", "0700", destination))
        self._copy_to_container(Path(target["releaseRoot"]), destination, contents=True)
        self._exec(
            (
                "/bin/sh",
                "-ec",
                f"chown -R 0:0 {destination}; "
                f"find {destination} -type d -exec chmod 0700 {{}} +; "
                f"find {destination} -type f -exec chmod go-rwx {{}} +",
            ),
            timeout=300,
        )
        return target

    def prepare_baseline(self, descriptor: Path, *, accept_minecraft_eula: bool) -> dict[str, Any]:
        if not accept_minecraft_eula:
            raise Blocked("baseline preparation requires explicit Minecraft EULA acceptance")
        boundary = self._build_and_boot()
        staged = self.stage_baseline_inputs(descriptor, accept_minecraft_eula=True)
        baseline = staged["baseline"]
        completed, result = self.fixture(
            "prepare",
            baseline["commit"],
            baseline["manifestSha256"],
            accept_minecraft_eula=True,
            timeout=1800,
            check=False,
        )
        if completed.returncode or result.get("status") != "PASS":
            raise LabError(f"fresh baseline fixture failed: {result.get('error', result.get('status'))}")
        external = validate_container_inspect(self._container_inspect(), self.project, self.container)
        if external["ports"]:
            raise LabError("fresh baseline fixture published a host port")
        return {"boundary": boundary, "fixture": result["observation"], "stagedInputs": staged}

    def create_rollback_point(self, commit: str, manifest_sha256: str) -> dict[str, Any]:
        _, before = self.fixture("fingerprint", commit, manifest_sha256)
        self.run(("stop", "--time", "60", self.container), timeout=120)
        stopped = self._container_inspect().get("State") or {}
        if stopped.get("Running") is not False or int(stopped.get("ExitCode", -1)) != 0:
            raise LabError("rollback-point shutdown did not stop systemd cleanly")
        name = f"{self.project}-rollback:{commit[:12]}"
        commit_result = self.run(
            (
                "container",
                "commit",
                "--change",
                f"LABEL {PROJECT_LABEL}={self.project}",
                "--change",
                f"LABEL {PURPOSE_LABEL}=rollback-point",
                self.container,
                name,
            ),
            timeout=900,
        )
        image_id = commit_result.stdout.strip()
        values = self.json(("image", "inspect", image_id))
        if not isinstance(values, list) or len(values) != 1:
            raise LabError("rollback-point image inspection differs")
        verify_owned(values[0], self.project, "images")
        labels = (values[0].get("Config") or {}).get("Labels") or {}
        if labels.get(PURPOSE_LABEL) != "rollback-point" or (values[0].get("Config") or {}).get("ExposedPorts"):
            raise LabError("rollback-point image boundary differs")
        self.run(("start", self.container), timeout=120)
        state = self._wait_systemd(120)
        _, after_observe = self.fixture(
            "observe",
            commit,
            manifest_sha256,
            require_startup_evidence=True,
            timeout=1800,
        )
        descriptor = {
            "baselineFingerprintSha256": _sha256_json(before["observation"]),
            "commit": commit,
            "imageId": values[0].get("Id"),
            "imageSize": values[0].get("Size"),
            "manifestSha256": manifest_sha256,
            "project": self.project,
            "schemaVersion": 1,
        }
        return {
            "descriptor": descriptor,
            "descriptorSha256": _sha256_json(descriptor),
            "imageLabels": labels,
            "restartObservation": after_observe["observation"],
            "systemState": state,
        }

    def deployer(
        self,
        target: Mapping[str, Any],
        command: str,
        arguments: Iterable[str],
        *,
        timeout: int = 1800,
        check: bool = True,
    ) -> Completed:
        root = "/var/lib/private/lkjmc-drr-target"
        argv = (
            f"{root}/source/lkjmc-deploy-release",
            command,
            "--release-root",
            root,
            "--manifest-sha256",
            str(target["manifestSha256"]),
            *tuple(arguments),
        )
        return self._exec(argv, timeout=timeout, check=check)

    def update_restart_matrix(
        self,
        baseline: Mapping[str, Any],
        target: Mapping[str, Any],
        rollback: Mapping[str, Any],
    ) -> dict[str, Any]:
        baseline_commit = str(baseline["commit"])
        baseline_manifest = str(baseline["manifestSha256"])
        target_commit = str(target["commit"])
        target_manifest = str(target["manifestSha256"])
        _, negative_before = self.fixture("fingerprint", baseline_commit, baseline_manifest)
        wrong_target = dict(target)
        wrong_target["manifestSha256"] = "0" * 64
        negative = self.deployer(
            wrong_target,
            "update",
            ("--from-commit", baseline_commit),
            timeout=300,
            check=False,
        )
        if negative.returncode == 0 or "manifest differs" not in negative.stderr:
            raise LabError("wrong-manifest preflight did not fail with the expected class")
        _, negative_after = self.fixture("fingerprint", baseline_commit, baseline_manifest)
        if negative_before["observation"] != negative_after["observation"]:
            raise LabError("wrong-manifest preflight changed product state")

        backup = f"/var/backups/lkjmc/{self.project}-update/rollback.dump"
        changed = self.deployer(
            target,
            "update",
            (
                "--from-commit",
                baseline_commit,
                "--backup",
                backup,
                "--rollback-snapshot",
                f"docker-{self.project}-{str(rollback['descriptorSha256'])[:16]}",
            ),
        )
        try:
            changed_value = json.loads(changed.stdout)
        except (TypeError, ValueError) as error:
            raise LabError("changed update returned invalid JSON") from error
        if changed_value.get("result") != "updated" or changed_value.get("fromCommit") != baseline_commit \
                or changed_value.get("toCommit") != target_commit:
            raise LabError("changed update receipt differs")
        _, backup_result = self.fixture(
            "verify-backup",
            baseline_commit,
            baseline_manifest,
            backup=backup,
            timeout=300,
        )
        _, accepted = self.fixture(
            "observe",
            target_commit,
            target_manifest,
            require_startup_evidence=True,
            timeout=1800,
        )

        _, noop_before = self.fixture("fingerprint", target_commit, target_manifest)
        noop = self.deployer(target, "update", ("--from-commit", target_commit), timeout=300)
        try:
            noop_value = json.loads(noop.stdout)
        except (TypeError, ValueError) as error:
            raise LabError("exact no-op returned invalid JSON") from error
        if noop_value != {"schemaVersion": 1, "result": "no-op", "commit": target_commit}:
            raise LabError("exact no-op receipt differs")
        _, noop_after = self.fixture("fingerprint", target_commit, target_manifest)
        if noop_before["observation"] != noop_after["observation"]:
            raise LabError("exact no-op changed product state")

        restart_before = noop_after["observation"]
        self._exec(("systemctl", "restart", SERVICE), timeout=1800)
        _, service_restart = self.fixture(
            "observe",
            target_commit,
            target_manifest,
            require_startup_evidence=True,
            timeout=1800,
        )
        _, restart_after = self.fixture("fingerprint", target_commit, target_manifest)
        if restart_before["systemd"].get("MainPID") == restart_after["observation"]["systemd"].get("MainPID") \
                or restart_before["processes"] == restart_after["observation"]["processes"]:
            raise LabError("service restart did not replace the owned process group")

        container_before = validate_container_inspect(self._container_inspect(), self.project, self.container)
        init_before = self._process_observation()
        self.run(("restart", "--timeout", "60", self.container), timeout=150)
        system_state = self._wait_systemd(180)
        container_after = validate_container_inspect(self._container_inspect(), self.project, self.container)
        init_after = self._process_observation()
        if container_before["hostPid"] == container_after["hostPid"] \
                or container_before["startedAt"] == container_after["startedAt"] \
                or init_before["initStartTicks"] == init_after["initStartTicks"]:
            raise LabError("Docker restart did not replace container and PID 1 identities")
        _, docker_restart = self.fixture(
            "observe",
            target_commit,
            target_manifest,
            require_startup_evidence=True,
            timeout=1800,
        )
        return {
            "backup": {"path": backup, "verification": backup_result["observation"]},
            "changedUpdate": changed_value,
            "dockerRestart": {
                "after": container_after,
                "before": container_before,
                "observation": docker_restart["observation"],
                "systemState": system_state,
            },
            "negativePreflight": {
                "exit": negative.returncode,
                "fingerprintSha256": _sha256_json(negative_before["observation"]),
            },
            "noOp": {
                "fingerprintSha256": _sha256_json(noop_before["observation"]),
                "receipt": noop_value,
            },
            "serviceRestart": service_restart["observation"],
            "targetAcceptance": accepted["observation"],
        }

    def copy_backup_out(
        self,
        backup_path: str,
        destination: Path,
        verification: Mapping[str, Any],
    ) -> dict[str, Any]:
        destination = destination.absolute()
        if os.path.lexists(destination):
            raise LabError("refusing an existing private backup handoff directory")
        destination.mkdir(mode=0o700, parents=False)
        self.run(
            (
                "container",
                "cp",
                f"{self.container}:{Path(backup_path).parent}/.",
                str(destination),
            ),
            timeout=300,
        )
        expected = {item["name"]: item for item in verification.get("members") or []}
        actual = {path.name: path for path in destination.iterdir()}
        if set(actual) != set(expected) or len(actual) != 4:
            raise LabError("private backup handoff closure differs")
        observations = []
        for name, path in sorted(actual.items()):
            metadata = path.lstat()
            if not stat.S_ISREG(metadata.st_mode) or path.is_symlink() \
                    or metadata.st_uid != os.getuid() or stat.S_IMODE(metadata.st_mode) & 0o077 \
                    or metadata.st_size != expected[name]["size"] \
                    or _sha256_path(path) != expected[name]["sha256"]:
                raise LabError("private backup handoff member differs")
            observations.append({"name": name, "sha256": expected[name]["sha256"], "size": metadata.st_size})
        return {
            "directory": str(destination),
            "directoryDigest": _sha256_json(observations),
            "members": observations,
        }

    def stage_restore_inputs(self, descriptor: Path, backup_directory: Path) -> dict[str, Any]:
        inputs = self.stage_baseline_inputs(descriptor, accept_minecraft_eula=False)
        target = self.stage_target_input(descriptor)
        destination = "/var/lib/private/lkjmc-drr-restore-input"
        self._exec(("/usr/bin/install", "-d", "-m", "0700", destination))
        self._copy_to_container(backup_directory, destination, contents=True)
        self._exec(
            (
                "/bin/sh",
                "-ec",
                f"chown -R 0:0 {destination}; chmod 0700 {destination}; "
                f"find {destination} -type f -exec chmod 0600 {{}} +; "
                f"test \"$(find {destination} -mindepth 1 -maxdepth 1 -type f | wc -l)\" -eq 4",
            )
        )
        dump_names = [path.name for path in backup_directory.iterdir() if path.suffix == ".dump"]
        if len(dump_names) != 1:
            raise LabError("private restore handoff has no unique dump")
        return {
            "backup": f"{destination}/{dump_names[0]}",
            "baseline": inputs["baseline"],
            "target": target,
        }

    def restore_boundary(
        self,
        descriptor: Path,
        backup_directory: Path,
    ) -> dict[str, Any]:
        boundary = self._build_and_boot()
        staged = self.stage_restore_inputs(descriptor, backup_directory)
        completed, result = self.fixture(
            "prepare-restore",
            staged["target"]["commit"],
            staged["target"]["manifestSha256"],
            backup=staged["backup"],
            baseline_commit=staged["baseline"]["commit"],
            baseline_manifest_sha256=staged["baseline"]["manifestSha256"],
            timeout=1800,
            check=False,
        )
        if completed.returncode or result.get("status") != "PASS":
            raise LabError(f"isolated restore boundary failed: {result.get('error', result.get('status'))}")
        external = validate_container_inspect(self._container_inspect(), self.project, self.container)
        if external["ports"]:
            raise LabError("isolated restore environment published a host port")
        return {"boundary": boundary, "fixture": result["observation"], "staged": staged}

    def _updater_identity(self, unit: str, target_root: str) -> dict[str, Any]:
        deadline = time.monotonic() + 30
        pid = 0
        while time.monotonic() < deadline:
            result = self._exec(
                ("systemctl", "show", unit, "--property=MainPID", "--value"),
                timeout=10,
                check=False,
                record=False,
            )
            text_value = result.stdout.strip()
            if text_value.isdigit() and int(text_value) > 1:
                pid = int(text_value)
                break
            time.sleep(0.05)
        if pid <= 1:
            raise MissedInterruptWindow("updater process exited before identity observation")
        proc = f"/proc/{pid}"
        executable = self._exec(("readlink", "-f", f"{proc}/exe"), record=False).stdout.strip()
        fields = self._exec(("cat", f"{proc}/stat"), record=False).stdout.strip().split()
        cgroup = self._exec(("cat", f"{proc}/cgroup"), record=False).stdout.strip()
        command = self._exec(("cat", f"{proc}/cmdline"), record=False).stdout.replace("\x00", " ").strip()
        if len(fields) < 22 or not cgroup or target_root not in command \
                or "lkjmc-deploy-release" not in command or " update " not in f" {command} ":
            raise LabError("updater process identity differs")
        return {
            "cgroup": cgroup,
            "commandSha256": _sha256_bytes(command.encode()),
            "executable": executable,
            "pid": pid,
            "startTicks": int(fields[21]),
        }

    def _watch_prepared_fence(self, pid: int, timeout: int = 900) -> None:
        probe = (
            "import json,pathlib;"
            "f=pathlib.Path('/etc/lkjmc/deployment-fence.json');"
            "print('waiting' if not f.is_file() else "
            "json.loads((pathlib.Path(json.loads(f.read_text())['stateDirectory'])/'deployment.json').read_text()).get('result','unknown'))"
        )
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if not self._exec(("test", "-d", f"/proc/{pid}"), check=False, record=False).returncode == 0:
                raise MissedInterruptWindow("updater exited before the prepared fence was observed")
            result = self._exec(("python3", "-c", probe), check=False, record=False)
            state = result.stdout.strip()
            if state == "prepared":
                stopped = self._exec(("kill", "-STOP", str(pid)), check=False, record=False)
                if stopped.returncode:
                    raise MissedInterruptWindow("updater left the prepared window before SIGSTOP")
                return
            if state not in {"", "waiting"}:
                raise MissedInterruptWindow(f"updater passed the prepared window: {state[:64]}")
            time.sleep(0.05)
        raise MissedInterruptWindow("updater did not reach the prepared fence within the bound")

    def interruption_recovery(
        self,
        baseline: Mapping[str, Any],
        target: Mapping[str, Any],
        rollback: Mapping[str, Any],
        *,
        changed_ledger: bool,
    ) -> dict[str, Any]:
        baseline_commit = str(baseline["commit"])
        baseline_manifest = str(baseline["manifestSha256"])
        target_commit = str(target["commit"])
        target_manifest = str(target["manifestSha256"])
        _, before = self.fixture("fingerprint", baseline_commit, baseline_manifest)
        before_marker = before["observation"]["migrationMarker"]
        backup = f"/var/backups/lkjmc/{self.project}-interrupt/rollback.dump"
        snapshot = f"docker-{self.project}-{str(rollback['descriptorSha256'])[:16]}"
        root = "/var/lib/private/lkjmc-drr-target"
        unit = "lkjmc-drr-interrupted-update.service"
        launch = self._exec(
            (
                "systemd-run",
                "--unit",
                unit,
                "--collect",
                "--property=Type=exec",
                f"{root}/source/lkjmc-deploy-release",
                "update",
                "--release-root",
                root,
                "--manifest-sha256",
                target_manifest,
                "--from-commit",
                baseline_commit,
                "--backup",
                backup,
                "--rollback-snapshot",
                snapshot,
            ),
            timeout=30,
        )
        identity = self._updater_identity(unit, root)
        self._watch_prepared_fence(identity["pid"])
        _, frozen = self.fixture("deployment-state", target_commit, target_manifest)
        frozen_value = frozen["observation"]
        process_state = self._exec(
            ("python3", "-c", f"print(open('/proc/{identity['pid']}/stat').read().split()[2])"),
            record=False,
        ).stdout.strip()
        if process_state not in {"T", "t"} or frozen_value["state"].get("result") != "prepared" \
                or frozen_value["migrationMarker"] != before_marker:
            self._exec(("kill", "-CONT", str(identity["pid"])), check=False, record=False)
            raise MissedInterruptWindow("frozen updater predicates were not simultaneously accepted")
        killed = self._exec(("kill", "-KILL", str(identity["pid"])), check=False, record=False)
        if killed.returncode:
            raise LabError("exact frozen updater process could not be killed")
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            if self._exec(("test", "-d", f"/proc/{identity['pid']}"), check=False, record=False).returncode:
                break
            time.sleep(0.05)
        else:
            raise LabError("killed updater process remained observable")
        self._exec(("systemctl", "stop", SERVICE), timeout=180, check=False)
        ordinary = self._exec(("systemctl", "start", SERVICE), timeout=120, check=False)
        if ordinary.returncode == 0:
            raise LabError("ordinary systemd start bypassed the durable fence")
        _, ordinary_state = self.fixture("deployment-state", target_commit, target_manifest)
        if ordinary_state["observation"]["processes"]:
            raise LabError("fenced ordinary start left a service-user process")

        container_before = validate_container_inspect(self._container_inspect(), self.project, self.container)
        self.run(("restart", "--timeout", "60", self.container), timeout=150)
        system_state = self._wait_systemd(180)
        container_after = validate_container_inspect(self._container_inspect(), self.project, self.container)
        _, restart_state = self.fixture("deployment-state", target_commit, target_manifest)
        if restart_state["observation"]["processes"] \
                or container_before["startedAt"] == container_after["startedAt"]:
            raise LabError("Docker restart bypassed the durable deployment fence")

        ledger_mutation = None
        if changed_ledger:
            mutation = self._exec(
                (
                    "runuser",
                    "-u",
                    "postgres",
                    "--",
                    "psql",
                    "-d",
                    "lkjmc",
                    "-X",
                    "--quiet",
                    "-v",
                    "ON_ERROR_STOP=1",
                    "-c",
                    "insert into schema_migrations(version,name,checksum) values "
                    "(999999,'docker-lab-changed-ledger','0000000000000000000000000000000000000000000000000000000000000000')",
                )
            )
            ledger_mutation = {"exit": mutation.returncode, "kind": "disposable-negative-migration-marker"}
            recovery = self.deployer(
                target,
                "recover",
                ("--to-commit", target_commit),
                timeout=300,
                check=False,
            )
            if recovery.returncode == 0 or "migration ledger changed or is unreadable" not in recovery.stderr \
                    or backup not in recovery.stderr or snapshot not in recovery.stderr:
                raise LabError("changed-ledger recovery did not refuse binary rollback exactly")
            _, refused = self.fixture("deployment-state", target_commit, target_manifest)
            refused_value = refused["observation"]
            if refused_value["state"].get("result") != "restore-required" \
                    or refused_value["systemd"].get("ActiveState") == "active" \
                    or refused_value["processes"] or refused_value["migrationMarker"] == before_marker:
                raise LabError("changed-ledger refusal terminal state differs")
            return {
                "changedLedger": ledger_mutation,
                "containerRestart": {"after": container_after, "before": container_before, "systemState": system_state},
                "frozenState": frozen_value,
                "ordinaryStartExit": ordinary.returncode,
                "recoveryExit": recovery.returncode,
                "refusedState": refused_value,
                "updaterIdentity": identity,
                "watcherLaunchExit": launch.returncode,
            }

        recovery = self.deployer(
            target,
            "recover",
            ("--to-commit", target_commit),
            timeout=1800,
        )
        try:
            receipt = json.loads(recovery.stdout)
        except (TypeError, ValueError) as error:
            raise LabError("packaged recovery returned invalid JSON") from error
        if receipt != {"schemaVersion": 1, "result": "recovered", "commit": baseline_commit}:
            raise LabError("packaged recovery receipt differs")
        _, recovered = self.fixture(
            "observe",
            baseline_commit,
            baseline_manifest,
            require_startup_evidence=True,
            timeout=1800,
        )
        retained = self._exec(
            (
                "python3",
                "-c",
                "import json,pathlib; p=pathlib.Path('/var/lib/private/lkjmc-deployments')/"
                f"'{target_commit}'/'deployment.json'; print(json.loads(p.read_text())['result'])",
            )
        ).stdout.strip()
        if retained != "recovered-verified" \
                or self._exec(("test", "-d", f"/opt/lkjmc/releases/{target_commit}"), check=False).returncode:
            raise LabError("recovery retained diagnostic state differs")
        return {
            "containerRestart": {"after": container_after, "before": container_before, "systemState": system_state},
            "frozenState": frozen_value,
            "ordinaryStartExit": ordinary.returncode,
            "receipt": receipt,
            "recovered": recovered["observation"],
            "retainedDeploymentResult": retained,
            "updaterIdentity": identity,
            "watcherLaunchExit": launch.returncode,
        }

    def systemd_probe(self) -> dict[str, Any]:
        preflight = self.preflight()
        model = json.loads(self.compose(("config", "--format", "json")).stdout)
        compose_boundary = validate_compose_model(model, self.project)
        self.compose(("build", "--pull", "host"), timeout=1800)
        images = self.owned_objects()["images"]
        if len(images) != 1:
            raise LabError("the project-owned runtime image closure differs")
        image_values = self.json(("image", "inspect", images[0]))
        if not isinstance(image_values, list) or len(image_values) != 1:
            raise LabError("runtime image inspection differs")
        verify_owned(image_values[0], self.project, "images")
        if (image_values[0].get("Config") or {}).get("ExposedPorts"):
            raise LabError("runtime image unexpectedly exposes a port")
        cgroup_probe = self.compose(
            (
                "run",
                "--rm",
                "--no-deps",
                "--no-TTY",
                "--entrypoint",
                "/bin/sh",
                "host",
                "-ec",
                "findmnt -n -o FSTYPE,OPTIONS /sys/fs/cgroup; "
                "mount -t cgroup2 -o nosuid,nodev,noexec cgroup2 /sys/fs/cgroup; "
                "findmnt -n -o FSTYPE,OPTIONS /sys/fs/cgroup; "
                "mkdir /sys/fs/cgroup/lkjmc-drr-write-probe; "
                "rmdir /sys/fs/cgroup/lkjmc-drr-write-probe",
            ),
            timeout=60,
            check=False,
        )
        if cgroup_probe.returncode:
            detail = (cgroup_probe.stdout + "\n" + cgroup_probe.stderr).strip().splitlines()
            suffix = " | ".join(line[:512] for line in detail[-10:]) if detail else "no cgroup diagnostic"
            raise Blocked(f"private container cgroup is not writable for real systemd: {suffix}")
        self.compose(("up", "--detach", "--no-build", "host"), timeout=180)
        system_state = self._wait_systemd()
        first_inspect_raw = self._container_inspect()
        first_container = validate_container_inspect(first_inspect_raw, self.project, self.container)
        first_process = self._process_observation()
        network = self._network_observation()
        host_cgroup = Path(f"/proc/{first_container['hostPid']}/cgroup").read_text().strip()
        if not host_cgroup:
            raise LabError("host-side cgroup observation is empty")
        self.run(("restart", "--timeout", "45", self.container), timeout=120)
        restarted_state = self._wait_systemd()
        second_inspect_raw = self._container_inspect()
        second_container = validate_container_inspect(second_inspect_raw, self.project, self.container)
        second_process = self._process_observation()
        if first_container["hostPid"] == second_container["hostPid"] \
                or first_container["startedAt"] == second_container["startedAt"]:
            raise LabError("Docker restart did not replace the container start identity")
        if first_process["initStartTicks"] == second_process["initStartTicks"]:
            raise LabError("Docker restart did not replace the PID 1 start identity")
        if first_process["mainStartTicks"] == second_process["mainStartTicks"]:
            raise LabError("Docker restart did not replace the systemd-owned process identity")
        self.run(("stop", "--time", "45", self.container), timeout=120)
        stopped = self._container_inspect().get("State") or {}
        if stopped.get("Running") is not False or int(stopped.get("ExitCode", -1)) != 0:
            raise LabError("systemd container did not stop gracefully")
        return {
            "composeBoundary": compose_boundary,
            "firstContainer": first_container,
            "firstProcess": first_process,
            "hostCgroup": host_cgroup,
            "image": {
                "id": image_values[0].get("Id"),
                "labels": (image_values[0].get("Config") or {}).get("Labels"),
                "repoDigests": image_values[0].get("RepoDigests") or [],
                "size": image_values[0].get("Size"),
            },
            "privateCgroupMount": cgroup_probe.stdout.strip(),
            "network": network,
            "preflight": preflight,
            "restartSystemState": restarted_state,
            "secondContainer": second_container,
            "secondProcess": second_process,
            "stopped": {"exitCode": stopped.get("ExitCode"), "running": stopped.get("Running")},
            "systemState": system_state,
        }

    def cleanup(self) -> dict[str, Any]:
        failures: list[str] = []
        self.compose(("down", "--volumes", "--remove-orphans", "--rmi", "local"), timeout=180, check=False)
        for kind, identifiers in self.owned_objects().items():
            for identifier in identifiers:
                inspect_kind = "container" if kind == "containers" else kind[:-1]
                values = self.json((inspect_kind, "inspect", identifier))
                if not isinstance(values, list) or len(values) != 1:
                    failures.append(f"{kind}:{identifier[:12]}:inspect")
                    continue
                verify_owned(values[0], self.project, kind)
                removal = {
                    "containers": ("container", "rm", "--force", identifier),
                    "networks": ("network", "rm", identifier),
                    "volumes": ("volume", "rm", identifier),
                    "images": ("image", "rm", identifier),
                }[kind]
                if self.run(removal, check=False).returncode:
                    failures.append(f"{kind}:{identifier[:12]}:remove")
        residual = {kind: values for kind, values in self.owned_objects().items() if values}
        status = "PASS" if not failures and not residual else "FAILED"
        return {"failures": failures, "residual": residual, "status": status}


def execute(
    mode: str,
    project: str,
    *,
    input_descriptor: Path | None = None,
) -> tuple[int, dict[str, Any]]:
    lab = DockerLab(project, matrix_resources=mode == "preflight")
    result: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "mode": mode,
        "project": project,
        "startedAtUnix": int(time.time()),
        "status": "FAILED",
    }
    code = 1
    try:
        if mode == "preflight":
            result["observation"] = lab.preflight()
        elif mode == "systemd-probe":
            result["observation"] = lab.systemd_probe()
        elif mode == "fixture-consent-gate":
            if input_descriptor is None:
                raise LabError("fixture consent gate requires an input descriptor")
            result["observation"] = lab.consent_gate(input_descriptor)
        else:
            raise LabError(f"unsupported lab mode: {mode}")
        result["status"] = "PASS"
        code = 0
    except Blocked as error:
        result["status"] = "BLOCKED"
        result["error"] = str(error)
        code = 2
    except LabError as error:
        result["error"] = str(error)
    finally:
        cleanup = lab.cleanup()
        result["cleanup"] = cleanup
        result["commands"] = lab.commands
        result["finishedAtUnix"] = int(time.time())
        if cleanup["status"] != "PASS":
            result["status"] = "FAILED"
            code = 1
    return code, result


def _project_variant(base: str, label: str, attempt: int | None = None) -> str:
    suffix = f"-{label}{attempt if attempt is not None else ''}"
    maximum_base = 48 - len(suffix)
    value = f"{base[:maximum_base].rstrip('-')}{suffix}"
    return validate_project(value)


def execute_full_matrix(
    descriptor: Path,
    project: str,
    work_root: Path,
    *,
    accept_minecraft_eula: bool,
) -> tuple[int, dict[str, Any]]:
    project = validate_project(project)
    descriptor_observation = validate_input_descriptor(
        descriptor,
        require_target=True,
        require_consent=not accept_minecraft_eula,
    )
    if not accept_minecraft_eula and descriptor_observation["minecraftEulaAccepted"] is not True:
        raise Blocked("explicit Minecraft EULA acceptance is absent")
    baseline = descriptor_observation["baseline"]
    target = descriptor_observation["target"]
    if target is None:
        raise Blocked("exact target release input is absent")
    work_root = work_root.absolute()
    if work_root.parent == work_root or os.path.lexists(work_root) \
            or not work_root.name.startswith(f".{project}-work"):
        raise LabError("private full-matrix work root is unsafe or already exists")
    parent = work_root.parent.lstat()
    if not stat.S_ISDIR(parent.st_mode) or work_root.parent.is_symlink() \
            or parent.st_uid != os.getuid() or stat.S_IMODE(parent.st_mode) & 0o077:
        raise LabError("private full-matrix work parent differs")
    work_root.mkdir(mode=0o700)
    result: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "mode": "full-matrix",
        "project": project,
        "startedAtUnix": int(time.time()),
        "status": "FAILED",
        "inputs": descriptor_observation,
        "minecraftEulaAcceptanceSupplied": True,
        "scenarios": {},
        "cleanup": {},
    }
    source = DockerLab(_project_variant(project, "update"), matrix_resources=True)
    restore: DockerLab | None = None
    active_labs: list[DockerLab] = [source]
    code = 1
    try:
        source_baseline = source.prepare_baseline(descriptor, accept_minecraft_eula=True)
        target_input = source.stage_target_input(descriptor)
        rollback = source.create_rollback_point(baseline["commit"], baseline["manifestSha256"])
        update = source.update_restart_matrix(baseline, target_input, rollback)
        result["scenarios"]["update"] = {
            "baseline": source_baseline,
            "rollbackPoint": rollback,
            "transitions": update,
        }
        handoff = work_root / "updater-backup"
        copied = source.copy_backup_out(
            update["backup"]["path"],
            handoff,
            update["backup"]["verification"],
        )
        _, source_before_restore = source.fixture(
            "fingerprint",
            target["commit"],
            target["manifestSha256"],
        )
        restore = DockerLab(_project_variant(project, "restore"))
        active_labs.append(restore)
        restored = restore.restore_boundary(descriptor, handoff)
        _, source_after_restore = source.fixture(
            "fingerprint",
            target["commit"],
            target["manifestSha256"],
        )
        if source_before_restore["observation"] != source_after_restore["observation"]:
            raise LabError("isolated restore changed the updated source environment")
        _, source_running = source.fixture(
            "observe",
            target["commit"],
            target["manifestSha256"],
            require_startup_evidence=True,
            timeout=1800,
        )
        result["scenarios"]["restore"] = {
            "backupHandoff": copied,
            "restored": restored,
            "sourceFingerprintSha256": _sha256_json(source_before_restore["observation"]),
            "sourceRemainedAccepted": source_running["observation"],
        }
        restore_cleanup = restore.cleanup()
        result["cleanup"][restore.project] = restore_cleanup
        result["commandsRestore"] = restore.commands
        active_labs.remove(restore)
        restore = None
        if restore_cleanup["status"] != "PASS":
            raise LabError("isolated restore cleanup failed")
        source.run(("stop", "--time", "60", source.container), timeout=120)

        def recovery_scenario(label: str, changed: bool) -> tuple[dict[str, Any], list[dict[str, Any]]]:
            invalid = []
            for attempt in range(1, 4):
                lab = DockerLab(_project_variant(project, label, attempt), matrix_resources=True)
                active_labs.append(lab)
                try:
                    baseline_result = lab.prepare_baseline(descriptor, accept_minecraft_eula=True)
                    scenario_target = lab.stage_target_input(descriptor)
                    scenario_rollback = lab.create_rollback_point(
                        baseline["commit"],
                        baseline["manifestSha256"],
                    )
                    transitions = lab.interruption_recovery(
                        baseline,
                        scenario_target,
                        scenario_rollback,
                        changed_ledger=changed,
                    )
                    scenario = {
                        "attempt": attempt,
                        "baseline": baseline_result,
                        "rollbackPoint": scenario_rollback,
                        "transitions": transitions,
                    }
                except MissedInterruptWindow as error:
                    invalid.append({"attempt": attempt, "reason": str(error), "status": "INVALID"})
                    scenario = None
                finally:
                    cleanup = lab.cleanup()
                    result["cleanup"][lab.project] = cleanup
                    invalid_commands = list(lab.commands)
                    active_labs.remove(lab)
                if cleanup["status"] != "PASS":
                    raise LabError(f"{label} attempt cleanup failed")
                if scenario is not None:
                    return scenario, invalid + [{"commands": invalid_commands, "status": "PASS"}]
                invalid[-1]["commands"] = invalid_commands
            raise LabError(f"{label} missed the unmodified updater window in three fresh projects")

        recovery, recovery_attempts = recovery_scenario("recover", False)
        result["scenarios"]["interruptionRecovery"] = recovery
        result["scenarios"]["interruptionRecoveryAttempts"] = recovery_attempts
        refusal, refusal_attempts = recovery_scenario("ledger", True)
        result["scenarios"]["changedLedgerRefusal"] = refusal
        result["scenarios"]["changedLedgerAttempts"] = refusal_attempts
        shutil.rmtree(handoff)
        if handoff.exists():
            raise LabError("private backup handoff cleanup failed")
        result["privateBackupHandoffCleanup"] = "PASS"
        result["status"] = "PASS"
        code = 0
    except Blocked as error:
        result["status"] = "BLOCKED"
        result["error"] = str(error)
        code = 2
    except (LabError, OSError, ValueError) as error:
        result["error"] = str(error)
    finally:
        for lab in list(reversed(active_labs)):
            cleanup = lab.cleanup()
            result["cleanup"][lab.project] = cleanup
            if lab is source:
                result["commandsUpdate"] = lab.commands
            else:
                result.setdefault("commandsResidual", {})[lab.project] = lab.commands
            if cleanup["status"] != "PASS":
                result["status"] = "FAILED"
                code = 1
        if work_root.exists():
            expected_children = {"updater-backup"}
            observed = {path.name for path in work_root.iterdir()}
            if not observed.issubset(expected_children):
                result["status"] = "FAILED"
                result["workCleanupError"] = f"unexpected private work children: {sorted(observed)}"
                code = 1
            else:
                shutil.rmtree(work_root)
        result["workRootCleanup"] = "PASS" if not work_root.exists() else "FAILED"
        result["finishedAtUnix"] = int(time.time())
        if any(value.get("status") != "PASS" for value in result["cleanup"].values()):
            result["status"] = "FAILED"
            code = 1
    return code, result
