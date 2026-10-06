import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("ci_failure_diagnostics", ROOT / "scripts/ci_failure_diagnostics.py")
DIAGNOSTICS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(DIAGNOSTICS)
COMMIT = "a" * 40
RUN = ".local/ux/real-012345abcdef"


class FailureDiagnostics(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def put(self, relative, value):
        target = self.root / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(value if isinstance(value, bytes) else value.encode())
        return target

    def acceptance(self, commit=COMMIT, status="failed"):
        self.put(".local/ci/reports/acceptance.json", json.dumps({"commit": commit, "status": status,
                                                              "started": 10, "finished": 20}))

    def network_manifest(self, run=RUN, tag="abcdef", started=15, **changes):
        manifest = {"schema": 1, "fixture_id": run.rsplit("real-", 1)[1],
                    "network_id": tag, "started_at": started,
                    "logs": [f".local/game/network-{tag}-{role}.log" for role in DIAGNOSTICS.NETWORK_ROLES]}
        manifest.update(changes)
        self.put(run + "/network-logs.json", json.dumps(manifest))
        return manifest

    def test_raw_network_stack_is_bound_to_current_fixture_and_redacted(self):
        self.acceptance()
        manifest = self.network_manifest()
        self.put(".local/game/official-token", "official-fixture-secret")
        for relative in manifest["logs"]:
            self.put(relative, "Accessing poi chunk off-main\n\tat net.minecraft.PoiManager.get(PoiManager.java:42)\n"
                     "\tat com.example.ChunkWorker.run(ChunkWorker.java:123)\nofficial-fixture-secret\n"
                     "Authorization: Bearer arbitrary-secret\n")
        stale = self.network_manifest(".local/ux/real-fedcba543210", tag="123456", started=9)
        for relative in stale["logs"]:
            self.put(relative, "unrelated-stale-network")
        self.put(".local/game/network-fedcba-official.log", "unowned-current-network")
        self.put(".local/game/official/logs/latest.log", "not-an-owned-network-log")
        report = DIAGNOSTICS.collect(self.root, COMMIT, {})
        self.assertEqual([entry["path"] for entry in report["files"]], manifest["logs"])
        encoded = DIAGNOSTICS.serialize(report).decode()
        self.assertIn("PoiManager.java:42", encoded)
        self.assertIn("ChunkWorker.java:123", encoded)
        for excluded in ["official-fixture-secret", "arbitrary-secret", "unrelated-stale-network",
                         "unowned-current-network", "not-an-owned-network-log"]:
            self.assertNotIn(excluded, encoded)

    def test_network_manifest_requires_exact_identity_paths_and_failed_job_time(self):
        self.acceptance()
        manifest = self.network_manifest()
        for relative in manifest["logs"]:
            self.put(relative, "must-not-be-exported")
        for changes in [{"fixture_id": "fedcba543210"}, {"network_id": "../bad"},
                        {"logs": [".local/game/official/logs/latest.log"]},
                        {"logs": list(reversed(manifest["logs"]))},
                        {"started_at": 9}, {"started_at": 21}, {"started_at": True},
                        {"started_at": float("nan")}, {"started_at": 10 ** 1000},
                        {"schema": 2}, {"schema": True}]:
            with self.subTest(changes=changes):
                self.network_manifest(**changes)
                self.assertEqual(DIAGNOSTICS.collect(self.root, COMMIT, {})["files"], [])
        for raw in ["not-json", "[]", "x" * (DIAGNOSTICS.MAX_SECRET + 1)]:
            self.put(RUN + "/network-logs.json", raw)
            self.assertEqual(DIAGNOSTICS.collect(self.root, COMMIT, {})["files"], [])
        self.network_manifest()
        (self.root / ".local/ci/reports/acceptance.json").unlink()
        self.assertEqual(DIAGNOSTICS.collect(self.root, COMMIT, {})["files"], [])

    def test_raw_network_and_manifest_links_are_refused(self):
        self.acceptance()
        manifest = self.network_manifest()
        outside = self.put("outside.log", "must-not-be-exported")
        for relative, hardlink in zip(manifest["logs"], [False, True, False]):
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            if hardlink:
                os.link(outside, target)
            else:
                target.symlink_to(outside)
        self.assertEqual(DIAGNOSTICS.collect(self.root, COMMIT, {})["files"], [])
        for relative in manifest["logs"]:
            (self.root / relative).unlink()
            self.put(relative, "owned-log")
        manifest_path = self.root / RUN / "network-logs.json"
        original = self.put("manifest-original.json", manifest_path.read_bytes())
        manifest_path.unlink()
        manifest_path.symlink_to(original)
        self.assertEqual(DIAGNOSTICS.collect(self.root, COMMIT, {})["files"], [])
        manifest_path.unlink()
        self.network_manifest()
        (self.root / ".local/game").rename(self.root / "game-original")
        (self.root / ".local/game").symlink_to(self.root / "game-original", target_is_directory=True)
        # Redaction sources beneath a replaced game root fail closed as well.
        with self.assertRaises(OSError):
            DIAGNOSTICS.collect(self.root, COMMIT, {})

    def test_raw_network_tails_share_existing_byte_and_file_limits(self):
        self.acceptance()
        manifest = self.network_manifest()
        content = b"old stack\n" + b"\x00\xff\n" * 100000 + b"FINAL STACK FRAME\n"
        for relative in manifest["logs"]:
            self.put(relative, content)
        for index in range(12):
            run = ".local/ux/real-" + format(index, "012x")
            for name in DIAGNOSTICS.FIXTURE_LOGS:
                self.put(run + "/" + name, content)
        # Keep the current fixture among the bounded recent run set.
        os.utime(self.root / RUN, None)
        report = DIAGNOSTICS.collect(self.root, COMMIT, {})
        self.assertTrue(set(manifest["logs"]) <= {entry["path"] for entry in report["files"]})
        self.assertLessEqual(len(DIAGNOSTICS.serialize(report)), DIAGNOSTICS.MAX_REPORT)
        self.assertLessEqual(len(report["files"]), DIAGNOSTICS.MAX_FILES)
        self.assertTrue(all(entry["truncated"] for entry in report["files"]))
        self.assertTrue(all(len(entry["tail"].encode()) <= DIAGNOSTICS.MAX_TAIL for entry in report["files"]))

    def test_assertion_logs_survive_without_exporting_fixture_sessions_or_config(self):
        self.acceptance()
        values = ["ci-job-secret-value", "fixture-browser-session-value", "fixture-csrf-value",
                  "fixture-cookie-value", "fixture-official-token-value", "fixture-forwarding-value"]
        self.put(RUN + "/browser-session.json", json.dumps({"token": values[1], "csrf": values[2],
                  "cookies": [{"name": "session", "value": values[3]}]}))
        self.put(".local/game/official-token", values[4] + "\n")
        self.put(".local/game/forwarding-secret", values[5])
        self.put(".local/dev.json", '{"session_secret":"excluded-config-secret"}')
        self.put(RUN + "/core.log", "excluded-core-log")
        self.put(".local/ci/private/browser-integration.log", "ASSERT expected Play\n" + "\n".join(values))
        self.put(RUN + "/game-expeditions.log", "ASSERT journal persisted\nCookie: arbitrary-session=unknown-cookie\nAuthorization: Bearer unknown-header\npassword='unknown-password'\n{\"credential\":\"unknown-credential\"}\npostgres://username:unknown-database-password@127.0.0.1/test\n")
        report = DIAGNOSTICS.collect(self.root, COMMIT, {"CI_JOB_TOKEN": values[0], "EMPTY_TOKEN": ""})
        encoded = DIAGNOSTICS.serialize(report).decode()
        self.assertIn("ASSERT expected Play", encoded)
        self.assertIn("ASSERT journal persisted", encoded)
        for value in [*values, "unknown-cookie", "unknown-header", "unknown-password", "unknown-credential", "unknown-database-password", "excluded-config-secret", "excluded-core-log"]:
            self.assertNotIn(value, encoded)
        self.assertEqual({entry["path"] for entry in report["files"]},
                         {".local/ci/private/browser-integration.log", RUN + "/game-expeditions.log"})
        self.assertTrue(all(not Path(entry["path"]).is_absolute() for entry in report["files"]))

    def test_fixture_failure_before_integration_retains_bounded_redacted_assertions(self):
        self.put(".local/ci/reports/acceptance.json", json.dumps({
            "commit": COMMIT, "status": "failed", "started": 10, "finished": 20,
            "checks": [{"name": "browser-fixture", "passed": False}],
        }))
        secret = "fixture-ci-token-value"
        self.put(".local/ci/private/browser-fixture.log", "earlier fixture output\n" * 3000 +
                 "FAIL page heading: expected Play\n" + secret + "\n" +
                 "Authorization: Bearer unknown-fixture-header\n" +
                 "Cookie: mock=unknown-fixture-cookie\n" +
                 "csrf='unknown-fixture-csrf'\n" + "FINAL FIXTURE ASSERTION\n")
        self.put(".local/browser-results/failed-fixture/error-context.md", "excluded page snapshot")
        self.put(".local/browser-results/failed-fixture/test-failed-1.png", b"excluded screenshot")
        self.put(".local/ci/private/web-state.log", "excluded earlier stage output")
        self.assertFalse((self.root / ".local/ux").exists())
        report = DIAGNOSTICS.collect(self.root, COMMIT, {"CI_JOB_TOKEN": secret})
        self.assertTrue(report["acceptance_receipt_present"])
        self.assertEqual([entry["path"] for entry in report["files"]],
                         [".local/ci/private/browser-fixture.log"])
        entry = report["files"][0]
        self.assertTrue(entry["truncated"])
        self.assertLessEqual(len(entry["tail"].encode()), DIAGNOSTICS.MAX_TAIL)
        self.assertIn("expected Play", entry["tail"])
        self.assertIn("FINAL FIXTURE ASSERTION", entry["tail"])
        self.assertIn("[REDACTED]", entry["tail"])
        encoded = DIAGNOSTICS.serialize(report)
        self.assertLessEqual(len(encoded), DIAGNOSTICS.MAX_REPORT)
        for value in [secret, "unknown-fixture-header", "unknown-fixture-cookie", "unknown-fixture-csrf",
                      "excluded page snapshot", "excluded screenshot", "excluded earlier stage output"]:
            self.assertNotIn(value, encoded.decode())
        DIAGNOSTICS.write_report(self.root, report)
        self.assertEqual((self.root / ".local/ci/reports/failure-diagnostics.json").stat().st_mode & 0o777,
                         0o600)

    def test_symlink_file_and_directory_and_hardlink_cannot_escape_workspace(self):
        outside = self.root.parent / (self.root.name + "-outside-secret")
        outside.write_text("must-not-be-exported")
        self.addCleanup(outside.unlink)
        link = self.root / ".local/ci/private/browser-integration.log"
        link.parent.mkdir(parents=True)
        link.symlink_to(outside)
        hardlink = self.root / ".local/ci/private/game-protocol.log"
        os.link(outside, hardlink)
        fixture_link = self.root / ".local/ci/private/browser-fixture.log"
        fixture_link.symlink_to(outside)
        ux = self.root / ".local/ux"
        ux.mkdir()
        (ux / "real-012345abcdef").symlink_to(self.root.parent, target_is_directory=True)
        report = DIAGNOSTICS.collect(self.root, COMMIT, {})
        self.assertEqual(report["files"], [])
        (self.root / ".local/ci/private").rename(self.root / "private-original")
        (self.root / ".local/ci/private").symlink_to(self.root / "private-original", target_is_directory=True)
        self.assertEqual(DIAGNOSTICS.collect(self.root, COMMIT, {})["files"], [])

    def test_mismatched_or_successful_acceptance_refuses_collection(self):
        self.put(".local/ci/private/browser-integration.log", "must not be returned")
        for commit, status in [("b" * 40, "failed"), (COMMIT, "passed")]:
            self.acceptance(commit, status)
            with self.subTest(commit=commit, status=status), self.assertRaises(ValueError):
                DIAGNOSTICS.collect(self.root, COMMIT, {})

    def test_report_byte_limit_handles_control_bytes_invalid_utf8_and_many_runs(self):
        self.acceptance()
        content = b"old assertion\n" + (b"\x00\xff\n" * 100000) + b"FINAL ASSERTION\n"
        for index in range(12):
            run = ".local/ux/real-" + format(index, "012x")
            for name in DIAGNOSTICS.FIXTURE_LOGS:
                self.put(run + "/" + name, content)
        for name in DIAGNOSTICS.OUTER_LOGS:
            self.put(".local/ci/private/" + name, content)
        report = DIAGNOSTICS.collect(self.root, COMMIT, {})
        encoded = DIAGNOSTICS.serialize(report)
        self.assertLessEqual(len(encoded), DIAGNOSTICS.MAX_REPORT)
        self.assertGreater(len(report["files"]), 0)
        self.assertLessEqual(len(report["files"]), DIAGNOSTICS.MAX_FILES)
        self.assertTrue(all("FINAL ASSERTION" in entry["tail"] for entry in report["files"]))
        self.assertTrue(all(entry["truncated"] for entry in report["files"]))
        self.assertTrue(all(len(entry["tail"].encode()) <= DIAGNOSTICS.MAX_TAIL for entry in report["files"]))
        self.assertLessEqual(len({entry["path"].split("/")[3] for entry in report["files"] if "/ux/" in entry["path"]}), DIAGNOSTICS.MAX_RUNS)

    def test_partial_tail_assignment_is_discarded_and_output_symlink_is_refused(self):
        self.put(".local/ci/private/browser-integration.log", "password=" + "x" * 40000 + "\nSAFE ASSERTION\n")
        report = DIAGNOSTICS.collect(self.root, COMMIT, {})
        self.assertEqual(report["files"][0]["tail"], "SAFE ASSERTION\n")
        DIAGNOSTICS.write_report(self.root, report)
        target = self.root / ".local/ci/reports/failure-diagnostics.json"
        self.assertEqual(target.stat().st_mode & 0o777, 0o600)
        self.assertLessEqual(target.stat().st_size, DIAGNOSTICS.MAX_REPORT)
        target.unlink()
        outside = self.put("untouched.txt", "original")
        target.symlink_to(outside)
        with self.assertRaises(OSError):
            DIAGNOSTICS.write_report(self.root, report)
        self.assertEqual(outside.read_text(), "original")

    def test_fixture_token_crossing_tail_boundary_is_redacted_before_clipping(self):
        secret = "fixture-boundary-secret-" * 8
        self.put(RUN + "/browser-session.json", json.dumps({"token": secret}))
        body = "old line" * 20000 + "\nassertion=" + secret + " tail " + "f" * (DIAGNOSTICS.MAX_TAIL - len(secret) // 2) + "\nFINAL ASSERTION\n"
        self.assertIn(secret[-32:], body.encode()[-DIAGNOSTICS.MAX_TAIL:].decode())
        self.put(RUN + "/browser-integration.log", body)
        report = DIAGNOSTICS.collect(self.root, COMMIT, {})
        tail = report["files"][0]["tail"]
        self.assertNotIn(secret[-32:], tail)
        self.assertIn("[REDACTED]", tail)
        self.assertIn("FINAL ASSERTION", tail)

    def test_missing_receipt_allows_only_explicit_earlier_failure_subset(self):
        self.put(".local/ci/private/browser-integration.log", "actual assertion")
        self.put(".local/ci/private/postgres-init.log", "not allowlisted")
        self.put(".local/ci/private/environment.log", "not allowlisted")
        report = DIAGNOSTICS.collect(self.root, COMMIT, {})
        self.assertFalse(report["acceptance_receipt_present"])
        self.assertEqual([entry["path"] for entry in report["files"]], [".local/ci/private/browser-integration.log"])


if __name__ == "__main__":
    unittest.main()
