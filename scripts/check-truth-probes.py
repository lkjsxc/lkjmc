#!/usr/bin/env python3
"""Reject known weak shapes; expected mode records debt without promoting it."""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

from truth_contracts import issues as contract_issues
from truth_mutations import mutation_tests

ROOT = Path(__file__).resolve().parents[1]
FORENSIC = Path("tmp/lkjmc-autonomous-evolution-plan/.control/artifacts/O-FORENSIC/prior-acceptance-map.json")
PACKET_PROBES = {"prior-items-have-probes", "old-runtime-shape-rejected", "generic-schema-rejected", "reactor-blocking-detected", "contracts-size-detected", "probe-mutation-tests"}
DETAIL_PROBES = {"payload-consumers-required", "menu-goldens-required", "doc-source-paths-required", "restore-drill-required"}
PROBES = PACKET_PROBES | DETAIL_PROBES
EXPECTED_CURRENT = {"old-runtime-shape-rejected", "menu-goldens-required", "doc-source-paths-required", "contracts-size-detected", "restore-drill-required"}
GOLDENS = {"root", "admin", "server", "shop", "docs", "settings"}
PREFIXES = ("crates/", "platforms/", "scripts/", "contracts/", "config/")
BOUNDARIES = (("crates/lkjmc-daemon/src/transport/command.rs", "api::dispatch_as("), ("crates/lkjmc-daemon/src/web/routes.rs", "handle_request("))
def text(root, path):
    target = root / path
    return target.read_text(encoding="utf-8") if target.is_file() else ""

def add(errors, probe, message):
    errors.append((probe, message))

def forensic_file(root):
    direct = root / FORENSIC
    if direct.is_file(): return direct
    result = subprocess.run(["git", "-C", str(root), "worktree", "list", "--porcelain"], capture_output=True, text=True, check=False)
    for line in result.stdout.splitlines():
        candidate = Path(line[9:]) / FORENSIC if line.startswith("worktree ") else direct
        if candidate.is_file(): return candidate
    return direct

def reopened_ids(root, errors):
    try:
        items = json.loads(forensic_file(root).read_text(encoding="utf-8"))["items"]
    except (OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        add(errors, "prior-items-have-probes", f"invalid forensic source: {error}")
        return set()
    reopened = {item.get("priorTask") for item in items if isinstance(item, dict) and item.get("classification") == "reopened"}
    if not reopened or not all(isinstance(item, str) and item for item in reopened):
        add(errors, "prior-items-have-probes", "forensic source has no valid reopened IDs")
    return reopened
def mapping(root, errors):
    expected = reopened_ids(root, errors)
    try:
        items = json.loads(text(root, "contracts/truth-probe-mapping.json"))["items"]
    except (KeyError, TypeError, json.JSONDecodeError) as error:
        add(errors, "prior-items-have-probes", f"invalid mapping: {error}")
        return
    actual = []
    for item in items if isinstance(items, list) else []:
        if not isinstance(item, dict):
            add(errors, "prior-items-have-probes", "mapping item is not an object")
            continue
        prior = item.get("priorItem")
        actual.append(prior)
        for field in ("probe", "futureTask"):
            if not isinstance(item.get(field), str) or not item[field].strip():
                add(errors, "prior-items-have-probes", f"{prior!r} has no {field}")
        if isinstance(item.get("probe"), str) and item["probe"] not in PROBES:
            add(errors, "prior-items-have-probes", f"{prior!r} has unknown probe {item['probe']}")
    found = {item for item in actual if isinstance(item, str) and item}
    if found != expected or len(actual) != len(found):
        missing, extra = sorted(expected - found), sorted(found - expected)
        add(errors, "prior-items-have-probes", f"mapping must cover reopened IDs exactly: missing={missing} extra={extra} duplicates={len(actual) - len(found)}")
def contracts(root, errors):
    errors.extend(contract_issues(root))
    directory = root / "platforms/jvm/common/src/test/resources/menu-goldens"
    missing = sorted(name for name in GOLDENS if not (directory / f"{name}.json").is_file())
    if missing:
        add(errors, "menu-goldens-required", f"missing menu goldens: {', '.join(missing)}")
def docs(root, errors):
    stale = []
    for path in (root / "docs").rglob("*.md"):
        for value in re.findall(r"`([^`]+)`", path.read_text(encoding="utf-8")):
            source = value.split()[0] if value.split() else ""
            if source.startswith(PREFIXES) and not any(token in source for token in ("*", "...")) and not (root / source).exists():
                stale.append(f"{path.relative_to(root)}: {source}")
    if stale:
        add(errors, "doc-source-paths-required", stale[0])
    checker = text(root, "scripts/check-docs.py")
    if "def check_state_sources" in checker and "ROOT / 'state'" in checker:
        add(errors, "contracts-size-detected", "source-path validation is limited to docs/state")
def runtime(root, errors):
    mutex = r"Arc\s*<\s*Mutex\s*<\s*Box\s*<\s*dyn\s+RuntimeAdapter\s*>\s*>\s*>"
    if re.search(mutex, text(root, "crates/lkjmc-daemon/src/app.rs")):
        add(errors, "old-runtime-shape-rejected", "daemon-wide RuntimeAdapter mutex remains")
    for path, call in BOUNDARIES:
        source, position = text(root, path), text(root, path).find(call)
        start = max(source.rfind("\nfn ", 0, position), source.rfind("\nasync fn ", 0, position))
        if position < 0 or "tokio::task::spawn_blocking" not in source[max(start, 0):position]:
            add(errors, "reactor-blocking-detected", f"{path} dispatch escapes its blocking boundary")
def check(root):
    errors = []
    mapping(root, errors)
    contracts(root, errors)
    docs(root, errors)
    runtime(root, errors)
    if not (root / "tests/restore/clean-room-restore.sh").is_file():
        add(errors, "restore-drill-required", "no executed clean-room restore proof")
    return errors
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--expected-failures", action="store_true")
    parser.add_argument("--probe", choices=sorted(PROBES))
    args = parser.parse_args()
    if args.probe == "probe-mutation-tests":
        failures = mutation_tests(ROOT, reopened_ids, check, FORENSIC, GOLDENS, BOUNDARIES)
        print("ok probe-mutation-tests" if not failures else "failed: " + "; ".join(failures))
        return bool(failures)
    errors = check(ROOT)
    if args.probe:
        errors = [error for error in errors if error[0] == args.probe]
    if args.expected_failures:
        mapping_errors = [error for error in errors if error[0] == "prior-items-have-probes"]
        missing = sorted(EXPECTED_CURRENT - {probe for probe, _ in errors})
        mutations = mutation_tests(ROOT, reopened_ids, check, FORENSIC, GOLDENS, BOUNDARIES)
        if mapping_errors or missing or mutations:
            print("failed truth harness: " + "; ".join([message for _, message in mapping_errors] + missing + mutations))
            return 1
        print(f"ok expected truth failures detected={len({probe for probe, _ in errors})} mutations=12")
        return 0
    if errors:
        print("\n".join(f"{probe}: {message}" for probe, message in errors))
        return 1
    print("ok check-truth-probes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
