#!/usr/bin/env python3
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import data_workflow_checks as checks

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

    def test_transaction_discovery_uses_rust_syntax(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates/lkjmc-store/src/example.rs"
            source.parent.mkdir(parents=True)
            source.write_text(
                '// client.transaction()\nconst NOTE: &str = ".transaction()";\n',
                encoding="utf-8",
            )
            config = root / "config/data-workflows.json"
            config.parent.mkdir()
            config.write_text(
                json.dumps({"classifications": [], "schema": "lkjmc-data-workflows-one"}),
                encoding="utf-8",
            )
            with mock.patch.object(checks, "ROOT", root):
                self.assertEqual(checks.inventory_errors(), [])
                source.write_text(
                    "fn persist(client: &mut Client) { let _tx = client.transaction(); }",
                    encoding="utf-8",
                )
                self.assertEqual(
                    checks.inventory_errors(),
                    ["unclassified transaction owner: crates/lkjmc-store/src/example.rs"],
                )


if __name__ == "__main__":
    unittest.main()
