#!/bin/sh
set -eu
bundle=${1:?trusted Git bundle required}
commit=${LKJMC_SOURCE_COMMIT:?LKJMC_SOURCE_COMMIT is required}
case $commit in
    *[!0-9a-f]*|'') echo 'LKJMC_SOURCE_COMMIT must be lowercase hexadecimal' >&2; exit 1 ;;
esac
[ "${#commit}" -eq 40 ] || {
    echo 'LKJMC_SOURCE_COMMIT must contain 40 characters' >&2
    exit 1
}
[ ! -e .git ] || { echo 'refusing to replace existing Git metadata' >&2; exit 1; }
[ -f "$bundle" ] && [ ! -L "$bundle" ] || {
    echo 'trusted Git bundle must be a regular non-symlink file' >&2
    exit 1
}
cleanup() { code=$?; trap - EXIT HUP INT TERM; rm -rf .git; exit "$code"; }
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
git init --quiet
git fetch --quiet "$bundle" HEAD
[ "$(git rev-parse FETCH_HEAD)" = "$commit" ] || {
    echo 'Git bundle HEAD differs from LKJMC_SOURCE_COMMIT' >&2
    exit 1
}
git update-ref HEAD "$commit"
git read-tree "$commit"
status=$(git status --porcelain=v1 --untracked-files=normal)
[ -z "$status" ] || {
    echo 'exported source differs from bundled Git object' >&2
    printf '%s\n' "$status" | sed -n '1,64p' >&2
    exit 1
}
trap - EXIT HUP INT TERM
printf 'ok source-git-attached commit=%s\n' "$commit"
