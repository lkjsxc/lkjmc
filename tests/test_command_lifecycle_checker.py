#!/usr/bin/env python3
"""Deterministic regressions for lifecycle database prerequisite policy."""
import contextlib
import io
import sys
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

from command_lifecycle_checker import DATABASE_PROBES, run_cli  # noqa: E402


class DatabasePrerequisiteTests(unittest.TestCase):
    def run_checker(self, arguments):
        output = io.StringIO()
        executed = []
        def execute(name):
            executed.append(name)
            return True
        with contextlib.redirect_stdout(output):
            result = run_cli(execute, arguments, environ={})
        return result, output.getvalue(), executed

    def test_named_required_database_probes_fail_without_url(self):
        for probe in ("duplicate-mutations-pass", "timeout-outcome-pass"):
            with self.subTest(probe=probe):
                result, output, executed = self.run_checker(["--probe", probe])
                self.assertNotEqual(result, 0)
                self.assertIn(f"failed {probe}:", output)
                self.assertNotIn("SKIP", output)
                self.assertEqual(executed, [])

    def test_explicit_aggregate_allow_skip_reports_all_database_probes(self):
        result, output, executed = self.run_checker([
            "--all", "--allow-database-skip",
        ])
        self.assertEqual(result, 0)
        for probe in DATABASE_PROBES:
            self.assertIn(f"SKIP {probe}:", output)
        self.assertTrue(executed)


if __name__ == "__main__":
    unittest.main()
