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
from runtime_adoption_source import RESERVED_EFFECT_METHODS


class RuntimeAdoptionCheckerTests(unittest.TestCase):
    def injected_errors(self, filename, addition):
        original = lambda path: path.read_text(encoding="utf-8")

        def injected(path):
            text = original(path)
            return text + addition if path.name == filename else text

        return checks.old_shape_errors(ROOT, injected)

    def assert_inventory_rejected(self, filename, addition):
        errors = self.injected_errors(filename, addition)
        self.assertTrue(any("direct runtime effect path changed" in item for item in errors))

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
                "runtime: Arc<dyn RuntimeAdapter>\nLifecycleCoordinator\n"
                "type Old = Arc<Mutex<Box<dyn RuntimeAdapter>>>;\n",
                encoding="utf-8",
            )
            errors = checks.old_shape_errors(root)
            self.assertTrue(any("daemon-wide runtime mutex" in item for item in errors))

    def test_reserved_calls_ignore_receiver_inference(self):
        cases = {
            "type alias": "type E = Arc<dyn RuntimeAdapter>; fn f(x: E) { x.runtime_start(); }",
            "accessor clone as_ref": (
                "fn f(state: &Shelf) { "
                "state.engine().clone().as_ref().runtime_stop(); }"
            ),
            "AppState": "fn f(state: &AppState) { state.any().runtime_status(); }",
            "non AppState": "fn f(state: &Other) { state.any().runtime_observe(); }",
            "generic": "fn f<T>(x: T) { x.runtime_adopt(); }",
            "arbitrary alias": (
                "fn f(value: Unknown) { let potato = value; "
                "potato.runtime_logs(); }"
            ),
            "parenthesized": "fn f(x: X) { ((&x)).runtime_delete(); }",
            "qualified": "fn f(x: &X) { Trait::runtime_shutdown(x); }",
        }
        expected = list(RESERVED_EFFECT_METHODS)
        for (label, source), method in zip(cases.items(), expected):
            with self.subTest(label=label):
                self.assertEqual(checks.runtime_effect_calls(source), [method])

    def test_raw_reserved_identifier_is_detected(self):
        self.assertEqual(
            checks.runtime_effect_calls("fn f(x: X) { x.r#runtime_start(); }"),
            ["runtime_start"],
        )

    def test_every_reserved_receiver_mutation_is_rejected(self):
        mutations = [
            "type Engine = Arc<dyn RuntimeAdapter>; fn f(x: Engine) { x.runtime_start(); }",
            "fn f(state: &Shelf) { state.get().clone().as_ref().runtime_stop(); }",
            "fn f(state: &AppState) { state.other().runtime_status(); }",
            "fn f(state: &NotAppState) { state.other().runtime_observe(); }",
            "fn f<T>(x: T) { x.runtime_adopt(); }",
            "fn f(x: X) { let arbitrary = x; arbitrary.runtime_logs(); }",
            "fn f(x: X) { x.runtime_delete(); }",
            "fn f(x: &X) { Trait::runtime_shutdown(x); }",
        ]
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                self.assert_inventory_rejected("instance_wake_runtime.rs", "\n" + mutation)

    def test_approved_path_count_mutation_is_rejected(self):
        self.assert_inventory_rejected(
            "reconcile_plan.rs",
            "\nfn bypass(x: X) { x.runtime_start(); }\n",
        )

    def test_comments_and_literals_do_not_create_calls(self):
        source = r'''
            // value.runtime_start()
            /* outer.runtime_stop(/* nested.runtime_status() */) */
            let a = "x.runtime_adopt()";
            let b = r###"x.runtime_logs()"###;
            let c = 'x';
            let d = b'x';
        '''
        self.assertEqual(checks.runtime_effect_calls(source), [])

    def test_old_generic_method_names_are_benign(self):
        source = """
            fn http(x: &HttpAdapter) { x.start(); x.stop(); x.status(); x.logs(); }
            fn tokio(x: &TokioRuntime) { x.observe(); x.adopt(); x.delete(); x.shutdown(); }
            fn generic<T>(x: T) { x.start(); x.shutdown(); }
        """
        self.assertEqual(checks.runtime_effect_calls(source), [])
        self.assertEqual(self.injected_errors("instance_wake_runtime.rs", source), [])

    def test_current_exact_inventory_passes(self):
        self.assertEqual(checks.old_shape_errors(ROOT), [])

    def test_database_probes_cannot_skip(self):
        environment = os.environ.copy()
        environment.pop("LKJMC_STORE_TEST_DATABASE_URL", None)
        for probe in sorted(checks.DB_PROBES):
            result = subprocess.run(
                [str(ROOT / "scripts/check-runtime-adoption.py"), "--probe", probe],
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
