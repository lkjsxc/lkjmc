#!/usr/bin/env python3
"""Compatibility entrypoint for the command-contract check."""
import subprocess
import sys


def main():
    result = subprocess.run(["./scripts/check-contracts.py"], check=False)
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
