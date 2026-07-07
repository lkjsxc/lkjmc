#!/usr/bin/env python3
from pathlib import Path
import json
import re
import sys

LOCALES = [Path('config/locales/en.json'), Path('config/locales/ja.json')]
CATALOG_DOC = Path('docs/product/i18n/catalog.md')
JAVA_ROOTS = [Path('platforms/jvm/common/src'), Path('platforms/jvm/paper/src'), Path('platforms/jvm/velocity/src')]
KEY_RE = re.compile(r'"([a-z][a-z0-9_.-]+)"')
JA_RE = re.compile(r'[ぁ-んァ-ン一-龯]')
ASCII_RE = re.compile(r'[A-Za-z]')
ALLOW_JA_ASCII = {'menu.admin.ban', 'menu.decorative'}
REF_PREFIXES = ('velocity.',)
TAG_RE = re.compile(r'<[^>]+>')


def load(path):
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f'{path}: locale catalog must be an object')
    bad = [key for key, item in value.items() if not isinstance(item, str)]
    if bad:
        raise ValueError(f'{path}: non-string key {bad[0]}')
    return value


def referenced_keys(base):
    keys = set()
    for root in JAVA_ROOTS:
        for path in root.rglob('*.java'):
            text = path.read_text()
            for match in KEY_RE.finditer(text):
                key = match.group(1)
                if key in base or key.startswith(REF_PREFIXES):
                    keys.add(key)
    return keys


def locale_quality(path, values):
    if path.name != 'ja.json':
        return []
    errors = []
    for key, value in values.items():
        if key in ALLOW_JA_ASCII:
            continue
        plain = TAG_RE.sub('', value).replace('\\<', '').replace('\\>', '')
        if ASCII_RE.search(plain) and not JA_RE.search(plain) and not allowed_ascii(plain):
            errors.append(f'{path}: {key} appears untranslated: {value}')
    return errors


def allowed_ascii(value):
    text = value.strip()
    if not text:
        return True
    if text.startswith('/') or text.startswith('http'):
        return True
    if re.fullmatch(r'[A-Z0-9_./: -]+', text):
        return True
    return False


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
            errors += locale_quality(path, values)
        missing = referenced_keys(base) - base
        errors += [f'locale refs: missing {key}' for key in sorted(missing)]
    if errors:
        print('\n'.join(errors))
        return 1
    print('ok check-locales')
    return 0


if __name__ == '__main__':
    sys.exit(main())
