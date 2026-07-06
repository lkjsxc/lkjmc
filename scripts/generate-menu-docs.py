#!/usr/bin/env python3
from pathlib import Path
import argparse
import json
import sys

MENU_DIR = Path('contracts/menus')
OUT_DIR = Path('docs/product/gui/routes')
INDEX = MENU_DIR / 'README.json'
THEMES = ['root', 'network', 'travel', 'claims', 'economy', 'social',
          'profile', 'settings', 'staff', 'adventure', 'danger', 'docs']


def load_docs():
    docs = []
    for path in sorted(MENU_DIR.glob('*.json')):
        if path == INDEX:
            continue
        item = json.loads(path.read_text(encoding='utf-8'))
        item['_path'] = path
        docs.append(item)
    return docs


def grouped(docs):
    groups = {theme: [] for theme in THEMES}
    for doc in sorted(docs, key=lambda item: item['id']):
        groups.setdefault(doc['theme'], []).append(doc)
    return {theme: groups[theme] for theme in groups if groups[theme]}


def rel_contract(doc):
    return '../../../../contracts/menus/' + doc['id'] + '.json'


def render_readme(groups):
    lines = [
        '# Menu route catalog', '', '## Purpose', '',
        'This generated directory lists menu route documents by theme.', '',
        '## Status', '', 'implemented', '', '## Table of contents', '',
    ]
    for theme in groups:
        lines.append(f'- [{theme.title()} routes]({theme}.md)')
    lines += ['', '## Verification', '',
              '`scripts/generate-menu-docs.py --check` verifies this catalog.', '']
    return '\n'.join(lines)


def data_summary(doc):
    data = doc.get('data') or {}
    binding = data.get('binding', '—')
    source = data.get('source', '—')
    commands = ', '.join(f'`{cmd}`' for cmd in data.get('commands', [])) or '—'
    return binding, source, commands


def render_theme(theme, docs):
    lines = [
        f'# {theme.title()} menu routes', '', '## Purpose', '',
        f'This generated file lists `{theme}` menu routes from',
        '[contracts/menus](../../../../contracts/menus).', '',
        '## Status', '', 'implemented', '', '## Routes', '',
        '| Route | Kind | Parent | Binding | Source | Data commands | Confirmation |',
        '| --- | --- | --- | --- | --- | --- | --- |',
    ]
    for doc in docs:
        binding, source, commands = data_summary(doc)
        parent = doc.get('parent') or '—'
        confirm = doc.get('confirmation') or '—'
        route = f'[`{doc["id"]}`]({rel_contract(doc)})'
        lines.append(f'| {route} | {doc["kind"]} | {parent} | {binding} | '
                     f'{source} | {commands} | {confirm} |')
    lines.append('')
    return '\n'.join(lines)


def rendered_files():
    groups = grouped(load_docs())
    files = {OUT_DIR / 'README.md': render_readme(groups)}
    for theme, docs in groups.items():
        files[OUT_DIR / f'{theme}.md'] = render_theme(theme, docs)
    return files


def index_text():
    ids = [doc['id'] for doc in sorted(load_docs(), key=lambda item: item['id'])]
    return json.dumps({'menus': ids}, indent=2) + '\n'


def write_files(files):
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for old in OUT_DIR.glob('*.md'):
        if old not in files:
            old.unlink()
    for path, text in files.items():
        path.write_text(text, encoding='utf-8')
    INDEX.write_text(index_text(), encoding='utf-8')


def check_files(files):
    errors = []
    if not OUT_DIR.exists():
        errors.append(f'{OUT_DIR}: run scripts/generate-menu-docs.py')
    for path, text in files.items():
        if not path.exists():
            errors.append(f'{path}: run scripts/generate-menu-docs.py')
        elif path.read_text(encoding='utf-8') != text:
            errors.append(f'{path}: run scripts/generate-menu-docs.py')
        if len(text.splitlines()) > 200:
            errors.append(f'{path}: generated route doc exceeds 200 lines')
    expected_index = index_text()
    if not INDEX.exists() or INDEX.read_text(encoding='utf-8') != expected_index:
        errors.append(f'{INDEX}: run scripts/generate-menu-docs.py')
    if OUT_DIR.exists():
        for path in OUT_DIR.glob('*.md'):
            if path not in files:
                errors.append(f'{path}: stale generated menu route catalog file')
    return errors


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    files = rendered_files()
    if args.check:
        errors = check_files(files)
        if errors:
            print('\n'.join(errors))
            return 1
        print('ok generate-menu-docs')
        return 0
    write_files(files)
    print(f'generated {OUT_DIR}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
