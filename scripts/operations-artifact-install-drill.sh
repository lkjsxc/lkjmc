#!/bin/sh
set -eu
out=${1:?release evidence directory required}
rm -rf "$out"; mkdir -p "$out/source"
cp target/release/lkjmc target/release/lkjmc-daemon "$out/source/"
for jar in platforms/jvm/*/build/libs/*.jar; do
  [ -f "$jar" ] || continue
  component=$(basename "$(dirname "$(dirname "$(dirname "$jar")")")")
  cp "$jar" "$out/source/$component-$(basename "$jar")"
done
set -- "$out/source/lkjmc" "$out/source/lkjmc-daemon"
for jar in "$out"/source/*.jar; do [ -f "$jar" ] && set -- "$@" "$jar"; done
[ "$#" -gt 2 ] || { echo 'no release jar built' >&2; exit 1; }
scripts/artifact-manifest.py --output "$out/artifact-manifest.json" "$@"
chmod -R a+rX "$out/source" "$out/artifact-manifest.json"
uid=$(id -u gradle); gid=$(id -g gradle); chown "$uid:$gid" "$out"

system=$out/system; user=$out/user; rootless=$out/rootless
scripts/install-artifacts.sh --scope system --manifest "$out/artifact-manifest.json" \
  --source "$out/source" --root "$system" --service-uid "$uid" --service-gid "$gid"
system_hash=$(sha256sum "$system/bin/lkjmc")
scripts/install-artifacts.sh --scope system --manifest "$out/artifact-manifest.json" \
  --source "$out/source" --root "$system" --service-uid "$uid" --service-gid "$gid"
[ "$(sha256sum "$system/bin/lkjmc")" = "$system_hash" ]
if LKJMC_INSTALL_FAULT=after-publish scripts/install-artifacts.sh --scope system \
  --manifest "$out/artifact-manifest.json" --source "$out/source" --root "$system" \
  --service-uid "$uid" --service-gid "$gid" >/dev/null 2>&1; then
  echo 'injected system publish fault passed' >&2; exit 1
fi
[ "$(sha256sum "$system/bin/lkjmc")" = "$system_hash" ]
for scope in user rootless; do
  dest=$out/$scope
  runuser -u gradle -- scripts/install-artifacts.sh --scope "$scope" \
    --manifest "$out/artifact-manifest.json" --source "$out/source" --root "$dest"
  first=$(sha256sum "$dest/bin/lkjmc")
  runuser -u gradle -- scripts/install-artifacts.sh --scope "$scope" \
    --manifest "$out/artifact-manifest.json" --source "$out/source" --root "$dest"
  [ "$(sha256sum "$dest/bin/lkjmc")" = "$first" ]
  if runuser -u gradle -- env LKJMC_INSTALL_FAULT=after-publish scripts/install-artifacts.sh \
    --scope "$scope" --manifest "$out/artifact-manifest.json" --source "$out/source" \
    --root "$dest" >/dev/null 2>&1; then echo "$scope rollback fault passed" >&2; exit 1; fi
  [ "$(sha256sum "$dest/bin/lkjmc")" = "$first" ]
done
[ "$(stat -c %u:%g:%a "$system/bin/lkjmc")" = "0:$gid:750" ]
[ "$(stat -c %u:%g:%a "$rootless/bin/lkjmc")" = "$uid:$gid:750" ]
! find "$out" -maxdepth 1 -name '.lkjmc-*' -print -quit | grep -q .
evidence_owner=$(stat -c %u:%g "$(dirname "$out")")
chown -R "$evidence_owner" "$out"
find "$out" -type d -exec chmod 0700 {} +
find "$out" -type f -exec chmod 0600 {} +
printf '%s\n' 'ok artifact-install-drill scopes=system,user,rootless reruns=2 rollback=pass'
