#!/usr/bin/env python3
import argparse
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import network_adoption_checks as checks

COMMANDS = {
    "inspect-apply-pass": [
        ["cargo", "test", "-p", "lkjmc-core", "network_intent_tests::inspect_is_exact"],
        ["cargo", "test", "-p", "lkjmc-store", "--test", "network_intent_store", "desired_intent_and_partial_history_are_durable"],
        ["cargo", "test", "-p", "lkjmc-daemon", "network_apply_real_local_boundary_and_reapply"],
        ["cargo", "test", "-p", "lkjmc-daemon", "size_one_pool_is_available"],
    ],
    "reapply-pass": [
        ["cargo", "test", "-p", "lkjmc-daemon", "network_apply_real_local_boundary_and_reapply"],
        ["cargo", "test", "-p", "lkjmc-daemon", "network_reapply_repairs_killed_owned_proxy"],
        ["cargo", "test", "-p", "lkjmc-daemon", "network_apply_denies_unowned_listener"],
    ],
    "partial-failure-pass": [
        ["cargo", "test", "-p", "lkjmc-store", "--test", "network_intent_store", "desired_intent_and_partial_history_are_durable"],
        ["cargo", "test", "-p", "lkjmc-daemon", "network_apply_recovers_after_partial_process_failure"],
        ["cargo", "test", "-p", "lkjmc-daemon", "recovery_matrix", "--", "--test-threads=1"],
    ],
    "local-kube-capabilities": [
        ["cargo", "test", "-p", "lkjmc-core", "config_tests::kubernetes_mount_capabilities_fail_before_use"],
        ["cargo", "test", "-p", "lkjmc-daemon", "runtime::kubernetes_tests::kubernetes_destructive_paths_deny_before_effect"],
    ],
    "config-example-pass": [[str(ROOT / "scripts/check-config-examples.py")]],
}


def run(probe: str) -> int:
    if probe == "network-path-single":
        errors = checks.source_errors(ROOT)
        if errors:
            print("\n".join(errors))
            return 1
        print("ok network-path-single")
        return 0
    if probe in checks.DB_PROBES and not os.environ.get("LKJMC_STORE_TEST_DATABASE_URL"):
        print(f"{probe}: LKJMC_STORE_TEST_DATABASE_URL is required")
        return 2
    for command in COMMANDS[probe]:
        result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
        if result.returncode:
            print(result.stdout)
            print(result.stderr)
            return result.returncode
    print(f"ok {probe}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--probe", choices=checks.PROBES)
    group.add_argument("--all", action="store_true")
    args = parser.parse_args()
    probes = checks.PROBES if args.all else [args.probe]
    for probe in probes:
        code = run(probe)
        if code:
            return code
    print("ok check-network-adoption")
    return 0

if __name__ == "__main__":
    sys.exit(main())
