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
id="smoke-$$"
cleanup() {
    if [ "${daemon_pid:-}" ]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    rm -f "$socket" "$daemon_log" "$out"
    rm -rf "$log_root" "$data_root"
}
trap cleanup EXIT
LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL cargo run -p lkjmc-cli -- db migrate >"$out" 2>&1
cargo run -p lkjmc-daemon -- \
    --socket "$socket" \
    --http none \
    --database-url "$LKJMC_STORE_TEST_DATABASE_URL" \
    --log-root "$log_root" \
    --data-root "$data_root" >"$daemon_log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 100); do
    [ -S "$socket" ] && break
    sleep 0.1
done
[ -S "$socket" ] || { cat "$daemon_log"; exit 1; }
cmd='echo lkjmc-ready; trap "exit 0" TERM; sleep 30 & wait'
cargo run -p lkjmc-cli -- --socket "$socket" instance create \
    --id "$id" --kind vanilla-custom --template process-smoke --command "$cmd" >"$out" 2>&1
cargo run -p lkjmc-cli -- --socket "$socket" instance start "$id" >"$out" 2>&1
for _ in $(seq 1 50); do
    cargo run -p lkjmc-cli -- --socket "$socket" --json instance list >"$out" 2>&1
    grep -q '"observedState":"process-healthy"' "$out" && break
    sleep 0.1
done
grep -q "$id" "$out"
grep -q '"observedState":"process-healthy"' "$out"
[ -f "$data_root/$id/eula.txt" ]
[ -f "$data_root/$id/server.properties" ]
if cargo run -p lkjmc-cli -- --socket "$socket" instance delete "$id" --yes >"$out" 2>&1; then
    cat "$out"
    exit 1
fi
cargo run -p lkjmc-cli -- --socket "$socket" instance logs "$id" --lines 5 >"$out" 2>&1
grep -q 'lkjmc-ready' "$out"
cargo run -p lkjmc-cli -- --socket "$socket" instance stop "$id" >"$out" 2>&1
cargo run -p lkjmc-cli -- --socket "$socket" --json instance list >"$out" 2>&1
grep -q '"observedState":"process-absent"' "$out"
cargo run -p lkjmc-cli -- --socket "$socket" instance delete "$id" --yes >"$out" 2>&1
printf '%s\n' 'ok process-runtime'
