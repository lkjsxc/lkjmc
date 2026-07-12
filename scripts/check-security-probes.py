#!/usr/bin/env python3
import argparse
import os
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DAEMON = ["cargo", "test", "-p", "lkjmc-daemon", "--bin", "lkjmc-daemon"]
STORE = ["cargo", "test", "-p", "lkjmc-store"]


def cargo(probe, command):
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
    if result.returncode:
        print(f"failed {probe}")
        print(result.stdout, end="")
        print(result.stderr, end="")
        return False
    print(f"ok {probe}")
    return True


def daemon_test(probe, test):
    return cargo(probe, [*DAEMON, test, "--", "--nocapture"])


def store_test(probe, test):
    return cargo(probe, [*STORE, test, "--", "--nocapture"])


def database_test(probe, test, store=False):
    if not os.environ.get("LKJMC_STORE_TEST_DATABASE_URL"):
        print(f"SKIP {probe}: LKJMC_STORE_TEST_DATABASE_URL is unset")
        return True
    runner = store_test if store else daemon_test
    return runner(probe, test)


def forgery_suite_pass():
    return daemon_test("forgery-suite-pass", "forged_actor_and_body_principals")


def surface_policy_complete():
    return daemon_test("surface-policy-complete", "registry_policy_covers")


def cache_capacity_expiry():
    return (
        daemon_test("cache-capacity-expiry", "capacity_evicts_lowest_hash")
        and daemon_test("cache-expiry", "expired_credential_is_not_returned")
        and daemon_test("cache-fractional-expiry", "fractional_expiry_is_not_rounded_up_in_cache")
    )


def cache_revision_hit():
    return database_test(
        "cache-revision-hit", "database_authentication_uses_a_cache_hit_without_bumping_revision"
    )


def revocation_race():
    return database_test(
        "revocation-race", "revocation_waits_for_an_inflight_revision_checked_authentication", True
    )


def fractional_expiry_boundary():
    return database_test(
        "fractional-expiry-boundary", "find_active_preserves_fractional_expiry_microseconds", True
    )


def outage_fail_closed():
    return daemon_test("outage-fail-closed", "unavailable_database_denies_even_a_cached_credential")


def canary_audit():
    return database_test(
        "canary-audit", "creation_and_revocation_audit_only_redacted_credential_data"
    )


def reactor_blocking_absent():
    return daemon_test("reactor-blocking-absent", "tcp_command_denies_missing_or_bootstrap_secret")


def root_routine_absent():
    return daemon_test("root-routine-absent", "credential_replaces_untrusted_actor_attribution")


def web_csrf_session_pass():
    return daemon_test("web-csrf-session-pass", "web_login_session_and_csrf_gate_forms")


def secret_canary_pass():
    return daemon_test("secret-canary-pass", "unavailable_secret_has_no_value_in_its_error")


PROBES = {
    "forgery-suite-pass": forgery_suite_pass,
    "surface-policy-complete": surface_policy_complete,
    "cache-capacity-expiry": cache_capacity_expiry,
    "cache-revision-hit": cache_revision_hit,
    "revocation-race": revocation_race,
    "fractional-expiry-boundary": fractional_expiry_boundary,
    "outage-fail-closed": outage_fail_closed,
    "canary-audit": canary_audit,
    "reactor-blocking-absent": reactor_blocking_absent,
    "root-routine-absent": root_routine_absent,
    "web-csrf-session-pass": web_csrf_session_pass,
    "secret-canary-pass": secret_canary_pass,
}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--probe", choices=sorted(PROBES))
    args = parser.parse_args()
    names = [args.probe] if args.probe else sorted(PROBES)
    return not all(PROBES[name]() for name in names)


if __name__ == "__main__":
    raise SystemExit(main())
