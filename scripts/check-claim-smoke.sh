#!/bin/sh
set -eu
if [ "${LKJMC_CLAIM_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok claim smoke skipped'
    exit 0
fi
[ -n "${LKJMC_STORE_TEST_DATABASE_URL:-}" ] || {
    echo 'LKJMC_STORE_TEST_DATABASE_URL is required for claim smoke' >&2
    exit 1
}
work=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-claim.XXXXXX")
socket="$work/daemon.sock"
out="$work/out"
helper="$work/call_daemon.py"
daemon_pid=''
cleanup() {
    [ -z "${daemon_pid:-}" ] || kill "$daemon_pid" 2>/dev/null || true
    [ -z "${daemon_pid:-}" ] || wait "$daemon_pid" 2>/dev/null || true
    rm -rf "$work"
}
trap cleanup EXIT
run() { "$@" >"$out" 2>&1 || { cat "$out" >&2; exit 1; }; }
cat >"$helper" <<'PY'
import json, socket, sys, uuid
path, command, body, expected = sys.argv[1], sys.argv[2], json.loads(sys.argv[3]), sys.argv[4]
envelope = {"requestId": str(uuid.uuid4()), "actor": {"kind": "cli", "name": "claim-smoke"}, "command": command, "body": body}
payload = json.dumps(envelope).encode()
request = b"".join((b"POST /command HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n", f"Content-Length: {len(payload)}\r\nConnection: close\r\n\r\n".encode(), payload))
with socket.socket(socket.AF_UNIX) as sock:
    sock.settimeout(5)
    sock.connect(path)
    sock.sendall(request)
    response = b"".join(iter(lambda: sock.recv(4096), b""))
head, separator, body = response.partition(b"\r\n\r\n")
if not separator or not head.startswith(b"HTTP/1.1 200"):
    raise SystemExit("claim smoke daemon response was-not-successful")
reply = json.loads(body.decode())
if expected == "denied":
    if reply.get("ok") or reply.get("error", {}).get("code") != "auth.surface_denied":
        raise SystemExit("claim smoke internal command was not denied")
else:
    if not reply.get("ok"):
        raise SystemExit("claim smoke cli command was unsuccessful")
print(json.dumps(reply.get("body") or {}, separators=(",", ":")))
PY
run cargo build -p lkjmc-cli -p lkjmc-daemon
run env LKJMC_TEST_RESET_DATABASE=1 "LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL" target/debug/lkjmc db reset-test
run env "LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL" target/debug/lkjmc db migrate
target/debug/lkjmc-daemon --socket "$socket" --http none --database-url "$LKJMC_STORE_TEST_DATABASE_URL" --log-root "$work/logs" --data-root "$work/data" >"$work/daemon.log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 300); do [ -S "$socket" ] && break; sleep 0.1; done
[ -S "$socket" ] || { cat "$work/daemon.log" >&2; exit 1; }
owner='00000000-0000-0000-0000-000000000101'
create_body='{"ownerUuid":"'$owner'","ownerName":"Owner","name":"Base","instanceId":"survival","worldName":"world","chunkX":1,"chunkZ":2}'
run python3 "$helper" "$socket" claim.create "$create_body" denied
run python3 "$helper" "$socket" claim.snapshot '{"instanceId":"survival"}' ok
python3 - "$out" <<'PY'
import json, sys
assert json.load(open(sys.argv[1])).get("chunks") == []
PY
run target/debug/lkjmc --socket "$socket" --json claim list --instance survival
printf '%s\n' 'ok claim smoke internal-create-denied cli-read-ok'
