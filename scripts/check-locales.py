#!/usr/bin/env python3
from pathlib import Path
import json
import sys

CATALOGS = [
    Path('config/locales/en.json'),
    Path('config/locales/ja.json'),
    Path('platforms/jvm/common/src/main/resources/locales/en.json'),
    Path('platforms/jvm/common/src/main/resources/locales/ja.json'),
]
CATALOG_DOC = Path('docs/product/i18n/catalog.md')


def leaf_keys(value, prefix=''):
    if isinstance(value, dict):
        keys = set()
        for key, child in value.items():
            child_prefix = f'{prefix}.{key}' if prefix else key
            keys |= leaf_keys(child, child_prefix)
        return keys
    return {prefix}


def main():
    errors = []
    key_sets = {}
    for path in CATALOGS:
        if str(path) not in CATALOG_DOC.read_text():
            errors.append(f'locale docs: missing {path}')
        key_sets[path] = leaf_keys(json.loads(path.read_text()))
    base_path = CATALOGS[0]
    base = key_sets[base_path]
    for path, keys in key_sets.items():
        missing = base - keys
        extra = keys - base
        errors += [f'{path}: missing {key}' for key in sorted(missing)]
        errors += [f'{path}: extra {key}' for key in sorted(extra)]
    if errors:
        print('\n'.join(errors))
        return 1
    print('ok check-locales')
    return 0


if __name__ == '__main__':
    sys.exit(main())
