#!/usr/bin/env python3
"""Deterministic and fail-closed tests for the canonical release handoff."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import stat
import tarfile
import tempfile
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = str(ROOT / "scripts")
if SCRIPTS not in os.sys.path:
    os.sys.path.insert(0, SCRIPTS)
SPEC = importlib.util.spec_from_file_location(
    "release_archive", ROOT / "scripts/release_archive.py"
)
ARCHIVE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ARCHIVE)

COMMIT = "a" * 40
VERSION = "1.2.3-test.1"
RUN_ID = "123456"
ATTEMPT = 2
REPOSITORY = "lkjsxc/lkjmc"
JOB = "verify-compose"
CONTRACT = [
    {
        "component": "data",
        "destination": "data.jar",
        "kind": "jar",
        "source": "build/data.jar",
    },
    {
        "component": "tool",
        "destination": "tool",
        "kind": "binary",
        "source": "build/tool",
    },
]


def common_arguments():
    return {
        "repository": REPOSITORY,
        "outer_artifact_name": ARCHIVE.canonical_artifact_name(COMMIT, RUN_ID, ATTEMPT),
        "workflow_event": "push",
        "workflow_ref": "refs/heads/main",
        "workflow_run_id": RUN_ID,
        "workflow_run_attempt": ATTEMPT,
        "producer_job": JOB,
    }


def namespace(**values):
    return type("Arguments", (), values)()


def make_manifest(release):
    artifacts = []
    for item in sorted(CONTRACT, key=lambda value: value["destination"]):
        raw = (release / "source" / item["destination"]).read_bytes()
        artifacts.append(
            {
                "component": item["component"],
                "kind": item["kind"],
                "path": item["destination"],
                "provenance": f"pinned build at {COMMIT}",
                "sha256": hashlib.sha256(raw).hexdigest(),
                "size": len(raw),
                "source": item["source"],
            }
        )
    return {
        "schemaVersion": 1,
        "commit": COMMIT,
        "artifacts": artifacts,
        "components": [],
        "contracts": [],
        "images": [],
    }


def create_release(parent, tool=b"tool\n", data=b"jar\n"):
    release = parent / "release"
    source = release / "source"
    source.mkdir(parents=True, mode=0o700)
    release.chmod(0o700)
    source.chmod(0o700)
    (source / "data.jar").write_bytes(data)
    (source / "data.jar").chmod(0o600)
    (source / "tool").write_bytes(tool)
    (source / "tool").chmod(0o700)
    manifest = make_manifest(release)
    raw = ARCHIVE.canonical_json(manifest, pretty=True)
    (release / "artifact-manifest.json").write_bytes(raw)
    (release / "artifact-manifest.json").chmod(0o600)
    (release / "artifact-manifest.json.sha256").write_text(
        f"{hashlib.sha256(raw).hexdigest()}  artifact-manifest.json\n",
        encoding="ascii",
    )
    (release / "artifact-manifest.json.sha256").chmod(0o600)
    return release, manifest


def fixture_owners(manifest):
    return mock.patch.multiple(
        ARCHIVE,
        commit=mock.Mock(return_value=COMMIT),
        workspace_package_value=mock.Mock(return_value=VERSION),
        release_contract=mock.Mock(return_value=CONTRACT),
        expected=mock.Mock(return_value=manifest),
    )


def pack_fixture(release, manifest, output):
    arguments = namespace(
        release_root=str(release), output=str(output), **common_arguments()
    )
    with fixture_owners(manifest):
        return ARCHIVE.pack(arguments)


def archive_path(handoff):
    return next(path for path in handoff.iterdir() if path.suffix == ".tar")


def checksum_header(block):
    value = bytearray(block)
    value[148:156] = b"        "
    checksum = sum(value)
    value[148:156] = f"{checksum:06o}\0 ".encode("ascii")
    return bytes(value)


def custom_header(name, kind=tarfile.DIRTYPE, mode=0o700, size=0, **fields):
    item = tarfile.TarInfo(name)
    item.type = kind
    item.mode = mode
    item.size = size
    item.uid = fields.get("uid", 0)
    item.gid = fields.get("gid", 0)
    item.mtime = fields.get("mtime", 0)
    item.uname = fields.get("uname", "")
    item.gname = fields.get("gname", "")
    item.linkname = fields.get("linkname", "")
    item.devmajor = fields.get("devmajor", 0)
    item.devminor = fields.get("devminor", 0)
    return item.tobuf(fields.get("format", tarfile.USTAR_FORMAT), encoding="utf-8")


def raw_archive(*members, trailing=b""):
    value = bytearray()
    for header, data in members:
        value.extend(header)
        value.extend(data)
        value.extend(bytes((-len(data)) % 512))
    value.extend(bytes(1024))
    value.extend(trailing)
    return bytes(value)


def inspect_raw(raw):
    with tempfile.TemporaryFile() as stream:
        stream.write(raw)
        stream.flush()
        return ARCHIVE.inspect_archive(stream.fileno())


def rewrite_handoff(handoff, transform):
    tar = archive_path(handoff)
    raw = transform(tar.read_bytes())
    tar.write_bytes(raw)
    tar.chmod(0o600)
    digest = hashlib.sha256(raw).hexdigest()
    (handoff / f"{tar.name}.sha256").write_text(
        f"{digest}  {tar.name}\n", encoding="ascii"
    )
    (handoff / f"{tar.name}.sha256").chmod(0o600)
    descriptor_path = handoff / ARCHIVE.DESCRIPTOR_NAME
    descriptor = json.loads(descriptor_path.read_text())
    descriptor["archiveSha256"] = digest
    descriptor["archiveSize"] = len(raw)
    descriptor_path.write_bytes(ARCHIVE.canonical_json(descriptor))
    descriptor_path.chmod(0o600)


class ReleaseArchiveTest(unittest.TestCase):
    def test_consume_runs_independent_verifiers_and_removes_extracted_state(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-archive-consume-") as raw:
            root = Path(raw)
            release, manifest = create_release(root)
            handoff = root / "handoff"
            pack_fixture(release, manifest, handoff)
            work = root / "consumer"
            work.mkdir(mode=0o700)
            arguments = namespace(
                artifact_dir=str(handoff),
                work_parent=str(work),
                receipt=str(work / "receipt.json"),
                artifact_id="987654",
                artifact_digest="c" * 64,
                **common_arguments(),
            )
            results = [
                "ok artifact-manifest-verified commit=" + COMMIT + " artifacts=2 contracts=0",
                "ok built-identity version=" + VERSION + " commit=" + COMMIT,
            ]
            with fixture_owners(manifest), mock.patch.object(
                ARCHIVE, "run_verifier", side_effect=results
            ) as verifier:
                receipt = ARCHIVE.consume_command(arguments)
            self.assertEqual(verifier.call_count, 2)
            self.assertEqual(receipt["status"], "release-artifact-verified")
            self.assertEqual(receipt["artifactServiceDigest"], "sha256:" + "c" * 64)
            self.assertEqual(
                [path.name for path in work.iterdir()], ["receipt.json"]
            )
            self.assertEqual(json.loads((work / "receipt.json").read_text()), receipt)

    def test_two_packs_are_identical_and_extract_exact_modes(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-archive-") as raw:
            root = Path(raw)
            release, manifest = create_release(root)
            first = root / "first"
            second = root / "second"
            first_result = pack_fixture(release, manifest, first)
            os.utime(release, (1_000_000, 1_000_000))
            os.utime(release / "source/tool", (2_000_000, 2_000_000))
            second_result = pack_fixture(release, manifest, second)
            self.assertEqual(first_result, second_result)
            self.assertEqual(archive_path(first).read_bytes(), archive_path(second).read_bytes())
            self.assertEqual(
                (first / ARCHIVE.DESCRIPTOR_NAME).read_bytes(),
                (second / ARCHIVE.DESCRIPTOR_NAME).read_bytes(),
            )

            common = {
                "repository": REPOSITORY,
                "sourceCommit": COMMIT,
                "productVersion": VERSION,
                "outerArtifactName": common_arguments()["outer_artifact_name"],
                "workflowEvent": "push",
                "workflowRef": "refs/heads/main",
                "workflowRunId": RUN_ID,
                "workflowRunAttempt": ATTEMPT,
                "producerJob": JOB,
            }
            extracted = root / "extracted"
            with fixture_owners(manifest):
                inspection = ARCHIVE.inspect_handoff(first, common)
                try:
                    ARCHIVE.extract_inspection(inspection, extracted)
                finally:
                    os.close(inspection["archiveFd"])
            self.assertEqual((extracted / "source/tool").read_bytes(), b"tool\n")
            self.assertEqual((extracted / "source/data.jar").read_bytes(), b"jar\n")
            self.assertEqual(stat.S_IMODE((extracted / "source/tool").stat().st_mode), 0o700)
            self.assertEqual(stat.S_IMODE((extracted / "source/data.jar").stat().st_mode), 0o600)
            self.assertEqual(stat.S_IMODE((extracted / "source").stat().st_mode), 0o700)

    def test_content_change_changes_archive_digest(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-archive-content-") as raw:
            root = Path(raw)
            first_release, first_manifest = create_release(root, tool=b"first\n")
            first = pack_fixture(first_release, first_manifest, root / "first")
            shutil.rmtree(first_release)
            second_release, second_manifest = create_release(root, tool=b"second\n")
            second = pack_fixture(second_release, second_manifest, root / "second")
            self.assertNotEqual(first["archiveSha256"], second["archiveSha256"])

    def test_wrong_release_mode_and_changed_sidecar_fail_without_output(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-archive-input-") as raw:
            root = Path(raw)
            release, manifest = create_release(root)
            (release / "source/tool").chmod(0o600)
            with self.assertRaisesRegex(RuntimeError, "path/type/mode"):
                pack_fixture(release, manifest, root / "wrong-mode")
            self.assertFalse((root / "wrong-mode").exists())
            (release / "source/tool").chmod(0o700)
            (release / "artifact-manifest.json.sha256").write_text("0" * 64 + "\n")
            with self.assertRaisesRegex(RuntimeError, "checksum sidecar"):
                pack_fixture(release, manifest, root / "wrong-sidecar")
            self.assertFalse((root / "wrong-sidecar").exists())

    def test_existing_pack_and_extract_targets_are_unchanged(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-archive-collision-") as raw:
            root = Path(raw)
            release, manifest = create_release(root)
            existing = root / "existing"
            existing.mkdir(mode=0o700)
            marker = existing / "marker"
            marker.write_text("preserve\n")
            with self.assertRaisesRegex(RuntimeError, "existing handoff"):
                pack_fixture(release, manifest, existing)
            self.assertEqual(marker.read_text(), "preserve\n")

            handoff = root / "handoff"
            pack_fixture(release, manifest, handoff)
            common = {
                "repository": REPOSITORY,
                "sourceCommit": COMMIT,
                "productVersion": VERSION,
                "outerArtifactName": common_arguments()["outer_artifact_name"],
                "workflowEvent": "push",
                "workflowRef": "refs/heads/main",
                "workflowRunId": RUN_ID,
                "workflowRunAttempt": ATTEMPT,
                "producerJob": JOB,
            }
            with fixture_owners(manifest):
                inspection = ARCHIVE.inspect_handoff(handoff, common)
                try:
                    with self.assertRaisesRegex(RuntimeError, "existing extraction"):
                        ARCHIVE.extract_inspection(inspection, existing)
                finally:
                    os.close(inspection["archiveFd"])
            self.assertEqual(marker.read_text(), "preserve\n")

    def test_cleanup_refuses_replacement_inode(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-archive-cleanup-") as raw:
            root = Path(raw)
            owned = root / "owned"
            owned.mkdir(mode=0o700)
            owned_identity = ARCHIVE.inode_identity(owned.stat())
            moved = root / "moved"
            owned.rename(moved)
            owned.mkdir(mode=0o700)
            marker = owned / "marker"
            marker.write_text("replacement\n")
            with self.assertRaisesRegex(RuntimeError, "replaced staging"):
                ARCHIVE.remove_owned_directory(owned, owned_identity)
            self.assertEqual(marker.read_text(), "replacement\n")

    def test_raw_header_path_and_type_mutations_fail(self):
        valid = custom_header("top")
        cases = {
            "absolute": custom_header("/absolute"),
            "empty": custom_header(""),
            "dot": custom_header("top/."),
            "dotdot": custom_header("top/../escape"),
            "backslash": custom_header("top\\ambiguous"),
            "overlong": custom_header("prefix/" + "x" * 94),
            "link": custom_header("top/link", tarfile.SYMTYPE, linkname="target"),
            "hardlink": custom_header("top/link", tarfile.LNKTYPE, linkname="target"),
            "character": custom_header("top/device", tarfile.CHRTYPE, devmajor=1, devminor=3),
            "pax": custom_header("top/pax", tarfile.XHDTYPE),
            "gnu": custom_header("top/gnu", tarfile.GNUTYPE_LONGNAME),
            "sparse": custom_header("top/sparse", tarfile.GNUTYPE_SPARSE),
            "wrong-owner": custom_header("top", uid=1),
            "wrong-mtime": custom_header("top", mtime=1),
        }
        non_utf8 = bytearray(valid)
        non_utf8[0] = 0xFF
        cases["non-utf8"] = checksum_header(non_utf8)
        for name, header in cases.items():
            with self.subTest(name=name):
                with self.assertRaises(RuntimeError):
                    inspect_raw(raw_archive((header, b"")))

    def test_raw_duplicate_order_padding_checksum_and_bounds_fail(self):
        top = custom_header("top")
        file_header = custom_header("top/file", tarfile.REGTYPE, 0o600, 1)
        cases = {
            "duplicate": raw_archive((top, b""), (top, b"")),
            "order": raw_archive((custom_header("z"), b""), (custom_header("a"), b"")),
            "truncated": raw_archive((file_header, b"x"))[:-700],
            "checksum": bytes([top[0] ^ 1]) + top[1:] + bytes(1024),
            "trailing": raw_archive((top, b""), trailing=b"extra"),
            "oversized": raw_archive(
                (custom_header("top/file", tarfile.REGTYPE, 0o600,
                               ARCHIVE.TREE_LIMITS.max_file_bytes + 1), b"")
            ),
        }
        padding = bytearray(raw_archive((file_header, b"x")))
        padding[513] = 1
        cases["padding"] = bytes(padding)
        for name, value in cases.items():
            with self.subTest(name=name):
                with self.assertRaises(RuntimeError):
                    inspect_raw(value)

    def test_outer_missing_extra_wrong_descriptor_and_wrong_mode_fail(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-archive-outer-") as raw:
            root = Path(raw)
            release, manifest = create_release(root)
            common = {
                "repository": REPOSITORY,
                "sourceCommit": COMMIT,
                "productVersion": VERSION,
                "outerArtifactName": common_arguments()["outer_artifact_name"],
                "workflowEvent": "push",
                "workflowRef": "refs/heads/main",
                "workflowRunId": RUN_ID,
                "workflowRunAttempt": ATTEMPT,
                "producerJob": JOB,
            }

            missing = root / "missing"
            pack_fixture(release, manifest, missing)
            (missing / ARCHIVE.DESCRIPTOR_NAME).unlink()
            with fixture_owners(manifest), self.assertRaises(RuntimeError):
                ARCHIVE.inspect_handoff(missing, common)

            extra = root / "extra"
            pack_fixture(release, manifest, extra)
            (extra / "undeclared").write_text("extra\n")
            (extra / "undeclared").chmod(0o600)
            with fixture_owners(manifest), self.assertRaises(RuntimeError):
                ARCHIVE.inspect_handoff(extra, common)

            descriptor_extra = root / "descriptor-extra"
            pack_fixture(release, manifest, descriptor_extra)
            path = descriptor_extra / ARCHIVE.DESCRIPTOR_NAME
            value = json.loads(path.read_text())
            value["extra"] = True
            path.write_bytes(ARCHIVE.canonical_json(value))
            with fixture_owners(manifest), self.assertRaises(RuntimeError):
                ARCHIVE.inspect_handoff(descriptor_extra, common)

            wrong_mode = root / "wrong-mode"
            pack_fixture(release, manifest, wrong_mode)

            def mutate(raw_value):
                value = bytearray(raw_value)
                # The first source file is data.jar. Rewrite only its canonical mode field.
                offset = 0
                while offset < len(value):
                    block = bytes(value[offset:offset + 512])
                    if block == bytes(512):
                        break
                    item = tarfile.TarInfo.frombuf(block, encoding="utf-8", errors="strict")
                    if item.name.endswith("/source/data.jar"):
                        value[offset:offset + 512] = custom_header(
                            item.name, tarfile.REGTYPE, 0o700, item.size
                        )
                        return bytes(value)
                    offset += 512 + ((item.size + 511) // 512) * 512
                raise AssertionError("fixture archive member missing")

            rewrite_handoff(wrong_mode, mutate)
            with fixture_owners(manifest), self.assertRaisesRegex(RuntimeError, "closure differs"):
                ARCHIVE.inspect_handoff(wrong_mode, common)

    def test_wrong_commit_repository_and_artifact_name_fail(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-archive-facts-") as raw:
            root = Path(raw)
            release, manifest = create_release(root)
            handoff = root / "handoff"
            pack_fixture(release, manifest, handoff)
            common = {
                "repository": REPOSITORY,
                "sourceCommit": COMMIT,
                "productVersion": VERSION,
                "outerArtifactName": common_arguments()["outer_artifact_name"],
                "workflowEvent": "push",
                "workflowRef": "refs/heads/main",
                "workflowRunId": RUN_ID,
                "workflowRunAttempt": ATTEMPT,
                "producerJob": JOB,
            }
            for field, changed in (
                ("repository", "other/repository"),
                ("sourceCommit", "b" * 40),
                ("outerArtifactName", "wrong-artifact"),
            ):
                expected = dict(common)
                expected[field] = changed
                with self.subTest(field=field), fixture_owners(manifest), self.assertRaises(
                    RuntimeError
                ):
                    ARCHIVE.inspect_handoff(handoff, expected)

    def test_strict_sidecar_rejects_alternate_spelling(self):
        digest = "b" * 64
        for value in (
            f"{digest} archive.tar\n",
            f"{digest.upper()}  archive.tar\n",
            f"{digest}  ./archive.tar\n",
            f"{digest}  archive.tar\nextra\n",
        ):
            with self.subTest(value=value):
                with self.assertRaises(RuntimeError):
                    ARCHIVE.strict_sidecar(value.encode("ascii"), "archive.tar", digest)


if __name__ == "__main__":
    unittest.main(verbosity=2)
