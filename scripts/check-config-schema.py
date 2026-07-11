#!/usr/bin/env python3
"""Check that local-safe Java plugins have no daemon config mirror."""
from pathlib import Path
import sys

RUST = Path("crates/lkjmc-core/src/config.rs")
JAVA_CONFIG = Path("platforms/jvm/common/src/main/java/com/lkjmc/common/config")
JAVA_RESOURCE = Path("platforms/jvm/common/src/main/resources/lkjmc-config-contract.json")
DOC = Path("docs/contracts/config-schema.md")


def main():
    errors = []
    if "pub mod schema;" in RUST.read_text(encoding="utf-8"):
        errors.append("config schema: withdrawn Java schema module is still exported")
    if JAVA_CONFIG.exists() or JAVA_RESOURCE.exists():
        errors.append("config schema: local-safe Java plugin retains daemon config input")
    text = DOC.read_text(encoding="utf-8")
    if "consume no daemon URL" not in text:
        errors.append("config schema docs: missing local-safe Java boundary")
    if errors:
        print("\n".join(errors))
        return 1
    print("ok check-config-schema")
    return 0


if __name__ == "__main__":
    sys.exit(main())
