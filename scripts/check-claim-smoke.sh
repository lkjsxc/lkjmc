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
socket=$(mktemp -u "${TMPDIR:-/tmp}/lkjmc-claim.XXXXXX.sock")
work=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-claim.XXXXXX")
out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-claim.XXXXXX.out")
helper="$work/call_daemon.py"
daemon_pid=''
cleanup() {
    if [ "${daemon_pid:-}" ]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    rm -f "$socket" "$out"
    rm -rf "$work"
}
trap cleanup EXIT
cat >"$helper" <<'PY'
import json, socket, sys, uuid
path, command, body = sys.argv[1], sys.argv[2], json.loads(sys.argv[3])
envelope = {
    "requestId": str(uuid.uuid4()),
    "actor": {"kind": "cli", "name": "claim-smoke"},
    "command": command,
    "body": body,
}
with socket.socket(socket.AF_UNIX) as sock:
    sock.connect(path)
    sock.sendall(json.dumps(envelope).encode() + b"\n")
    chunks = []
    while True:
        chunk = sock.recv(4096)
        if not chunk:
            break
        chunks.append(chunk)
response = json.loads(b"".join(chunks).decode())
if not response.get("ok"):
    raise SystemExit(json.dumps(response))
print(json.dumps(response.get("body") or {}, separators=(",", ":")))
PY
cargo build -p lkjmc-cli -p lkjmc-daemon >"$out" 2>&1
LKJMC_TEST_RESET_DATABASE=1 LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL \
    target/debug/lkjmc db reset-test >"$out" 2>&1
LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL target/debug/lkjmc db migrate >"$out" 2>&1
target/debug/lkjmc-daemon --socket "$socket" --http none \
    --database-url "$LKJMC_STORE_TEST_DATABASE_URL" \
    --log-root "$work/logs" --data-root "$work/data" >"$work/daemon.log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 1200); do
    [ -S "$socket" ] && break
    sleep 0.1
done
[ -S "$socket" ] || { cat "$work/daemon.log"; exit 1; }
owner='00000000-0000-0000-0000-000000000101'
trusted='00000000-0000-0000-0000-000000000102'
create_body='{"ownerUuid":"'$owner'","ownerName":"Owner","name":"Base","instanceId":"survival","worldName":"world","chunkX":1,"chunkZ":2}'
python3 "$helper" "$socket" claim.create "$create_body" >"$out"
claim_id=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["claimId"])' "$out")
trust_body='{"ownerUuid":"'$owner'","trustedUuid":"'$trusted'","trustedName":"Friend","instanceId":"survival","worldName":"world","chunkX":1,"chunkZ":2}'
python3 "$helper" "$socket" claim.trust "$trust_body" >"$out"
python3 "$helper" "$socket" claim.snapshot '{"instanceId":"survival"}' >"$out"
python3 - "$out" "$claim_id" "$trusted" <<'PY'
import json, sys
body = json.load(open(sys.argv[1])); claim_id, trusted = sys.argv[2], sys.argv[3]
chunks = body.get("chunks", [])
assert len(chunks) == 1, chunks
assert chunks[0]["claimId"] == claim_id, chunks
assert chunks[0]["trusts"][0]["uuid"] == trusted, chunks
PY
untrust_name_body='{"ownerUuid":"'$owner'","trustedUuid":"'$trusted'","name":"Base"}'
python3 "$helper" "$socket" claim.untrust "$untrust_name_body" >"$out"
python3 "$helper" "$socket" claim.snapshot '{"instanceId":"survival"}' >"$out"
python3 - "$out" <<'PY'
import json, sys
chunks = json.load(open(sys.argv[1])).get("chunks", [])
assert chunks and chunks[0].get("trusts") == [], chunks
PY
trust_name_body='{"ownerUuid":"'$owner'","trustedUuid":"'$trusted'","trustedName":"Friend","name":"Base"}'
python3 "$helper" "$socket" claim.trust "$trust_name_body" >"$out"
target/debug/lkjmc --socket "$socket" --json claim list --instance survival >"$out" 2>&1
grep -q "$claim_id" "$out"
target/debug/lkjmc --socket "$socket" --json claim delete "$claim_id" --yes >"$out" 2>&1
python3 "$helper" "$socket" claim.snapshot '{"instanceId":"survival"}' >"$out"
python3 - "$out" <<'PY'
import json, sys
assert json.load(open(sys.argv[1])).get("chunks") == []
PY
printf '%s\n' 'ok claim smoke'
