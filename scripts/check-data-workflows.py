#!/usr/bin/env python3
import argparse
import os
import sys
from urllib.parse import urlparse

from data_workflow_checks import ALL_PROBES, DB_PROBES, cargo_test, inventory_errors
from data_workflow_mutations import old_path_errors


def database_ready():
    value = os.environ.get("LKJMC_STORE_TEST_DATABASE_URL", "")
    try:
        parsed = urlparse(value)
        return parsed.scheme in {"postgres", "postgresql"} and bool(parsed.hostname and parsed.path.strip("/"))
    except ValueError:
        return False


def run(probe, allow_database_skip=False):
    if probe == "all-multiwrites-classified":
        errors = inventory_errors()
        if errors:
            print("\n".join(errors), file=sys.stderr)
            return 1
        return 0
    if probe == "old-workflows-absent":
        errors = old_path_errors()
        if errors:
            print("\n".join(errors), file=sys.stderr)
            return 1
        return 0
    if probe == "profile-format-safe-complete":
        return cargo_test("lkjmc-core", "profile_")
    if probe in DB_PROBES:
        if not database_ready():
            if allow_database_skip:
                print(f"skip check-data-workflows probe={probe} reason=database-url-absent")
                return 0
            print(f"{probe}: valid LKJMC_STORE_TEST_DATABASE_URL is required", file=sys.stderr)
            return 2
        return cargo_test("lkjmc-store", probe.replace("-", "_"), "data_workflows")
    raise ValueError(probe)


def main():
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--probe", choices=ALL_PROBES)
    group.add_argument("--all", action="store_true")
    parser.add_argument("--allow-database-skip", action="store_true")
    args = parser.parse_args()
    if args.allow_database_skip and not args.all:
        parser.error("--allow-database-skip requires --all")
    probes = ALL_PROBES if args.all else [args.probe]
    for probe in probes:
        skipped = probe in DB_PROBES and not database_ready() and args.allow_database_skip
        result = run(probe, args.allow_database_skip)
        if result:
            return result
        if not skipped:
            print(f"ok check-data-workflows probe={probe}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
