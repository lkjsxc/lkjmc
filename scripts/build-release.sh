#!/bin/sh
set -eu
out_arg=${1:?private release directory required}
root=$(git rev-parse --show-toplevel 2>/dev/null) || {
    echo 'release build requires a Git checkout' >&2
    exit 1
}
cd "$root"
[ -z "$(git status --porcelain=v1 --untracked-files=normal)" ] || {
    echo 'release build requires a clean worktree' >&2
    exit 1
}
commit=$(git rev-parse HEAD)
case ${LKJMC_SOURCE_COMMIT:-$commit} in
    "$commit") ;;
    *) echo 'LKJMC_SOURCE_COMMIT differs from Git HEAD' >&2; exit 1 ;;
esac
out=$(python3 - "$root" "$out_arg" <<'PY'
import os,re,stat,sys
root=os.path.realpath(sys.argv[1]); value=os.path.abspath(sys.argv[2])
name=os.path.basename(value); parent=os.path.realpath(os.path.dirname(value))
if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]*',name):
 raise SystemExit('unsafe release output name')
try: metadata=os.stat(parent,follow_symlinks=False)
except FileNotFoundError: raise SystemExit('release output parent must already exist')
if not stat.S_ISDIR(metadata.st_mode): raise SystemExit('release output parent is not a directory')
if metadata.st_mode&0o022:
 raise SystemExit('release output parent must not be group/other writable')
if os.path.commonpath((root,parent))==root:
 raise SystemExit('release output must be outside the source checkout')
print(os.path.join(parent,name))
PY
)
[ ! -e "$out" ] && [ ! -L "$out" ] || {
    echo "release output already exists: $out" >&2
    exit 1
}
umask 077
build_root=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-release-build.XXXXXX")
source_root=$build_root/source
worktree_added=0
out_created=0
out_identity=
cleanup() {
    code=$?
    trap - EXIT HUP INT TERM
    if [ "$worktree_added" -eq 1 ]; then
        git -C "$root" worktree remove --force "$source_root" >/dev/null 2>&1 || code=1
    fi
    rm -rf "$build_root"
    if [ "$code" -ne 0 ] && [ "$out_created" -eq 1 ]; then
        current_identity=$(stat -c '%d:%i' "$out" 2>/dev/null || true)
        if [ -d "$out" ] && [ ! -L "$out" ] && [ "$current_identity" = "$out_identity" ]; then
            rm -rf "$out"
        else
            echo 'refusing cleanup of replaced release output' >&2
        fi
    fi
    exit "$code"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
git worktree add --quiet --detach "$source_root" "$commit"
worktree_added=1
mkdir -m 0700 "$out"
out_identity=$(stat -c '%d:%i' "$out")
out_created=1
mkdir -m 0700 "$out/source"
nonce=$(python3 -c 'import secrets; print(secrets.token_hex(16))')
(
    cd "$source_root"
    export LKJMC_SOURCE_COMMIT=$commit
    export LKJMC_BUILD_NONCE=$nonce
    export CARGO_TARGET_DIR=$source_root/target
    cargo build --locked --release -p lkjmc-cli -p lkjmc-daemon -p lkjmc-discord -p lkjmc-ops
    ./gradlew --no-daemon --no-build-cache shadowJar
    python3 - "$out/source" <<'PY'
import json,pathlib,shutil,stat,sys
root=pathlib.Path.cwd(); target=pathlib.Path(sys.argv[1])
data=json.loads((root/'config/release-artifacts.json').read_text())
for item in data['artifacts']:
    source=root/item['source']; destination=target/item['destination']
    mode=source.lstat().st_mode
    if not stat.S_ISREG(mode) or source.is_symlink():
        raise SystemExit(f'invalid fresh built artifact: {source}')
    shutil.copyfile(source,destination)
    destination.chmod(0o700 if item['kind']=='binary' else 0o600)
PY
    scripts/verify-built-identity.py --source "$out/source"
    scripts/artifact-manifest.py --release-root "$out" --output "$out/artifact-manifest.json"
    scripts/verify-artifact-manifest.py --release-root "$out" --manifest "$out/artifact-manifest.json"
)
git -C "$root" worktree remove --force "$source_root"
worktree_added=0
rm -rf "$build_root"
trap - EXIT HUP INT TERM
printf 'ok release-built root=%s commit=%s\n' "$out" "$commit"
