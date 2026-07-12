#!/usr/bin/env python3
"""Run fail-closed daemon command lifecycle probes."""
import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DAEMON = ["cargo", "test", "-p", "lkjmc-daemon", "--bin", "lkjmc-daemon"]
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
    "auth-budget-sql": "auth_budget_leaves_only_remaining_sql_time",
}
DATABASE_PROBES = {"timeout-outcome-pass", "duplicate-mutations-pass", "auth-budget-sql"}


def run(command):
    return subprocess.run(command, cwd=ROOT, check=False).returncode == 0


def cargo_probe(name):
    if name in DATABASE_PROBES and not os.environ.get("LKJMC_STORE_TEST_DATABASE_URL"):
        print(f"SKIP {name}: LKJMC_STORE_TEST_DATABASE_URL is unset")
        return True
    return run([*DAEMON, PROBES[name], "--", "--nocapture"])


def effect_classes_enforced():
    return run(["./scripts/check-contracts.py"]) and cargo_probe("effect-classes-enforced")


def timeout_outcome_pass():
    lifecycle = (ROOT / "crates/lkjmc-daemon/src/command_lifecycle.rs").read_text(encoding="utf-8")
    store_error = (ROOT / "crates/lkjmc-store/src/error.rs").read_text(encoding="utf-8")
    status_store = (ROOT / "crates/lkjmc-store/src/status.rs").read_text(encoding="utf-8")
    required = ("SqlState::QUERY_CANCELED", "SqlState::LOCK_NOT_AVAILABLE")
    if "contains(" in lifecycle or not all(token in store_error for token in required):
        print("failed timeout-outcome-pass: SQLSTATE timeout normalization is not structural")
        return False
    if status_store.count("query_one(") != 1:
        print("failed timeout-outcome-pass: status counts are not one aggregate query")
        return False
    status_probe = run([*DAEMON, "status_timeout_outcome_pass", "--", "--nocapture"])
    return status_probe and cargo_probe("timeout-outcome-pass")


def reactor_clean():
    paths = [
        "crates/lkjmc-daemon/src/transport/command.rs",
        "crates/lkjmc-daemon/src/transport/auth.rs",
        "crates/lkjmc-daemon/src/transport/peer.rs",
        "crates/lkjmc-daemon/src/web/routes.rs",
    ]
    forbidden = ("database_connection(", "database_pool(", "pool.get(")
    for path in paths:
        source = (ROOT / path).read_text(encoding="utf-8")
        for segment in source.split("async fn ")[1:]:
            if any(token in segment.split("\nfn ", 1)[0] for token in forbidden):
                print(f"failed reactor-clean: synchronous database call in {path}")
                return False
    print("ok reactor-clean")
    return True


def admission_contained():
    checks = {
        "crates/lkjmc-daemon/src/transport/auth.rs": "run_blocking(",
        "crates/lkjmc-daemon/src/transport/peer.rs": "run_blocking(",
        "crates/lkjmc-daemon/src/transport/command.rs": "run_blocking(",
        "crates/lkjmc-daemon/src/web/routes.rs": "run_blocking(",
    }
    for path, marker in checks.items():
        source = (ROOT / path).read_text(encoding="utf-8").split("#[cfg(test)]")[0]
        if "spawn_blocking(" in source or marker not in source:
            print(f"failed command-load-budget: admission bypass in {path}")
            return False
    routes = (ROOT / "crates/lkjmc-daemon/src/transport/routes.rs").read_text(encoding="utf-8")
    admission = (ROOT / "crates/lkjmc-daemon/src/app/admission.rs").read_text(encoding="utf-8")
    workers = (ROOT / "crates/lkjmc-daemon/src/app/admission/workers.rs").read_text(encoding="utf-8")
    app = (ROOT / "crates/lkjmc-daemon/src/app.rs").read_text(encoding="utf-8")
    database = (ROOT / "crates/lkjmc-daemon/src/app/database.rs").read_text(encoding="utf-8")
    pool = (ROOT / "crates/lkjmc-store/src/pool.rs").read_text(encoding="utf-8")
    worker_source = admission + workers
    required = ("JoinHandle<()>", "track(worker)", "take_workers()", "worker.await", "start.send(())")
    if "super::admission::require" not in routes:
        print("failed command-load-budget: router has no shared admission")
        return False
    if admission.count("spawn_blocking(") != 1 or not all(token in worker_source for token in required):
        print("failed command-load-budget: request worker handle is not tracked and joined")
        return False
    if re.search(r"timeout_at\([^)]*,\s*worker\)", worker_source):
        print("failed command-load-budget: timeout drops a request worker handle")
        return False
    if ("set_deadlines" not in pool or "configure(&mut config, ceiling)" not in pool
            or "config.connect_timeout(ceiling)" not in pool or "remaining_request_budget" not in admission
            or "request_database_connection" not in database
            or "get_timeout(remaining)" not in database or "set_deadlines" not in database
            or "crate::command_lifecycle::DEADLINE" not in app):
        print("failed command-load-budget: request database budget is not propagated")
        return False
    fixed_limit = re.compile(r"(?:statement|lock)_timeout\\s*=\\s*['\\\"]?\\d+(?:ms|s)")
    if fixed_limit.search(pool):
        print("failed command-load-budget: fixed PostgreSQL request limit")
        return False
    print("ok command-load-budget containment")
    return True


def shutdown_pass():
    return cargo_probe("shutdown-pass") and run(
        ["env", "LKJMC_ASSERT_SHUTDOWN=1", "./scripts/check-daemon-cli.sh"]
    )


def selected(name):
    if name == "effect-classes-enforced":
        return effect_classes_enforced()
    if name == "timeout-outcome-pass":
        return timeout_outcome_pass()
    if name == "reactor-clean":
        return reactor_clean()
    if name == "shutdown-pass":
        return shutdown_pass()
    if name == "command-load-budget":
        return cargo_probe(name) and admission_contained()
    return cargo_probe(name)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--probe", choices=sorted([*PROBES, "reactor-clean", "shutdown-pass"]))
    parser.add_argument("--all", action="store_true")
    args = parser.parse_args()
    names = [args.probe] if args.probe else [
        "effect-classes-enforced", "queues-bounded", "timeout-outcome-pass",
        "duplicate-mutations-pass", "config-apply-truthful", "shutdown-pass",
        "outer-cancellation", "deadline-cleanup", "auth-budget-sql",
        "reactor-clean", "command-load-budget",
    ]
    if not args.probe and not args.all:
        parser.error("choose --probe or --all")
    return 0 if all(selected(name) for name in names) else 1


if __name__ == "__main__":
    sys.exit(main())
