#!/usr/bin/env python3
"""Run fail-closed daemon command lifecycle probes."""
import os
import re
import subprocess
import sys
from pathlib import Path

from command_lifecycle_checker import (
    DATABASE_PROBES, DATABASE_URL, PROBES, database_url_state, run_cli,
)

ROOT = Path(__file__).resolve().parents[1]
DAEMON = ["cargo", "test", "-p", "lkjmc-daemon", "--bin", "lkjmc-daemon"]

def run(command):
    return subprocess.run(command, cwd=ROOT, check=False).returncode == 0

def cargo_probe(name):
    if name in DATABASE_PROBES and database_url_state(os.environ) != "supplied":
        print(f"failed {name}: {DATABASE_URL} is required")
        return False
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
    cache = (ROOT / "crates/lkjmc-daemon/src/credential_cache.rs").read_text(encoding="utf-8")
    http_tokens = (ROOT / "crates/lkjmc-daemon/src/app/http_tokens.rs").read_text(encoding="utf-8")
    auth = (ROOT / "crates/lkjmc-daemon/src/transport/auth.rs").read_text(encoding="utf-8")
    if ("Result<Option<DaemonTokenRecord>, StoreError>" not in cache
            or "map_err(|_| ())" in cache or "credential lookup unavailable" in http_tokens
            or "Err(error) if error.is_deadline()" not in auth):
        print("failed timeout-outcome-pass: credential timeout is laundered into denial")
        return False
    web_routes = (ROOT / "crates/lkjmc-daemon/src/web/routes.rs").read_text(encoding="utf-8")
    required_web = ("command.deadline_exceeded", "Ok(Err(error)) if error.is_deadline() => deadline()", "Err(crate::app::BlockingError::Deadline) => deadline()")
    if "request deadline exceeded" in web_routes or not all(token in web_routes for token in required_web):
        print("failed timeout-outcome-pass: web deadline is not structured")
        return False
    transport = (ROOT / "crates/lkjmc-daemon/src/transport/command.rs").read_text(encoding="utf-8")
    admission = (ROOT / "crates/lkjmc-daemon/src/transport/admission.rs").read_text(encoding="utf-8")
    desired = (ROOT / "crates/lkjmc-daemon/src/commands/player_settings.rs").read_text(encoding="utf-8")
    timeout_test = (ROOT / "crates/lkjmc-daemon/src/tests/command_operation_tests/timeout.rs").read_text(encoding="utf-8")
    required_outcome = ("command::lookup", '"cancelled"', "response.request_id")
    if ("admission.correlate" not in transport or "admission.request_id()" not in admission
            or "execute_desired" not in desired
            or not all(token in timeout_test for token in required_outcome)):
        print("failed timeout-outcome-pass: request-correlated durable mutation outcome is absent")
        return False
    status_probe = run([*DAEMON, "status_timeout_outcome_pass", "--", "--nocapture"])
    auth_probe = run([*DAEMON, "sqlstate_deadline_is_not_auth_denied", "--", "--nocapture"])
    return status_probe and auth_probe and cargo_probe("timeout-outcome-pass") and cargo_probe("credential-cache-deadline")

def discord_boundary():
    source = "\n".join(path.read_text(encoding="utf-8") for path in (ROOT / "crates/lkjmc-discord/src").glob("*.rs"))
    if any(token in source for token in ("TcpListener::bind", "axum::serve", "spawn_blocking(")):
        print("failed discord-boundary: executable interaction listener remains")
        return False
    return run(["cargo", "test", "-p", "lkjmc-discord", "interaction_bind_is_rejected", "--", "--nocapture"])

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
    required = ("JoinHandle<()>", "register_pending()", "worker.await", "start.send(())")
    if "super::admission::require" not in routes:
        print("failed command-load-budget: router has no shared admission")
        return False
    if admission.count("spawn_blocking(") != 1 or not all(token in worker_source for token in required):
        print("failed command-load-budget: request worker handle is not tracked and joined")
        return False
    order = [admission.find(token) for token in (
        "register_pending()", "spawn_blocking(", "state.attach(worker_id, worker)", "start.send(())",
    )]
    if order != sorted(order) or ".retain(" in workers or "Worker::Joining" not in workers:
        print("failed command-load-budget: pending worker or observed-handle invariant is absent")
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
    if name == "duplicate-mutations-pass":
        return (cargo_probe(name)
                and run([*DAEMON, "conflicting_duplicate_is_denied", "--", "--nocapture"])
                and run([*DAEMON, "journal_failure_rolls_back_mutation", "--", "--nocapture"])
                and run([*DAEMON, "failed_worker_leaves_no_requested_journal", "--", "--nocapture"])
                and run([*DAEMON, "panicked_mutation_releases_transaction_lock", "--", "--nocapture"]))
    if name == "reactor-clean":
        return reactor_clean()
    if name == "shutdown-pass":
        return shutdown_pass()
    if name == "command-load-budget":
        return cargo_probe(name) and admission_contained()
    if name == "discord-boundary":
        return discord_boundary()
    return cargo_probe(name)

def main():
    return run_cli(selected)


if __name__ == "__main__":
    sys.exit(main())
