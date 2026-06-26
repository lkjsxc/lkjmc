#!/bin/sh
set -eu
if [ "${LKJMC_PLUGIN_LIVE_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok check-plugin-assets skipped'
    exit 0
fi
: "${LKJMC_STORE_TEST_DATABASE_URL:?set LKJMC_STORE_TEST_DATABASE_URL for plugin asset smoke}"
root=$(mktemp -d)
cleanup() {
    if [ -n "${daemon_pid:-}" ]; then kill "$daemon_pid" 2>/dev/null || true; fi
    rm -rf "$root"
}
trap cleanup EXIT
mkdir -p "$root/config" "$root/logs" "$root/jars" "$root/data" "$root/run"
printf '%s\n' 'pw' >"$root/config/db.secret"
printf '%s\n' 'token' >"$root/config/http.token"
cat >"$root/config/lkjmc.json" <<JSON
{"installRoot":"$root","configRoot":"$root/config","dataRoot":"$root/data","logRoot":"$root/logs","socketPath":"$root/run/daemon.sock","database":{"host":"127.0.0.1","port":5432,"database":"lkjmc","user":"lkjmc","secretFile":"$root/config/db.secret"},"network":{"defaultLocale":"en","fallbackServer":"hub","onlineMode":true,"velocityForwarding":"modern"},"jars":{"root":"$root/jars","defaultChannel":"stable","userAgent":"lkjmc (+https://github.com/lkjsxc/lkjmc)"},"runtime":{"adapter":"local-process","defaultJavaMemoryMb":1024,"stopTimeoutSeconds":30}}
JSON
cargo build -p lkjmc-daemon -p lkjmc-cli >/dev/null
LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL cargo run -p lkjmc-cli -- db migrate >/dev/null
LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL cargo run -p lkjmc-daemon -- --config "$root/config/lkjmc.json" --database-url "$LKJMC_STORE_TEST_DATABASE_URL" --socket "$root/run/daemon.sock" --http none >"$root/daemon.log" 2>&1 &
daemon_pid=$!
i=0
while [ "$i" -lt 60 ]; do [ -S "$root/run/daemon.sock" ] && break; i=$((i + 1)); sleep 1; done
[ -S "$root/run/daemon.sock" ] || { cat "$root/daemon.log"; exit 1; }
cargo run -p lkjmc-cli -- --socket "$root/run/daemon.sock" asset plugin sync --plugin viaversion >/dev/null
cargo run -p lkjmc-cli -- --socket "$root/run/daemon.sock" asset plugin inspect viaversion >/dev/null
printf '%s\n' 'ok check-plugin-assets'
