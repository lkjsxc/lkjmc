#!/bin/sh
set -eu
if [ "${LKJMC_INSTALLER_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok installer skipped'
    exit 0
fi
command -v docker >/dev/null 2>&1 || { printf '%s\n' 'docker required' >&2; exit 1; }
repo=$(pwd)
docker run --rm -v "$repo:/src:ro" ubuntu:24.04 sh -eu -c '
    apt-get update >/dev/null
    apt-get install -y --no-install-recommends ca-certificates git >/dev/null
    cp -a /src /work
    cd /work
    ./scripts/install.sh
    /opt/lkjmc/bin/lkjmc --socket /run/lkjmc/daemon.sock status --json >/tmp/status.json
    grep -q "daemon" /tmp/status.json
'
printf '%s\n' 'ok installer'
