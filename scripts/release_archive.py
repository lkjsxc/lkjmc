#!/usr/bin/env python3
"""Pack, verify, and safely extract the canonical release handoff archive."""
from __future__ import annotations

import argparse
import ctypes
import errno
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import secrets
import shutil
import signal
import stat
import subprocess
import sys
import tarfile
from typing import Any

from fd_tree import Entry, Limits, same, visit_file, walk
from release_inventory import (
    ROOT,
    commit,
    expected,
    release_contract,
    workspace_package_value,
)


ARCHIVE_FORMAT = "posix-ustar-uncompressed"
DESCRIPTOR_NAME = "release-handoff.json"
TRANSPORT_STATEMENT = (
    "The GitHub Actions artifact is transport only; the inner archive is the canonical payload."
)
DESCRIPTOR_FIELDS = {
    "archiveFilename",
    "archiveFormat",
    "archiveSha256",
    "archiveSize",
    "outerArtifactName",
    "producerJob",
    "productVersion",
    "releaseManifestSha256",
    "releaseManifestSidecarSha256",
    "repository",
    "schemaVersion",
    "sourceCommit",
    "topLevelDirectory",
    "transportStatement",
    "workflowEvent",
    "workflowRef",
    "workflowRunAttempt",
    "workflowRunId",
}
RECEIPT_FIELDS = {
    "archiveFilename",
    "archiveSha256",
    "artifactId",
    "artifactServiceDigest",
    "identityVerifier",
    "manifestVerifier",
    "outerArtifactName",
    "releaseManifestSha256",
    "schemaVersion",
    "sourceCommit",
    "status",
    "workflowRunAttempt",
    "workflowRunId",
}
TREE_LIMITS = Limits(
    max_entries=128,
    max_files=64,
    max_bytes=256 * 1024 * 1024,
    max_file_bytes=128 * 1024 * 1024,
    max_depth=4,
)
OUTER_LIMITS = Limits(
    max_entries=3,
    max_files=3,
    max_bytes=TREE_LIMITS.max_bytes + 2 * 1024 * 1024,
    max_file_bytes=TREE_LIMITS.max_bytes + 1024 * 1024,
    max_depth=1,
)
MAX_DESCRIPTOR_BYTES = 32 * 1024
MAX_SIDECAR_BYTES = 256
MAX_PATH_BYTES = 100
BLOCK = 512
HEX40 = re.compile(r"[0-9a-f]{40}")
HEX64 = re.compile(r"[0-9a-f]{64}")
SAFE_VERSION = re.compile(r"[A-Za-z0-9][A-Za-z0-9.-]{0,63}")
SAFE_REPOSITORY = re.compile(r"[A-Za-z0-9_.-]{1,100}/[A-Za-z0-9_.-]{1,100}")
SAFE_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,255}")
SAFE_EVENT = re.compile(r"[A-Za-z0-9_]{1,64}")
SAFE_JOB = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]{0,63}")
LIBC = ctypes.CDLL(None, use_errno=True)
RENAME_NOREPLACE = 1


def fail(message: str) -> None:
    raise RuntimeError(message)


def canonical_json(value: Any, *, pretty: bool = False) -> bytes:
    if pretty:
        text = json.dumps(value, indent=2, sort_keys=True)
    else:
        text = json.dumps(value, separators=(",", ":"), sort_keys=True)
    return (text + "\n").encode("utf-8")


