#!/bin/sh
set -eu
socket=$(mktemp -u "${TMPDIR:-/tmp}/lkjmc-daemon.XXXXXX.sock")
log=$(mktemp "${TMPDIR:-/tmp}/lkjmc-daemon.XXXXXX.log")
doctor_out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-doctor.XXXXXX.out")
status_out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-status.XXXXXX.out")
cleanup() {
    if [ "${daemon_pid:-}" ]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    rm -f "$socket" "$log" "$doctor_out" "$status_out"
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
target/debug/lkjmc --socket "$socket" doctor >"$doctor_out" 2>>"$log"
grep -qx 'ok doctor' "$doctor_out"
target/debug/lkjmc --socket "$socket" status --json >"$status_out" 2>>"$log"
grep -q '"daemon":"running"' "$status_out"
printf '%s\n' 'ok daemon-cli'
