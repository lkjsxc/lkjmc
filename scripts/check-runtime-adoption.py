#!/usr/bin/env python3
import argparse
import sys

from runtime_adoption_checks import PROBES, run


def main():
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--probe", choices=PROBES)
    group.add_argument("--all", action="store_true")
    args = parser.parse_args()
    probes = PROBES if args.all else [args.probe]
    for probe in probes:
        result = run(probe)
        if result:
            return result
        print(f"ok check-runtime-adoption probe={probe}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
