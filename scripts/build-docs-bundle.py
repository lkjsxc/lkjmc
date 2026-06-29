#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def title(lines, fallback):
    for line in lines:
        if line.startswith('# '):
            return line[2:].strip()
    return fallback


def links(lines):
    found = []
    pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')
    for number, line in enumerate(lines, 1):
        for match in pattern.finditer(line):
            found.append({'text': match.group(1), 'target': match.group(2), 'line': number})
    return found


def heading_slug(text):
    slug = re.sub(r'[^a-z0-9\s-]', '', text.lower()).strip()
    return re.sub(r'[\s-]+', '-', slug)


def headings(lines):
    values = []
    for number, line in enumerate(lines, 1):
        match = re.match(r'^(#{1,6})\s+(.+)$', line)
        if match:
            values.append({'level': len(match.group(1)), 'title': match.group(2).strip(),
                           'slug': heading_slug(match.group(2)), 'line': number})
    return values


def collect():
    files = [ROOT / 'README.md', ROOT / 'AGENTS.md'] + sorted((ROOT / 'docs').rglob('*.md'))
    entries = []
    for file in files:
        rel = file.relative_to(ROOT).as_posix()
        raw = file.read_text(encoding='utf-8').splitlines()
        entries.append({'path': rel, 'title': title(raw, file.stem), 'lines': raw,
                        'links': links(raw), 'headings': headings(raw)})
    return {'version': 1, 'files': entries}


def main():
    if len(sys.argv) != 2:
        print('usage: build-docs-bundle.py OUTPUT', file=sys.stderr)
        return 2
    out = Path(sys.argv[1])
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(collect(), ensure_ascii=False, separators=(',', ':')) + '\n', encoding='utf-8')
    print(f'ok docs bundle {out}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
