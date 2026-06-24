#!/usr/bin/env python3
from pathlib import Path
import re
import sys

ROOT = Path('docs')
LINK_RE = re.compile(r'\[[^\]]+\]\(([^)]+)\)')
BANNED = {
    'milestone label': re.compile(r'\bmilestone\b', re.I),
    'short release tag': re.compile(r'\bv\d+(?:\.\d+)*\b'),
    'old-behavior narration': re.compile(r'\blegacy\b', re.I),
}


def docs_dirs():
    return [ROOT] + sorted(p for p in ROOT.rglob('*') if p.is_dir())


def child_targets(directory: Path):
    targets = []
    for child in sorted(directory.iterdir()):
        if child.name == 'README.md':
            continue
        if child.is_file() and child.suffix == '.md':
            targets.append(child.name)
        elif child.is_dir():
            has_docs = any(p.suffix == '.md' for p in child.rglob('*.md'))
            if has_docs:
                targets.append(f'{child.name}/README.md')
    return targets


def check_readme(directory: Path, errors: list[str]):
    if not directory.exists():
        errors.append(f'missing docs directory: {directory}')
        return
    readmes = [p for p in directory.iterdir() if p.name == 'README.md']
    if len(readmes) != 1:
        errors.append(f'{directory}: expected exactly one README.md')
        return
    text = readmes[0].read_text(encoding='utf-8')
    if '## Table of contents' not in text:
        errors.append(f'{readmes[0]}: missing table of contents heading')
    for target in child_targets(directory):
        if f']({target})' not in text:
            errors.append(f'{readmes[0]}: missing link to {target}')


def check_file(path: Path, errors: list[str]):
    text = path.read_text(encoding='utf-8')
    lines = text.splitlines()
    if not lines or not lines[0].startswith('# ') or lines[0].startswith('## '):
        errors.append(f'{path}: first line must be one H1')
    if sum(1 for line in lines if line.startswith('# ')) != 1:
        errors.append(f'{path}: expected exactly one H1')
    if '## Purpose' not in text:
        errors.append(f'{path}: missing ## Purpose')
    for label, regex in BANNED.items():
        if regex.search(text):
            errors.append(f'{path}: contains banned {label}')
    for link in LINK_RE.findall(text):
        url = link.split('#', 1)[0]
        if not url or url.startswith(('http://', 'https://', 'mailto:')):
            continue
        target = (path.parent / url).resolve()
        if not target.exists():
            errors.append(f'{path}: broken link {link}')


def main() -> int:
    errors = []
    if not ROOT.exists():
        errors.append('docs directory missing')
    else:
        for directory in docs_dirs():
            check_readme(directory, errors)
        for path in sorted(ROOT.rglob('*.md')):
            check_file(path, errors)
        if not (ROOT / 'current-state.md').exists():
            errors.append('docs/current-state.md missing')
    if errors:
        for error in errors:
            print(error)
        return 1
    print('ok check-docs')
    return 0


if __name__ == '__main__':
    sys.exit(main())
