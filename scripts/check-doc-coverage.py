#!/usr/bin/env python3
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(subprocess.check_output(
    ['git', 'rev-parse', '--show-toplevel'], text=True,
).strip()).resolve()
DOCS = ROOT / 'docs'
INDEX = DOCS / 'execution/documentation-coverage.json'
CODE_RE = re.compile(r'`([^`]+)`')
HASH_RE = re.compile(r'^[0-9a-f]{64}$')
ACTIONS = {
    'pending', 'added', 'changed', 'confirmed', 'retain-with-boundary',
    'rewritten', 'unchanged',
}
REVIEW_STATES = {'pending', 'audited', 'reviewed'}
SOURCE_PREFIXES = ('crates/', 'platforms/', 'scripts/', 'contracts/', 'config/')
REQUIRED = {
    'path', 'contentHash', 'role', 'owner', 'status', 'reviewState',
    'reviewedAtCommit', 'action', 'sourceEvidence', 'checkEvidence',
    'contradictions', 'followUpTasks',
}


def load_json(path: Path, errors: list[str]):
    try:
        return json.loads(path.read_text(encoding='utf-8'))
    except (OSError, json.JSONDecodeError) as exc:
        errors.append(f'invalid coverage JSON {path}: {exc}')
        return None


def is_commit(value: object) -> bool:
    if not isinstance(value, str) or not re.fullmatch(r'[0-9a-f]{7,64}', value):
        return False
    result = subprocess.run(
        ['git', 'rev-parse', '--verify', f'{value}^{{commit}}'],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False,
    )
    return result.returncode == 0


def repository_path(value: object) -> bool:
    if not isinstance(value, str) or not value:
        return False
    candidate = Path(value)
    try:
        target = (candidate if candidate.is_absolute() else ROOT / candidate).resolve()
    except OSError:
        return False
    return target.is_relative_to(ROOT) and target.exists()


def proof_paths(proof: str) -> list[str]:
    paths = []
    for item in CODE_RE.findall(proof):
        candidate = item[2:] if item.startswith('./') else item
        if candidate.startswith(SOURCE_PREFIXES) and '*' not in candidate:
            paths.append(candidate)
    return paths


def check_entry(entry: object, errors: list[str]) -> str | None:
    if not isinstance(entry, dict) or not REQUIRED <= entry.keys():
        errors.append('invalid coverage entry')
        return None
    path = entry['path']
    if not isinstance(path, str):
        errors.append('invalid coverage path')
        return None
    document = Path(path)
    if not document.is_file():
        errors.append(f'missing documented path {path}')
        return path
    actual = hashlib.sha256(document.read_bytes()).hexdigest()
    if entry['contentHash'] != actual:
        errors.append(f'hash mismatch {path}')
    if entry['action'] not in ACTIONS:
        errors.append(f'invalid action {entry["action"]}')
    if entry['reviewState'] not in REVIEW_STATES:
        errors.append(f'invalid review state {entry["reviewState"]}')
    if not is_commit(entry['reviewedAtCommit']):
        errors.append(f'invalid review commit {entry["reviewedAtCommit"]}')
    for field in ('sourceEvidence', 'checkEvidence'):
        evidence = entry[field]
        if not isinstance(evidence, list):
            errors.append(f'invalid evidence {path}')
            continue
        for item in evidence:
            if not repository_path(item):
                errors.append(f'missing evidence path {item}')
    return path


def state_rows(errors: list[str]):
    for path in sorted((DOCS / 'state').glob('*.md')):
        lines = path.read_text(encoding='utf-8').splitlines()
        start = next((i for i, line in enumerate(lines)
                      if line.startswith('| Capability | Owner document |')), None)
        if start is None or '## Status\n\nimplemented' not in '\n'.join(lines):
            continue
        for line in lines[start + 2:]:
            if not line.startswith('|'):
                break
            cells = [cell.strip() for cell in line.strip('|').split('|')]
            if len(cells) < 4:
                continue
            source, proof = cells[2], cells[3]
            sources = [item for item in CODE_RE.findall(source)
                       if item.startswith(SOURCE_PREFIXES) and '*' not in item]
            if '`none`' in source or not sources or any(not repository_path(item) for item in sources):
                errors.append(f'{path}: implemented capability lacks source evidence')
            if '`none`' in proof or not CODE_RE.findall(proof):
                errors.append(f'{path}: implemented capability lacks deterministic proof')
            for item in proof_paths(proof):
                if not repository_path(item) or not (ROOT / item).is_file():
                    errors.append(f'{path}: missing deterministic proof path {item}')


def main() -> int:
    errors = []
    index = load_json(INDEX, errors)
    if not isinstance(index, dict) or not isinstance(index.get('shards'), list):
        errors.append('invalid coverage index')
        index = {'shards': []}
    if not is_commit(index.get('reviewCommit')):
        errors.append(f'invalid review commit {index.get("reviewCommit")}')
    covered = set()
    for shard_name in index['shards']:
        shard = Path(shard_name)
        data = load_json(shard, errors)
        if not isinstance(data, dict) or not isinstance(data.get('entries'), list):
            errors.append(f'invalid coverage shard {shard_name}')
            continue
        for entry in data['entries']:
            path = check_entry(entry, errors)
            if path in covered:
                errors.append(f'duplicate coverage {path}')
            elif path:
                covered.add(path)
    tracked = subprocess.run(
        ['git', 'ls-files', '*.md'], capture_output=True, text=True, check=False,
    )
    documents = set(tracked.stdout.splitlines())
    for path in sorted(documents - covered):
        errors.append(f'missing coverage {path}')
    for path in sorted(covered - documents):
        errors.append(f'stale coverage {path}')
    state_rows(errors)
    if errors:
        print('\n'.join(errors))
        return 1
    print('ok check-doc-coverage')
    return 0


if __name__ == '__main__':
    sys.exit(main())
