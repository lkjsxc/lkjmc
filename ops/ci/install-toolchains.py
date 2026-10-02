#!/usr/bin/env python3
"""Install only checksum-pinned public tool archives into the isolated CI image."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import zipfile

source = Path('/inputs/toolchains')
destination = Path('/opt/ci/toolchains')
destination.mkdir(parents=True)
lock = json.loads(Path('/inputs/toolchains.lock.json').read_text())
for name, pin in lock['tools'].items():
    archive = source / name
    assert hashlib.file_digest(archive.open('rb'), 'sha256').hexdigest() == pin['sha256']
    staging = Path('/tmp/install-' + name)
    staging.mkdir()
    if name == 'gradle':
        with zipfile.ZipFile(archive) as bundle:
            for member in bundle.infolist():
                p = Path(member.filename)
                assert not p.is_absolute() and '..' not in p.parts
            bundle.extractall(staging)
    else:
        with tarfile.open(archive) as bundle:
            bundle.extractall(staging, filter='data')
    items = list(staging.iterdir())
    assert len(items) == 1 and items[0].is_dir()
    if name in ['rustc', 'cargo', 'rust-std', 'rustfmt-preview']:
        subprocess.run(['sh', str(items[0] / 'install.sh'), '--prefix=/opt/ci/rust',
                        '--disable-ldconfig'], check=True)
    else:
        shutil.move(str(items[0]), destination / name)
    shutil.rmtree(staging)
(destination / 'gradle/bin/gradle').chmod(0o755)
