#!/usr/bin/env python3
from pathlib import Path
import sys

CHECKS = [
    (
        "docs/architecture/runtime/daemon/commands/bootstrap.md",
        ["bootstrap.plan", "bootstrap.apply", "bootstrap.status", "bootstrap.doctor"],
    ),
    (
        "docs/product/commands/ssh-cli.md",
        [
            "lkjmc bootstrap plan",
            "lkjmc bootstrap apply",
            "lkjmc bootstrap status",
            "lkjmc bootstrap doctor",
        ],
    ),
    (
        "docs/operations/quickstart/playable-network.md",
        ["acceptance record", "never writes", "hub", "survival"],
    ),
    (
        "docs/product/network/playable-default.md",
        ["TCP", "25565", "UDP", "19132", "modern", "hub"],
    ),
]


def main() -> int:
    errors = []
    for path_text, needles in CHECKS:
        path = Path(path_text)
        if not path.exists():
            errors.append(f"missing {path}")
            continue
        text = path.read_text(encoding="utf-8")
        for needle in needles:
            if needle not in text:
                errors.append(f"{path}: missing {needle}")
    if errors:
        print("\n".join(errors))
        return 1
    print("ok check-bootstrap-docs")
    return 0


if __name__ == "__main__":
    sys.exit(main())
