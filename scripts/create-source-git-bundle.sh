#!/bin/sh
set -eu

output_arg=${1:?private Git bundle output required}
requested_commit=${2:-HEAD}
source_ref=refs/bundles/lkjmc-source

root=$(git rev-parse --show-toplevel 2>/dev/null) || {
    echo 'source bundle creation requires a Git checkout' >&2
    exit 1
}
cd "$root"
[ "$(git rev-parse --is-shallow-repository)" = false ] || {
    echo 'source bundle creation requires complete non-shallow history' >&2
    exit 1
}
[ -z "$(git status --porcelain=v1 --untracked-files=normal)" ] || {
    echo 'source bundle creation requires a clean worktree' >&2
    exit 1
}
commit=$(git rev-parse --verify "$requested_commit^{commit}" 2>/dev/null) || {
    echo 'source bundle commit is not available' >&2
    exit 1
}
[ "$commit" = "$(git rev-parse HEAD)" ] || {
    echo 'source bundle commit differs from Git HEAD' >&2
    exit 1
}
case $commit in
    *[!0-9a-f]*|'') echo 'source bundle commit must be lowercase hexadecimal' >&2; exit 1 ;;
esac
[ "${#commit}" -eq 40 ] || {
    echo 'source bundle commit must contain 40 characters' >&2
    exit 1
}

output=$(python3 - "$output_arg" <<'PY'
import os
import stat
import sys

value = os.path.abspath(sys.argv[1])
parent = os.path.dirname(value)
try:
    metadata = os.stat(parent, follow_symlinks=False)
except FileNotFoundError:
    raise SystemExit('source bundle output parent must already exist')
if not stat.S_ISDIR(metadata.st_mode):
    raise SystemExit('source bundle output parent is not a directory')
print(value)
PY
)
[ ! -e "$output" ] && [ ! -L "$output" ] || {
    echo 'refusing existing source bundle output' >&2
    exit 1
}
git show-ref --verify --quiet "$source_ref" && {
    echo "refusing existing source bundle ref: $source_ref" >&2
    exit 1
}

umask 077
temporary=$(mktemp "$output.tmp.XXXXXX")
import_root=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-source-import.XXXXXX")
ref_created=0
output_created=0
output_identity=
cleanup() {
    code=$?
    trap - EXIT HUP INT TERM
    if [ "$ref_created" -eq 1 ]; then
        current=$(git rev-parse --verify "$source_ref" 2>/dev/null || true)
        if [ "$current" = "$commit" ]; then
            git update-ref -d "$source_ref" "$commit" || code=1
        else
            echo "refusing to delete changed source bundle ref: $source_ref" >&2
            code=1
        fi
    fi
    [ -z "$temporary" ] || rm -f "$temporary"
    [ -z "$import_root" ] || rm -rf "$import_root"
    if [ "$code" -ne 0 ] && [ "$output_created" -eq 1 ]; then
        current_identity=$(stat -c '%d:%i' "$output" 2>/dev/null || true)
        if [ -f "$output" ] && [ ! -L "$output" ] && [ "$current_identity" = "$output_identity" ]; then
            rm -f "$output"
        else
            echo 'refusing cleanup of replaced source bundle output' >&2
        fi
    fi
    exit "$code"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

git update-ref "$source_ref" "$commit" ''
ref_created=1
git bundle create "$temporary" "$source_ref"
git update-ref -d "$source_ref" "$commit"
ref_created=0
chmod 0600 "$temporary"

heads=$(git bundle list-heads "$temporary")
[ "$heads" = "$commit $source_ref" ] || {
    echo 'source bundle advertised refs differ from the exact contract' >&2
    exit 1
}
git -C "$import_root" init --quiet
git -C "$import_root" bundle verify "$temporary" >/dev/null
git -C "$import_root" fetch --quiet "$temporary" "$source_ref"
[ "$(git -C "$import_root" rev-parse FETCH_HEAD)" = "$commit" ] || {
    echo 'imported source bundle ref differs from the exact commit' >&2
    exit 1
}
git -C "$import_root" checkout --quiet --detach "$commit"
git -C "$import_root" fsck --full --strict --no-dangling
[ -z "$(git -C "$import_root" status --porcelain=v1 --untracked-files=normal)" ] || {
    echo 'imported source bundle checkout is not clean' >&2
    exit 1
}

ln "$temporary" "$output"
output_identity=$(stat -c '%d:%i' "$output")
output_created=1
rm -f "$temporary"
temporary=
python3 - "$output" <<'PY'
import os
import sys

path = sys.argv[1]
fd = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
try:
    os.fsync(fd)
finally:
    os.close(fd)
parent = os.open(os.path.dirname(path), os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
try:
    os.fsync(parent)
finally:
    os.close(parent)
PY

rm -rf "$import_root"
import_root=
output_created=0
trap - EXIT HUP INT TERM
printf 'ok source-git-bundle commit=%s ref=%s\n' "$commit" "$source_ref"
