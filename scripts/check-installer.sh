#!/bin/sh
set -eu
if [ "${LKJMC_INSTALLER_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok installer skipped'
    exit 0
fi
command -v docker >/dev/null 2>&1 || { printf '%s\n' 'docker required' >&2; exit 1; }
repo=$(pwd)
docker run --rm -v "$repo:/src:ro" ubuntu:24.04 sh -eu -c '
    fail() { printf "%s\n" "installer smoke: $*" >&2; exit 1; }
    run_install() {
        log=$1
        ./scripts/install.sh >"$log" 2>&1 || {
            tail -80 "$log" >&2
            fail "installer failed"
        }
    }
    daemon_count() {
        count=0
        for proc in /proc/[0-9]*; do
            uid=$(stat -c %u "$proc" 2>/dev/null || true)
            name=$(cat "$proc/comm" 2>/dev/null || true)
            if [ "$uid" = "$service_uid" ] && [ "$name" = lkjmc-daemon ]; then count=$((count + 1)); fi
        done
        printf "%s\n" "$count"
    }
    apt-get update >/dev/null
    apt-get install -y --no-install-recommends ca-certificates git >/dev/null
    cp -a /src /work
    chown -R 61234:61234 /work
    getent passwd 61234 >/dev/null 2>&1 && fail "test UID unexpectedly named"
    getent group 61234 >/dev/null 2>&1 && fail "test GID unexpectedly named"
    cd /work
    source_before=$(stat -c "%n %u:%g %a" Cargo.toml scripts/install.sh scripts/install-support.sh)
    run_install /tmp/install-1.log
    runuser -u lkjmc -- /opt/lkjmc/bin/lkjmc --socket /run/lkjmc/daemon.sock status --json >/tmp/status.json
    grep -q "daemon" /tmp/status.json || fail "daemon status missing"
    service_uid=$(id -u lkjmc); service_gid=$(id -g lkjmc)
    service_groups=$(id -G lkjmc)
    case " $service_groups " in *" 61234 "*) fail "unnamed checkout GID became supplemental" ;; esac
    for path in /opt/lkjmc/jars /opt/lkjmc/assets /var/lib/lkjmc /var/log/lkjmc /run/lkjmc; do
        [ "$(stat -c %u:%g "$path")" = "$service_uid:$service_gid" ] || fail "ownership mismatch: $path"
    done
    for secret in /etc/lkjmc/database.secret /etc/lkjmc/daemon-http.token /etc/lkjmc/forwarding.secret /etc/lkjmc/daemon.env; do
        [ "$(stat -c %u:%g:%a "$secret")" = "$service_uid:$service_gid:600" ] || fail "private mode mismatch: $secret"
    done
    secret_hashes=$(sha256sum /etc/lkjmc/database.secret /etc/lkjmc/daemon-http.token /etc/lkjmc/forwarding.secret)
    first_pid=$(cat /run/lkjmc/daemon.pid)
    count=$(daemon_count)
    [ "$count" = 1 ] || { ps -ef >&2; fail "first install daemon count drift: $count"; }
    run_install /tmp/install-2.log
    second_pid=$(cat /run/lkjmc/daemon.pid)
    [ "$first_pid" != "$second_pid" ] || fail "rerun did not replace daemon"
    kill -0 "$first_pid" 2>/dev/null && fail "rerun left old daemon alive"
    count=$(daemon_count)
    [ "$count" = 1 ] || { ps -ef >&2; fail "rerun daemon count drift: $count"; }
    [ "$(id -u lkjmc):$(id -g lkjmc):$(id -G lkjmc)" = "$service_uid:$service_gid:$service_groups" ] || fail "service privilege drift"
    [ "$(sha256sum /etc/lkjmc/database.secret /etc/lkjmc/daemon-http.token /etc/lkjmc/forwarding.secret)" = "$secret_hashes" ] || fail "rerun replaced secrets"
    [ "$(stat -c "%n %u:%g %a" Cargo.toml scripts/install.sh scripts/install-support.sh)" = "$source_before" ] || fail "source ownership drift"
    for secret in /etc/lkjmc/database.secret /etc/lkjmc/daemon-http.token /etc/lkjmc/forwarding.secret; do
        value=$(cat "$secret")
        grep -F "$value" /tmp/install-1.log /tmp/install-2.log >/dev/null && fail "installer printed a generated secret"
    done
    kill "$second_pid"
    i=0
    while kill -0 "$second_pid" 2>/dev/null && [ "$i" -lt 50 ]; do sleep 0.1; i=$((i + 1)); done
    [ "$(daemon_count)" = 0 ] || fail "daemon cleanup drift"
'
printf '%s\n' 'ok installer unnamed-gid rerun ownership privilege secret process'
