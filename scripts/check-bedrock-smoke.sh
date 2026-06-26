#!/bin/sh
set -eu
if [ "${LKJMC_BEDROCK_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok check-bedrock-smoke skipped'
    exit 0
fi
host=${LKJMC_BEDROCK_HOST:-127.0.0.1}
port=${LKJMC_BEDROCK_PORT:-19132}
python3 - "$host" "$port" <<'PY'
import socket, sys
host = sys.argv[1]
port = int(sys.argv[2])
magic = bytes.fromhex('00ffff00fefefefefdfdfdfd12345678')
payload = b'\x01' + (0).to_bytes(8, 'big') + magic + (1).to_bytes(8, 'big')
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.settimeout(3)
sock.sendto(payload, (host, port))
try:
    data, _ = sock.recvfrom(2048)
except TimeoutError:
    print(f'error: no Bedrock UDP response from {host}:{port}', file=sys.stderr)
    sys.exit(1)
if not data:
    print('error: empty Bedrock UDP response', file=sys.stderr)
    sys.exit(1)
print('ok check-bedrock-smoke')
PY
