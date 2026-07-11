#!/usr/bin/env python3
"""Compatibility entry point for the Java containment check."""
import subprocess
import sys

sys.exit(subprocess.call(["./scripts/check-jvm-containment.py"]))