def strict_json(raw: bytes, label: str, *, pretty: bool = False) -> Any:
    def pairs(values: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for name, value in values:
            if name in result:
                fail(f"duplicate JSON field in {label}: {name}")
            result[name] = value
        return result

    def constant(value: str) -> None:
        fail(f"non-finite JSON value in {label}: {value}")

    try:
        value = json.loads(
            raw.decode("utf-8", "strict"),
            object_pairs_hook=pairs,
            parse_constant=constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"invalid {label}: {error}")
    if raw != canonical_json(value, pretty=pretty):
        fail(f"noncanonical {label}")
    return value


def read_all(descriptor: int, size: int, limit: int, label: str) -> bytes:
    if size > limit:
        fail(f"oversized {label}")
    os.lseek(descriptor, 0, os.SEEK_SET)
    value = bytearray()
    while len(value) < size:
        block = os.read(descriptor, min(1024 * 1024, size - len(value)))
        if not block:
            fail(f"truncated {label}")
        value.extend(block)
    if os.read(descriptor, 1):
        fail(f"growing {label}")
    return bytes(value)


def digest_descriptor(descriptor: int, size: int) -> str:
    os.lseek(descriptor, 0, os.SEEK_SET)
    digest = hashlib.sha256()
    remaining = size
    while remaining:
        block = os.read(descriptor, min(1024 * 1024, remaining))
        if not block:
            fail("file changed size while hashing")
        digest.update(block)
        remaining -= len(block)
    if os.read(descriptor, 1):
        fail("file grew while hashing")
    return digest.hexdigest()


def strict_sidecar(raw: bytes, filename: str, expected_digest: str) -> None:
    try:
        value = raw.decode("ascii", "strict")
    except UnicodeDecodeError:
        fail("non-ASCII checksum sidecar")
    match = re.fullmatch(rf"([0-9a-f]{{64}})  {re.escape(filename)}\n", value)
    if match is None or match.group(1) != expected_digest:
        fail(f"checksum sidecar differs for {filename}")


def canonical_relative_path(value: str) -> tuple[str, ...]:
    try:
        encoded = value.encode("utf-8", "strict")
    except UnicodeEncodeError:
        fail("non-UTF-8 archive member")
    if not encoded or len(encoded) > MAX_PATH_BYTES:
        fail("empty or overlong archive member")
    if value.startswith("/") or value.endswith("/") or "\\" in value or "\x00" in value:
        fail(f"unsafe archive member: {value!r}")
    parts = tuple(value.split("/"))
    if any(part in ("", ".", "..") for part in parts):
        fail(f"noncanonical archive member: {value!r}")
    if str(PurePosixPath(*parts)) != value:
        fail(f"noncanonical archive member: {value!r}")
    return parts


def canonical_header(name: str, kind: bytes, mode: int, size: int) -> bytes:
    canonical_relative_path(name)
    if kind not in (tarfile.DIRTYPE, tarfile.REGTYPE):
        fail("unsupported canonical archive member type")
    if kind == tarfile.DIRTYPE and size != 0:
        fail("directory archive member has data")
    item = tarfile.TarInfo(name)
    item.mode = mode
    item.uid = 0
    item.gid = 0
    item.size = size
    item.mtime = 0
    item.type = kind
    item.linkname = ""
    item.uname = ""
    item.gname = ""
    item.devmajor = 0
    item.devminor = 0
    item.pax_headers = {}
    try:
        value = item.tobuf(tarfile.USTAR_FORMAT, encoding="utf-8", errors="strict")
    except (UnicodeError, ValueError) as error:
        fail(f"archive member cannot be represented as ustar: {error}")
    if len(value) != BLOCK or any(value[345:500]):
        fail("archive member requires a ustar prefix or extension")
    return value


def expected_release_modes() -> dict[str, tuple[str, int]]:
    values: dict[str, tuple[str, int]] = {
        ".": ("directory", 0o700),
        "artifact-manifest.json": ("file", 0o600),
        "artifact-manifest.json.sha256": ("file", 0o600),
        "source": ("directory", 0o700),
    }
    for item in release_contract():
        mode = 0o700 if item["kind"] == "binary" else 0o600
        values[f"source/{item['destination']}"] = ("file", mode)
    return values


def snapshot_release(root: Path) -> dict[str, dict[str, Any]]:
    values: dict[str, dict[str, Any]] = {
        ".": {"kind": "directory", "mode": 0o700, "size": 0, "sha256": None}
    }

    def directory(_descriptor: int, entry: Entry) -> None:
        canonical_relative_path(entry.path)
        values[entry.path] = {
            "kind": "directory",
            "mode": entry.mode,
            "size": 0,
            "sha256": None,
        }

    def regular(descriptor: int, entry: Entry) -> None:
        canonical_relative_path(entry.path)
        values[entry.path] = {
            "kind": "file",
            "mode": entry.mode,
            "size": entry.size,
            "sha256": digest_descriptor(descriptor, entry.size),
        }

    walk(root, regular, TREE_LIMITS, visit_directory=directory)
    expected_modes = expected_release_modes()
    observed_modes = {path: (value["kind"], value["mode"]) for path, value in values.items()}
    if observed_modes != expected_modes:
        differences = sorted(set(observed_modes) | set(expected_modes), key=os.fsencode)
        shown = ", ".join(
            path for path in differences if observed_modes.get(path) != expected_modes.get(path)
        )
        fail(f"release path/type/mode closure differs: {shown[:1024]}")
    return values


def validate_manifest_snapshot(
    root: Path, snapshot: dict[str, dict[str, Any]], source_commit: str
) -> tuple[bytes, bytes, dict[str, Any]]:
    manifest_path = root / "artifact-manifest.json"
    sidecar_path = root / "artifact-manifest.json.sha256"
    manifest_parts: list[bytes] = []
    sidecar_parts: list[bytes] = []
    visit_file(
        manifest_path,
        lambda fd, item: manifest_parts.append(
            read_all(fd, item.size, 1024 * 1024, "release manifest")
        ),
        TREE_LIMITS,
    )
    visit_file(
        sidecar_path,
        lambda fd, item: sidecar_parts.append(
            read_all(fd, item.size, MAX_SIDECAR_BYTES, "release manifest sidecar")
        ),
        TREE_LIMITS,
    )
    manifest = manifest_parts[0]
    sidecar = sidecar_parts[0]
    expected_manifest = expected(root, source_commit)
    expected_payload = canonical_json(expected_manifest, pretty=True)
    if manifest != expected_payload:
        fail("release manifest differs from independently derived closure")
    manifest_digest = hashlib.sha256(manifest).hexdigest()
    strict_sidecar(sidecar, "artifact-manifest.json", manifest_digest)
    if snapshot["artifact-manifest.json"]["sha256"] != manifest_digest:
        fail("release manifest changed after snapshot")
    if snapshot["artifact-manifest.json.sha256"]["sha256"] != hashlib.sha256(sidecar).hexdigest():
        fail("release manifest sidecar changed after snapshot")
    for item in expected_manifest["artifacts"]:
        observed = snapshot[f"source/{item['path']}"]
        if observed["size"] != item["size"] or observed["sha256"] != item["sha256"]:
            fail(f"release artifact differs from manifest: {item['path']}")
    return manifest, sidecar, expected_manifest


def validate_common(value: dict[str, Any]) -> None:
    if not SAFE_REPOSITORY.fullmatch(value["repository"]):
        fail("invalid repository identity")
    if not HEX40.fullmatch(value["sourceCommit"]):
        fail("invalid source commit")
    if not SAFE_VERSION.fullmatch(value["productVersion"]):
        fail("invalid canonical product version")
    if not SAFE_NAME.fullmatch(value["outerArtifactName"]):
        fail("invalid outer artifact name")
    if not SAFE_EVENT.fullmatch(value["workflowEvent"]):
        fail("invalid workflow event")
    ref = value["workflowRef"]
    if not isinstance(ref, str) or not ref.startswith("refs/") or len(ref) > 256 or any(
        character.isspace() or ord(character) < 0x20 for character in ref
    ):
        fail("invalid workflow ref")
    run_id = value["workflowRunId"]
    if not isinstance(run_id, str) or not re.fullmatch(r"[1-9][0-9]{0,19}", run_id):
        fail("invalid workflow run ID")
    attempt = value["workflowRunAttempt"]
    if isinstance(attempt, bool) or not isinstance(attempt, int) or not 1 <= attempt <= 1000:
        fail("invalid workflow run attempt")
    if not SAFE_JOB.fullmatch(value["producerJob"]):
        fail("invalid producer job")
    expected_artifact = canonical_artifact_name(value["sourceCommit"], run_id, attempt)
    if value["outerArtifactName"] != expected_artifact:
        fail("outer artifact name is not canonical for commit/run/attempt")


def canonical_artifact_name(source_commit: str, run_id: str, attempt: int) -> str:
    return f"lkjmc-release-{source_commit}-run-{run_id}-attempt-{attempt}"


def current_common(args: argparse.Namespace) -> dict[str, Any]:
    value = {
        "repository": args.repository,
        "sourceCommit": commit(),
        "productVersion": workspace_package_value("version"),
        "outerArtifactName": args.outer_artifact_name,
        "workflowEvent": args.workflow_event,
        "workflowRef": args.workflow_ref,
        "workflowRunId": args.workflow_run_id,
        "workflowRunAttempt": args.workflow_run_attempt,
        "producerJob": args.producer_job,
    }
    validate_common(value)
    return value


def descriptor_for(
    common: dict[str, Any], archive_size: int, archive_digest: str,
    manifest_digest: str, manifest_sidecar_digest: str,
) -> dict[str, Any]:
    top = f"lkjmc-{common['productVersion']}-{common['sourceCommit']}"
    archive = f"{top}.tar"
    value = dict(common)
    value.update(
        {
            "schemaVersion": 1,
            "archiveFormat": ARCHIVE_FORMAT,
            "archiveFilename": archive,
            "archiveSize": archive_size,
            "archiveSha256": archive_digest,
            "releaseManifestSha256": manifest_digest,
            "releaseManifestSidecarSha256": manifest_sidecar_digest,
            "topLevelDirectory": top,
            "transportStatement": TRANSPORT_STATEMENT,
        }
    )
    return value


def validate_descriptor(raw: bytes, common: dict[str, Any]) -> dict[str, Any]:
    value = strict_json(raw, "handoff descriptor")
    if not isinstance(value, dict) or set(value) != DESCRIPTOR_FIELDS:
        fail("handoff descriptor fields differ")
    if type(value.get("schemaVersion")) is not int or value["schemaVersion"] != 1:
        fail("unsupported handoff descriptor schema")
    for name, expected_value in common.items():
        if value.get(name) != expected_value:
            fail(f"handoff descriptor {name} differs")
    validate_common(common)
    top = f"lkjmc-{common['productVersion']}-{common['sourceCommit']}"
    if value.get("topLevelDirectory") != top or value.get("archiveFilename") != f"{top}.tar":
        fail("handoff descriptor archive name differs")
    if value.get("archiveFormat") != ARCHIVE_FORMAT:
        fail("handoff descriptor archive format differs")
    if value.get("transportStatement") != TRANSPORT_STATEMENT:
        fail("handoff descriptor transport statement differs")
    for name in ("archiveSha256", "releaseManifestSha256", "releaseManifestSidecarSha256"):
        if not isinstance(value.get(name), str) or not HEX64.fullmatch(value[name]):
            fail(f"invalid handoff descriptor digest: {name}")
    size = value.get("archiveSize")
    if isinstance(size, bool) or not isinstance(size, int) or not 1024 <= size <= OUTER_LIMITS.max_file_bytes:
        fail("invalid handoff descriptor archive size")
    return value


def private_parent(path: Path) -> tuple[int, os.stat_result]:
    try:
        before = os.lstat(path)
    except OSError as error:
        fail(f"unstatable private parent: {error}")
    if not stat.S_ISDIR(before.st_mode) or stat.S_ISLNK(before.st_mode):
        fail("private parent is not a no-follow directory")
    if stat.S_IMODE(before.st_mode) != 0o700:
        fail("private parent mode must be 0700")
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        current = os.fstat(descriptor)
        same(before, current, "private parent")
    except Exception:
        os.close(descriptor)
        raise
    return descriptor, current


def inode_identity(value: os.stat_result) -> tuple[int, int]:
    return value.st_dev, value.st_ino


def same_inode(before: os.stat_result, after: os.stat_result, label: str) -> None:
    if inode_identity(before) != inode_identity(after) or stat.S_IFMT(before.st_mode) != stat.S_IFMT(after.st_mode):
        fail(f"identity changed for {label}")


def create_stage(parent: Path, label: str) -> tuple[Path, tuple[int, int], int, os.stat_result]:
    if not SAFE_NAME.fullmatch(label):
        fail("unsafe publication target name")
    parent_fd, parent_stat = private_parent(parent)
    for _ in range(32):
        name = f".{label}.tmp-{secrets.token_hex(16)}"
        try:
            os.mkdir(name, 0o700, dir_fd=parent_fd)
        except FileExistsError:
            continue
        stage = parent / name
        metadata = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
        return stage, inode_identity(metadata), parent_fd, parent_stat
    os.close(parent_fd)
    fail("cannot allocate private staging directory")


def remove_owned_directory(path: Path, owned_identity: tuple[int, int]) -> None:
    try:
        current = os.lstat(path)
    except FileNotFoundError:
        return
    if not stat.S_ISDIR(current.st_mode) or stat.S_ISLNK(current.st_mode):
        fail("refusing cleanup of replaced staging directory")
    if inode_identity(current) != owned_identity:
        fail("refusing cleanup of replaced staging directory")
    shutil.rmtree(path)
    if os.path.lexists(path):
        fail("staging directory cleanup incomplete")


def rename_noreplace(parent_fd: int, source_name: str, target_name: str) -> None:
    function = getattr(LIBC, "renameat2", None)
    if function is None:
        fail("atomic no-replace rename is unavailable")
    result = function(
        parent_fd,
        ctypes.c_char_p(os.fsencode(source_name)),
        parent_fd,
        ctypes.c_char_p(os.fsencode(target_name)),
        RENAME_NOREPLACE,
    )
    if result != 0:
        error = ctypes.get_errno()
        if error == errno.EEXIST:
            fail("refusing existing publication target")
        raise OSError(error, os.strerror(error), target_name)


def write_all(descriptor: int, value: bytes | memoryview) -> None:
    view = memoryview(value)
    while view:
        written = os.write(descriptor, view)
        if written <= 0:
            fail("short filesystem write")
        view = view[written:]


def write_private(root_fd: int, name: str, value: bytes) -> None:
    descriptor = os.open(
        name,
        os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW,
        0o600,
        dir_fd=root_fd,
    )
    try:
        write_all(descriptor, value)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_archive(
    stage_fd: int, archive_name: str, release_root: Path,
    top: str, snapshot: dict[str, dict[str, Any]],
) -> None:
    descriptor = os.open(
        archive_name,
        os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW,
        0o600,
        dir_fd=stage_fd,
    )
    visited: set[str] = set()
    last_name: list[bytes] = []

    def emit_header(name: str, kind: bytes, mode: int, size: int) -> None:
        encoded = name.encode("utf-8", "strict")
        if last_name and encoded <= last_name[0]:
            fail("archive member order is not bytewise canonical")
        last_name[:] = [encoded]
        write_all(descriptor, canonical_header(name, kind, mode, size))

    def directory(_source_fd: int, entry: Entry) -> None:
        name = f"{top}/{entry.path}"
        emit_header(name, tarfile.DIRTYPE, entry.mode, 0)
        visited.add(entry.path)

    def regular(source_fd: int, entry: Entry) -> None:
        name = f"{top}/{entry.path}"
        emit_header(name, tarfile.REGTYPE, entry.mode, entry.size)
        digest = hashlib.sha256()
        remaining = entry.size
        while remaining:
            block = os.read(source_fd, min(1024 * 1024, remaining))
            if not block:
                fail(f"release file truncated during packing: {entry.path}")
            digest.update(block)
            write_all(descriptor, block)
            remaining -= len(block)
        if os.read(source_fd, 1):
            fail(f"release file grew during packing: {entry.path}")
        padding = (-entry.size) % BLOCK
        if padding:
            write_all(descriptor, bytes(padding))
        if digest.hexdigest() != snapshot[entry.path]["sha256"]:
            fail(f"release file changed between validation and packing: {entry.path}")
        visited.add(entry.path)

    try:
        emit_header(top, tarfile.DIRTYPE, 0o700, 0)
        walk(release_root, regular, TREE_LIMITS, visit_directory=directory)
        if visited != set(snapshot) - {"."}:
            fail("release closure changed while packing")
        write_all(descriptor, bytes(2 * BLOCK))
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def hash_private_file(root: Path, name: str) -> tuple[int, str]:
    values: list[tuple[int, str]] = []
    visit_file(
        root / name,
        lambda fd, item: values.append((item.size, digest_descriptor(fd, item.size))),
        OUTER_LIMITS,
    )
    return values[0]


def pack(args: argparse.Namespace) -> dict[str, Any]:
    common = current_common(args)
    release_root = Path(args.release_root).absolute()
    output = Path(args.output).absolute()
    if os.path.lexists(output):
        fail("refusing existing handoff output")
    snapshot = snapshot_release(release_root)
    manifest, sidecar, _manifest_data = validate_manifest_snapshot(
        release_root, snapshot, common["sourceCommit"]
    )
    top = f"lkjmc-{common['productVersion']}-{common['sourceCommit']}"
    archive_name = f"{top}.tar"
    stage, stage_identity, parent_fd, parent_stat = create_stage(output.parent, output.name)
    published = False
    primary_error: BaseException | None = None
    try:
        stage_fd = os.open(stage, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
        try:
            write_archive(stage_fd, archive_name, release_root, top, snapshot)
            archive_size, archive_digest = hash_private_file(stage, archive_name)
            descriptor = descriptor_for(
                common,
                archive_size,
                archive_digest,
                hashlib.sha256(manifest).hexdigest(),
                hashlib.sha256(sidecar).hexdigest(),
            )
            write_private(
                stage_fd,
                f"{archive_name}.sha256",
                f"{archive_digest}  {archive_name}\n".encode("ascii"),
            )
            write_private(stage_fd, DESCRIPTOR_NAME, canonical_json(descriptor))
            os.fsync(stage_fd)
        finally:
            os.close(stage_fd)
        inspection = inspect_handoff(stage, common)
        os.close(inspection["archiveFd"])
        same_inode(parent_stat, os.fstat(parent_fd), "private parent")
        same_inode(parent_stat, os.lstat(output.parent), "private parent path")
        rename_noreplace(parent_fd, stage.name, output.name)
        os.fsync(parent_fd)
        published = True
        return descriptor
    except BaseException as error:
        primary_error = error
        raise
    finally:
        cleanup_error: BaseException | None = None
        if not published:
            try:
                remove_owned_directory(stage, stage_identity)
            except BaseException as error:
                cleanup_error = error
        os.close(parent_fd)
        if cleanup_error is not None and primary_error is None:
            raise cleanup_error
        if cleanup_error is not None and primary_error is not None:
            print(f"release archive cleanup failed after {primary_error}: {cleanup_error}", file=sys.stderr)


def read_exact(descriptor: int, size: int, digest: hashlib._Hash) -> bytes:
    value = bytearray()
    while len(value) < size:
        block = os.read(descriptor, min(1024 * 1024, size - len(value)))
        if not block:
            fail("truncated archive")
        digest.update(block)
        value.extend(block)
    return bytes(value)


def inspect_archive(descriptor: int) -> dict[str, Any]:
    os.lseek(descriptor, 0, os.SEEK_SET)
    archive_digest = hashlib.sha256()
    records: list[dict[str, Any]] = []
    names: set[str] = set()
    total_data = 0
    manifest = b""
    manifest_sidecar = b""
    while True:
        header = read_exact(descriptor, BLOCK, archive_digest)
        if header == bytes(BLOCK):
            second = read_exact(descriptor, BLOCK, archive_digest)
            if second != bytes(BLOCK):
                fail("archive has only one canonical zero terminator")
            if os.read(descriptor, 1):
                fail("archive has trailing data after canonical terminator")
            break
        try:
            item = tarfile.TarInfo.frombuf(header, encoding="utf-8", errors="strict")
        except (tarfile.HeaderError, UnicodeError) as error:
            fail(f"invalid ustar header: {error}")
        if item.type not in (tarfile.DIRTYPE, tarfile.REGTYPE) or item.sparse is not None:
            fail(f"link, special, extension, or sparse archive member: {item.name!r}")
        canonical_relative_path(item.name)
        if item.name in names:
            fail(f"duplicate archive member: {item.name}")
        names.add(item.name)
        if item.uid != 0 or item.gid != 0 or item.uname or item.gname or item.mtime != 0:
            fail(f"nonnormalized archive metadata: {item.name}")
        if item.linkname or item.devmajor != 0 or item.devminor != 0 or item.pax_headers:
            fail(f"unsupported archive metadata: {item.name}")
        if item.type == tarfile.DIRTYPE and item.size != 0:
            fail(f"directory archive member has data: {item.name}")
        if item.size < 0 or item.size > TREE_LIMITS.max_file_bytes:
            fail(f"archive member size overflow: {item.name}")
        total_data += item.size
        if len(records) + 1 > TREE_LIMITS.max_entries or total_data > TREE_LIMITS.max_bytes:
            fail("archive closure exceeds release limits")
        mode = item.mode & 0o7777
        if header != canonical_header(item.name, item.type, mode, item.size):
            fail(f"noncanonical ustar header: {item.name}")
        data_offset = os.lseek(descriptor, 0, os.SEEK_CUR)
        content_digest = hashlib.sha256()
        captured = bytearray()
        remaining = item.size
        while remaining:
            block = os.read(descriptor, min(1024 * 1024, remaining))
            if not block:
                fail(f"truncated archive member: {item.name}")
            archive_digest.update(block)
            content_digest.update(block)
            if item.name.endswith("/artifact-manifest.json") or item.name.endswith(
                "/artifact-manifest.json.sha256"
            ):
                captured.extend(block)
            remaining -= len(block)
        padding_size = (-item.size) % BLOCK
        if padding_size:
            padding = read_exact(descriptor, padding_size, archive_digest)
            if any(padding):
                fail(f"nonzero archive padding: {item.name}")
        if item.name.endswith("/artifact-manifest.json"):
            if len(captured) > 1024 * 1024:
                fail("oversized release manifest")
            manifest = bytes(captured)
        elif item.name.endswith("/artifact-manifest.json.sha256"):
            if len(captured) > MAX_SIDECAR_BYTES:
                fail("oversized release manifest sidecar")
            manifest_sidecar = bytes(captured)
        records.append(
            {
                "name": item.name,
                "kind": "directory" if item.type == tarfile.DIRTYPE else "file",
                "mode": mode,
                "size": item.size,
                "sha256": None if item.type == tarfile.DIRTYPE else content_digest.hexdigest(),
                "offset": data_offset,
            }
        )
    if not records:
        fail("empty release archive")
    ordered = sorted((record["name"] for record in records), key=lambda value: value.encode("utf-8"))
    if [record["name"] for record in records] != ordered:
        fail("archive member order differs from bytewise POSIX order")
    return {
        "archiveSha256": archive_digest.hexdigest(),
        "archiveSize": os.fstat(descriptor).st_size,
        "records": records,
        "manifest": manifest,
        "manifestSidecar": manifest_sidecar,
    }


def validate_archive_closure(archive: dict[str, Any], descriptor: dict[str, Any]) -> None:
    if archive["archiveSha256"] != descriptor["archiveSha256"]:
        fail("archive digest differs from handoff descriptor")
    if archive["archiveSize"] != descriptor["archiveSize"]:
        fail("archive size differs from handoff descriptor")
    manifest_digest = hashlib.sha256(archive["manifest"]).hexdigest()
    sidecar_digest = hashlib.sha256(archive["manifestSidecar"]).hexdigest()
    if manifest_digest != descriptor["releaseManifestSha256"]:
        fail("release manifest digest differs from handoff descriptor")
    if sidecar_digest != descriptor["releaseManifestSidecarSha256"]:
        fail("release manifest sidecar digest differs from handoff descriptor")
    strict_sidecar(archive["manifestSidecar"], "artifact-manifest.json", manifest_digest)
    manifest = strict_json(archive["manifest"], "release manifest", pretty=True)
    if (
        not isinstance(manifest, dict)
        or type(manifest.get("schemaVersion")) is not int
        or manifest["schemaVersion"] != 1
    ):
        fail("unsupported release manifest schema")
    if manifest.get("commit") != descriptor["sourceCommit"]:
        fail("release manifest commit differs from handoff descriptor")
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list):
        fail("release manifest artifact closure missing")
    declared: dict[str, tuple[int, str]] = {}
    for item in artifacts:
        if not isinstance(item, dict):
            fail("invalid release manifest artifact")
        path = item.get("path")
        size = item.get("size")
        digest = item.get("sha256")
        if not isinstance(path, str) or Path(path).name != path or path in declared:
            fail("unsafe or duplicate release manifest artifact")
        if isinstance(size, bool) or not isinstance(size, int) or size < 0:
            fail(f"invalid release manifest size: {path}")
        if not isinstance(digest, str) or not HEX64.fullmatch(digest):
            fail(f"invalid release manifest digest: {path}")
        declared[path] = (size, digest)
    contract = release_contract()
    contract_names = {item["destination"] for item in contract}
    if set(declared) != contract_names or len(artifacts) != len(contract):
        fail("release manifest artifact paths differ from contract")
    top = descriptor["topLevelDirectory"]
    expected_records: dict[str, tuple[str, int, int | None, str | None]] = {
        top: ("directory", 0o700, 0, None),
        f"{top}/artifact-manifest.json": ("file", 0o600, len(archive["manifest"]), manifest_digest),
        f"{top}/artifact-manifest.json.sha256": (
            "file", 0o600, len(archive["manifestSidecar"]), sidecar_digest
        ),
        f"{top}/source": ("directory", 0o700, 0, None),
    }
    for item in contract:
        size, digest = declared[item["destination"]]
        mode = 0o700 if item["kind"] == "binary" else 0o600
        expected_records[f"{top}/source/{item['destination']}"] = ("file", mode, size, digest)
    actual_records = {
        record["name"]: (record["kind"], record["mode"], record["size"], record["sha256"])
        for record in archive["records"]
    }
    if actual_records != expected_records:
        fail("archive path/type/mode/size/digest closure differs")


def inspect_handoff(root: Path, common: dict[str, Any]) -> dict[str, Any]:
    descriptor_bytes: list[bytes] = []
    sidecars: dict[str, bytes] = {}
    archive_fds: dict[str, int] = {}

    def no_directory(_descriptor: int, entry: Entry) -> None:
        fail(f"outer artifact contains a directory: {entry.path}")

    def visitor(descriptor: int, entry: Entry) -> None:
        if entry.path == DESCRIPTOR_NAME:
            descriptor_bytes.append(read_all(descriptor, entry.size, MAX_DESCRIPTOR_BYTES, "handoff descriptor"))
        elif entry.path.endswith(".tar.sha256"):
            sidecars[entry.path] = read_all(descriptor, entry.size, MAX_SIDECAR_BYTES, "archive sidecar")
        elif entry.path.endswith(".tar"):
            archive_fds[entry.path] = os.dup(descriptor)

    try:
        entries = walk(root, visitor, OUTER_LIMITS, visit_directory=no_directory)
        paths = {entry.path for entry in entries}
        if len(descriptor_bytes) != 1 or len(sidecars) != 1 or len(archive_fds) != 1:
            fail("outer artifact does not contain one descriptor, archive, and sidecar")
        descriptor = validate_descriptor(descriptor_bytes[0], common)
        archive_name = descriptor["archiveFilename"]
        expected_paths = {DESCRIPTOR_NAME, archive_name, f"{archive_name}.sha256"}
        if paths != expected_paths or len(entries) != 3:
            fail("outer artifact file closure differs")
        archive_fd = archive_fds.pop(archive_name)
        archive = inspect_archive(archive_fd)
        strict_sidecar(sidecars[f"{archive_name}.sha256"], archive_name, archive["archiveSha256"])
        validate_archive_closure(archive, descriptor)
        return {"descriptor": descriptor, "archive": archive, "archiveFd": archive_fd}
    finally:
        for descriptor in archive_fds.values():
            os.close(descriptor)


def extract_inspection(inspection: dict[str, Any], output: Path) -> dict[str, Any]:
    if os.path.lexists(output):
        fail("refusing existing extraction target")
    stage, stage_identity, parent_fd, parent_stat = create_stage(output.parent, output.name)
    published = False
    primary_error: BaseException | None = None
    try:
        root_fd = os.open(stage, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
        directories: dict[str, int] = {".": root_fd}
        top = inspection["descriptor"]["topLevelDirectory"]
        archive_fd = inspection["archiveFd"]
        try:
            for record in inspection["archive"]["records"]:
                parts = canonical_relative_path(record["name"])
                if parts[0] != top:
                    fail("archive member escapes canonical top-level directory")
                relative = parts[1:]
                if not relative:
                    if record["kind"] != "directory" or record["mode"] != 0o700:
                        fail("invalid top-level archive directory")
                    continue
                parent_name = "." if len(relative) == 1 else "/".join(relative[:-1])
                if parent_name not in directories:
                    fail(f"archive parent directory is not explicit: {record['name']}")
                parent_descriptor = directories[parent_name]
                name = relative[-1]
                if record["kind"] == "directory":
                    os.mkdir(name, record["mode"], dir_fd=parent_descriptor)
                    child = os.open(
                        name,
                        os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC,
                        dir_fd=parent_descriptor,
                    )
                    directories["/".join(relative)] = child
                    continue
                child = os.open(
                    name,
                    os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC,
                    record["mode"],
                    dir_fd=parent_descriptor,
                )
                try:
                    digest = hashlib.sha256()
                    offset = record["offset"]
                    remaining = record["size"]
                    while remaining:
                        block = os.pread(archive_fd, min(1024 * 1024, remaining), offset)
                        if not block:
                            fail(f"archive changed during extraction: {record['name']}")
                        digest.update(block)
                        write_all(child, block)
                        offset += len(block)
                        remaining -= len(block)
                    if digest.hexdigest() != record["sha256"]:
                        fail(f"archive member digest changed during extraction: {record['name']}")
                    os.fchmod(child, record["mode"])
                    os.fsync(child)
                finally:
                    os.close(child)
            for name in sorted((name for name in directories if name != "."), reverse=True):
                os.fsync(directories[name])
                os.close(directories[name])
            directories = {".": root_fd}
            os.fsync(root_fd)
        finally:
            for name, descriptor in list(directories.items()):
                if name != ".":
                    os.close(descriptor)
            os.close(root_fd)
        snapshot = snapshot_release(stage)
        expected_records = {
            ".": {"kind": "directory", "mode": 0o700, "size": 0, "sha256": None}
        }
        for record in inspection["archive"]["records"]:
            name = record["name"]
            if name == top:
                continue
            relative = name[len(top) + 1:]
            expected_records[relative] = {
                key: record[key] for key in ("kind", "mode", "size", "sha256")
            }
        if snapshot != expected_records:
            fail("extracted release differs from verified archive closure")
        same_inode(parent_stat, os.fstat(parent_fd), "private parent")
        same_inode(parent_stat, os.lstat(output.parent), "private parent path")
        rename_noreplace(parent_fd, stage.name, output.name)
        os.fsync(parent_fd)
        published = True
        return inspection["descriptor"]
    except BaseException as error:
        primary_error = error
        raise
    finally:
        cleanup_error: BaseException | None = None
        if not published:
            try:
                remove_owned_directory(stage, stage_identity)
            except BaseException as error:
                cleanup_error = error
        os.close(parent_fd)
        if cleanup_error is not None and primary_error is None:
            raise cleanup_error
        if cleanup_error is not None and primary_error is not None:
            print(f"release extraction cleanup failed after {primary_error}: {cleanup_error}", file=sys.stderr)


def run_verifier(command: tuple[str, ...], label: str, source_commit: str) -> str:
    environment = os.environ | {"LKJMC_SOURCE_COMMIT": source_commit}
    result = subprocess.run(
        command,
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        timeout=180,
        check=False,
    )
    output = (result.stdout + result.stderr).strip()
    if result.returncode != 0:
        fail(f"{label} failed: {output[-4096:]}")
    if len(output) > 4096:
        fail(f"{label} output is unbounded")
    return output


def write_receipt(path: Path, receipt: dict[str, Any]) -> None:
    if set(receipt) != RECEIPT_FIELDS:
        fail("consumer receipt fields differ")
    if os.path.lexists(path):
        fail("refusing existing consumer receipt")
    if not SAFE_NAME.fullmatch(path.name):
        fail("unsafe consumer receipt name")
    parent_fd, parent_stat = private_parent(path.parent)
    try:
        write_private(parent_fd, path.name, canonical_json(receipt))
        same_inode(parent_stat, os.fstat(parent_fd), "receipt parent")
        same_inode(parent_stat, os.lstat(path.parent), "receipt parent path")
        os.fsync(parent_fd)
    finally:
        os.close(parent_fd)


def verify_command(args: argparse.Namespace) -> dict[str, Any]:
    common = current_common(args)
    inspection = inspect_handoff(Path(args.artifact_dir).absolute(), common)
    try:
        return inspection["descriptor"]
    finally:
        os.close(inspection["archiveFd"])


def extract_command(args: argparse.Namespace) -> dict[str, Any]:
    common = current_common(args)
    inspection = inspect_handoff(Path(args.artifact_dir).absolute(), common)
    try:
        return extract_inspection(inspection, Path(args.output).absolute())
    finally:
        os.close(inspection["archiveFd"])


def normalize_service_digest(value: str) -> str:
    match = re.fullmatch(r"(?:sha256:)?([0-9a-f]{64})", value)
    if match is None:
        fail("invalid artifact-service digest")
    return f"sha256:{match.group(1)}"


def consume_command(args: argparse.Namespace) -> dict[str, Any]:
    common = current_common(args)
    if not re.fullmatch(r"[1-9][0-9]{0,19}", args.artifact_id):
        fail("invalid artifact ID")
    service_digest = normalize_service_digest(args.artifact_digest)
    work_parent = Path(args.work_parent).absolute()
    receipt_path = Path(args.receipt).absolute()
    if receipt_path.parent != work_parent:
        fail("consumer receipt must be directly under its private work parent")
    check_fd, _check_stat = private_parent(work_parent)
    os.close(check_fd)
    output = work_parent / f"consumed-release-{secrets.token_hex(16)}"
    inspection = inspect_handoff(Path(args.artifact_dir).absolute(), common)
    extracted = False
    output_identity: tuple[int, int] | None = None
    primary_error: BaseException | None = None
    try:
        descriptor = extract_inspection(inspection, output)
        extracted = True
        output_identity = inode_identity(os.lstat(output))
        manifest_result = run_verifier(
            (
                sys.executable,
                str(ROOT / "scripts/verify-artifact-manifest.py"),
                "--manifest",
                str(output / "artifact-manifest.json"),
                "--release-root",
                str(output),
            ),
            "independent manifest verifier",
            common["sourceCommit"],
        )
        identity_result = run_verifier(
            (
                sys.executable,
                str(ROOT / "scripts/verify-built-identity.py"),
                "--source",
                str(output / "source"),
            ),
            "independent built-identity verifier",
            common["sourceCommit"],
        )
        remove_owned_directory(output, output_identity)
        extracted = False
        receipt = {
            "schemaVersion": 1,
            "status": "release-artifact-verified",
            "sourceCommit": descriptor["sourceCommit"],
            "workflowRunId": descriptor["workflowRunId"],
            "workflowRunAttempt": descriptor["workflowRunAttempt"],
            "outerArtifactName": descriptor["outerArtifactName"],
            "artifactId": args.artifact_id,
            "artifactServiceDigest": service_digest,
            "archiveFilename": descriptor["archiveFilename"],
            "archiveSha256": descriptor["archiveSha256"],
            "releaseManifestSha256": descriptor["releaseManifestSha256"],
            "manifestVerifier": manifest_result,
            "identityVerifier": identity_result,
        }
        if "built-identity" not in identity_result:
            fail("independent built-identity verifier result is not canonical")
        write_receipt(receipt_path, receipt)
        return receipt
    except BaseException as error:
        primary_error = error
        raise
    finally:
        os.close(inspection["archiveFd"])
        if extracted and output_identity is not None:
            try:
                remove_owned_directory(output, output_identity)
            except BaseException as cleanup_error:
                if primary_error is None:
                    raise
                print(
                    f"release consume cleanup failed after {primary_error}: {cleanup_error}",
                    file=sys.stderr,
                )


def add_common(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--repository", required=True)
    parser.add_argument("--outer-artifact-name", required=True)
    parser.add_argument("--workflow-event", required=True)
    parser.add_argument("--workflow-ref", required=True)
    parser.add_argument("--workflow-run-id", required=True)
    parser.add_argument("--workflow-run-attempt", required=True, type=int)
    parser.add_argument("--producer-job", required=True)


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    commands = value.add_subparsers(dest="command", required=True)
    pack_parser = commands.add_parser("pack")
    add_common(pack_parser)
    pack_parser.add_argument("--release-root", required=True)
    pack_parser.add_argument("--output", required=True)
    verify_parser = commands.add_parser("verify")
    add_common(verify_parser)
    verify_parser.add_argument("--artifact-dir", required=True)
    extract_parser = commands.add_parser("extract")
    add_common(extract_parser)
    extract_parser.add_argument("--artifact-dir", required=True)
    extract_parser.add_argument("--output", required=True)
    consume_parser = commands.add_parser("consume")
    add_common(consume_parser)
    consume_parser.add_argument("--artifact-dir", required=True)
    consume_parser.add_argument("--work-parent", required=True)
    consume_parser.add_argument("--receipt", required=True)
    consume_parser.add_argument("--artifact-id", required=True)
    consume_parser.add_argument("--artifact-digest", required=True)
    return value


def interrupted(signum: int, _frame: Any) -> None:
    raise InterruptedError(f"interrupted by signal {signum}")


def main() -> int:
    os.umask(0o077)
    for name in (signal.SIGHUP, signal.SIGINT, signal.SIGTERM):
        signal.signal(name, interrupted)
    args = parser().parse_args()
    functions = {
        "pack": pack,
        "verify": verify_command,
        "extract": extract_command,
        "consume": consume_command,
    }
    try:
        result = functions[args.command](args)
    except Exception as error:
        print(f"release archive {args.command} failed: {error}", file=sys.stderr)
        return 1
    print(canonical_json(result).decode("utf-8"), end="")
    return 0


if __name__ == "__main__":
    sys.exit(main())
