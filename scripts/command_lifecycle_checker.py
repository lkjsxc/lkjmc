#!/usr/bin/env python3
"""Selection and prerequisite policy for command lifecycle probes."""
import argparse
import os
from urllib.parse import urlsplit

PROBES = {
    "effect-classes-enforced": "effect_classes_enforced",
    "queues-bounded": "queues_bounded",
    "timeout-outcome-pass": "timeout_outcome_pass",
    "duplicate-mutations-pass": "duplicate_mutations_pass",
    "config-apply-truthful": "config_apply_truthful",
    "command-load-budget": "shared_admission_covers_auth_and_web",
    "shutdown-pass": "shutdown_waits_for_inflight_admission",
    "outer-cancellation": "outer_cancellation_keeps_worker_tracked_until_cleanup",
    "deadline-cleanup": "deadline_keeps_worker_tracked_until_cleanup",
    "completed-worker-observation": "completed_worker_is_observed_before_removal",
    "auth-budget-sql": "auth_budget_leaves_only_remaining_sql_time",
    "credential-cache-deadline": "lock_timeout_remains_deadline_through_cache",
    "tcp-db-deadline": "tcp_route_normalizes_real_database_deadlines",
    "web-db-deadline": "web_route_normalizes_real_database_deadlines",
}
DATABASE_PROBES = (
    "timeout-outcome-pass",
    "duplicate-mutations-pass",
    "auth-budget-sql",
    "credential-cache-deadline",
    "tcp-db-deadline",
    "web-db-deadline",
)
AGGREGATE_PROBES = (
    "effect-classes-enforced", "queues-bounded", "timeout-outcome-pass",
    "duplicate-mutations-pass", "config-apply-truthful", "shutdown-pass",
    "outer-cancellation", "deadline-cleanup", "completed-worker-observation",
    "auth-budget-sql", "tcp-db-deadline", "web-db-deadline",
    "reactor-clean", "command-load-budget", "discord-boundary",
)
EXTRA_PROBES = {"discord-boundary", "reactor-clean", "shutdown-pass"}
DATABASE_URL = "LKJMC_STORE_TEST_DATABASE_URL"


def database_url_state(environ):
    value = environ.get(DATABASE_URL, "").strip()
    if not value:
        return "absent"
    try:
        parsed = urlsplit(value)
    except ValueError:
        return "invalid"
    return "supplied" if parsed.scheme in {"postgres", "postgresql"} else "invalid"


def required_database_probes(names):
    requested = set(names)
    if "timeout-outcome-pass" in requested:
        requested.add("credential-cache-deadline")
    return tuple(name for name in DATABASE_PROBES if name in requested)


def run_cli(execute, argv=None, environ=None):
    parser = argparse.ArgumentParser()
    choice = parser.add_mutually_exclusive_group(required=True)
    choice.add_argument("--probe", choices=sorted([*PROBES, *EXTRA_PROBES]))
    choice.add_argument("--all", action="store_true")
    parser.add_argument("--allow-database-skip", action="store_true")
    args = parser.parse_args(argv)
    if args.allow_database_skip and not args.all:
        parser.error("--allow-database-skip requires --all")

    names = AGGREGATE_PROBES if args.all else (args.probe,)
    required = required_database_probes(names)
    state = database_url_state(os.environ if environ is None else environ)
    if state == "invalid" and required:
        for name in required:
            print(f"failed {name}: {DATABASE_URL} is not a PostgreSQL URL")
        return 1
    skipped = set()
    if state == "absent" and required:
        if not args.allow_database_skip:
            for name in required:
                print(f"failed {name}: {DATABASE_URL} is required")
            return 1
        skipped.update(required)
        for name in required:
            print(f"SKIP {name}: {DATABASE_URL} is unset (explicit aggregate allow-skip)")
    return 0 if all(name in skipped or execute(name) for name in names) else 1
