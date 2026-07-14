#!/usr/bin/env python3
from pathlib import Path
import re
import sys

DOCS = [
    Path("docs/architecture/assets/plugin-jars.md"),
    Path("docs/architecture/assets/download-policy.md"),
    Path("docs/architecture/plugin/third-party-policy.md"),
    Path("docs/product/network/bedrock-entry.md"),
    Path("docs/product/network/java-compatibility.md"),
]
PLUGIN_SOURCE = Path("crates/lkjmc-core/src/plugin.rs")


def plugin_ids() -> list[str]:
    text = PLUGIN_SOURCE.read_text(encoding="utf-8")
    body_match = re.search(r"pub fn as_str\(self\).*?match self \{(.*?)\n        \}", text, re.S)
    if not body_match:
        return []
    return sorted(set(re.findall(r'"([a-z0-9-]+)"', body_match.group(1))))


def main() -> int:
    errors = []
    docs_text = "\n".join(path.read_text(encoding="utf-8") for path in DOCS)
    for plugin_id in plugin_ids():
        if plugin_id not in docs_text:
            errors.append(f"docs: missing plugin id {plugin_id}")
    required_phrases = [
        "hash verification",
        "ViaBackwards requires ViaVersion",
        "Floodgate `key.pem` is never logged",
        "Geyser and Floodgate",
    ]
    lower_docs = docs_text.lower()
    for phrase in required_phrases:
        if phrase.lower() not in lower_docs:
            errors.append(f"docs: missing phrase {phrase}")
    if errors:
        print("\n".join(errors))
        return 1
    print("ok check-asset-docs")
    return 0


if __name__ == "__main__":
    sys.exit(main())
