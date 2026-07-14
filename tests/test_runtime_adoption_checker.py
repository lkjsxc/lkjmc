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

    def test_direct_effect_caller_mutation_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")

        def injected(path):
            text = original(path)
            if path.name == "instance_wake_runtime.rs":
                return text + (
                    "\nfn bypass(state: &State, executor: &dyn RuntimeAdapter) { "
                    "let mut client = state.database_connection(); executor.start(); }\n"
                )
            return text

        errors = checks.old_shape_errors(ROOT, injected)
        self.assertTrue(any("direct runtime effect path changed" in error for error in errors))

    def test_aliased_effect_caller_mutation_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")

        def injected(path):
            text = original(path)
            if path.name == "instance_wake_runtime.rs":
                return text + (
                    "\nfn bypass(state: &AppState) { let engine = state.runtime(); "
                    "let executor = engine; executor.stop(\"hub\", DEADLINE); }\n"
                )
            return text

        errors = checks.old_shape_errors(ROOT, injected)
        self.assertTrue(any("direct runtime effect path changed" in error for error in errors))

    def test_state_effect_caller_mutation_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")

        def injected(path):
            text = original(path)
            if path.name == "instance_wake_runtime.rs":
                return text + "\nfn bypass(state: &AppState) { state.runtime().observe(\"hub\"); }\n"
            return text

        errors = checks.old_shape_errors(ROOT, injected)
        self.assertTrue(any("direct runtime effect path changed" in error for error in errors))

    def test_qualified_effect_caller_mutation_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")

        def injected(path):
            text = original(path)
            if path.name == "instance_wake_runtime.rs":
                return text + "\nfn bypass(a: &dyn RuntimeAdapter) { RuntimeAdapter::status(a); }\n"
            return text

        errors = checks.old_shape_errors(ROOT, injected)
        self.assertTrue(any("direct runtime effect path changed" in error for error in errors))

    def test_runtime_field_effect_caller_is_detected(self):
        source = "impl AppState { fn bypass(&self) { self.runtime.delete(\"hub\"); } }"
        self.assertEqual(checks.runtime_effect_calls(source), ["delete"])

    def test_unrelated_start_method_is_not_a_runtime_effect(self):
        self.assertEqual(checks.runtime_effect_calls("fn f(process: P) { process.start(); }"), [])

    def test_http_adapter_named_adapter_is_benign(self):
        source = "fn serve(adapter: &HttpAdapter) { adapter.start(); }"
        self.assertEqual(checks.runtime_effect_calls(source), [])

    def test_tokio_runtime_named_runtime_is_benign(self):
        source = "fn close(runtime: &TokioRuntime) { runtime.shutdown(); }"
        self.assertEqual(checks.runtime_effect_calls(source), [])

    def test_benign_names_do_not_mutate_source_inventory(self):
        original = lambda path: path.read_text(encoding="utf-8")

        def injected(path):
            text = original(path)
            if path.name == "instance_wake_runtime.rs":
                return text + (
                    "\nfn serve(adapter: &HttpAdapter) { adapter.start(); }\n"
                    "fn close(runtime: &TokioRuntime) { runtime.shutdown(); }\n"
                )
            return text

        self.assertEqual(checks.old_shape_errors(ROOT, injected), [])

    def test_approved_effect_path_count_mutation_is_rejected(self):
        original = lambda path: path.read_text(encoding="utf-8")

        def injected(path):
            text = original(path)
            if path.name == "reconcile_plan.rs":
                return text + "\nfn bypass(executor: &dyn RuntimeAdapter) { executor.start(); }\n"
            return text

        errors = checks.old_shape_errors(ROOT, injected)
        self.assertTrue(any("direct runtime effect path changed" in error for error in errors))

    def test_database_probes_cannot_skip(self):
        environment = os.environ.copy()
        environment.pop("LKJMC_STORE_TEST_DATABASE_URL", None)
        for probe in sorted(checks.DB_PROBES):
            result = subprocess.run(
                [str(ROOT / "scripts/check-runtime-adoption.py"), "--probe", probe],
                cwd=ROOT, env=environment, capture_output=True, text=True, check=False,
            )
            self.assertEqual(result.returncode, 2)
            self.assertIn("LKJMC_STORE_TEST_DATABASE_URL is required", result.stdout)


if __name__ == "__main__":
    unittest.main()
