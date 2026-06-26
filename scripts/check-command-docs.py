#!/usr/bin/env python3
from pathlib import Path
import re
import sys

DAEMON_DOC = Path('docs/architecture/runtime/daemon/command-catalog.md')
CLI_DOC = Path('docs/product/commands/ssh-cli.md')
MC_DOC = Path('docs/product/commands/minecraft.md')
PAPER_YML = Path('platforms/jvm/paper/src/main/resources/plugin.yml')
VELOCITY = Path('platforms/jvm/velocity/src/main/java/com/lkjmc/velocity/VelocityCommands.java')


def command_blocks(text):
    starts = [m.start() for m in re.finditer(r'match\s+(?:request\.command|command_name)\.as_str\(\)\s*\{', text)]
    for start in starts:
        brace = text.find('{', start)
        depth = 0
        for index in range(brace, len(text)):
            if text[index] == '{':
                depth += 1
            elif text[index] == '}':
                depth -= 1
                if depth == 0:
                    yield text[brace:index]
                    break


def daemon_commands():
    commands = set()
    for path in Path('crates/lkjmc-daemon/src').glob('*.rs'):
        for block in command_blocks(path.read_text()):
            commands.update(re.findall(r'"([a-z][a-z0-9_.-]+)"\s*=>', block))
    return commands


def cli_families():
    text = Path('crates/lkjmc-cli/src/args.rs').read_text()
    return set(re.findall(r'cmd\s*==\s*"([a-z][a-z0-9-]+)"', text))


def cli_subcommands():
    pairs = set()
    for path in Path('crates/lkjmc-cli/src').glob('args_*.rs'):
        family = path.stem.removeprefix('args_').replace('_', '-')
        text = path.read_text()
        for sub in re.findall(r'sub\s*==\s*"([a-z][a-z0-9-]+)"', text):
            pairs.add((family, sub))
    return pairs


def paper_commands_and_permissions():
    commands = set()
    permissions = set()
    in_commands = False
    for line in PAPER_YML.read_text().splitlines():
        if line == 'commands:':
            in_commands = True
            continue
        if line == 'permissions:':
            break
        if in_commands:
            match = re.match(r'^  ([A-Za-z0-9_-]+):\s*$', line)
            if match:
                commands.add(match.group(1))
            perm = re.match(r'^    permission:\s*([^\s]+)', line)
            if perm:
                permissions.add(perm.group(1))
    return commands, permissions


def velocity_roots():
    text = VELOCITY.read_text()
    return set(re.findall(r'metaBuilder\("([A-Za-z0-9_-]+)"\)', text))


def report(label, missing):
    return [f'{label}: missing {item}' for item in sorted(missing)]


def main():
    errors = []
    daemon_doc = DAEMON_DOC.read_text()
    source_commands = daemon_commands()
    doc_commands = {
        value for value in re.findall(r'`([a-z][a-z0-9_.-]+)`', daemon_doc)
        if value in source_commands or not value.endswith('.rs')
    }
    errors += report('daemon docs', source_commands - doc_commands)
    errors += report('daemon docs extra', doc_commands - source_commands)

    cli_doc = CLI_DOC.read_text()
    for family in sorted(cli_families()):
        if f'lkjmc {family}' not in cli_doc:
            errors.append(f'cli docs: missing lkjmc {family}')
    for family, sub in sorted(cli_subcommands()):
        if f'lkjmc {family} {sub}' not in cli_doc:
            errors.append(f'cli docs: missing lkjmc {family} {sub}')

    mc_doc = MC_DOC.read_text()
    paper_commands, paper_permissions = paper_commands_and_permissions()
    for command in sorted(paper_commands):
        if f'/{command}' not in mc_doc:
            errors.append(f'minecraft docs: missing /{command}')
    for permission in sorted(paper_permissions):
        if permission not in mc_doc:
            errors.append(f'minecraft docs: missing {permission}')
    for command in sorted(velocity_roots()):
        if f'/{command}' not in mc_doc:
            errors.append(f'minecraft docs: missing velocity /{command}')

    if errors:
        print('\n'.join(errors))
        return 1
    print('ok check-command-docs')
    return 0


if __name__ == '__main__':
    sys.exit(main())
