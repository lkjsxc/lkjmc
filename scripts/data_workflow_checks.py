#!/usr/bin/env python3
import json
import re
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
    if set(data) != {"classifications", "schema"} or data["schema"] != "lkjmc-data-workflows-one":
        return ["invalid workflow inventory envelope"]
    rows = data["classifications"]
    required = {"id", "source", "owner", "transaction", "externalEffect"}
    errors = []
    sources = set()
    ids = set()
    for row in rows if isinstance(rows, list) else []:
        if set(row) != required:
            errors.append("workflow classification has wrong fields")
            continue
        if row["id"] in ids or row["source"] in sources:
            errors.append("duplicate workflow classification")
        ids.add(row["id"]); sources.add(row["source"])
        source = ROOT / row["source"]
        if not source.is_file(): errors.append(f"missing classified source: {row['source']}")
        if row["transaction"] not in {"local", "delegated"}:
            errors.append(f"invalid transaction owner: {row['id']}")
        if row["externalEffect"] not in {"none", "intent-only", "denied"}:
            errors.append(f"invalid effect edge: {row['id']}")
    actual = transaction_owners()
    for source in sorted(actual - sources):
        errors.append(f"unclassified transaction owner: {source}")
    for source in sorted(sources - actual):
        if source.endswith("data_workflows.rs"):
            continue
        errors.append(f"classification is not a transaction owner: {source}")
    return errors


def transaction_owners():
    found = set()
    roots = (ROOT / "crates/lkjmc-store/src", ROOT / "crates/lkjmc-daemon/src")
    for base in roots:
        for path in base.rglob("*.rs"):
            relative = path.relative_to(ROOT)
            if excluded(relative, path.name):
                continue
            source = rust_code(path.read_text(encoding="utf-8"))
            if re.search(r"\.\s*transaction\s*\(\s*\)", source):
                found.add(str(relative))
    return found


def excluded(relative, name):
    return ("tests" in relative.parts or name.endswith("_tests.rs")
            or "fault_harness" in relative.parts)


def rust_code(source):
    source = re.sub(r"/\*.*?\*/", " ", source, flags=re.S)
    source = re.sub(r"//[^\n]*", " ", source)
    raw = r'(?<![A-Za-z0-9_])(?:br|rb|r)(?P<h>#{0,16})".*?"(?P=h)'
    source = re.sub(raw, '""', source, flags=re.S)
    source = re.sub(r'(?:b)?"(?:\\.|[^"\\])*"', '""', source, flags=re.S)
    return source


def cargo_test(package, test_filter, integration=None):
    command = ["cargo", "test", "-p", package]
    if integration: command += ["--test", integration]
    command += [test_filter, "--", "--nocapture"]
    return subprocess.run(command, cwd=ROOT).returncode
