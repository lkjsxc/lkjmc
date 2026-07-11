#!/usr/bin/env python3
from pathlib import Path
import re
import sys

DOC = Path("docs/architecture/security/permissions.md")
PLUGIN = Path("platforms/jvm/paper/src/main/resources/plugin.yml")
PATTERN = re.compile(r"lkjmc\.(?:user|admin)\.[a-z0-9.]+")


def main():
    source = set(PATTERN.findall(PLUGIN.read_text(encoding="utf-8")))
    documented = set(PATTERN.findall(DOC.read_text(encoding="utf-8")))
    errors = [f"permissions docs: missing {value}" for value in sorted(source - documented)]
    errors += [f"permissions docs: unknown {value}" for value in sorted(documented - source)]
    if errors:
        print("\n".join(errors))
        return 1
    print("ok check-permissions")
    return 0


if __name__ == "__main__":
    sys.exit(main())
