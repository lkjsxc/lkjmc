#!/usr/bin/env python3
from pathlib import Path
import argparse
import json
import sys

CONTRACT = Path('contracts/commands.json')
CATALOG = Path('docs/architecture/runtime/daemon/command-catalog.md')


def load_commands():
    return json.loads(CONTRACT.read_text())['commands']


def render(commands):
    families = {}
    for command in commands:
        families.setdefault(command['family'], []).append(command)
    lines = [
        '# Command catalog',
        '',
        '## Purpose',
        '',
        'This generated document lists public daemon command literals from',
        '[contracts/commands.json](../../../../contracts/commands.json).',
        '',
    ]
    for family in sorted(families):
        lines += [f'## {family}', '']
        for command in families[family]:
            lines.append(f"- `{command['name']}` — {command['summary']}")
        lines.append('')
    lines += [
        '## Verification',
        '',
        '`scripts/check-command-docs.py` verifies this catalog, command docs,',
        'and daemon registration tests against `contracts/commands.json`.',
        '',
    ]
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    rendered = render(load_commands())
    if args.check:
        if CATALOG.read_text() != rendered:
            print(f'{CATALOG}: run scripts/generate-command-catalog.py')
            return 1
        print('ok generate-command-catalog')
        return 0
    CATALOG.write_text(rendered)
    print(f'generated {CATALOG}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
