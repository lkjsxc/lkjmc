#!/usr/bin/env python3
from pathlib import Path
import argparse
import json
import sys

CONTRACT = Path('contracts/commands.json')
CATALOG_DIR = Path('docs/architecture/runtime/daemon/commands')
OLD_CATALOG = Path('docs/architecture/runtime/daemon/command-catalog.md')


def load_commands():
    return json.loads(CONTRACT.read_text(encoding='utf-8'))['commands']


def families(commands):
    grouped = {}
    for command in commands:
        grouped.setdefault(command['family'], []).append(command)
    return {family: sorted(items, key=lambda item: item['name'])
            for family, items in sorted(grouped.items())}


def family_path(family):
    return CATALOG_DIR / f'{family}.md'


def render_readme(grouped):
    lines = [
        '# Daemon command families',
        '',
        '## Purpose',
        '',
        'This generated directory groups public daemon command literals by',
        'product family.',
        '',
        '## Status',
        '',
        'implemented',
        '',
        '## Table of contents',
        '',
    ]
    for family in grouped:
        lines.append(f'- [{family.title()} commands]({family}.md)')
    lines += [
        '',
        '## Verification',
        '',
        '`scripts/check-command-docs.py` verifies this generated catalog and',
        'the command registry against `contracts/commands.json`.',
        '',
    ]
    return '\n'.join(lines)


def render_family(family, commands):
    title = family.title()
    lines = [
        f'# {title} commands',
        '',
        '## Purpose',
        '',
        f'This generated file lists `{family}` daemon command literals from',
        '[contracts/commands.json](../../../../../contracts/commands.json).',
        '',
        '## Status',
        '',
        'implemented',
        '',
        '## Commands',
        '',
        '| Command | Authorization | Surfaces | Summary |',
        '| --- | --- | --- | --- |',
    ]
    for command in commands:
        surfaces = ', '.join(command['surfaces'])
        lines.append(
            f"| `{command['name']}` | {command['authorization']} | "
            f"{surfaces} | {command['summary']} |"
        )
    lines.append('')
    return '\n'.join(lines)


def rendered_files(commands):
    grouped = families(commands)
    files = {CATALOG_DIR / 'README.md': render_readme(grouped)}
    for family, items in grouped.items():
        files[family_path(family)] = render_family(family, items)
    return files


def write_catalog(files):
    CATALOG_DIR.mkdir(parents=True, exist_ok=True)
    for path in CATALOG_DIR.glob('*.md'):
        if path not in files:
            path.unlink()
    for path, text in files.items():
        path.write_text(text, encoding='utf-8')
    if OLD_CATALOG.exists():
        OLD_CATALOG.unlink()


def check_catalog(files):
    errors = []
    if OLD_CATALOG.exists():
        errors.append(f'{OLD_CATALOG}: remove monolithic command catalog')
    for path, text in files.items():
        if not path.exists():
            errors.append(f'{path}: run scripts/generate-command-catalog.py')
        elif path.read_text(encoding='utf-8') != text:
            errors.append(f'{path}: run scripts/generate-command-catalog.py')
    if CATALOG_DIR.exists():
        expected = set(files)
        for path in CATALOG_DIR.glob('*.md'):
            if path not in expected:
                errors.append(f'{path}: stale generated command catalog file')
    else:
        errors.append(f'{CATALOG_DIR}: run scripts/generate-command-catalog.py')
    return errors


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    files = rendered_files(load_commands())
    if args.check:
        errors = check_catalog(files)
        if errors:
            print('\n'.join(errors))
            return 1
        print('ok generate-command-catalog')
        return 0
    if CATALOG_DIR.exists() and not CATALOG_DIR.is_dir():
        print(f'{CATALOG_DIR}: not a directory')
        return 1
    write_catalog(files)
    print(f'generated {CATALOG_DIR}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
