#!/bin/sh
set -eu
out=${1:?release evidence directory required}
rm -rf "$out"
evidence_owner=$(stat -c %u:%g "$(dirname "$out")")
uid=${evidence_owner%:*}; gid=${evidence_owner#*:}; unnamed_gid=42424
finish() {
  status=$?
  trap - EXIT HUP INT TERM
  if [ -d "$out" ]; then scripts/private-artifact-handoff.py --owner "$evidence_owner" "$out" || status=1; fi
  exit "$status"
}
trap finish EXIT
trap 'exit 1' HUP INT TERM
scripts/build-release.sh "$out"
scripts/private-artifact-handoff.py --owner "$evidence_owner" "$out"
scripts/verify-artifact-manifest.py --manifest "$out/artifact-manifest.json" --release-root "$out"
scripts/scan-secrets.py --canary "${LKJMC_SECRET_CANARY:?generated canary required}" --path "$out"
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
scripts/private-artifact-handoff.py --owner "$evidence_owner" "$out"
printf '%s\n' 'ok artifact-install-drill scopes=system,user,rootless no-op=stable unnamed-gid=pass rollback=pass changed=atomic'
