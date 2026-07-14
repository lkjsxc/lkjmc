#!/usr/bin/env python3
import os
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import sync_adoption_checks as checks


class SyncAdoptionCheckerTests(unittest.TestCase):
    def mutate(self, relative, old, new):
        target = ROOT / relative

        def override(path):
            source = path.read_text(encoding="utf-8")
            return source.replace(old, new) if path == target else source

        return checks.source_errors(ROOT, override)

    def test_exact_probe_inventory(self):
        self.assertEqual(checks.PROBES, [
            "all-snapshots-revisioned",
            "freshness-bound-pass",
            "reconnect-storm-pass",
            "request-budget-pass",
            "auth-invalidation-pass",
            "shutdown-clean",
            "duplicate-pollers-absent",
        ])

    def test_unrevisioned_snapshot_mutation_is_rejected(self):
        errors = self.mutate(
            "crates/lkjmc-daemon/src/transport/sync.rs",
            '"revision": value.revision',
            '"version": value.revision',
        )
        self.assertTrue(any("revisioned daemon snapshot" in error for error in errors))

    def test_duplicate_poller_mutation_is_rejected(self):
        errors = self.mutate(
            "platforms/jvm/common/src/main/java/com/lkjmc/common/sync/SyncCoordinator.java",
            "scheduler.scheduleWithFixedDelay(this::tick",
            "scheduler.scheduleWithFixedDelay(this::tick\n        scheduler.scheduleWithFixedDelay(this::tick",
        )
        self.assertTrue(any("exactly one feed poller" in error for error in errors))

    def test_unbounded_cache_mutation_is_rejected(self):
        errors = self.mutate(
            "platforms/jvm/common/src/main/java/com/lkjmc/common/sync/SyncCache.java",
            "while (entries.size() > maxEntries || bytes > maxBytes)",
            "while (false)",
        )
        self.assertTrue(any("bounds are incomplete" in error for error in errors))

    def test_surface_policy_mutation_is_rejected(self):
        errors = self.mutate(
            "crates/lkjmc-daemon/src/authz.rs",
            'matches!(self.surface.as_str(), "paper" | "velocity")',
            "true",
        )
        self.assertTrue(any("surface/scope policy" in error for error in errors))

    def test_presence_routing_dependency_mutation_is_rejected(self):
        errors = self.mutate(
            "migrations/047-revisioned-sync.sql",
            "perform sync_touch('routing', 'network');",
            "perform sync_touch('presence', 'network');",
        )
        self.assertTrue(any("presence-to-routing" in error for error in errors))

    def test_payload_validator_mutation_is_rejected(self):
        errors = self.mutate(
            "platforms/jvm/common/src/main/java/com/lkjmc/common/sync/SyncCoordinator.java",
            "SyncPayloadValidator.valid(actual, body.get(\"payload\"))",
            "true",
        )
        self.assertTrue(any("payload validation" in error for error in errors))

    def test_retention_caller_mutation_is_rejected(self):
        errors = self.mutate(
            "crates/lkjmc-daemon/src/transport/server.rs",
            "state.start_maintenance()?;",
            "let _ = state;",
        )
        self.assertTrue(any("retention worker" in error for error in errors))

    def test_current_source_passes(self):
        self.assertEqual(checks.source_errors(ROOT), [])

    def test_database_prerequisite_cannot_skip(self):
        environment = os.environ.copy()
        environment.pop("LKJMC_STORE_TEST_DATABASE_URL", None)
        result = subprocess.run(
            [str(ROOT / "scripts/check-sync-adoption.py"), "--probe", "duplicate-pollers-absent"],
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("LKJMC_STORE_TEST_DATABASE_URL is required", result.stdout)


if __name__ == "__main__":
    unittest.main()
