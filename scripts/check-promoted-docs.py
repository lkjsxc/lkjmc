#!/usr/bin/env python3
from pathlib import Path
import sys

CHECKS = [
    (Path('docs/decisions/control-surface-scope.md'), 'not active product targets'),
    (Path('docs/research/web-control-surface.md'), 'not a current product target'),
    (Path('docs/research/kubernetes-runtime.md'), 'not a current product target'),
    (Path('docs/current-state.md'), 'lkjmc-installer'),
]
REQUIRED = [
    (Path('docs/current-state.md'), 'authenticated `/web` operator pages'),
    (Path('docs/current-state.md'), '`kubernetes` selectable'),
    (Path('docs/operations/smoke-checks.md'), 'check-web-smoke.sh'),
    (Path('docs/operations/smoke-checks.md'), 'check-kubernetes-smoke.sh'),
]


def main() -> int:
    errors = []
    for path, phrase in CHECKS:
        if phrase in path.read_text():
            errors.append(f'{path}: stale phrase {phrase}')
    for path, phrase in REQUIRED:
        if phrase not in path.read_text():
            errors.append(f'{path}: missing {phrase}')
    if errors:
        print('\n'.join(errors))
        return 1
    print('ok check-promoted-docs')
    return 0


if __name__ == '__main__':
    sys.exit(main())
