"""Root guest helper update, invoked only by a reviewed agent start operation.

Updates public helper bytes and disables autostart without touching game state,
runtime settings, receipts, credentials, or worlds. A stopped old VM is updated
on its next explicitly requested Minecraft start, never by a Console read.
"""
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import uuid


def install(directory, files):
    directory = Path(directory)
    for parent in (directory, *directory.parents):
        info = parent.lstat()
        assert stat.S_ISDIR(info.st_mode) and info.st_uid in (0, os.geteuid()) and not info.st_mode & 0o022
    for name, item in files.items():
        assert name in ('guest.py', 'managed-paths.json')
        raw = item['text'].encode()
        assert len(raw) <= 128 * 1024 and hashlib.sha256(raw).hexdigest() == item['sha256']
        target = directory / name
        if target.exists() or target.is_symlink():
            info = target.lstat()
            assert stat.S_ISREG(info.st_mode) and info.st_nlink == 1 and info.st_uid == os.geteuid()
        temporary = directory / ('.' + name + '-' + uuid.uuid4().hex)
        try:
            fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, item['mode'])
            with os.fdopen(fd, 'wb') as stream:
                stream.write(raw)
                stream.flush()
                os.fchmod(stream.fileno(), item['mode'])
                os.fsync(stream.fileno())
            os.replace(temporary, target)
        finally:
            temporary.unlink(missing_ok=True)
    fd = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def main():
    assert os.geteuid() == 0
    assert Path('/etc/lkjmc-guest-image').read_text().strip() == 'rebuild-v1'
    files = json.load(sys.stdin)
    assert set(files) == {'guest.py', 'managed-paths.json'}
    assert files['guest.py']['mode'] == 0o755 and files['managed-paths.json']['mode'] == 0o644
    # Disabling an active service preserves the current Minecraft process.
    subprocess.run(['systemctl', 'disable', 'lkjmc-game.service'], check=True, capture_output=True)
    install('/usr/local/lib/lkjmc', files)
    print(json.dumps({'updated': True, 'autostart': False}))


if __name__ == '__main__':
    main()
