#!/usr/bin/env python3
"""Run fail-closed product observability probes and source mutations."""
import argparse
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATABASE = "LKJMC_STORE_TEST_DATABASE_URL"
DAEMON_TEST = ["cargo", "test", "-p", "lkjmc-daemon", "--bin", "lkjmc-daemon"]
PROBES = (
    "correlation-pass",
    "fault-diagnostics-pass",
    "metrics-bounded",
    "support-bundle-pass",
    "secret-canary-pass",
    "overhead-budget",
)
DATABASE_PROBES = {"correlation-pass", "support-bundle-pass"}
REQUIRED = {
    "correlation-pass": (
        ("crates/lkjmc-daemon/src/tests/observability_correlation.rs", "for repeat in 0..30"),
        ("crates/lkjmc-daemon/src/observability/api.rs", ".run_blocking("),
        ("crates/lkjmc-store/tests/observability.rs", "reused_request_id_records_each_postgresql_attempt"),
        ("crates/lkjmc-store/tests/observability.rs", "malicious_attributes_retain_redacted_postgresql_event"),
        ("docs/architecture/runtime/daemon/observability.md", "does not\nsatisfy the research `B-O`"),
    ),
    "fault-diagnostics-pass": (
        ("crates/lkjmc-daemon/src/tests/observability_correlation.rs", "fault_diagnostics_pass_is_typed_http_non_success"),
        ("crates/lkjmc-daemon/src/observability/health.rs", 'unavailable("readiness_worker_failed")'),
    ),
    "metrics-bounded": (
        ("crates/lkjmc-daemon/src/observability/metrics.rs", "pub(crate) const SERIES_CAP: usize = 64"),
        ("crates/lkjmc-daemon/src/observability/metrics.rs", '"outcome=\\"succeeded\\""'),
    ),
    "support-bundle-pass": (
        ("crates/lkjmc-daemon/src/support/bundle/archive.rs", ".mode(0o600)"),
        ("crates/lkjmc-daemon/src/support/bundle/archive.rs", "fs::hard_link(temp, output.target())"),
        ("crates/lkjmc-daemon/src/support/bundle/archive.rs", "libc::O_NOFOLLOW"),
        ("crates/lkjmc-daemon/src/support/bundle.rs", "Duration::from_secs(7)"),
        ("crates/lkjmc-daemon/src/tests/observability_support.rs", "Sha256::digest(bytes)"),
        ("crates/lkjmc-daemon/src/tests/observability_support_security.rs", "nested_parent_symlink_swaps_never_write_outside"),
        ("crates/lkjmc-daemon/src/tests/observability_support_security.rs", "fifo_and_slow_fault_return_bounded_without_partial_output"),
    ),
    "secret-canary-pass": (
        ("crates/lkjmc-daemon/src/support/redaction.rs", 'window == b"-canary"'),
        ("crates/lkjmc-core/src/observability/validation.rs", '!lower.contains(\"://\")'),
        ("platforms/jvm/common/src/main/java/com/lkjmc/common/diagnostic/DiagnosticEmitter.java", "queue.offer(event)"),
    ),
    "overhead-budget": (
        ("crates/lkjmc-daemon/src/observability/mod.rs", "for _ in 0..10_000"),
        ("crates/lkjmc-daemon/src/observability/mod.rs", "Duration::from_secs(2)"),
    ),
}


def run(command: list[str]) -> bool:
    return subprocess.run(command, cwd=ROOT, check=False).returncode == 0


def source_failures(overrides=None) -> set[str]:
    overrides = overrides or {}
    failed = set()
    for probe, requirements in REQUIRED.items():
        for relative, marker in requirements:
            text = overrides.get(relative)
            if text is None:
                text = (ROOT / relative).read_text(encoding="utf-8")
            if marker not in text:
                failed.add(probe)
    return failed


def mutation_tests() -> bool:
    failures = []
    for probe, requirements in REQUIRED.items():
        relative, marker = requirements[0]
        original = (ROOT / relative).read_text(encoding="utf-8")
        mutated = original.replace(marker, "", 1)
        if probe not in source_failures({relative: mutated}):
            failures.append(probe)
    if failures:
        print("failed observability mutations: " + ",".join(failures))
        return False
    print("ok observability mutations=6")
    return True


def selected(probe: str) -> bool:
    if probe in source_failures():
        print(f"failed {probe}: required fail-closed source is absent")
        return False
    if probe in DATABASE_PROBES and not os.environ.get(DATABASE):
        print(f"failed {probe}: {DATABASE} is required")
        return False
    commands = {
        "correlation-pass": [
            [*DAEMON_TEST, "correlation_pass_uses_http_and_postgresql_thirty_times", "--", "--nocapture"],
            ["cargo", "test", "-p", "lkjmc-store", "--test", "observability", "reused_request_id_records_each_postgresql_attempt", "--", "--nocapture"],
            ["cargo", "test", "-p", "lkjmc-store", "--test", "observability", "malicious_attributes_retain_redacted_postgresql_event", "--", "--nocapture"],
        ],
        "fault-diagnostics-pass": [[*DAEMON_TEST, "fault_diagnostics_pass_is_typed_http_non_success", "--", "--nocapture"]],
        "metrics-bounded": [[*DAEMON_TEST, "labels_and_series_are_bounded", "--", "--nocapture"]],
        "support-bundle-pass": [
            [*DAEMON_TEST, "support_bundle_pass_is_private_sorted_hashed_and_redacted", "--", "--nocapture"],
            [*DAEMON_TEST, "bundle_rejects_nested_parent_and_target_symlinks_without_outside_writes", "--", "--nocapture"],
            [*DAEMON_TEST, "nested_parent_symlink_swaps_never_write_outside", "--", "--nocapture"],
            [*DAEMON_TEST, "fifo_and_slow_fault_return_bounded_without_partial_output", "--", "--nocapture"],
        ],
        "secret-canary-pass": [
            ["cargo", "test", "-p", "lkjmc-core", "rejects_urls_and_secret_canaries"],
            [*DAEMON_TEST, "every_secret_class_is_removed", "--", "--nocapture"],
            ["./gradlew", "--no-daemon", "--no-build-cache", ":platforms:jvm:common:test", "--tests", "com.lkjmc.common.diagnostic.DiagnosticEmitterTest"],
        ],
        "overhead-budget": [[*DAEMON_TEST, "overhead_budget", "--", "--nocapture"]],
    }
    passed = all(run(command) for command in commands[probe])
    print(("ok " if passed else "failed ") + probe)
    return passed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--probe", choices=PROBES)
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--mutations", action="store_true")
    parser.add_argument("--allow-database-skip", action="store_true")
    args = parser.parse_args()
    if args.mutations:
        return 0 if mutation_tests() else 1
    if args.probe:
        return 0 if selected(args.probe) else 1
    if not args.all:
        parser.error("choose --probe, --all, or --mutations")
    if source_failures() or not mutation_tests():
        return 1
    skipped = []
    for probe in PROBES:
        if probe in DATABASE_PROBES and not os.environ.get(DATABASE) and args.allow_database_skip:
            skipped.append(probe)
            continue
        if not selected(probe):
            return 1
    print("ok observability probes skipped=" + (",".join(skipped) if skipped else "none"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
