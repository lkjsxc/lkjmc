#!/bin/sh
set -eu
if [ -z "${LKJMC_STORE_TEST_DATABASE_URL:-}" ]; then
    printf '%s\n' 'ok process-runtime skipped'
    exit 0
fi
socket=$(mktemp -u "${TMPDIR:-/tmp}/lkjmc-runtime.XXXXXX.sock")
log_root=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-runtime-logs.XXXXXX")
data_root=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-runtime-data.XXXXXX")
daemon_log=$(mktemp "${TMPDIR:-/tmp}/lkjmc-runtime-daemon.XXXXXX.log")
out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-runtime.XXXXXX.out")
id="smoke-$(date +%s)-$$"
cleanup() {
    if [ "${daemon_pid:-}" ]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    rm -f "$socket" "$daemon_log" "$out"
    rm -rf "$log_root" "$data_root"
}
trap cleanup EXIT
cargo build -p lkjmc-cli -p lkjmc-daemon >"$out" 2>&1
LKJMC_TEST_RESET_DATABASE=1 LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL \
    target/debug/lkjmc db reset-test >"$out" 2>&1
LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL target/debug/lkjmc db migrate >"$out" 2>&1
target/debug/lkjmc-daemon \
    --socket "$socket" \
    --http none \
    --database-url "$LKJMC_STORE_TEST_DATABASE_URL" \
    --log-root "$log_root" \
    --data-root "$data_root" >"$daemon_log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 1200); do
    [ -S "$socket" ] && break
    sleep 0.1
done
[ -S "$socket" ] || { cat "$daemon_log"; exit 1; }
cmd='echo lkjmc-ready; while read line; do [ "$line" = stop ] && exit 0; done'
target/debug/lkjmc --socket "$socket" instance create \
    --id "$id" --kind vanilla-custom --template process-smoke --command "$cmd" >"$out" 2>&1
target/debug/lkjmc --socket "$socket" instance start "$id" >"$out" 2>&1
for _ in $(seq 1 50); do
    target/debug/lkjmc --socket "$socket" --json instance list >"$out" 2>&1
    grep -q '"observedState":"process-healthy"' "$out" && break
    sleep 0.1
done
grep -q "$id" "$out"
grep -q '"observedState":"process-healthy"' "$out"
[ -f "$data_root/$id/eula.txt" ]
[ -f "$data_root/$id/server.properties" ]
if target/debug/lkjmc --socket "$socket" instance delete "$id" --yes >"$out" 2>&1; then
    cat "$out"
    exit 1
fi
target/debug/lkjmc --socket "$socket" instance logs "$id" --lines 5 >"$out" 2>&1
grep -q 'lkjmc-ready' "$out"
target/debug/lkjmc --socket "$socket" instance stop "$id" >"$out" 2>&1
target/debug/lkjmc --socket "$socket" --json instance list >"$out" 2>&1
grep -q '"observedState":"process-absent"' "$out"
target/debug/lkjmc --socket "$socket" instance delete "$id" --yes >"$out" 2>&1
printf '%s\n' 'ok process-runtime'
