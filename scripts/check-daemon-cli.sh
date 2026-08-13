#!/bin/sh
set -eu
socket=$(mktemp -u "${TMPDIR:-/tmp}/lkjmc-daemon.XXXXXX.sock")
log=$(mktemp "${TMPDIR:-/tmp}/lkjmc-daemon.XXXXXX.log")
doctor_out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-doctor.XXXXXX.out")
status_out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-status.XXXXXX.out")
status_human_out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-status-human.XXXXXX.out")
version_out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-version.XXXXXX.out")
cleanup() {
    if [ "${daemon_pid:-}" ]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    rm -f "$socket" "$log" "$doctor_out" "$status_out" "$status_human_out" "$version_out"
}
trap cleanup EXIT
cargo build -p lkjmc-cli -p lkjmc-daemon >"$log" 2>&1
target/debug/lkjmc-daemon --socket "$socket" --http none >"$log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 1200); do
    if [ -S "$socket" ]; then
        break
    fi
    sleep 0.1
done
if [ ! -S "$socket" ]; then
    cat "$log"
    exit 1
fi
if target/debug/lkjmc --socket "$socket" doctor >"$doctor_out" 2>&1; then
    cat "$doctor_out"
    exit 1
fi
grep -q 'command.effect_denied' "$doctor_out"
target/debug/lkjmc --socket "$socket" status --json >"$status_out" 2>>"$log"
grep -q '"daemon":"running"' "$status_out"
grep -q '"version":"0.1.0-alpha.1"' "$status_out"
grep -Eq '"commit":"([0-9a-f]{40}|unknown)"' "$status_out"
grep -q '"instances":null' "$status_out"
grep -q '"runtimeRefresh":false' "$status_out"
target/debug/lkjmc --socket "$socket" status >"$status_human_out" 2>>"$log"
grep -Eq '^build: version=0\.1\.0-alpha\.1 commit=([0-9a-f]{40}|unknown) dirty=(false|unknown)$' "$status_human_out"
grep -q '^instances: unknown$' "$status_human_out"
target/debug/lkjmc --json version >"$version_out"
grep -q '"version":"0.1.0-alpha.1"' "$version_out"
grep -Eq '"commit":"([0-9a-f]{40}|unknown)"' "$version_out"
target/debug/lkjmc-daemon --version | grep -Eq \
    '^lkjmc-daemon 0\.1\.0-alpha\.1 commit=([0-9a-f]{40}|unknown) dirty=(false|unknown)$'
if [ "${LKJMC_ASSERT_SHUTDOWN:-0}" = 1 ]; then
    kill -TERM "$daemon_pid"
    wait "$daemon_pid"
    daemon_pid=""
    printf '%s\n' 'ok shutdown-pass'
fi
printf '%s\n' 'ok daemon-cli'
