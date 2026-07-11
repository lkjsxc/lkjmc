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
    if [ "${daemon_pid:-}" ]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    rm -rf "$work"
}
trap cleanup EXIT
run_phase() {
    phase=$1
    shift
    if ! "$@" >"$out" 2>&1; then
        printf 'claim smoke failed phase=%s\n' "$phase" >&2
        exit 1
    fi
}
daemon_state() {
    if kill -0 "$daemon_pid" 2>/dev/null; then
        printf '%s' running
    elif wait "$daemon_pid" 2>/dev/null; then
        printf '%s' exited-0
    else
        printf 'exited-%s' "$?"
    fi
}
fail_daemon() {
    printf 'claim smoke failed phase=%s daemon=%s\n' "$1" "$(daemon_state)" >&2
    exit 1
}
call_daemon() {
    if ! python3 "$helper" "$socket" "$1" "$2" >"$out"; then
        fail_daemon "request-$1"
    fi
}
cat >"$helper" <<'PY'
import json, socket, sys, uuid
path, command, body = sys.argv[1], sys.argv[2], json.loads(sys.argv[3])
envelope = {
    "requestId": str(uuid.uuid4()),
    "actor": {"kind": "cli", "name": "claim-smoke"},
    "command": command,
    "body": body,
}
def fail(reason):
    raise SystemExit(f"claim smoke daemon response {reason}")
payload = json.dumps(envelope).encode()
request = b"".join((
    b"POST /command HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n",
    f"Content-Length: {len(payload)}\r\nConnection: close\r\n\r\n".encode(),
    payload,
))
try:
    with socket.socket(socket.AF_UNIX) as sock:
        sock.settimeout(5)
        sock.connect(path)
        sock.sendall(request)
        chunks = []
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            chunks.append(chunk)
except OSError:
    fail("transport-failed")
head, separator, body = b"".join(chunks).partition(b"\r\n\r\n")
if not separator:
    fail("was-empty-or-non-http")
if not head.startswith(b"HTTP/1.1 200"):
    fail("was-http-error")
try:
    response = json.loads(body.decode())
except (UnicodeDecodeError, json.JSONDecodeError):
    fail("was-invalid-json")
if not isinstance(response, dict) or not response.get("ok"):
    fail("was-unsuccessful")
print(json.dumps(response.get("body") or {}, separators=(",", ":")))
PY
run_phase build cargo build -p lkjmc-cli -p lkjmc-daemon
run_phase reset-db env LKJMC_TEST_RESET_DATABASE=1 "LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL" \
    target/debug/lkjmc db reset-test
run_phase migrate-db env "LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL" target/debug/lkjmc db migrate
target/debug/lkjmc-daemon --socket "$socket" --http none \
    --database-url "$LKJMC_STORE_TEST_DATABASE_URL" \
    --log-root "$work/logs" --data-root "$work/data" >"$work/daemon.log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 300); do
    [ -S "$socket" ] && break
    sleep 0.1
done
[ -S "$socket" ] || fail_daemon socket-ready
owner='00000000-0000-0000-0000-000000000101'
trusted='00000000-0000-0000-0000-000000000102'
create_body='{"ownerUuid":"'$owner'","ownerName":"Owner","name":"Base","instanceId":"survival","worldName":"world","chunkX":1,"chunkZ":2}'
call_daemon claim.create "$create_body"
claim_id=$(python3 - "$out" <<'PY'
import json, sys
try:
    claim_id = json.load(open(sys.argv[1]))["claimId"]
except (OSError, json.JSONDecodeError, KeyError, TypeError):
    raise SystemExit("claim smoke response missing claimId")
print(claim_id)
PY
)
trust_body='{"ownerUuid":"'$owner'","trustedUuid":"'$trusted'","trustedName":"Friend","instanceId":"survival","worldName":"world","chunkX":1,"chunkZ":2}'
call_daemon claim.trust "$trust_body"
call_daemon claim.snapshot '{"instanceId":"survival"}'
python3 - "$out" "$claim_id" "$trusted" <<'PY'
import json, sys
try:
    chunks = json.load(open(sys.argv[1])).get("chunks", [])
except (OSError, json.JSONDecodeError, AttributeError):
    raise SystemExit("claim smoke response body invalid")
assert len(chunks) == 1, chunks
assert chunks[0]["claimId"] == sys.argv[2], chunks
assert chunks[0]["trusts"][0]["uuid"] == sys.argv[3], chunks
PY
untrust_body='{"ownerUuid":"'$owner'","trustedUuid":"'$trusted'","name":"Base"}'
call_daemon claim.untrust "$untrust_body"
call_daemon claim.snapshot '{"instanceId":"survival"}'
python3 - "$out" <<'PY'
import json, sys
try:
    chunks = json.load(open(sys.argv[1])).get("chunks", [])
except (OSError, json.JSONDecodeError, AttributeError):
    raise SystemExit("claim smoke response body invalid")
assert chunks and chunks[0].get("trusts") == [], chunks
PY
trust_name_body='{"ownerUuid":"'$owner'","trustedUuid":"'$trusted'","trustedName":"Friend","name":"Base"}'
call_daemon claim.trust "$trust_name_body"
run_phase claim-list target/debug/lkjmc --socket "$socket" --json claim list --instance survival
grep -q "$claim_id" "$out" || { echo 'claim smoke failed phase=claim-list-result' >&2; exit 1; }
run_phase claim-delete target/debug/lkjmc --socket "$socket" --json claim delete "$claim_id" --yes
call_daemon claim.snapshot '{"instanceId":"survival"}'
python3 - "$out" <<'PY'
import json, sys
try:
    chunks = json.load(open(sys.argv[1])).get("chunks")
except (OSError, json.JSONDecodeError, AttributeError):
    raise SystemExit("claim smoke response body invalid")
assert chunks == []
PY
printf '%s\n' 'ok claim smoke'
