#!/usr/bin/env python3
"""Privileged fail-closed systemd pre-start check for deployment fencing."""
import json
import os
from pathlib import Path
import re
import stat
import sys

FENCE = Path("/etc/lkjmc/deployment-fence.json")
PERMIT = Path("/run/lkjmc-deploy-start-permit")
HEX40 = re.compile(r"[0-9a-f]{40}")
SAFE_LABEL = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]{0,127}")


class FenceError(RuntimeError):
    pass


def fail(message):
    raise FenceError(message)


def root_directory(path, expected_uid):
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        fail(f"missing control directory: {path}")
    if not stat.S_ISDIR(metadata.st_mode) or path.is_symlink() \
            or metadata.st_uid != expected_uid or stat.S_IMODE(metadata.st_mode) & 0o022:
        fail(f"unsafe control directory: {path}")


def root_ancestry(path, expected_uid, trusted_root):
    root = trusted_root.resolve(strict=True)
    parent = path.parent.resolve(strict=True)
    try:
        relative = parent.relative_to(root)
    except ValueError:
        fail("deployment control path escapes its trusted root")
    root_directory(root, expected_uid)
    current = root
    for part in relative.parts:
        current /= part
        root_directory(current, expected_uid)


def regular_control(path, mode, expected_uid, expected_bytes=None, trusted_root=Path("/")):
    root_ancestry(path, expected_uid, trusted_root)
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        fail(f"missing deployment control file: {path}")
    if not stat.S_ISREG(metadata.st_mode) or path.is_symlink() \
            or metadata.st_uid != expected_uid or stat.S_IMODE(metadata.st_mode) != mode:
        fail(f"unsafe deployment control file: {path}")
    raw = path.read_bytes()
    if expected_bytes is not None and raw != expected_bytes:
        fail(f"deployment control contents differ: {path}")
    return raw


def validate_fence(raw):
    if len(raw) > 65536:
        fail("deployment fence is too large")
    try:
        value = json.loads(raw)
    except (TypeError, ValueError) as error:
        fail(f"invalid deployment fence JSON: {error}")
    fields = {
        "schemaVersion", "fromCommit", "toCommit", "stateDirectory", "backup", "rollbackSnapshot",
    }
    if not isinstance(value, dict) or set(value) != fields or value.get("schemaVersion") != 1 \
            or not isinstance(value.get("fromCommit"), str) \
            or not HEX40.fullmatch(value["fromCommit"]) \
            or not isinstance(value.get("toCommit"), str) \
            or not HEX40.fullmatch(value["toCommit"]) \
            or value.get("stateDirectory") != f'/var/lib/private/lkjmc-deployments/{value["toCommit"]}' \
            or not isinstance(value.get("backup"), str) \
            or not value["backup"].startswith("/var/backups/lkjmc/") \
            or not isinstance(value.get("rollbackSnapshot"), str) \
            or not SAFE_LABEL.fullmatch(value["rollbackSnapshot"]):
        fail("deployment fence fields differ")


def check(fence=FENCE, permit=PERMIT, expected_uid=0, trusted_root=Path("/")):
    fence_present = fence.exists() or fence.is_symlink()
    permit_present = permit.exists() or permit.is_symlink()
    if not fence_present:
        if permit_present:
            fail("start permit exists without a deployment fence")
        root_ancestry(fence, expected_uid, trusted_root)
        root_ancestry(permit, expected_uid, trusted_root)
        return "unfenced"
    raw = regular_control(fence, 0o600, expected_uid, trusted_root=trusted_root)
    validate_fence(raw)
    if not permit_present:
        fail("deployment fence blocks service start")
    regular_control(
        permit,
        0o400,
        expected_uid,
        expected_bytes=b"lkjmc-deploy-start-permit\n",
        trusted_root=trusted_root,
    )
    permit.unlink()
    descriptor = os.open(permit.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    return "permitted-once"


def main():
    if os.geteuid() != 0:
        fail("deployment fence check requires privileged systemd execution")
    print(f"lkjmc deployment fence: {check()}")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"lkjmc deployment fence blocked start: {error}", file=sys.stderr)
        sys.exit(1)
