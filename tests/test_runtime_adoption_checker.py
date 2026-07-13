#!/usr/bin/env python3
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import runtime_adoption_checks as checks


class RuntimeAdoptionCheckerTests(unittest.TestCase):
    def test_exact_probe_inventory(self):
        self.assertEqual(checks.PROBES, [
            "runtime-global-mutex-absent",
            "cross-instance-hang-pass",
            "same-instance-race-pass",
            "reconcile-idempotent",
            "effect-crash-recovery",
            "adapter-capability-pass",
            "runtime-load-budget",
        ])

    def test_old_global_mutex_mutation_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "crates/lkjmc-daemon/src"
            source.mkdir(parents=True)
            (source / "app.rs").write_text(
                "use std::sync::{Arc, Mutex};\n"
                "runtime: Arc<dyn RuntimeAdapter>\n"
                "LifecycleCoordinator\n"
                "type Old = Arc<Mutex<Box<dyn RuntimeAdapter>>>;\n",
                encoding="utf-8",
            )
            errors = checks.old_shape_errors(root)
            self.assertTrue(any("daemon-wide runtime mutex" in error for error in errors))

    def test_database_probes_cannot_skip(self):
        environment = os.environ.copy()
        environment.pop("LKJMC_STORE_TEST_DATABASE_URL", None)
        for probe in ["reconcile-idempotent", "effect-crash-recovery"]:
            result = subprocess.run(
                [str(ROOT / "scripts/check-runtime-adoption.py"), "--probe", probe],
                cwd=ROOT, env=environment, capture_output=True, text=True, check=False,
            )
            self.assertEqual(result.returncode, 2)
            self.assertIn("LKJMC_STORE_TEST_DATABASE_URL is required", result.stdout)


if __name__ == "__main__":
    unittest.main()
