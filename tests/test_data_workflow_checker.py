#!/usr/bin/env python3
import importlib.util
import os
import subprocess
import sys
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import data_workflow_mutations as mutations

SPEC = importlib.util.spec_from_file_location(
    "check_data_workflows", ROOT / "scripts/check-data-workflows.py"
)
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


class DataWorkflowCheckerTests(unittest.TestCase):
    def test_named_database_probe_requires_url(self):
        environment = os.environ.copy()
        environment.pop("LKJMC_STORE_TEST_DATABASE_URL", None)
        result = subprocess.run(
            [str(ROOT / "scripts/check-data-workflows.py"), "--probe", "fencing-pass"],
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("LKJMC_STORE_TEST_DATABASE_URL is required", result.stderr)

    def test_database_skip_is_aggregate_only(self):
        with mock.patch.object(runner, "database_ready", return_value=False):
            self.assertEqual(runner.run("fencing-pass", allow_database_skip=True), 0)
        result = subprocess.run(
            [
                str(ROOT / "scripts/check-data-workflows.py"),
                "--probe",
                "fencing-pass",
                "--allow-database-skip",
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires --all", result.stderr)

    def test_old_workflow_mutations_are_rejected(self):
        self.assertEqual(mutations.old_path_errors(), [])
        original = mutations.read

        def injected(path):
            value = original(path)
            if path.endswith("command_registrations.rs"):
                return value + '\n"player.transfer.saved"'
            return value

        with mock.patch.object(mutations, "read", side_effect=injected):
            self.assertIn(
                "audit-only transfer command remains registered",
                mutations.old_path_errors(),
            )


if __name__ == "__main__":
    unittest.main()
