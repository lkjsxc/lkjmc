#!/usr/bin/env python3
"""Compare two private release trees byte-for-byte after no-follow traversal."""
import argparse
import hashlib
import os
from pathlib import Path
import stat
import sys

from fd_tree import Limits, walk


LIMITS = Limits(
    max_entries=128,
    max_files=64,
    max_bytes=256 * 1024 * 1024,
    max_file_bytes=128 * 1024 * 1024,
    max_depth=4,
)


def inventory(root):
    root = Path(root)
    files = {}

    def digest(descriptor, entry):
        value = hashlib.sha256()
        while True:
            block = os.read(descriptor, 1024 * 1024)
            if not block:
                break
            value.update(block)
        files[entry.path] = ("file", entry.mode, entry.size, value.hexdigest())

    walk(root, digest, LIMITS)
    entries = {".": ("directory", stat.S_IMODE(root.lstat().st_mode), 0, "-")}
    for path in root.rglob("*"):
        metadata = path.lstat()
        relative = path.relative_to(root).as_posix()
        if stat.S_ISDIR(metadata.st_mode):
            entries[relative] = ("directory", stat.S_IMODE(metadata.st_mode), 0, "-")
        elif stat.S_ISREG(metadata.st_mode) and not path.is_symlink():
            entries[relative] = files[relative]
        else:
            raise RuntimeError(f"release tree contains a symlink or special file: {relative}")
    if set(files) != {path for path, value in entries.items() if value[0] == "file"}:
        raise RuntimeError("release file closure changed after validated traversal")
    return entries


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("first")
    parser.add_argument("second")
    args = parser.parse_args()
    first = Path(args.first)
    second = Path(args.second)
    if first.resolve(strict=True) == second.resolve(strict=True):
        raise RuntimeError("release roots must be distinct")
    first_entries = inventory(first)
    second_entries = inventory(second)
    if first_entries != second_entries:
        differences = [
            path for path in sorted(set(first_entries) | set(second_entries))
            if first_entries.get(path) != second_entries.get(path)
        ]
        shown = ", ".join(differences[:16])
        suffix = "" if len(differences) <= 16 else f" (+{len(differences) - 16} more)"
        raise RuntimeError(f"release roots differ: {shown}{suffix}")
    files = sum(value[0] == "file" for value in first_entries.values())
    print(f"ok release-roots-reproducible entries={len(first_entries)} files={files}")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"release comparison failed: {error}", file=sys.stderr)
        sys.exit(1)
