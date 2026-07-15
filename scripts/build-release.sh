#!/bin/sh
set -eu
out=${1:?private release directory required}
[ ! -e "$out" ] || { echo "release output already exists: $out" >&2; exit 1; }
umask 077
mkdir -m 0700 "$out" "$out/source"
cleanup() { code=$?; [ "$code" -eq 0 ] || rm -rf "$out"; exit "$code"; }
trap cleanup EXIT HUP INT TERM
python3 - "$out/source" <<'PY'
import json,os,pathlib,shutil,stat,sys
root=pathlib.Path(__file__).resolve().parent if '__file__' in globals() else pathlib.Path.cwd()
root=pathlib.Path.cwd(); target=pathlib.Path(sys.argv[1])
data=json.loads((root/'config/release-artifacts.json').read_text())
for item in data['artifacts']:
    source=root/item['source']; destination=target/item['destination']
    mode=source.lstat().st_mode
    if not stat.S_ISREG(mode) or source.is_symlink(): raise SystemExit(f'invalid built artifact: {source}')
    shutil.copyfile(source,destination)
    os.chmod(destination,0o700 if item['kind']=='binary' else 0o600)
PY
scripts/artifact-manifest.py --release-root "$out" --output "$out/artifact-manifest.json"
scripts/verify-artifact-manifest.py --release-root "$out" --manifest "$out/artifact-manifest.json"
trap - EXIT HUP INT TERM
printf 'ok release-built root=%s\n' "$out"
