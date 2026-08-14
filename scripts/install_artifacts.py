#!/usr/bin/env python3
"""Verify an anchored release and atomically publish its immutable artifact tree."""
import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
from pathlib import Path

HEX64 = re.compile(r"[0-9a-f]{64}")
ARTIFACT_FIELDS = {"component", "kind", "path", "provenance", "sha256", "size", "source"}


def fail(message):
    raise RuntimeError(message)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def regular(path, label):
    try:
        mode = path.lstat().st_mode
    except FileNotFoundError:
        fail(f"missing {label}: {path}")
    if not stat.S_ISREG(mode) or path.is_symlink():
        fail(f"{label} is not a regular file: {path}")


def fsync_dir(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def verify_portable(manifest, source, expected_digest):
    if not HEX64.fullmatch(expected_digest or ""):
        fail("--manifest-sha256 must be one lowercase SHA-256 digest")
    regular(manifest, "manifest")
    sidecar = manifest.with_suffix(manifest.suffix + ".sha256")
    regular(sidecar, "manifest sidecar")
    raw = manifest.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    if digest != expected_digest:
        fail("manifest differs from the operator-supplied SHA-256")
    if sidecar.read_text(encoding="ascii") != f"{digest}  artifact-manifest.json\n":
        fail("manifest checksum sidecar differs")
    try:
        data = json.loads(raw)
    except (TypeError, ValueError) as error:
        fail(f"invalid release manifest: {error}")
    if not isinstance(data, dict) or data.get("schemaVersion") != 1:
        fail("unsupported release manifest schema")
    if not re.fullmatch(r"[0-9a-f]{40}", data.get("commit", "")):
        fail("invalid release commit")
    artifacts = data.get("artifacts")
    if not isinstance(artifacts, list) or not artifacts:
        fail("release manifest has no artifacts")
    names = set()
    for item in artifacts:
        if not isinstance(item, dict) or set(item) != ARTIFACT_FIELDS:
            fail("release artifact fields differ")
        name = item.get("path")
        kind = item.get("kind")
        size = item.get("size")
        if not isinstance(name, str) or Path(name).name != name or name in ("", ".", ".."):
            fail("unsafe release artifact path")
        if name in names:
            fail("duplicate release artifact path")
        names.add(name)
        if kind not in ("binary", "jar", "config"):
            fail(f"unsupported release artifact kind: {kind}")
        if (name.endswith(".jar")) != (kind == "jar"):
            fail("release artifact kind differs from path")
        if not HEX64.fullmatch(item.get("sha256", "")):
            fail("invalid release artifact SHA-256")
        if isinstance(size, bool) or not isinstance(size, int) or size < 0:
            fail("invalid release artifact size")
        path = source / name
        regular(path, "release artifact")
        metadata = path.stat()
        if metadata.st_size != size or sha256(path) != item["sha256"]:
            fail(f"release artifact differs: {name}")
        if stat.S_IMODE(metadata.st_mode) & 0o022:
            fail(f"release artifact is group/other writable: {name}")
    if source.is_symlink() or not source.is_dir():
        fail("release source directory is unsafe")
    actual = {path.name for path in source.iterdir() if path.is_file() and not path.is_symlink()}
    if actual != names or any(path.is_symlink() or not path.is_file() for path in source.iterdir()):
        fail("release artifact closure differs from manifest")
    return data


def metadata(data, manifest):
    value = {
        "commit": data["commit"],
        "manifestSha256": hashlib.sha256(manifest.read_bytes()).hexdigest(),
        "schemaVersion": 1,
    }
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def artifact_location(item):
    if item["kind"] == "jar":
        return "jars", 0o640
    if item["kind"] == "config":
        return "share", 0o640
    return "bin", 0o750


def expected_files(data, manifest):
    result = {}
    for item in data["artifacts"]:
        folder, mode = artifact_location(item)
        result[f'{folder}/{item["path"]}'] = (item["sha256"], item["size"], mode)
    for name in ("artifact-manifest.json", "artifact-manifest.json.sha256"):
        path = manifest.parent / name
        result[f"meta/{name}"] = (sha256(path), path.stat().st_size, 0o640)
    payload = metadata(data, manifest)
    result[".lkjmc-install.json"] = (hashlib.sha256(payload).hexdigest(), len(payload), 0o640)
    return result


def valid_tree(root, data, manifest, uid, gid, dir_mode):
    if root.is_symlink() or not root.is_dir():
        return False
    expected = expected_files(data, manifest)
    actual = {str(path.relative_to(root)) for path in root.rglob("*") if path.is_file() and not path.is_symlink()}
    if actual != set(expected) or any(path.is_symlink() for path in root.rglob("*")):
        return False
    directories = {root}
    directories.update((root / relative).parent for relative in expected)
    for directory in directories:
        value = directory.stat()
        if not directory.is_dir() or stat.S_IMODE(value.st_mode) != dir_mode:
            return False
        if (value.st_uid, value.st_gid) != (uid, gid):
            return False
    for relative, (digest, size, mode) in expected.items():
        path = root / relative
        value = path.stat()
        if not stat.S_ISREG(value.st_mode) or (value.st_uid, value.st_gid) != (uid, gid):
            return False
        if value.st_size != size or sha256(path) != digest or stat.S_IMODE(value.st_mode) != mode:
            return False
    return True


def copy_file(source, destination, mode, uid, gid):
    with source.open("rb") as incoming, destination.open("xb") as outgoing:
        shutil.copyfileobj(incoming, outgoing)
        outgoing.flush()
        os.fsync(outgoing.fileno())
    os.chmod(destination, mode)
    os.chown(destination, uid, gid)


def stage_tree(stage, source, data, manifest, uid, gid, dir_mode):
    folders = {artifact_location(item)[0] for item in data["artifacts"]} | {"meta"}
    for folder in sorted(folders):
        os.mkdir(stage / folder, dir_mode)
    for item in data["artifacts"]:
        folder, mode = artifact_location(item)
        copy_file(source / item["path"], stage / folder / item["path"], mode, uid, gid)
    for name in ("artifact-manifest.json", "artifact-manifest.json.sha256"):
        copy_file(manifest.parent / name, stage / "meta" / name, 0o640, uid, gid)
    install_metadata = stage / ".lkjmc-install.json"
    with install_metadata.open("xb") as output:
        output.write(metadata(data, manifest))
        output.flush()
        os.fsync(output.fileno())
    os.chmod(install_metadata, 0o640)
    os.chown(install_metadata, uid, gid)
    for directory in [stage / folder for folder in sorted(folders)] + [stage]:
        os.chmod(directory, dir_mode)
        os.chown(directory, uid, gid)
        fsync_dir(directory)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--scope", required=True, choices=("system", "user", "rootless"))
    parser.add_argument("--manifest", required=True)
    parser.add_argument("--manifest-sha256")
    parser.add_argument("--root", required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--service-uid", type=int)
    parser.add_argument("--service-gid", type=int)
    args = parser.parse_args()
    current = os.geteuid()
    if args.scope == "system":
        if current != 0 or args.service_uid is None or args.service_gid is None:
            fail("system scope requires root and numeric service UID/GID")
        uid = 0
        gid = args.service_gid
        dir_mode = 0o750
    else:
        if current == 0:
            fail(f"{args.scope} scope refuses root")
        uid = current
        gid = os.getegid()
        dir_mode = 0o700
    manifest = Path(args.manifest).resolve()
    release = manifest.parent
    source = Path(args.source).resolve()
    root = Path(os.path.abspath(args.root))
    if source != release / "source" or root == Path("/") or ".." in Path(args.root).parts or root.is_symlink():
        fail("unsafe release source or install root")
    if args.manifest_sha256:
        data = verify_portable(manifest, source, args.manifest_sha256)
    else:
        verifier = Path(__file__).with_name("verify-artifact-manifest.py")
        subprocess.run(
            (sys.executable, str(verifier), "--manifest", str(manifest), "--release-root", str(release)),
            check=True,
        )
        data = json.loads(manifest.read_bytes())
    if valid_tree(root, data, manifest, uid, gid, dir_mode):
        print(f'ok artifact-install scope={args.scope} root={root} result=no-op version={data["commit"]}')
        return
    parent = root.parent
    parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".lkjmc-stage-", dir=parent))
    os.chmod(stage, 0o700)
    rollback = Path(tempfile.mkdtemp(prefix=".lkjmc-rollback-", dir=parent))
    rollback.rmdir()
    prior = False
    published = False
    committed = False
    try:
        stage_tree(stage, source, data, manifest, uid, gid, dir_mode)
        if os.environ.get("LKJMC_INSTALL_FAULT") == "after-stage":
            fail("injected failure after stage")
        if root.exists():
            os.replace(root, rollback)
            prior = True
            fsync_dir(parent)
        os.replace(stage, root)
        published = True
        fsync_dir(parent)
        if os.environ.get("LKJMC_INSTALL_FAULT") in ("after-publish", "validation", "status"):
            fail("injected post-publish validation failure")
        if not valid_tree(root, data, manifest, uid, gid, dir_mode):
            fail("post-publish validation failed")
        committed = True
        if os.environ.get("LKJMC_INSTALL_FAULT") == "after-commit":
            fail("injected failure after committed publication")
        if prior:
            shutil.rmtree(rollback)
        fsync_dir(parent)
    except Exception:
        if not committed:
            if published and root.exists():
                shutil.rmtree(root)
            if prior and rollback.exists():
                os.replace(rollback, root)
            fsync_dir(parent)
        raise
    finally:
        if stage.exists():
            shutil.rmtree(stage)
        if rollback.exists() and not prior:
            shutil.rmtree(rollback)
    print(f'ok artifact-install scope={args.scope} root={root} result=updated version={data["commit"]}')


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"artifact install failed: {error}", file=sys.stderr)
        sys.exit(1)
