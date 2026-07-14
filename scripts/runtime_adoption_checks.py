import os
import re
import subprocess
from pathlib import Path
from urllib.parse import urlparse

from runtime_adoption_source import runtime_effect_calls

ROOT = Path(__file__).resolve().parents[1]
PROBES = [
    "runtime-global-mutex-absent",
    "cross-instance-hang-pass",
    "same-instance-race-pass",
    "reconcile-idempotent",
    "effect-crash-recovery",
    "adapter-capability-pass",
    "runtime-load-budget",
]
DB_PROBES = {
    "cross-instance-hang-pass",
    "same-instance-race-pass",
    "reconcile-idempotent",
    "effect-crash-recovery",
}
EXPECTED_EFFECTS = {
    "app.rs": ["shutdown"],
    "commands/instance_read.rs": ["logs"],
    "runtime/reconcile_observation.rs": ["adopt", "status"],
    "runtime/reconcile_plan.rs": ["delete", "start", "stop"],
}


def database_ready():
    try:
        parsed = urlparse(os.environ.get("LKJMC_STORE_TEST_DATABASE_URL", ""))
        return parsed.scheme in {"postgres", "postgresql"} and bool(parsed.hostname and parsed.path.strip("/"))
    except ValueError:
        return False


def old_shape_errors(root=ROOT, override=None):
    source = root / "crates/lkjmc-daemon/src"
    errors = []
    for path in source.rglob("*.rs"):
        text = override(path) if override else path.read_text(encoding="utf-8")
        relative = path.relative_to(root)
        if re.search(r"Arc\s*<\s*Mutex\s*<\s*Box\s*<\s*dyn\s+RuntimeAdapter", text):
            errors.append(f"daemon-wide runtime mutex: {relative}")
        if re.search(r"\.runtime\s*\.lock\s*\(", text):
            errors.append(f"runtime effect lock: {relative}")
        if "runtime lock poisoned" in text:
            errors.append(f"old runtime lock diagnostic: {relative}")
        if not path.stem.endswith("_tests"):
            calls = runtime_effect_calls(text)
            expected = sorted(EXPECTED_EFFECTS.get(str(path.relative_to(source)), []))
            if calls != expected:
                errors.append(f"direct runtime effect path changed: {relative}: {calls}")
    if (source / "reconcile").exists():
        errors.append("alternate lifecycle directory remains: crates/lkjmc-daemon/src/reconcile")
    app_path = source / "app.rs"
    app = override(app_path) if override else app_path.read_text(encoding="utf-8")
    if "runtime: Arc<dyn RuntimeAdapter>" not in app:
        errors.append("shareable runtime adapter field absent")
    if "LifecycleCoordinator" not in app:
        errors.append("keyed lifecycle coordinator absent")
    return errors


def cargo_test(package, name, test_target=None):
    command = ["cargo", "test", "-p", package]
    if test_target:
        command.extend(["--test", test_target])
    command.extend([name, "--", "--exact"])
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
    if result.returncode:
        print(result.stdout)
        print(result.stderr)
    return result.returncode


def run(probe):
    if probe == "runtime-global-mutex-absent":
        errors = old_shape_errors()
        if errors:
            print("\n".join(errors))
            return 1
        return 0
    if probe in DB_PROBES and not database_ready():
        print(f"{probe}: valid LKJMC_STORE_TEST_DATABASE_URL is required")
        return 2
    tests = {
        "cross-instance-hang-pass": [
            ("lkjmc-daemon", "runtime::coordinator::tests::unrelated_key_proceeds_while_key_is_held", None),
            ("lkjmc-daemon", "runtime::adoption_concurrency_tests::cross_instance_database_process_hang", None),
        ],
        "same-instance-race-pass": [
            ("lkjmc-daemon", "runtime::coordinator::tests::same_instance_race_is_serialized", None),
            ("lkjmc-daemon", "runtime::adoption_concurrency_tests::same_instance_database_process_race", None),
        ],
        "reconcile-idempotent": [
            ("lkjmc-store", "reconcile_idempotent", "runtime_adoption"),
            ("lkjmc-daemon", "runtime::adoption_tests::reconcile_idempotent_process_boundary", None),
        ],
        "effect-crash-recovery": [
            ("lkjmc-store", "effect_crash_recovery", "runtime_adoption"),
            ("lkjmc-daemon", "runtime::adoption_tests::effect_crash_recovery_process_boundary", None),
        ],
        "adapter-capability-pass": [
            ("lkjmc-daemon", "runtime::adapter::tests::adapter_capability_pass", None),
            ("lkjmc-daemon", "runtime::kubernetes_tests::kubernetes_plan_fails_closed_without_access", None),
            ("lkjmc-daemon", "runtime::kubernetes_tests::kubernetes_hung_kubectl_respects_total_deadline", None),
            ("lkjmc-daemon", "runtime::kubernetes_tests::kubernetes_destructive_paths_deny_before_effect", None),
            ("lkjmc-daemon", "runtime::local::tests::pid_start_and_executable_mismatches_are_fenced", None),
        ],
        "runtime-load-budget": [
            ("lkjmc-daemon", "runtime::coordinator::tests::runtime_load_budget", None),
            ("lkjmc-daemon", "runtime::local::tests::shutdown_respects_total_deadline", None),
        ],
    }
    for test in tests[probe]:
        result = cargo_test(*test)
        if result:
            return result
    return 0
