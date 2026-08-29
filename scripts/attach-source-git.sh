#!/bin/sh
set -eu
bundle=${1:?trusted Git bundle required}
commit=${LKJMC_SOURCE_COMMIT:?LKJMC_SOURCE_COMMIT is required}
source_ref=refs/bundles/lkjmc-source
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
heads=$(git bundle list-heads "$bundle")
[ "$heads" = "$commit $source_ref" ] || {
    echo 'Git bundle advertised refs differ from the exact source contract' >&2
    exit 1
}
git bundle verify "$bundle" >/dev/null
git fetch --quiet "$bundle" "$source_ref"
[ "$(git rev-parse FETCH_HEAD)" = "$commit" ] || {
    echo 'Git bundle source ref differs from LKJMC_SOURCE_COMMIT' >&2
    exit 1
}
git update-ref --no-deref HEAD "$commit"
git read-tree "$commit"
git fsck --full --strict --no-dangling
status=$(git status --porcelain=v1 --untracked-files=normal)
[ -z "$status" ] || {
    echo 'exported source differs from bundled Git object' >&2
    printf '%s\n' "$status" | sed -n '1,64p' >&2
    exit 1
}
trap - EXIT HUP INT TERM
printf 'ok source-git-attached commit=%s ref=%s\n' "$commit" "$source_ref"
