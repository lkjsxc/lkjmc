#!/usr/bin/env python3
from pathlib import Path
import sys

LIMIT = 200
SKIP_DIRS = {
    '.git', 'tmp', 'target', '.gradle', 'build', '.idea', '.vscode',
    'node_modules', 'data', 'runtime', 'logs', 'out', '__pycache__',
    'contracts',
}
SKIP_SUFFIXES = {
    '.jar', '.db', '.sqlite', '.sqlite3', '.log', '.pid', '.sock', '.lock',
}
CHECK_SUFFIXES = {
    '.md', '.rs', '.java', '.kt', '.kts', '.toml', '.py', '.sh', '.yml',
    '.yaml', '.json', '.sql', '.gradle', '.properties', '.txt',
}
CHECK_NAMES = {'.gitignore', '.dockerignore', 'Dockerfile', 'AGENTS.md', 'gradlew'}


def skipped(path: Path) -> bool:
    if path.parts and path.parts[0] in SKIP_DIRS:
        return True
    if path.parts[:2] == ('platforms', 'jvm') and 'build' in path.parts[2:-1]:
        return True
    return path.suffix in SKIP_SUFFIXES


def tracked_text(path: Path) -> bool:
    return path.name in CHECK_NAMES or path.suffix in CHECK_SUFFIXES


def line_count(path: Path) -> int:
    try:
        text = path.read_text(encoding='utf-8')
    except UnicodeDecodeError:
        return 0
    return len(text.splitlines())


def main() -> int:
    violations = []
    for path in sorted(Path('.').rglob('*')):
        if not path.is_file() or skipped(path) or not tracked_text(path):
            continue
        count = line_count(path)
        if count > LIMIT:
            violations.append((str(path), count))
    if violations:
        for path, count in violations:
            print(f'{path}: {count} lines exceeds {LIMIT}')
        return 1
    print('ok check-lines')
    return 0


if __name__ == '__main__':
    sys.exit(main())
