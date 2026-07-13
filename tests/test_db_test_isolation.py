#!/usr/bin/env python3
"""Regression tests for deterministic Cargo test-harness selection."""
import os
import subprocess
import tempfile
import textwrap
import unittest
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check-db-test-isolation.sh"


class IsolationHarnessSelectionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.log = self.directory / "invocations.log"
        self.harness = self.make_executable("selected-harness", """
            import os
            import sys
            from pathlib import Path
            if "--list" in sys.argv:
                if os.environ.get("EMPTY_FILTER") != sys.argv[1]:
                    print(f"tests::{sys.argv[1]}::case: test")
                raise SystemExit(0)
            with Path(os.environ["INVOCATION_LOG"]).open("a", encoding="utf-8") as log:
                log.write(sys.argv[1] + "\\n")
        """)
        self.other = self.make_executable("other-harness", "raise SystemExit(0)")
        self.make_fake_cargo()
        self.decoys = []
        deps = ROOT / "target/debug/deps"
        deps.mkdir(parents=True, exist_ok=True)
        for _ in range(2):
            path = deps / f"lkjmc_daemon-{uuid.uuid4().hex}"
            path.write_text(
                "#!/bin/sh\nprintf 'decoy\\n' >>\"$INVOCATION_LOG\"\nexit 97\n",
                encoding="utf-8",
            )
            path.chmod(0o755)
            self.decoys.append(path)
        self.addCleanup(self.remove_decoys)

    def make_executable(self, name, body):
        path = self.directory / name
        source = "#!/usr/bin/env python3\n" + textwrap.dedent(body).lstrip()
        path.write_text(source, encoding="utf-8")
        path.chmod(0o755)
        return path

    def make_fake_cargo(self):
        self.make_executable("cargo", """
            import json
            import os
            import sys
            from pathlib import Path
            if "--no-run" in sys.argv and "--message-format=json" in sys.argv:
                count = int(os.environ.get("ARTIFACT_COUNT", "1"))
                for index in range(count):
                    executable = os.environ[
                        "FAKE_HARNESS" if index == 0 else "OTHER_HARNESS"
                    ]
                    print(json.dumps({
                        "reason": "compiler-artifact",
                        "manifest_path": str(Path("crates/lkjmc-daemon/Cargo.toml").resolve()),
                        "target": {"name": "lkjmc-daemon", "kind": ["bin"]},
                        "profile": {"test": True},
                        "executable": executable,
                    }))
                raise SystemExit(0)
            if "lkjmc-store" in sys.argv:
                raise SystemExit(0)
            raise SystemExit(91)
        """)

    def remove_decoys(self):
        for path in self.decoys:
            path.unlink(missing_ok=True)

    def run_script(self, **changes):
        environment = os.environ.copy()
        environment.update({
            "PATH": f"{self.directory}:{environment['PATH']}",
            "LKJMC_STORE_TEST_DATABASE_URL": "postgres://fixture.invalid/test",
            "FAKE_HARNESS": str(self.harness),
            "OTHER_HARNESS": str(self.other),
            "INVOCATION_LOG": str(self.log),
        })
        environment.update(changes)
        return subprocess.run(
            [str(SCRIPT)], cwd=ROOT, env=environment,
            capture_output=True, text=True, check=False,
        )

    def test_decoy_hashed_executables_are_never_run(self):
        result = self.run_script()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        invocations = self.log.read_text(encoding="utf-8").splitlines()
        expected = {
            "deadline_route_tests", "timeout_outcome_pass",
            "status_commands_share_bounded_pool",
        }
        self.assertEqual(len(invocations), 6)
        self.assertEqual(set(invocations), expected)
        self.assertNotIn("decoy", invocations)

    def test_unknown_filter_fails_before_concurrent_execution(self):
        result = self.run_script(EMPTY_FILTER="timeout_outcome_pass")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no tests matched filter: timeout_outcome_pass", result.stderr)
        self.assertFalse(self.log.exists())

    def test_zero_or_ambiguous_metadata_fails(self):
        for count in ("0", "2"):
            with self.subTest(count=count):
                result = self.run_script(ARTIFACT_COUNT=count)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(f"found {count}", result.stderr)
                self.assertFalse(self.log.exists())


if __name__ == "__main__":
    unittest.main()
