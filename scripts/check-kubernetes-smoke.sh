#!/bin/sh
set -eu
if [ "${LKJMC_KUBERNETES_SMOKE:-}" != "1" ]; then
    printf '%s\n' 'ok kubernetes-smoke skipped'
    exit 0
fi
: "${LKJMC_KUBERNETES_CONFIG:?set LKJMC_KUBERNETES_CONFIG}"
: "${LKJMC_KUBERNETES_DATABASE_URL:?set LKJMC_KUBERNETES_DATABASE_URL}"
socket=$(mktemp -u "${TMPDIR:-/tmp}/lkjmc-k8s.XXXXXX.sock")
out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-k8s.XXXXXX.out")
log=$(mktemp "${TMPDIR:-/tmp}/lkjmc-k8s.XXXXXX.log")
id="k8s-smoke-$$"
cleanup() {
    if [ "${daemon_pid:-}" ]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    rm -f "$socket" "$out" "$log"
}
trap cleanup EXIT
cargo build -p lkjmc-cli -p lkjmc-daemon >"$out" 2>&1
LKJMC_DATABASE_URL=$LKJMC_KUBERNETES_DATABASE_URL target/debug/lkjmc db migrate >"$out" 2>&1
target/debug/lkjmc-daemon --config "$LKJMC_KUBERNETES_CONFIG" --socket "$socket" --http none >"$log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 1200); do [ -S "$socket" ] && break; sleep 0.1; done
[ -S "$socket" ] || { cat "$log"; exit 1; }
target/debug/lkjmc --socket "$socket" instance create --id "$id" --kind vanilla-custom --template k8s-smoke --command true >"$out" 2>&1
target/debug/lkjmc --socket "$socket" instance start "$id" >"$out" 2>&1
for _ in $(seq 1 60); do
    target/debug/lkjmc --socket "$socket" --json instance list >"$out" 2>&1 || true
    grep -q "$id" "$out" && break
    sleep 2
done
grep -q "$id" "$out"
target/debug/lkjmc --socket "$socket" instance logs "$id" --lines 20 >"$out" 2>&1 || true
target/debug/lkjmc --socket "$socket" instance stop "$id" >"$out" 2>&1
target/debug/lkjmc --socket "$socket" instance delete "$id" --yes --force >"$out" 2>&1
printf '%s\n' 'ok kubernetes-smoke'
