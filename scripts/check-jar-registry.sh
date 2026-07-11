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
forwarding_secret=$(mktemp "${TMPDIR:-/tmp}/lkjmc-jar.XXXXXX.secret")
printf '%s\n' 'jar-smoke-forwarding-secret' >"$forwarding_secret"
id="jar-smoke-$(date +%s)-$$"
cleanup() {
    if [ "${daemon_pid:-}" ]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    rm -f "$socket" "$daemon_log" "$out" "$forwarding_secret"
    rm -rf "$work"
}
trap cleanup EXIT
run_out() {
    if ! "$@" >"$out" 2>&1; then
        cat "$out"
        exit 1
    fi
}
mkdir -p "$work/classes" "$work/jars" "$work/logs" "$work/data"
cat >"$work/Smoke.java" <<'JAVA'
public final class Smoke {
    public static void main(String[] args) throws Exception {
        System.out.println("lkjmc-jar-ready");
        var in = new java.io.BufferedReader(new java.io.InputStreamReader(System.in));
        while (true) {
            if ("stop".equals(in.readLine())) return;
        }
    }
}
JAVA
javac -d "$work/classes" "$work/Smoke.java"
printf '%s\n' 'Main-Class: Smoke' >"$work/manifest.txt"
jar cfm "$work/smoke.jar" "$work/manifest.txt" -C "$work/classes" .
run_out cargo build -p lkjmc-cli -p lkjmc-daemon
LKJMC_TEST_RESET_DATABASE=1 LKJMC_DATABASE_URL="$LKJMC_STORE_TEST_DATABASE_URL" \
    run_out target/debug/lkjmc db reset-test
LKJMC_DATABASE_URL="$LKJMC_STORE_TEST_DATABASE_URL" run_out target/debug/lkjmc db migrate
target/debug/lkjmc-daemon \
    --socket "$socket" \
    --http none \
    --database-url "$LKJMC_STORE_TEST_DATABASE_URL" \
    --log-root "$work/logs" \
    --jar-root "$work/jars" \
    --data-root "$work/data" >"$daemon_log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 1200); do
    [ -S "$socket" ] && break
    sleep 0.1
done
[ -S "$socket" ] || { cat "$daemon_log"; exit 1; }
run_out target/debug/lkjmc --socket "$socket" --json jar import \
    --kind custom --name smoke.jar --path "$work/smoke.jar"
asset_id=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["id"])' "$out")
asset_path=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["path"])' "$out")
run_out target/debug/lkjmc --socket "$socket" jar list
run_out target/debug/lkjmc --socket "$socket" instance create \
    --id "$id" --kind velocity --template jar-smoke \
    --jar-asset "$asset_id" --memory-mb 256 \
    --forwarding-secret-file "$forwarding_secret"
run_out target/debug/lkjmc --socket "$socket" instance start "$id"
for _ in $(seq 1 300); do
    target/debug/lkjmc --socket "$socket" instance logs "$id" --lines 10 >"$out" 2>&1 || true
    grep -q 'lkjmc-jar-ready' "$out" && break
    sleep 0.1
done
grep -q 'lkjmc-jar-ready' "$out" || { cat "$out"; exit 1; }
run_out target/debug/lkjmc --socket "$socket" instance stop "$id"
printf '%s' bad >>"$asset_path"
if target/debug/lkjmc --socket "$socket" instance start "$id" >"$out" 2>&1; then
    cat "$out"
    exit 1
fi
grep -q 'checksum mismatch' "$out" || { cat "$out"; exit 1; }
run_out target/debug/lkjmc --socket "$socket" instance delete "$id" --yes --force
run_out target/debug/lkjmc --socket "$socket" jar prune --yes
[ ! -e "$asset_path" ] || { echo 'asset was not pruned'; exit 1; }
if [ "${LKJMC_JAR_LIVE_SMOKE:-0}" = "1" ]; then
    run_out target/debug/lkjmc --socket "$socket" jar sync \
        --project paper --channel stable
    grep -q 'ok jar sync' "$out" || { cat "$out"; exit 1; }
fi
printf '%s\n' 'ok jar-registry'
