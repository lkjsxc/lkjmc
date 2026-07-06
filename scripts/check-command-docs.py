#!/usr/bin/env python3
from pathlib import Path
import json
import subprocess
import sys

CONTRACT = Path('contracts/commands.json')
AUTH = {'open', 'player', 'admin', 'operator'}
SURFACES = {'paper', 'velocity', 'cli', 'web', 'discord'}


def load_commands():
    try:
        commands = json.loads(CONTRACT.read_text())['commands']
    except (KeyError, json.JSONDecodeError) as error:
        raise ValueError(f'{CONTRACT}: invalid command registry: {error}')
    return commands


def validate_registry(commands):
    errors = []
    names = [command.get('name', '') for command in commands]
    if names != sorted(names):
        errors.append(f'{CONTRACT}: commands must be sorted by name')
    if len(names) != len(set(names)):
        errors.append(f'{CONTRACT}: command names must be unique')
    for command in commands:
        name = command.get('name', '')
        if command.get('authorization') not in AUTH:
            errors.append(f'{CONTRACT}: {name} has invalid authorization')
        surfaces = set(command.get('surfaces', []))
        if not surfaces or surfaces - SURFACES:
            errors.append(f'{CONTRACT}: {name} has invalid surfaces')
        family = command.get('family', '')
        expected_doc = Path(f'docs/architecture/runtime/daemon/commands/{family}.md')
        doc = Path(command.get('doc', ''))
        if doc != expected_doc:
            errors.append(f'{CONTRACT}: {name} doc must be {expected_doc}')
        if not doc.is_file():
            errors.append(f'{CONTRACT}: {name} doc missing: {doc}')
        elif name not in doc.read_text():
            errors.append(f'{doc}: missing command {name}')
    return errors


def check_catalog():
    result = subprocess.run(
        ['./scripts/generate-command-catalog.py', '--check'],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if result.returncode == 0:
        return []
    return [line for line in result.stdout.splitlines() if line]


def main():
    try:
        commands = load_commands()
    except ValueError as error:
        print(error)
        return 1
    errors = validate_registry(commands) + check_catalog()
    if errors:
        print('\n'.join(errors))
        return 1
    print('ok check-command-docs')
    return 0


if __name__ == '__main__':
    sys.exit(main())
