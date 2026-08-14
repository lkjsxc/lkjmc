#!/usr/bin/env python3
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DB_PROBES = {
    "transfer-crash-matrix",
    "delivery-crash-matrix",
    "adventure-crash-matrix",
    "fencing-pass",
    "schema-cutover-pass",
}
ALL_PROBES = [
    "transfer-crash-matrix",
    "delivery-crash-matrix",
    "adventure-crash-matrix",
    "profile-format-safe-complete",
    "fencing-pass",
    "schema-cutover-pass",
    "old-workflows-absent",
]


def cargo_test(package, test_filter, integration=None):
    command = ["cargo", "test", "-p", package]
    if integration: command += ["--test", integration]
    command += [test_filter, "--", "--nocapture"]
    return subprocess.run(command, cwd=ROOT).returncode
