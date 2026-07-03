#!/usr/bin/env python3
from pathlib import Path
import json
import re
import sys

LOCALES = [Path('config/locales/en.json'), Path('config/locales/ja.json')]
CATALOG_DOC = Path('docs/product/i18n/catalog.md')
JAVA_ROOTS = [Path('platforms/jvm/common/src'), Path('platforms/jvm/paper/src'), Path('platforms/jvm/velocity/src')]
KEY_RE = re.compile(r'"([a-z][a-z0-9_.-]+)"')


def load(path):
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f'{path}: locale catalog must be an object')
    bad = [key for key, item in value.items() if not isinstance(item, str)]
    if bad:
        raise ValueError(f'{path}: non-string key {bad[0]}')
    return value


def referenced_keys():
    keys = set()
    for root in JAVA_ROOTS:
        for path in root.rglob('*.java'):
            text = path.read_text()
            keys |= {match.group(1) for match in KEY_RE.finditer(text) if match.group(1).startswith('velocity.')}
    return keys


def main():
    errors = []
    doc = CATALOG_DOC.read_text()
    catalogs = {}
    for path in LOCALES:
        if str(path) not in doc:
            errors.append(f'locale docs: missing {path}')
        try:
            catalogs[path] = load(path)
        except ValueError as error:
            errors.append(str(error))
    if catalogs:
        base = set(catalogs[LOCALES[0]])
        for path, values in catalogs.items():
            keys = set(values)
            errors += [f'{path}: missing {key}' for key in sorted(base - keys)]
            errors += [f'{path}: extra {key}' for key in sorted(keys - base)]
        missing = referenced_keys() - base
        errors += [f'locale refs: missing {key}' for key in sorted(missing)]
    if errors:
        print('\n'.join(errors))
        return 1
    print('ok check-locales')
    return 0


if __name__ == '__main__':
    sys.exit(main())
