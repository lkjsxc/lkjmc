#!/usr/bin/env python3
import re
from dataclasses import dataclass
from pathlib import Path

from data_workflow_source import effect_kinds, sql_writes

ROOTS = ("crates/lkjmc-store/src", "crates/lkjmc-daemon/src")
IGNORED_CALLS = {"if", "for", "while", "match", "return", "execute", "query", "query_one",
                 "query_opt", "format", "vec", "some", "ok", "err", "from", "new"}


@dataclass
class Symbol:
    path: str
    name: str
    body: str
    masked: str
    writes: list[str]
    effects: set[str]
    calls: set[str]
    qualified_calls: set[str]
    local: bool


def mask(source):
    pattern = re.compile(r'/\*.*?\*/|//[^\n]*|r(?P<h>#{0,16})".*?"(?P=h)|(?:b)?"(?:\\.|[^"\\])*"', re.S)
    return pattern.sub(lambda found: " " * len(found.group(0)), source)


def functions(path, root):
    source = path.read_text(encoding="utf-8")
    masked = mask(source)
    header = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^{};]*>)?\s*\([^;{}]*\)[^{;]*\{")
    results = []
    for found in header.finditer(masked):
        attributes = source[max(0, found.start() - 200):found.start()]
        if re.search(r"#\s*\[\s*(?:tokio::)?test", attributes): continue
        start = found.end() - 1
        depth = 0
        end = None
        for offset in range(start, len(masked)):
            if masked[offset] == "{": depth += 1
            elif masked[offset] == "}":
                depth -= 1
                if depth == 0:
                    end = offset + 1
                    break
        if end is None: continue
        body, clean = source[start:end], masked[start:end]
        writes = sql_writes(body)
        effects = effect_kinds(clean)
        calls = {name.lower() for name in re.findall(r"(?<![.])\b([a-z_][a-z0-9_]*)\s*\(", clean)}
        qualified = {name.lower() for name in re.findall(r"::\s*([a-z_][a-z0-9_]*)\s*\(", clean)}
        calls -= IGNORED_CALLS | {found.group(1).lower()}
        results.append(Symbol(str(path.relative_to(root)), found.group(1), body, clean,
                              writes, effects, calls, qualified, ".transaction" in clean))
    return results


def discover(root):
    symbols = []
    for directory in ROOTS:
        base = root / directory
        if not base.exists(): continue
        for path in base.rglob("*.rs"):
            relative = path.relative_to(root)
            if ("tests" in relative.parts or "network_probe_tests" in relative.parts
                    or "fault_harness" in relative.parts or path.name.endswith("_tests.rs")
                    or path.name == "tests.rs"): continue
            symbols.extend(functions(path, root))
    by_name = {(symbol.path, symbol.name.lower()): symbol for symbol in symbols}
    global_names = {}
    for symbol in symbols: global_names.setdefault(symbol.name.lower(), []).append(symbol)
    operations = {id(item): {(item.path, item.name, offset)
                              for offset in range(len(item.writes))} for item in symbols}
    changed = True
    while changed:
        changed = False
        for item in symbols:
            writes, effects, ops = set(item.writes), set(item.effects), set(operations[id(item)])
            for call in item.calls:
                nested = by_name.get((item.path, call))
                if nested is None and call in item.qualified_calls:
                    matches = global_names.get(call, [])
                    nested = matches[0] if len(matches) == 1 else None
                if nested is None: continue
                writes.update(nested.writes); effects.update(nested.effects)
                ops.update(operations[id(nested)])
            merged = sorted(writes)
            if merged != item.writes or effects != item.effects or ops != operations[id(item)]:
                item.writes, item.effects, operations[id(item)] = merged, effects, ops
                changed = True
    found = {}
    for item in symbols:
        if len(operations[id(item)]) >= 2 or item.effects:
            key = (item.path, item.name)
            owner = "local" if item.local else ("delegated" if item.writes else "none")
            found[key] = {"writes": item.writes, "effects": sorted(item.effects),
                          "transactionOwner": owner}
    return found
