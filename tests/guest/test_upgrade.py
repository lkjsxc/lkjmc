import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('guest_upgrade', Path(__file__).resolve().parents[2] / 'ops/guest/upgrade.py')
upgrade = importlib.util.module_from_spec(spec)
spec.loader.exec_module(upgrade)


class GuestUpgrade(unittest.TestCase):
    def test_replay_preserves_unrelated_bytes_and_refuses_linked_or_corrupt_inputs(self):
        with tempfile.TemporaryDirectory(prefix='.lkjmc-upgrade-', dir=Path.home()) as name:
            directory = Path(name)
            untouched = directory / 'world-receipt'
            untouched.write_bytes(b'keep')
            files = {'guest.py': {'text': 'new helper', 'sha256': hashlib.sha256(b'new helper').hexdigest(), 'mode': 0o755}}
            upgrade.install(directory, files)
            upgrade.install(directory, files)
            self.assertEqual(untouched.read_bytes(), b'keep')
            self.assertEqual((directory / 'guest.py').read_text(), 'new helper')
            files['guest.py']['sha256'] = '0' * 64
            with self.assertRaises(AssertionError):
                upgrade.install(directory, files)
            files['guest.py']['sha256'] = hashlib.sha256(b'new helper').hexdigest()
            (directory / 'guest.py').unlink()
            (directory / 'guest.py').symlink_to(untouched)
            with self.assertRaises(AssertionError):
                upgrade.install(directory, files)
            self.assertEqual(untouched.read_bytes(), b'keep')
