#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROPERTIES=$ROOT/gradle/wrapper/gradle-wrapper.properties
GRADLE_USER_HOME=${GRADLE_USER_HOME:-"$HOME/.gradle"}
umask 077
fail() { printf 'gradle bootstrap failed: %s\n' "$*" >&2; exit 1; }
property() {
  python3 - "$PROPERTIES" "$1" <<'PY'
import pathlib,sys
path,key=sys.argv[1:]
values={}
for raw in pathlib.Path(path).read_text(encoding='utf-8').splitlines():
    line=raw.strip()
    if not line or line.startswith(('#','!')) or '=' not in line: continue
    name,value=line.split('=',1)
    values[name.strip()]=value.strip().replace(r'\:', ':').replace(r'\\', '\\')
value=values.get(key,'')
if '\n' in value or '\r' in value: raise SystemExit(1)
print(value)
PY
}
URL=$(property distributionUrl) || fail 'cannot parse distributionUrl'
CHECKSUM=$(property distributionSha256Sum) || fail 'cannot parse distributionSha256Sum'
case "$URL" in https://*/gradle-*-bin.zip) ;; *) fail 'distributionUrl must be an HTTPS Gradle binary zip';; esac
case "$CHECKSUM" in
  *[!0-9a-f]*|'') fail 'distributionSha256Sum must be 64 lowercase hex characters';;
esac
[ "${#CHECKSUM}" -eq 64 ] || fail 'distributionSha256Sum must be 64 lowercase hex characters'
[ "$CHECKSUM" != 0000000000000000000000000000000000000000000000000000000000000000 ] || fail 'zero distribution checksum'
ARCHIVE=${URL##*/}; VERSION=${ARCHIVE#gradle-}; VERSION=${VERSION%-bin.zip}
CACHE=$GRADLE_USER_HOME/wrapper/dists/lkjmc/$CHECKSUM
ZIP=$CACHE/distribution.zip; DIST=$CACHE/gradle-$VERSION; BIN=$DIST/bin/gradle
mkdir -p "$CACHE"; chmod 0700 "$GRADLE_USER_HOME" "$GRADLE_USER_HOME/wrapper" \
  "$GRADLE_USER_HOME/wrapper/dists" "$GRADLE_USER_HOME/wrapper/dists/lkjmc" "$CACHE" 2>/dev/null || true
tmp_zip=; tmp_dir=
cleanup() { [ -z "$tmp_zip" ] || rm -f "$tmp_zip"; [ -z "$tmp_dir" ] || rm -rf "$tmp_dir"; }
trap cleanup EXIT HUP INT TERM
verify_zip() {
  [ -f "$1" ] && [ ! -L "$1" ] || fail 'distribution cache is not a regular file'
  actual=$(sha256sum "$1" | cut -d' ' -f1)
  [ "$actual" = "$CHECKSUM" ] || fail 'distribution checksum differs'
  python3 - "$1" "gradle-$VERSION" <<'PY'
import pathlib,stat,sys,zipfile
archive=pathlib.Path(sys.argv[1]); root=sys.argv[2]
with zipfile.ZipFile(archive) as z:
    bad=z.testzip()
    if bad: raise SystemExit('corrupt zip member: '+bad)
    for info in z.infolist():
        p=pathlib.PurePosixPath(info.filename)
        mode=info.external_attr >> 16
        if p.is_absolute() or '..' in p.parts or not p.parts or p.parts[0]!=root:
            raise SystemExit('unsafe distribution member')
        if stat.S_ISLNK(mode): raise SystemExit('symlink distribution member')
PY
}
if [ -e "$ZIP" ]; then
  verify_zip "$ZIP"
else
  tmp_zip=$(mktemp "$CACHE/.download.XXXXXX")
  chmod 0600 "$tmp_zip"
  curl -fsSL "$URL" -o "$tmp_zip" || fail 'distribution download failed'
  verify_zip "$tmp_zip"
  mv "$tmp_zip" "$ZIP"; tmp_zip=
fi
validate_dist() {
  python3 - "$ZIP" "$DIST" <<'PY'
import hashlib,pathlib,stat,sys,zipfile
archive,root=pathlib.Path(sys.argv[1]),pathlib.Path(sys.argv[2])
if root.is_symlink() or not root.is_dir(): raise SystemExit('invalid extracted cache')
with zipfile.ZipFile(archive) as z:
    expected={pathlib.PurePosixPath(i.filename).relative_to(root.name) for i in z.infolist() if not i.is_dir()}
    members=list(root.rglob('*'))
    if any(p.is_symlink() or (not p.is_dir() and not p.is_file()) for p in members): raise SystemExit('unsafe extracted cache member')
    actual={p.relative_to(root) for p in members if p.is_file()}
    if actual!=set(map(pathlib.Path,expected)): raise SystemExit('extracted cache closure differs')
    for info in z.infolist():
        if info.is_dir(): continue
        rel=pathlib.PurePosixPath(info.filename).relative_to(root.name)
        disk=root/pathlib.Path(rel)
        if disk.is_symlink() or hashlib.sha256(disk.read_bytes()).digest()!=hashlib.sha256(z.read(info)).digest():
            raise SystemExit('extracted cache bytes differ')
PY
}
if [ -e "$DIST" ]; then
  validate_dist
else
  tmp_dir=$(mktemp -d "$CACHE/.extract.XXXXXX")
  chmod 0700 "$tmp_dir"; unzip -q "$ZIP" -d "$tmp_dir"
  [ -x "$tmp_dir/gradle-$VERSION/bin/gradle" ] || fail 'distribution launcher missing'
  mv "$tmp_dir/gradle-$VERSION" "$DIST"; rm -rf "$tmp_dir"; tmp_dir=
  validate_dist
fi
[ -x "$BIN" ] || fail 'verified launcher is not executable'
trap - EXIT HUP INT TERM
exec "$BIN" "$@"
