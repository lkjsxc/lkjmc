#!/usr/bin/env python3
import os
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import network_adoption_checks as checks

class NetworkAdoptionCheckerTests(unittest.TestCase):
    def test_exact_probe_inventory(self):
        self.assertEqual(checks.PROBES, [
            "network-path-single", "inspect-apply-pass", "reapply-pass",
            "partial-failure-pass", "local-kube-capabilities", "config-example-pass",
        ])

    def test_current_inventory_passes(self):
        self.assertEqual(checks.source_errors(ROOT), [])

    def test_legacy_path_mutation_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")
        def mutated(path):
            value = original(path)
            if path.name == "install.sh":
                value += "\n# javaEntry legacy launch\n"
            return value
        errors = checks.source_errors(ROOT, mutated)
        self.assertTrue(any("legacy network path" in error for error in errors))

    def test_missing_closed_member_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")
        def mutated(path):
            value = original(path)
            if path.name == "network_intent.rs" and "config" in str(path):
                value = value.replace("pub capabilities:", "pub removed_capabilities:")
            return value
        errors = checks.source_errors(ROOT, mutated)
        self.assertIn("network intent member missing: capabilities", errors)

    def test_database_probes_cannot_skip(self):
        environment = os.environ.copy()
        environment.pop("LKJMC_STORE_TEST_DATABASE_URL", None)
        for probe in sorted(checks.DB_PROBES):
            result = subprocess.run(
                [str(ROOT / "scripts/check-network-adoption.py"), "--probe", probe],
                cwd=ROOT, env=environment, capture_output=True, text=True, check=False,
            )
            self.assertEqual(result.returncode, 2)
            self.assertIn("LKJMC_STORE_TEST_DATABASE_URL is required", result.stdout)

if __name__ == "__main__":
    unittest.main()
