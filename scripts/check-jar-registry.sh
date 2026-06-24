#!/bin/sh
set -eu
if [ -z "${LKJMC_STORE_TEST_DATABASE_URL:-}" ]; then
    printf '%s\n' 'ok jar-registry skipped'
    exit 0
fi
work=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-jar.XXXXXX")
socket=$(mktemp -u "${TMPDIR:-/tmp}/lkjmc-jar.XXXXXX.sock")
daemon_log=$(mktemp "${TMPDIR:-/tmp}/lkjmc-jar-daemon.XXXXXX.log")
out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-jar.XXXXXX.out")
id="jar-smoke-$$"
cleanup() {
    if [ "${daemon_pid:-}" ]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    rm -f "$socket" "$daemon_log" "$out"
    rm -rf "$work"
}
trap cleanup EXIT
mkdir -p "$work/classes" "$work/jars" "$work/logs" "$work/data"
cat >"$work/Smoke.java" <<'JAVA'
public final class Smoke {
    public static void main(String[] args) throws Exception {
        System.out.println("lkjmc-jar-ready");
        Thread.sleep(30000L);
    }
}
JAVA
javac -d "$work/classes" "$work/Smoke.java"
printf '%s\n' 'Main-Class: Smoke' >"$work/manifest.txt"
jar cfm "$work/smoke.jar" "$work/manifest.txt" -C "$work/classes" .
LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL cargo run -p lkjmc-cli -- db migrate >"$out" 2>&1
cargo run -p lkjmc-daemon -- \
    --socket "$socket" \
    --http none \
    --database-url "$LKJMC_STORE_TEST_DATABASE_URL" \
    --log-root "$work/logs" \
    --jar-root "$work/jars" \
    --data-root "$work/data" >"$daemon_log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 100); do
    [ -S "$socket" ] && break
    sleep 0.1
done
[ -S "$socket" ] || { cat "$daemon_log"; exit 1; }
cargo run -p lkjmc-cli -- --socket "$socket" --json jar import \
    --kind custom --name smoke.jar --path "$work/smoke.jar" >"$out" 2>&1
asset_id=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["id"])' "$out")
asset_path=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["path"])' "$out")
cargo run -p lkjmc-cli -- --socket "$socket" jar list >"$out" 2>&1
cargo run -p lkjmc-cli -- --socket "$socket" instance create \
    --id "$id" --kind vanilla-custom --template jar-smoke \
    --jar-asset "$asset_id" --memory-mb 128 >"$out" 2>&1
cargo run -p lkjmc-cli -- --socket "$socket" instance start "$id" >"$out" 2>&1
for _ in $(seq 1 50); do
    cargo run -p lkjmc-cli -- --socket "$socket" instance logs "$id" --lines 10 >"$out" 2>&1 || true
    grep -q 'lkjmc-jar-ready' "$out" && break
    sleep 0.1
done
grep -q 'lkjmc-jar-ready' "$out"
[ -f "$work/data/$id/eula.txt" ]
[ -f "$work/data/$id/server.properties" ]
cargo run -p lkjmc-cli -- --socket "$socket" instance stop "$id" >"$out" 2>&1
printf '%s' bad >>"$asset_path"
if cargo run -p lkjmc-cli -- --socket "$socket" instance start "$id" >"$out" 2>&1; then
    cat "$out"
    exit 1
fi
grep -q 'checksum mismatch' "$out"
cargo run -p lkjmc-cli -- --socket "$socket" instance delete "$id" --yes --force >"$out" 2>&1
cargo run -p lkjmc-cli -- --socket "$socket" jar prune --yes >"$out" 2>&1
[ ! -e "$asset_path" ]
printf '%s\n' 'ok jar-registry'
