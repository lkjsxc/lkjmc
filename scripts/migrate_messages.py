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


def argument_spans(fragment: str) -> list[tuple[int, int]]:
    depth, quoted, escaped, start = 0, False, False, 0
    spans = []
    for i, char in enumerate(fragment):
        if quoted:
            if escaped: escaped = False
            elif char == "\\": escaped = True
            elif char == '"': quoted = False
        elif char == '"': quoted = True
        elif char in '([{': depth += 1
        elif char in ')]}': depth -= 1
        elif char == ',' and depth == 0:
            spans.append((start, i)); start = i + 1
    spans.append((start, len(fragment)))
    return spans


def template_expression(fragment: str, mapping: dict[str, str]) -> str:
    fragment = re.sub('(' + LITERAL + r')(?:\s*\+\s*' + LITERAL + ')+',
        lambda m: json.dumps(''.join(json.loads(x.group()) for x in TOKEN.finditer(m.group())), ensure_ascii=False), fragment)
    # Only a literal template or ternary result is translatable. Conditions and
    # data arguments (including values that happen to equal a catalog source) stay exact.
    def replace(match):
        prefix = fragment[:match.start()].rstrip()
        eligible = not prefix or prefix[-1] in '?:'
        literal = json.loads(match.group())
        return json.dumps(mapping.get(literal, literal), ensure_ascii=False) if eligible else match.group()
    return TOKEN.sub(replace, fragment)


def migrate_source(source: str, mapping: dict[str, str], route_labels: bool = False) -> str:
    spans = []
    for match in CALL.finditer(source):
        end = call_end(source, match.end())
        spans.append((match.end(), end, match.group()))
    for start, end, call in reversed(spans):
        end = call_end(source, start)
        fragment = source[start:end]
        args = argument_spans(fragment)
        index = 0 if re.match(r'^(?:t|message)\b', call.lstrip()) is not None else 1
        if index >= len(args): continue
        left, right = args[index]
        fragment = fragment[:left] + template_expression(fragment[left:right], mapping) + fragment[right:]
        source = source[:start] + fragment + source[end:]
    source = ERROR.sub(lambda m: m.group().replace(m.group(1), json.dumps(mapping.get(json.loads(m.group(1)), json.loads(m.group(1))))), source)
    fmt = re.compile(r'Error::(?:invalid|conflict|unavailable)\s*\(\s*format!\s*\(')
    for match in reversed(list(fmt.finditer(source))):
        end = call_end(source, match.end())
        parts = argument_spans(source[match.end():end])
        fragment = source[match.end():end]
        template = json.loads(fragment[parts[0][0]:parts[0][1]].strip())
        if template not in mapping: continue
        params = re.findall(r'\{([A-Za-z_0-9]*)\}', template)
        explicit = [fragment[a:b].strip() for a,b in parts[1:] if fragment[a:b].strip()]
        replacement = 'crate::system_message::SystemMessage::new(' + json.dumps(mapping[template]) + ')'
        position = 0
        for name in dict.fromkeys(params):
            if name:
                replacement += '.with(' + json.dumps(name) + ', ' + name + ')'
            else:
                if position >= len(explicit): raise ValueError('Unbound format parameter: ' + template)
                replacement += '.with(' + json.dumps(str(position)) + ', ' + explicit[position] + ')'
                position += 1
        prefix = source[match.start():match.end()]
        outer = prefix[:prefix.index('format!')]
        source = source[:match.start()] + outer + replacement + source[end+1:]
    # These explicit first-party ID tables feed dynamic t() calls.
    table_markers = ['export const states:', 'const names:', 'export const hostingActionReasons', 'const noticeNames:']
    for marker in table_markers:
        start = source.find(marker)
        if start < 0: continue
        end = source.find('};', start)
        if end < 0: continue
        source = source[:start] + TOKEN.sub(lambda m: json.dumps(mapping.get(json.loads(m.group()), json.loads(m.group())), ensure_ascii=False), source[start:end]) + source[end:]
    if route_labels:
        # The route module's literal tuples contain only component/path keys and
        # authored display labels. Translate the last label, retaining earlier keys.
        array = re.compile(r'\[(?:' + LITERAL + r'\s*,\s*){1,4}' + LITERAL + r'\]')
        def labels(match):
            values = json.loads(match.group())
            positions = [1, 2] if len(values) == 5 else [len(values) - 1]
            for index in positions: values[index] = mapping.get(values[index], values[index])
            return json.dumps(values, ensure_ascii=False)
        source = array.sub(labels, source)
        set_calls = re.compile(r'\bset\s*\(')
        for match in reversed(list(set_calls.finditer(source))):
            end = call_end(source, match.end()); fragment = source[match.end():end]
            args = argument_spans(fragment)
            if len(args) < 3: continue
            left,right = args[2]
            source = source[:match.end()+left] + template_expression(fragment[left:right], mapping) + source[match.end()+right:]
        source = re.sub(r'\b(title|description)\s*:\s*(' + LITERAL + ')',
            lambda m: m.group(1) + ':' + json.dumps(mapping.get(json.loads(m.group(2)), json.loads(m.group(2)))), source)
        source = re.sub(r'\?\?\s*(' + LITERAL + ')',
            lambda m: '??' + json.dumps(mapping.get(json.loads(m.group(1)), json.loads(m.group(1)))), source)
        # Route maps contain system names only; keys themselves are identifiers.
        source = re.sub(r'([a-zA-Z_][a-zA-Z_0-9]*\s*:)\s*(' + LITERAL + ')',
            lambda m: m.group(1) + json.dumps(mapping.get(json.loads(m.group(2)), json.loads(m.group(2)))), source)
    return source


