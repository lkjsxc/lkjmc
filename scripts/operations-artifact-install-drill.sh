#!/bin/sh
set -eu
out=${1:?release evidence directory required}
rm -rf "$out"
scripts/build-release.sh "$out"
scripts/verify-artifact-manifest.py --manifest "$out/artifact-manifest.json" --release-root "$out"
scripts/scan-secrets.py --canary "${LKJMC_SECRET_CANARY:?generated canary required}" --path "$out"
evidence_owner=$(stat -c %u:%g "$(dirname "$out")")
uid=${evidence_owner%:*}; gid=${evidence_owner#*:}; unnamed_gid=42424
chown -R "$uid:$gid" "$out"
manifest=$out/artifact-manifest.json; source=$out/source
fingerprint() { stat -c '%i:%y:%u:%g:%a' "$1"; }
assert_no_temp() { ! find "$out" -maxdepth 1 \( -name '.lkjmc-stage-*' -o -name '.lkjmc-rollback-*' \) -print -quit | grep -q .; }
as_owner() { setpriv --reuid="$uid" --regid="$gid" --clear-groups -- "$@"; }
install_scope() {
 scope=$1; dest=$out/$scope
 if [ "$scope" = system ]; then
  scripts/install-artifacts.sh --scope system --manifest "$manifest" --source "$source" \
   --root "$dest" --service-uid "$uid" --service-gid "$unnamed_gid"
 else
  as_owner scripts/install-artifacts.sh --scope "$scope" --manifest "$manifest" --source "$source" --root "$dest"
 fi
 first=$(fingerprint "$dest/bin/lkjmc")
 if [ "$scope" = system ]; then
  scripts/install-artifacts.sh --scope system --manifest "$manifest" --source "$source" \
   --root "$dest" --service-uid "$uid" --service-gid "$unnamed_gid"
 else
  as_owner scripts/install-artifacts.sh --scope "$scope" --manifest "$manifest" --source "$source" --root "$dest"
 fi
 [ "$(fingerprint "$dest/bin/lkjmc")" = "$first" ] || { echo "$scope identical rerun changed inode, mtime, owner, or mode" >&2; exit 1; }
}
install_scope system
install_scope user
install_scope rootless
[ "$(stat -c %u:%g:%a "$out/system/bin/lkjmc")" = "0:$unnamed_gid:750" ]
[ "$(stat -c %u:%g:%a "$out/rootless/bin/lkjmc")" = "$uid:$gid:750" ]
changed=$out/changed
cp -a "$out/source" "$changed-source"
printf '\nchanged-release\n' >>"$changed-source/lkjmc"
mkdir -m 0700 "$changed"; mv "$changed-source" "$changed/source"
LKJMC_SOURCE_COMMIT=${LKJMC_SOURCE_COMMIT:?} scripts/artifact-manifest.py --release-root "$changed" --output "$changed/artifact-manifest.json"
chown -R "$uid:$gid" "$changed"
old=$(sha256sum "$out/user/bin/lkjmc")
if as_owner env LKJMC_SOURCE_COMMIT="$LKJMC_SOURCE_COMMIT" LKJMC_INSTALL_FAULT=status \
 scripts/install-artifacts.sh --scope user --manifest "$changed/artifact-manifest.json" \
 --source "$changed/source" --root "$out/user" >/dev/null 2>&1; then
 echo 'injected status validation failure passed' >&2; exit 1
fi
[ "$(sha256sum "$out/user/bin/lkjmc")" = "$old" ] || { echo 'rollback did not restore prior bytes' >&2; exit 1; }
as_owner env LKJMC_SOURCE_COMMIT="$LKJMC_SOURCE_COMMIT" scripts/install-artifacts.sh \
 --scope user --manifest "$changed/artifact-manifest.json" --source "$changed/source" --root "$out/user"
[ "$(sha256sum "$out/user/bin/lkjmc")" != "$old" ] || { echo 'changed release was not published' >&2; exit 1; }
assert_no_temp
chown -R "$evidence_owner" "$out"
find "$out" -type d -exec chmod 0700 {} +
find "$out" -type f -exec chmod 0600 {} +
printf '%s\n' 'ok artifact-install-drill scopes=system,user,rootless no-op=stable unnamed-gid=pass rollback=pass changed=atomic'
