#!/usr/bin/env python3
"""Research-only policy module: data in, decision out, no effects."""
import json
import sys

ALLOW = {"subject": "operator", "operation": "inspect"}

for line in sys.stdin:
    try:
        request = json.loads(line)
        decision = "allow" if request == ALLOW else "deny"
    except (json.JSONDecodeError, TypeError):
        decision = "deny"
    print(json.dumps({"decision": decision}, separators=(",", ":")), flush=True)
