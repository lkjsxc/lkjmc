#!/usr/bin/env python3
import argparse
import sys

from sync_adoption_checks import PROBES, ROOT, command, prerequisites, run_probe, source_errors


def main():
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--probe", choices=PROBES)
    group.add_argument("--all", action="store_true")
    args = parser.parse_args()
    errors = prerequisites() + source_errors()
    if errors:
        print("\n".join(errors))
        return 2
    if args.probe:
        result = run_probe(args.probe)
        if result:
            return result
        print(f"ok check-sync-adoption probe={args.probe}")
        return 0
    if command(["cargo", "test", "-p", "lkjmc-store", "--test", "sync", "--test", "sync_coherence", "--", "--nocapture"]):
        return 1
    if command(["cargo", "build", "-p", "lkjmc-daemon"]):
        return 1
    result = command([
        "./gradlew", "--no-daemon", "--no-build-cache",
        ":platforms:jvm:common:syncHarness", "-PsyncProbe=all",
    ])
    if result:
        return result
    for probe in PROBES:
        print(f"ok check-sync-adoption probe={probe}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
