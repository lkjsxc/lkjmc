#!/usr/bin/env python3
"""Migrate first-party authoring literals to stable IDs; never used at runtime.

Run after integrating UI/game branches: python scripts/migrate_messages.py --write.
New translated strings may be supplied as {English: Japanese} via --additions.
A --check run verifies sources and catalogs without modifying anything.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
LITERAL = r'"(?:[^"\\]|\\.)*"'
TOKEN = re.compile(LITERAL)
SLOT = re.compile(r'\{([A-Za-z_0-9]+)\}')
CALL = re.compile(r'\b(?:t|tr|message|Messages\.text|ctx\.text)\s*\(')
ERROR = re.compile(r'Error::(?:invalid|conflict|unavailable)\s*\(\s*(' + LITERAL + r')')


def stable_id(source: str) -> str:
    stem = re.sub(r'[^a-z0-9]+', '_', source.lower()).strip('_')
    if len(stem) <= 65 and stem:
        return 'text.' + stem
    return 'text.' + stem[:55].rstrip('_') + '_' + hashlib.sha256(source.encode()).hexdigest()[:10]


def call_end(source: str, start: int) -> int:
    depth, quoted, escaped = 1, False, False
    for i in range(start, len(source)):
        c = source[i]
        if quoted:
            if escaped: escaped = False
            elif c == '\\': escaped = True
            elif c == '"': quoted = False
        elif c == '"': quoted = True
        elif c == '(': depth += 1
        elif c == ')':
            depth -= 1
            if depth == 0: return i
    raise ValueError('Unclosed localization call')


def migrate_source(source: str, mapping: dict[str, str]) -> str:
    spans = []
    for match in CALL.finditer(source):
        end = call_end(source, match.end())
        spans.append((match.end(), end))
    # Work right-to-left, merge nested calls into their outer call.
    outer = []
    for start, end in spans:
        if not outer or start > outer[-1][1]: outer.append((start, end))
    for start, end in reversed(outer):
        fragment = source[start:end]
        # Java split string constants are one translation source.
        fragment = re.sub('(' + LITERAL + r')(?:\s*\+\s*' + LITERAL + ')+',
            lambda m: json.dumps(''.join(json.loads(x.group()) for x in TOKEN.finditer(m.group())), ensure_ascii=False), fragment)
        fragment = TOKEN.sub(lambda m: json.dumps(mapping.get(json.loads(m.group()), json.loads(m.group())), ensure_ascii=False), fragment)
        source = source[:start] + fragment + source[end:]
    source = ERROR.sub(lambda m: m.group().replace(m.group(1), json.dumps(mapping.get(json.loads(m.group(1)), json.loads(m.group(1))))), source)
    # Existing API tables feed dynamic t() calls; their values are system text.
    if 'export const states:' in source:
        for marker in ['export const states:', 'const names:']:
            start = source.find(marker)
            if start < 0: continue
            end = source.find('\n  };' if marker.startswith('const') else '\n};', start)
            if end < 0: continue
            source = source[:start] + TOKEN.sub(lambda m: json.dumps(mapping.get(json.loads(m.group()), json.loads(m.group())), ensure_ascii=False), source[start:end]) + source[end:]
    return source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--catalogs-only', action='store_true')
    parser.add_argument('--additions', type=Path)
    args = parser.parse_args()
    manifest_path = ROOT / 'locales/messages.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    mapping = {entry['source']: key for key, entry in manifest.items()}
    en = json.loads((ROOT / 'locales/en.json').read_text())
    ja = json.loads((ROOT / 'locales/ja.json').read_text())
    additions = json.loads(args.additions.read_text()) if args.additions else {}
    originals = {key: value for key, value in en.items() if key not in manifest}
    originals.update({key: key for key in additions})
    for source in originals:
        key = mapping.get(source, stable_id(source))
        if key in manifest and manifest[key]['source'] != source:
            key += '_' + hashlib.sha256(source.encode()).hexdigest()[:8]
        translated = additions.get(source, ja.get(source))
        if not translated:
            raise ValueError('Missing Japanese translation: ' + source)
        mapping[source] = key
        manifest[key] = {'source': source, 'params': sorted(set(SLOT.findall(source)))}
        en[key], ja[key] = originals[source], translated
        if key != source:
            en.pop(source, None); ja.pop(source, None)
    unknown = []
    candidates = [*ROOT.glob('web/src/*.ts'), *ROOT.glob('web/src/*.tsx'), *ROOT.glob('plugins/**/src/main/java/**/*.java'), *ROOT.glob('crates/core/src/**/*.rs')]
    if not args.catalogs_only:
        for path in candidates:
            if path.name in {'i18n.ts', 'Messages.java', 'system_message.rs'}: continue
            source = path.read_text()
            for match in CALL.finditer(source):
                fragment = source[match.end():call_end(source, match.end())]
                for token in TOKEN.finditer(fragment):
                    literal = json.loads(token.group())
                    # Only initial t() argument is required; conditionals may use unrelated keys.
                    if token.start() == len(fragment) - len(fragment.lstrip()) and literal not in mapping and literal not in manifest:
                        unknown.append(f'{path.relative_to(ROOT)}: {literal}')
            updated = migrate_source(source, mapping)
            if args.write and updated != source: path.write_text(updated)
    if unknown: raise ValueError('Untranslated authoring sources:\n' + '\n'.join(unknown))
    if args.write:
        for name, data in [('messages', manifest), ('en', en), ('ja', ja)]:
            (ROOT / f'locales/{name}.json').write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
    print(f'{len(manifest)} shared message IDs; {len(candidates)} source files examined')

if __name__ == '__main__': main()
