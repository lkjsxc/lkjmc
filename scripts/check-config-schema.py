#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

RUST = Path('crates/lkjmc-core/src/config/schema.rs')
JAVA = Path('platforms/jvm/common/src/main/resources/lkjmc-config-contract.json')
DOC = Path('docs/contracts/config-schema.md')


def rust_fields(text: str) -> set[str]:
    block = re.search(r'REQUIRED_JAVA_FIELDS:\s*&\[&str\]\s*=\s*&\[(.*?)\];', text, re.S)
    if not block:
        return set()
    return set(re.findall(r'"([A-Z0-9_]+)"', block.group(1)))


def main() -> int:
    errors = []
    fields = rust_fields(RUST.read_text())
    resource = set(json.loads(JAVA.read_text()).get('fields', []))
    if not fields:
        errors.append('config schema: no Rust fields found')
    for field in sorted(fields - resource):
        errors.append(f'config schema: Java resource missing {field}')
    for field in sorted(resource - fields):
        errors.append(f'config schema: Java resource extra {field}')
    doc_text = DOC.read_text()
    for field in sorted(fields):
        if field not in doc_text:
            errors.append(f'config schema docs: missing {field}')
    if errors:
        print('\n'.join(errors))
        return 1
    print('ok check-config-schema')
    return 0


if __name__ == '__main__':
    sys.exit(main())
