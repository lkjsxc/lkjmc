#!/bin/sh
set -eu
if [ "${LKJMC_WEB_SMOKE:-}" != "1" ]; then
    printf '%s\n' 'ok web-smoke skipped'
    exit 0
fi
socket=$(mktemp -u "${TMPDIR:-/tmp}/lkjmc-web.XXXXXX.sock")
token_file=$(mktemp "${TMPDIR:-/tmp}/lkjmc-web-token.XXXXXX")
out=$(mktemp "${TMPDIR:-/tmp}/lkjmc-web.XXXXXX.out")
log=$(mktemp "${TMPDIR:-/tmp}/lkjmc-web.XXXXXX.log")
addr="127.0.0.1:18765"
printf '%s\n' 'web-smoke-token' >"$token_file"
cleanup() {
    if [ "${daemon_pid:-}" ]; then
        kill "$daemon_pid" 2>/dev/null || true
        wait "$daemon_pid" 2>/dev/null || true
    fi
    rm -f "$socket" "$token_file" "$out" "$log"
}
trap cleanup EXIT
cargo build -p lkjmc-daemon >"$out" 2>&1
target/debug/lkjmc-daemon --socket "$socket" --http "$addr" --http-token-file "$token_file" >"$log" 2>&1 &
daemon_pid=$!
for _ in $(seq 1 1200); do
    python3 - "$addr" >/dev/null 2>&1 <<'PY' && break || true
import socket,sys
host,port=sys.argv[1].split(':')
s=socket.create_connection((host,int(port)),0.1); s.close()
PY
    sleep 0.1
done
python3 - "$addr" >"$out" <<'PY'
import http.client,re,sys,urllib.parse
host,port=sys.argv[1].split(':')
conn=http.client.HTTPConnection(host,int(port),timeout=2)
conn.request('GET','/web')
assert conn.getresponse().status == 403
conn=http.client.HTTPConnection(host,int(port),timeout=2)
conn.request('GET','/web',headers={'Authorization':'Bearer web-smoke-token'})
resp=conn.getresponse(); body=resp.read().decode()
assert resp.status == 200, resp.status
assert 'Status' in body and 'Authorization' not in body
login_body=urllib.parse.urlencode({'password':'web-smoke-token'})
conn=http.client.HTTPConnection(host,int(port),timeout=2)
conn.request('POST','/web/login',body=login_body,headers={'Content-Type':'application/x-www-form-urlencoded'})
resp=conn.getresponse(); body=resp.read().decode()
assert resp.status == 200, resp.status
cookie=resp.getheader('set-cookie').split(';',1)[0]
csrf=re.search(r'name=_csrf value="([^"]+)"', body).group(1)
conn=http.client.HTTPConnection(host,int(port),timeout=2)
conn.request('GET','/web',headers={'Cookie':cookie})
resp=conn.getresponse(); body=resp.read().decode()
assert resp.status == 200 and 'Status' in body
conn=http.client.HTTPConnection(host,int(port),timeout=2)
conn.request('POST','/web/logout',headers={'Cookie':cookie})
assert conn.getresponse().status == 403
logout_body=urllib.parse.urlencode({'_csrf':csrf})
conn=http.client.HTTPConnection(host,int(port),timeout=2)
conn.request('POST','/web/logout',body=logout_body,headers={'Cookie':cookie})
assert conn.getresponse().status == 200
print('ok web-smoke')
PY
cat "$out"
