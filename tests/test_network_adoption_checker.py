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

    def test_process_and_pool_regressions_are_wired_to_database_probes(self):
        runner = (ROOT / "scripts/check-network-adoption.py").read_text(encoding="utf-8")
        for test in (
            "size_one_pool_is_available",
            "network_reapply_repairs_killed_owned_proxy",
            "network_apply_denies_unowned_listener",
        ):
            self.assertEqual(runner.count(test), 1)

    def test_superseded_path_mutation_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")
        def mutated(path):
            value = original(path)
            if path.name == "daemon.json.example":
                value = value.replace(
                    '"network": {',
                    '"network": {\n    "javaEntry": "superseded",',
                    1,
                )
            return value
        errors = checks.source_errors(ROOT, mutated)
        self.assertTrue(any("superseded network path" in error for error in errors))

    def test_alternate_compiler_and_compatibility_export_are_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")
        def mutated(path):
            value = original(path)
            if path.as_posix().endswith("lkjmc-core/src/network_intent.rs"):
                value += "\nfn renamed(input: &NetworkConfig) -> NetworkInspection { todo!() }\n"
                value += "pub struct DesiredNetwork;\n"
            return value
        errors = checks.source_errors(ROOT, mutated)
        self.assertTrue(any("compiler inventory" in error for error in errors))
        self.assertTrue(any("compiler export" in error for error in errors))

    def test_alternate_rust_process_path_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")
        def mutated(path):
            value = original(path)
            if path.as_posix().endswith("bootstrap_api/apply.rs"):
                injected = "\nfn renamed() { let _ = std::process::Command::new(\"java\").spawn(); }\n"
                value = value.replace("\n#[cfg(test)]", injected + "\n#[cfg(test)]", 1)
            return value
        errors = checks.source_errors(ROOT, mutated)
        self.assertTrue(any("Rust process entrypoint inventory" in error for error in errors))

    def test_libc_process_api_is_rejected_without_flag_false_positive(self):
        original = lambda path: path.read_text(encoding="utf-8")
        def mutated(path):
            value = original(path)
            if path.as_posix().endswith("bootstrap_api/apply.rs"):
                injected = "\nfn renamed() { unsafe { libc::fork(); } }\n"
                value = value.replace("\n#[cfg(test)]", injected + "\n#[cfg(test)]", 1)
            return value
        errors = checks.source_errors(ROOT, mutated)
        self.assertTrue(any("alternate Rust process path" in error for error in errors))
        self.assertFalse(any("support/bundle" in error for error in errors))

    def test_java_and_shell_launch_mutations_are_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")
        def mutated(path):
            value = original(path)
            if path.name == "AttestationVerifier.java":
                value += "\n// new ProcessBuilder(\"java\").start();\n"
            if path.name == "check-minecraft-smoke.sh":
                value += "\nexec \"$JAVA_HOME/bin/java\" -jar injected.jar\n"
            return value
        errors = checks.source_errors(ROOT, mutated)
        self.assertTrue(any("Java process launch path" in error for error in errors))
        self.assertTrue(any("shell Java launch path" in error for error in errors))

    def test_missing_closed_member_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")
        def mutated(path):
            value = original(path)
            if path.name == "network_intent.rs" and "config" in str(path):
                value = value.replace("pub capabilities:", "pub removed_capabilities:")
            return value
        errors = checks.source_errors(ROOT, mutated)
        self.assertIn("network intent member missing: capabilities", errors)

    def test_placeholder_and_mutable_compose_are_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")
        def mutated(path):
            value = original(path)
            if path.name == "daemon.json.example":
                value = value.replace(
                    '"network": {',
                    '"network": {\n    "testArtifact": "https://example.invalid/file",',
                    1,
                )
            if path.name == "docker-compose.yml":
                value = value.replace("@sha256:", "# mutable-")
            return value
        errors = checks.source_errors(ROOT, mutated)
        self.assertTrue(any("placeholder input" in error for error in errors))
        self.assertIn("Compose PostgreSQL image is not immutable", errors)

    def test_repeated_digest_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")
        def mutated(path):
            value = original(path)
            if path.name == "daemon.json.example":
                value = value.replace(
                    '"network": {',
                    '"network": {\n    "testDigest": "' + "11" * 32 + '",',
                    1,
                )
            return value
        errors = checks.source_errors(ROOT, mutated)
        self.assertTrue(any("repeated digest" in error for error in errors))

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
