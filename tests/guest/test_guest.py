"""Filesystem fault tests. Does not run Incus, systemd, Java, or a production guest."""
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import uuid
import zipfile

spec = importlib.util.spec_from_file_location(
    "guest", Path(__file__).resolve().parents[2] / "ops/guest/guest.py"
)
guest = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guest)


class PowerLoss(BaseException):
    pass


class GuestFiles(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="lkjmc-guest-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for key, value in {
            "ROOT": self.root / "server",
            "CONTROL": self.root / "control",
            "FIFO": self.root / "console",
            "CONFIG": self.root / "etc/lkjmc/server.json",
        }.items():
            self.enterContext(patch.object(guest, key, value))
        guest.ROOT.mkdir()
        (guest.CONTROL / "incoming").mkdir(parents=True)
        self.enterContext(patch.object(guest, "server_stopped"))
        self.enterContext(patch.object(guest, "owner"))
        self.enterContext(patch.object(guest.os, "sync"))

    def test_runtime_is_readable_under_private_bootstrap_umask_without_exposing_credentials(self):
        old = os.umask(0o077)
        try:
            guest.runtime_configuration(25, 2048, 'paper')
            token = guest.CONFIG.parent / 'role-token'
            token.write_text('private fixture')
            self.assertEqual(token.stat().st_mode & 0o777, 0o600)
            original = guest.CONFIG.read_bytes()
            guest.runtime_configuration(17, 4096, 'custom')
            self.assertEqual(guest.CONFIG.read_bytes(), original)
            self.assertEqual(guest.CONFIG.parent.stat().st_mode & 0o777, 0o755)
            self.assertEqual(guest.CONFIG.stat().st_mode & 0o777, 0o644)
            self.assertEqual(token.stat().st_mode & 0o777, 0o600)
        finally:
            os.umask(old)

    def test_runtime_configuration_rejects_symlinks(self):
        guest.CONFIG.parent.mkdir(parents=True)
        target = self.root / 'untouched'
        target.write_text('original')
        guest.CONFIG.symlink_to(target)
        with self.assertRaises(ValueError):
            guest.runtime_configuration(25, 2048, 'paper')
        self.assertEqual(target.read_text(), 'original')

    def archive(self, members, *, tar=False):
        path = self.root / str(uuid.uuid4())
        if tar:
            with tarfile.open(path, "w") as archive:
                for name, data in members:
                    entry = tarfile.TarInfo(name)
                    entry.size = len(data)
                    archive.addfile(entry, io.BytesIO(data))
        else:
            with zipfile.ZipFile(path, "w") as archive:
                for name, data in members:
                    archive.writestr(name, data)
        return path

    def request(self, data=b"new server jar", kind="jar", target="server.jar"):
        artifact = str(uuid.uuid4())
        source = guest.CONTROL / "incoming" / artifact
        source.write_bytes(data)
        return dict(job_id=str(uuid.uuid4()), artifact_id=artifact,
                    sha256=hashlib.sha256(data).hexdigest(), kind=kind,
                    path=target, storage_mib=1024)

    def test_nested_world_zip_and_tar_preserve_bytes(self):
        for tar in (False, True):
            with self.subTest(tar=tar):
                source = self.archive([("./world/level.dat", b"level"),
                                       ("./world/region/r.0.0.mca", b"region")], tar=tar)
                destination = self.root / str(uuid.uuid4())
                destination.mkdir()
                self.assertEqual(guest.extract_world(source, destination, 100), 11)
                self.assertEqual((destination / "level.dat").read_bytes(), b"level")
                self.assertEqual((destination / "region/r.0.0.mca").read_bytes(), b"region")

    def test_archive_cannot_escape_or_exceed_storage(self):
        for name in ("../escaped", "/tmp/escaped", "world/../../escaped", "a\\escaped"):
            for tar in (False, True):
                with self.subTest(name=name, tar=tar):
                    destination = self.root / str(uuid.uuid4())
                    destination.mkdir()
                    with self.assertRaises(ValueError):
                        guest.extract_world(self.archive([(name, b"x")], tar=tar), destination, 100)
        destination = self.root / "quota"
        destination.mkdir()
        with self.assertRaises(ValueError):
            guest.extract_world(self.archive([("level.dat", b"x" * 101)]), destination, 100)
        self.assertFalse((self.root / "escaped").exists())

    def test_archive_links_are_rejected(self):
        for kind in (tarfile.SYMTYPE, tarfile.LNKTYPE, tarfile.CHRTYPE):
            source = self.root / str(uuid.uuid4())
            with tarfile.open(source, "w") as archive:
                entry = tarfile.TarInfo("level.dat")
                entry.type = kind
                entry.linkname = "/etc/passwd"
                archive.addfile(entry)
            destination = self.root / str(uuid.uuid4())
            destination.mkdir()
            with self.assertRaises(ValueError):
                guest.extract_world(source, destination, 100)
        source = self.root / "links.zip"
        with zipfile.ZipFile(source, "w") as archive:
            entry = zipfile.ZipInfo("level.dat")
            entry.create_system = 3
            entry.external_attr = 0o120777 << 16
            archive.writestr(entry, "/etc/passwd")
        with self.assertRaises(ValueError):
            guest.extract_world(source, self.root / "unzip", 100)

    def test_install_rolls_forward_after_each_rename(self):
        for interruption in (1, 2):
            with self.subTest(interruption=interruption):
                target = guest.ROOT / ('world'+str(interruption))
                target.mkdir()
                (target/'level.dat').write_bytes(b'old world')
                source = self.archive([('level.dat', b'new world')])
                request = self.request(source.read_bytes(), 'world', target.name)
                original = guest.rename_new
                calls = 0
                def rename(*args):
                    nonlocal calls
                    original(*args)
                    calls += 1
                    if calls == interruption:
                        raise PowerLoss()
                with patch.object(guest, 'rename_new', rename), self.assertRaises(PowerLoss):
                    guest.install(request)
                self.assertEqual(json.loads(guest.receipt_path(request['job_id']).read_text())['phase'], 'prepared')
                result = guest.install(request)
                self.assertEqual(result['effect'], 'committed')
                self.assertEqual((target/'level.dat').read_bytes(), b'new world')
                self.assertEqual(guest.install(request), result)
                self.assertEqual(list(guest.ROOT.glob('.lkjmc-before-*')), [])
                with self.assertRaises(ValueError):
                    guest.install({**request, 'path': 'changed'})

    def test_bad_world_and_checksum_do_not_replace_existing_world(self):
        target = guest.ROOT / "world"
        target.mkdir()
        (target / "level.dat").write_bytes(b"old level")
        source = self.archive([("../bad", b"x")])
        request = self.request(source.read_bytes(), "world", "world")
        with self.assertRaises(ValueError):
            guest.install(request)
        self.assertFalse(guest.receipt_path(request["job_id"]).exists())
        self.assertEqual((target / "level.dat").read_bytes(), b"old level")
        request = self.request()
        request["sha256"] = "0" * 64
        with self.assertRaises(ValueError):
            guest.install(request)
        self.assertFalse((guest.ROOT / "server.jar").exists())

    def test_committed_install_cleans_displaced_files_after_restart(self):
        target=guest.ROOT / 'world'
        target.mkdir()
        (target/'level.dat').write_bytes(b'old')
        request=self.request(self.archive([('level.dat',b'new')]).read_bytes(),'world','world')
        original=guest.atomic
        def atomic(path,value):
            original(path,value)
            if value['phase']=='committed':raise PowerLoss()
        with patch.object(guest,'atomic',atomic),self.assertRaises(PowerLoss):
            guest.install(request)
        self.assertEqual(len(list(guest.ROOT.glob('.lkjmc-before-*'))),1)
        self.assertEqual(guest.install(request)['effect'],'committed')
        self.assertEqual((target/'level.dat').read_bytes(),b'new')
        self.assertEqual(list(guest.ROOT.glob('.lkjmc-before-*')),[])

    def test_managed_paths_and_symlink_escape_are_rejected(self):
        (guest.ROOT / "outside").symlink_to(self.root)
        for value in ("eula.txt", "lkjmc-control", "../server.jar", "outside/escape", "/root/x", ".hidden"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                guest.checked_path(value)

    def test_console_never_replays_a_line_with_uncertain_delivery(self):
        os.mkfifo(guest.FIFO)
        reader = os.open(guest.FIFO, os.O_RDONLY | os.O_NONBLOCK)
        self.addCleanup(os.close, reader)
        request = dict(job_id=str(uuid.uuid4()), line="say exactly once")
        atomic = guest.atomic

        def crash_before_commit(path, value):
            if value["phase"] == "committed":
                raise PowerLoss()
            atomic(path, value)

        with patch.object(guest, "atomic", crash_before_commit), self.assertRaises(PowerLoss):
            guest.console(request)
        self.assertEqual(os.read(reader, 4096), b"say exactly once\n")
        self.assertEqual(guest.console(request)["effect"], "uncertain")
        self.assertEqual(os.read(reader, 4096), b"")
        with self.assertRaises(ValueError):
            guest.console({**request, "line": "say changed"})


if __name__ == "__main__":
    unittest.main()