def generated_types(manifest: dict) -> str:
    lines = ['// Generated by scripts/migrate_messages.py; edit shared catalogs instead.',
             'export type MessageParameter = string | number | boolean | null;',
             'export interface MessageParameters {']
    for key, entry in manifest.items():
        params = entry['params']
        shape = '{ ' + '; '.join(json.dumps(name) + ': MessageParameter' for name in params) + ' }' if params else 'Record<string, never>'
        lines.append('  ' + json.dumps(key) + ': ' + shape + ';')
    lines.extend(['}', 'export type MessageId = keyof MessageParameters;', 'export interface MessageArguments {'])
    for key, entry in manifest.items():
        params = entry['params']
        if not params: args = '[]'
        elif all(name.isdigit() for name in params) and set(map(int, params)) == set(range(len(params))):
            args = '[' + ', '.join('MessageParameter' for _ in params) + '] | [MessageParameters[' + json.dumps(key) + ']]'
        else: args = '[MessageParameters[' + json.dumps(key) + ']]'
        lines.append('  ' + json.dumps(key) + ': ' + args + ';')
    lines.extend(['}', 'export type KnownSystemMessage = { [K in MessageId]: { id: K; params: MessageParameters[K] } }[MessageId];', ''])
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--catalogs-only', action='store_true')
    parser.add_argument('--surface', choices=['all', 'web', 'game', 'core'], default='all')
    parser.add_argument('--additions', type=Path)
    args = parser.parse_args()
    manifest_path = ROOT / 'locales/messages.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    mapping = {entry['source']: key for key, entry in manifest.items()}
    en = json.loads((ROOT / 'locales/en.json').read_text())
    ja = json.loads((ROOT / 'locales/ja.json').read_text())
    additions = json.loads(args.additions.read_text()) if args.additions else {}
    if 'en' in additions and 'ja' in additions:
        additions = additions['ja']
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
    if args.surface != 'all':
        prefix = {'web': 'web/', 'game': 'plugins/', 'core': 'crates/core/'}[args.surface]
        candidates = [path for path in candidates if str(path.relative_to(ROOT)).startswith(prefix)]
    if not args.catalogs_only:
        for path in candidates:
            if path.name in {'i18n.ts', 'messages.generated.ts', 'Messages.java', 'system_message.rs'}: continue
            source = path.read_text()
            for match in CALL.finditer(source):
                fragment = source[match.end():call_end(source, match.end())]
                for token in TOKEN.finditer(fragment):
                    literal = json.loads(token.group())
                    # Only initial t() argument is required; conditionals may use unrelated keys.
                    if token.start() == len(fragment) - len(fragment.lstrip()) and literal not in mapping and literal not in manifest:
                        unknown.append(f'{path.relative_to(ROOT)}: {literal}')
            for literal in ERROR.finditer(source):
                text = json.loads(literal.group(1))
                if text not in mapping and text not in manifest: unknown.append(f'{path.relative_to(ROOT)}: {text}')
            updated = migrate_source(source, mapping, route_labels=path.name == 'routes.ts')
            if args.write and updated != source: path.write_text(updated)
            if args.check and updated != source: unknown.append(f'{path.relative_to(ROOT)}: authoring literals still need migration')
    if unknown: raise ValueError('Untranslated authoring sources:\n' + '\n'.join(unknown))
    if args.write:
        (ROOT / 'web/src/messages.generated.ts').write_text(generated_types(manifest))
        for name, data in [('messages', manifest), ('en', en), ('ja', ja)]:
            (ROOT / f'locales/{name}.json').write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
    print(f'{len(manifest)} shared message IDs; {len(candidates)} source files examined')

if __name__ == '__main__': main()
