#!/bin/sh
set -eu
scope=; manifest=; root=; source_root=; service_uid=; service_gid=
fail() { echo "artifact install failed: $*" >&2; exit 1; }
while [ "$#" -gt 0 ]; do
  case "$1" in
    --scope) scope=${2:-}; shift;; --manifest) manifest=${2:-}; shift;;
    --root) root=${2:-}; shift;; --source) source_root=${2:-}; shift;;
    --service-uid) service_uid=${2:-}; shift;; --service-gid) service_gid=${2:-}; shift;;
    *) fail "unknown argument: $1";;
  esac; shift
done
case "$scope" in
  system) [ "$(id -u)" = 0 ] || fail 'system scope requires root'
    case "$service_uid:$service_gid" in *[!0-9:]*|:|*:|:*:*) fail 'system scope requires numeric service UID/GID';; esac
    owner_uid=0; dir_mode=0750;;
  user|rootless) [ "$(id -u)" != 0 ] || fail "$scope scope refuses root"
    service_uid=$(id -u); service_gid=$(id -g); owner_uid=$service_uid; dir_mode=0700;;
  *) fail 'scope must be system, user, or rootless';;
esac
[ -f "$manifest" ] && [ -d "$source_root" ] && [ -n "$root" ] || fail 'manifest, source, and root are required'
case "$root" in /|''|*'/../'*|../*|*/..) fail 'unsafe install root';; esac
umask 077
parent=$(dirname "$root"); mkdir -p "$parent"
stage=$parent/.lkjmc-stage-$$; rollback=$parent/.lkjmc-rollback-$$
cleanup() { rm -rf "$stage"; }; trap cleanup EXIT HUP INT TERM
mkdir -m 0700 "$stage" "$stage/bin" "$stage/jars"
python3 - "$manifest" "$source_root" >"$stage/inventory" <<'PY'
import hashlib,json,pathlib,sys
manifest=pathlib.Path(sys.argv[1]); source=pathlib.Path(sys.argv[2]).resolve()
data=json.loads(manifest.read_text())
if data.get('schemaVersion')!=1 or not isinstance(data.get('artifacts'),list): raise SystemExit('invalid manifest schema')
seen=set()
for a in data['artifacts']:
 if set(a)!={"component","kind","path","provenance","sha256","size","source"}: raise SystemExit('invalid artifact fields')
 p=pathlib.Path(a['path']).resolve()
 if p.parent!=source or p.name in seen or any(c.isspace() for c in p.name): raise SystemExit('artifact path is not a unique source child')
 if a['kind'] not in ('binary','jar'): raise SystemExit('unsupported artifact kind')
 raw=p.read_bytes()
 if len(raw)!=a['size'] or hashlib.sha256(raw).hexdigest()!=a['sha256']: raise SystemExit('artifact checksum differs')
 seen.add(p.name); print(a['kind'],p.name,a['sha256'],sep='\t')
PY
while IFS="	" read -r kind name checksum; do
  [ -n "$name" ] || continue
  case "$kind" in binary) dest=$stage/bin/$name; mode=0750;; jar) dest=$stage/jars/$name; mode=0640;; *) fail 'invalid inventory kind';; esac
  cp "$source_root/$name" "$dest"; chmod "$mode" "$dest"
  [ "$(sha256sum "$dest" | cut -d' ' -f1)" = "$checksum" ] || fail "staged checksum differs: $name"
done <"$stage/inventory"
[ "${LKJMC_INSTALL_FAULT:-}" != after-stage ] || fail 'injected failure after stage'
chown -R "$owner_uid:$service_gid" "$stage"
find "$stage" -type d -exec chmod "$dir_mode" {} +
if [ "$scope" = rootless ] && find "$stage" -perm /6000 -print -quit | grep -q .; then fail 'rootless setid file'; fi
python3 - "$stage" <<'PY'
import os,pathlib,sys
for p in pathlib.Path(sys.argv[1]).rglob('*'):
 if p.is_file():
  f=os.open(p,os.O_RDONLY); os.fsync(f); os.close(f)
PY
if [ -d "$root" ]; then mv "$root" "$rollback"; fi
if ! mv "$stage" "$root"; then [ ! -d "$rollback" ] || mv "$rollback" "$root"; fail 'publish failed'; fi
if [ "${LKJMC_INSTALL_FAULT:-}" = after-publish ]; then
  rm -rf "$root"; [ ! -d "$rollback" ] || mv "$rollback" "$root"; fail 'injected failure after publish'
fi
rm -rf "$rollback"; trap - EXIT HUP INT TERM
printf 'ok artifact-install scope=%s root=%s service=%s:%s\n' "$scope" "$root" "$service_uid" "$service_gid"
