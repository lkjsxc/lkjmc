#!/usr/bin/env python3
"""Run fail-closed daemon command lifecycle probes."""
import argparse
import os
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
    "command-load-budget": "command_load_budget_rejects_without_enqueuing",
}
DATABASE_PROBES = {"timeout-outcome-pass", "duplicate-mutations-pass"}


def run(command):
    return subprocess.run(command, cwd=ROOT, check=False).returncode == 0


def cargo_probe(name):
    if name in DATABASE_PROBES and not os.environ.get("LKJMC_STORE_TEST_DATABASE_URL"):
        print(f"SKIP {name}: LKJMC_STORE_TEST_DATABASE_URL is unset")
        return True
    return run([*DAEMON, PROBES[name], "--", "--nocapture"])


def effect_classes_enforced():
    return run(["./scripts/check-contracts.py"]) and cargo_probe("effect-classes-enforced")


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


def shutdown_pass():
    return run(["env", "LKJMC_ASSERT_SHUTDOWN=1", "./scripts/check-daemon-cli.sh"])


def selected(name):
    if name == "effect-classes-enforced":
        return effect_classes_enforced()
    if name == "reactor-clean":
        return reactor_clean()
    if name == "shutdown-pass":
        return shutdown_pass()
    return cargo_probe(name)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--probe", choices=sorted([*PROBES, "reactor-clean", "shutdown-pass"]))
    parser.add_argument("--all", action="store_true")
    args = parser.parse_args()
    names = [args.probe] if args.probe else [
        "effect-classes-enforced", "queues-bounded", "timeout-outcome-pass",
        "duplicate-mutations-pass", "config-apply-truthful", "shutdown-pass",
        "reactor-clean", "command-load-budget",
    ]
    if not args.probe and not args.all:
        parser.error("choose --probe or --all")
    return 0 if all(selected(name) for name in names) else 1


if __name__ == "__main__":
    sys.exit(main())
