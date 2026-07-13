#!/usr/bin/env python3
import json
import subprocess
from pathlib import Path

from data_workflow_inventory import discover

ROOT = Path(__file__).resolve().parents[1]
DB_PROBES = {
    "transfer-crash-matrix",
    "delivery-crash-matrix",
    "adventure-crash-matrix",
    "fencing-pass",
    "schema-cutover-pass",
}
ALL_PROBES = [
    "all-multiwrites-classified",
    "transfer-crash-matrix",
    "delivery-crash-matrix",
    "adventure-crash-matrix",
    "profile-format-safe-complete",
    "fencing-pass",
    "schema-cutover-pass",
    "old-workflows-absent",
]


def inventory_errors():
    path = ROOT / "config/data-workflows.json"
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"invalid workflow inventory: {error}"]
    if set(data) != {"classifications", "schema"} or data["schema"] != "lkjmc-data-workflows-two":
        return ["invalid workflow inventory envelope"]
    rows = data["classifications"]
    required = {"id", "source", "symbol", "owner", "writes", "effects", "transactionOwner"}
    errors, classified, ids = [], {}, set()
    for row in rows if isinstance(rows, list) else []:
        if set(row) != required:
            errors.append("workflow classification has wrong fields")
            continue
        key = (row["source"], row["symbol"])
        if row["id"] in ids or key in classified:
            errors.append("duplicate workflow classification")
        ids.add(row["id"]); classified[key] = row
        if not (ROOT / row["source"]).is_file():
            errors.append(f"missing classified source: {row['source']}")
        if row["transactionOwner"] not in {"local", "delegated", "none"}:
            errors.append(f"invalid transaction owner: {row['id']}")
        if row["writes"] != sorted(set(row["writes"])):
            errors.append(f"writes must be sorted and unique: {row['id']}")
        if row["effects"] != sorted(set(row["effects"])):
            errors.append(f"effects must be sorted and unique: {row['id']}")
    actual = discover(ROOT)
    for key in sorted(actual.keys() - classified.keys()):
        errors.append(f"unclassified multiwrite/effect: {key[0]}::{key[1]}")
    for key in sorted(classified.keys() - actual.keys()):
        errors.append(f"classification has no discovered multiwrite/effect: {key[0]}::{key[1]}")
    for key in sorted(actual.keys() & classified.keys()):
        row, observed = classified[key], actual[key]
        for field in ("writes", "effects", "transactionOwner"):
            if row[field] != observed[field]:
                errors.append(f"classification {field} mismatch: {key[0]}::{key[1]}")
    return errors


def cargo_test(package, test_filter, integration=None):
    command = ["cargo", "test", "-p", package]
    if integration: command += ["--test", integration]
    command += [test_filter, "--", "--nocapture"]
    return subprocess.run(command, cwd=ROOT).returncode
