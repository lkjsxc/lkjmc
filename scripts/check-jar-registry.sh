#!/bin/sh
set -eu
socket=$(mktemp -u "${TMPDIR:-/tmp}/lkjmc-jar.XXXXXX.sock")
log=$(mktemp "${TMPDIR:-/tmp}/lkjmc-jar.XXXXXX.log")
out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-jar.XXXXXX.out")
cleanup() {
    [ -z "${daemon_pid:-}" ] || kill "$daemon_pid" 2>/dev/null || true
    [ -z "${daemon_pid:-}" ] || wait "$daemon_pid" 2>/dev/null || true
    rm -f "$socket" "$log" "$out"
}
trap cleanup EXIT
cargo build -p lkjmc-cli -p lkjmc-daemon >"$log" 2>&1
target/debug/lkjmc-daemon --socket "$socket" --http none >"$log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 1200); do [ -S "$socket" ] && break; sleep 0.1; done
[ -S "$socket" ] || { cat "$log"; exit 1; }
if target/debug/lkjmc --socket "$socket" jar import --kind custom --name denied.jar --path /missing >"$out" 2>&1; then
    cat "$out"
    exit 1
fi
grep -q 'command.effect_denied' "$out"
printf '%s\n' 'ok jar-registry denied-before-filesystem'
