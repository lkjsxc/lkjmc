#!/usr/bin/env python3
"""Build an explicit, secret-free release payload; never deploys or changes a host.

The same script can verify an archive on the host using only Python's standard
library. Passing verification is a byte-integrity check, not public acceptance.
"""
import argparse
import gzip
import hashlib
import io
import json
import os
import shutil
from pathlib import Path, PurePosixPath
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parent.parent


def run(*args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def clean():
    if git('status', '--porcelain'):
        raise ValueError('Commit or preserve outstanding work before building a release')


def checksum(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def allowed_name(name):
    path = PurePosixPath(name)
    return bool(name) and not path.is_absolute() and '\\' not in name and all(
        part not in ('', '.', '..') for part in name.split('/'))


def verify(path):
    with tarfile.open(path, 'r:gz') as archive:
        members = archive.getmembers()
        names = [entry.name for entry in members]
        if len(names) != len(set(names)) or any(
                not entry.isfile() or not allowed_name(entry.name) or entry.size > 1024**3
                for entry in members):
            raise ValueError('Archive contains duplicate/unsafe paths or non-regular files')
        header = archive.getmember('MANIFEST.json')
        if header.size > 1024**2:
            raise ValueError('Manifest is too large')
        manifest = json.load(archive.extractfile(header))
        if manifest.get('version') != 1 or manifest.get('production_acceptance') is not False:
            raise ValueError('Unsupported release manifest')
        expected = manifest['files']
        if set(names) != set(expected) | {'MANIFEST.json'}:
            raise ValueError('Archive file set differs from its manifest')
        for entry in members:
            if entry.name == 'MANIFEST.json':
                continue
            recorded = expected[entry.name]
            with archive.extractfile(entry) as source:
                actual = hashlib.file_digest(source, 'sha256').hexdigest()
            if (actual != recorded['sha256'] or entry.size != recorded['bytes']
                    or entry.mode != recorded['mode']):
                raise ValueError('Release file differs: ' + entry.name)
    return {'git_commit': manifest['git_commit'], 'files': len(expected),
            'bytes': path.stat().st_size, 'sha256': checksum(path),
            'production_acceptance': False}


def build(output):
    clean()
    commit = git('rev-parse', 'HEAD')
    timestamp = int(git('show', '-s', '--format=%ct', 'HEAD'))
    tools = Path(os.environ.get('LKJMC_TOOLCHAINS', ROOT / '.local/toolchains'))
    java = Path(os.environ.get('JAVA_HOME', tools / 'jdk-25.0.4.1+1'))
    gradle = Path(os.environ.get('LKJMC_GRADLE', tools / 'gradle-9.8.0/bin/gradle'))
    cargo = os.environ.get('LKJMC_CARGO') or shutil.which('cargo') or str(Path.home() / '.cargo/bin/cargo')
    env = os.environ.copy()
    env['JAVA_HOME'] = str(java)
    run(str(cargo), 'build', '--release', '--locked', '--offline', '--workspace', cwd=ROOT)
    run(str(gradle), '--no-daemon', '--max-workers=2', '-p', 'plugins',
        ':common:checkMessages', ':paper:checkTeamMenus', ':paper:jar', ':proxy:jar', ':floodgate-link:jar', '--offline', cwd=ROOT, env=env)
    run('npm', 'ci', '--offline', cwd=ROOT / 'web')
    run('npm', 'run', 'build', cwd=ROOT / 'web')
    run('python3', 'scripts/game_artifacts.py', '--offline', cwd=ROOT)
    files = {}

    def add(name, source, mode=0o644):
        if name in files or not allowed_name(name) or source.is_symlink() or not source.is_file():
            raise ValueError('Invalid release input: ' + name)
        files[name] = (source, mode)

    for name in ['lkjmc-core', 'lkjmc-agent']:
        add('bin/' + name, ROOT / 'target/release' / name, 0o755)
    add('plugins/shared/lkjmc-paper.jar', ROOT / 'plugins/paper/build/libs/lkjmc-paper.jar')
    add('plugins/proxy/lkjmc-velocity.jar', ROOT / 'plugins/proxy/build/libs/lkjmc-velocity.jar')
    add('extensions/floodgate/floodgate-lkjmc-database.jar',
        ROOT / 'plugins/floodgate-link/build/libs/floodgate-lkjmc-database.jar')
    for name in ['paper', 'velocity']:
        add('servers/' + name + '.jar', ROOT / '.local/artifacts' / (name + '.jar'))
    for name in ['worldedit', 'worldguard', 'viaversion', 'viabackwards']:
        add('plugins/shared/' + name + '.jar', ROOT / '.local/artifacts' / (name + '.jar'))
    for name in ['geyser', 'floodgate']:
        add('plugins/proxy/' + name + '.jar', ROOT / '.local/artifacts' / (name + '.jar'))
    for source in sorted((ROOT / 'web/dist').rglob('*')):
        if source.is_file() and source.suffix != '.map':
            add('web/' + source.relative_to(ROOT / 'web/dist').as_posix(), source)
    for source in sorted((ROOT / 'migrations').glob('*.sql')):
        add('migrations/' + source.name, source)
    add('guest/guest.py', ROOT / 'ops/guest/guest.py', 0o755)
    add('guest/managed-paths.json', ROOT / 'ops/guest/managed-paths.json')
    add('guest/upgrade.py', ROOT / 'ops/guest/upgrade.py')
    add('verify-release.py', Path(__file__), 0o755)
    add('component-candidates.json', ROOT / 'ops/component-candidates.json')
    manifest = {'version': 1, 'git_commit': commit, 'source_timestamp': timestamp,
                'production_acceptance': False, 'files': {name: {
                    'sha256': checksum(source), 'bytes': source.stat().st_size, 'mode': mode
                } for name, (source, mode) in sorted(files.items())}}
    clean()
    if commit != git('rev-parse', 'HEAD'):
        raise ValueError('Source revision changed during the build')
    output.mkdir(mode=0o700, parents=True, exist_ok=True)
    target = output / f'lkjmc-{commit}.tar.gz'
    if target.exists():
        raise ValueError('Release already exists; keep the original artifact and receipt')
    fd, temporary = tempfile.mkstemp(prefix='.release-', dir=output)
    try:
        with os.fdopen(fd, 'wb') as raw:
            with gzip.GzipFile(filename='', mode='wb', fileobj=raw, mtime=0) as compressed:
                with tarfile.open(fileobj=compressed, mode='w') as archive:
                    data = json.dumps(manifest, sort_keys=True, indent=2).encode() + b'\n'
                    info = tarfile.TarInfo('MANIFEST.json')
                    info.size, info.mode, info.mtime = len(data), 0o644, timestamp
                    archive.addfile(info, io.BytesIO(data))
                    for name, (source, mode) in sorted(files.items()):
                        info = tarfile.TarInfo(name)
                        info.size, info.mode, info.mtime = source.stat().st_size, mode, timestamp
                        with source.open('rb') as stream:
                            archive.addfile(info, stream)
            raw.flush()
            os.fsync(raw.fileno())
        receipt = verify(Path(temporary))
        os.rename(temporary, target)
        receipt_path = target.with_suffix('.receipt.json')
        with receipt_path.open('x') as destination:
            json.dump(receipt, destination, indent=2)
            destination.write('\n')
            destination.flush()
            os.fsync(destination.fileno())
        directory = os.open(output, os.O_RDONLY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
        print(json.dumps({'archive': str(target), **receipt}, indent=2))
    finally:
        Path(temporary).unlink(missing_ok=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    create = commands.add_parser('build')
    create.add_argument('--output', type=Path, default=ROOT / '.local/releases')
    check = commands.add_parser('verify')
    check.add_argument('archive', type=Path)
    args = parser.parse_args()
    if args.command == 'build':
        build(args.output)
    else:
        print(json.dumps(verify(args.archive), indent=2))
