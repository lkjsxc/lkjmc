#!/usr/bin/env python3
from pathlib import Path
import re
import sys

PERMISSION_DOC = Path('docs/architecture/security/permissions.md')
PERMISSION_NODES = Path('platforms/jvm/common/src/main/java/com/lkjmc/common/permission/PermissionNodes.java')
PLUGIN_YML = Path('platforms/jvm/paper/src/main/resources/plugin.yml')
PERM_RE = re.compile(r'lkjmc\.[a-z0-9.]+')


def java_permissions():
    text = PERMISSION_NODES.read_text()
    return set(re.findall(r'"(lkjmc\.[^"]+)"', text))


def plugin_permissions():
    permissions = set()
    in_permissions = False
    for line in PLUGIN_YML.read_text().splitlines():
        command_perm = re.match(r'^    permission:\s*([^\s]+)', line)
        if command_perm:
            permissions.add(command_perm.group(1))
        if line == 'permissions:':
            in_permissions = True
            continue
        if in_permissions:
            key = re.match(r'^  (lkjmc\.[a-z0-9.]+):\s*$', line)
            if key:
                permissions.add(key.group(1))
    return permissions


def main():
    source = java_permissions() | plugin_permissions()
    docs = set(PERM_RE.findall(PERMISSION_DOC.read_text()))
    missing = source - docs
    extra = docs - source
    errors = []
    errors += [f'permissions docs: missing {item}' for item in sorted(missing)]
    errors += [f'permissions docs: unknown {item}' for item in sorted(extra)]
    if errors:
        print('\n'.join(errors))
        return 1
    print('ok check-permissions')
    return 0


if __name__ == '__main__':
    sys.exit(main())
