#!/usr/bin/env bash
set -euo pipefail

if [ "${LKJMC_MINECRAFT_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok minecraft smoke skipped'
    exit 0
fi

command -v java >/dev/null 2>&1 || { echo 'java is required' >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo 'python3 is required' >&2; exit 1; }

work=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-mc-smoke.XXXXXX")
paper_pid=''
velocity_pid=''
daemon_pid=''
cleanup() {
    for pid in "$paper_pid" "$velocity_pid" "$daemon_pid"; do
        [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
        [ -n "$pid" ] && wait "$pid" 2>/dev/null || true
    done
    rm -rf "$work"
}
trap cleanup EXIT

resolve_url() {
    project=$1
    version=${2:-}
    python3 - "$project" "$version" <<'PY'
import json, os, sys, urllib.request
project, version = sys.argv[1], sys.argv[2]
base = f"https://api.papermc.io/v2/projects/{project}"
with urllib.request.urlopen(base, timeout=30) as response:
    meta = json.load(response)
version = version or meta["versions"][-1]
with urllib.request.urlopen(f"{base}/versions/{version}/builds", timeout=30) as response:
    builds = json.load(response)["builds"]
build = builds[-1]
name = build["downloads"]["application"]["name"]
print(f"{base}/versions/{version}/builds/{build['build']}/downloads/{name}")
PY
}

download() {
    url=$1
    target=$2
    python3 - "$url" "$target" <<'PY'
import shutil, sys, urllib.request
url, target = sys.argv[1], sys.argv[2]
with urllib.request.urlopen(url, timeout=120) as response, open(target, "wb") as out:
    shutil.copyfileobj(response, out)
PY
}

wait_for_log() {
    log=$1
    pattern=$2
    pid=$3
    name=$4
    for _ in $(seq 1 240); do
        grep -q "$pattern" "$log" 2>/dev/null && return 0
        kill -0 "$pid" 2>/dev/null || { cat "$log"; echo "$name exited" >&2; exit 1; }
        sleep 0.5
    done
    cat "$log"
    echo "$name did not report ready" >&2
    exit 1
}

pick_port() {
    python3 - <<'PY'
import socket
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}

wait_for_socket() {
    socket=$1
    log=$2
    for _ in $(seq 1 100); do
        [ -S "$socket" ] && return 0
        sleep 0.1
    done
    cat "$log"
    echo 'daemon socket did not appear' >&2
    exit 1
}

wait_for_http() {
    port=$1
    token=$2
    for _ in $(seq 1 100); do
        python3 - "$port" "$token" >/dev/null 2>&1 <<'PY' && return 0
import json, sys, urllib.request, uuid
port, token = sys.argv[1], sys.argv[2]
body = json.dumps({"requestId": str(uuid.uuid4()), "actor": {"kind": "cli", "name": "smoke"}, "command": "doctor", "body": {}}).encode()
req = urllib.request.Request(f"http://127.0.0.1:{port}", data=body, headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"})
with urllib.request.urlopen(req, timeout=1) as response:
    raise SystemExit(0 if json.load(response).get("ok") else 1)
PY
        sleep 0.1
    done
    echo 'daemon HTTP did not become ready' >&2
    exit 1
}

./gradlew --no-daemon shadowJar >/dev/null
mkdir -p "$work/paper/plugins" "$work/velocity/plugins"
cp platforms/jvm/paper/build/libs/paper-*-all.jar "$work/paper/plugins/lkjmc-paper.jar"
cp platforms/jvm/velocity/build/libs/velocity-*-all.jar "$work/velocity/plugins/lkjmc-velocity.jar"

paper_url=${LKJMC_PAPER_JAR_URL:-$(resolve_url paper "${LKJMC_PAPER_VERSION:-}")}
velocity_url=${LKJMC_VELOCITY_JAR_URL:-$(resolve_url velocity "${LKJMC_VELOCITY_VERSION:-}")}
download "$paper_url" "$work/paper.jar"
download "$velocity_url" "$work/velocity.jar"

paper_port=${LKJMC_SMOKE_PAPER_PORT:-$(pick_port)}
velocity_port=${LKJMC_SMOKE_VELOCITY_PORT:-$(pick_port)}
daemon_http_port=${LKJMC_SMOKE_DAEMON_PORT:-$(pick_port)}
daemon_token=${LKJMC_SMOKE_DAEMON_TOKEN:-minecraft-smoke}
paper_env=(env LKJMC_INSTANCE_ID=paper)
velocity_env=(env)
if [ "${LKJMC_MINECRAFT_PLAYER_SMOKE:-0}" = "1" ]; then
    [ -n "${LKJMC_STORE_TEST_DATABASE_URL:-}" ] || {
        echo 'LKJMC_STORE_TEST_DATABASE_URL is required for player smoke' >&2
        exit 1
    }
    socket=$(mktemp -u "$work/lkjmc-daemon.XXXXXX.sock")
    LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL cargo run -p lkjmc-cli -- db migrate >/dev/null
    cargo run -p lkjmc-daemon -- --socket "$socket" --http "127.0.0.1:$daemon_http_port" \
        --http-token "$daemon_token" --database-url "$LKJMC_STORE_TEST_DATABASE_URL" \
        --log-root "$work/logs" --data-root "$work/data" >"$work/daemon.log" 2>&1 &
    daemon_pid=$!
    wait_for_socket "$socket" "$work/daemon.log"
    wait_for_http "$daemon_http_port" "$daemon_token"
    smoke_uuid=$(scripts/minecraft_login_probe.py offline-uuid SmokeBanned)
    cargo run -p lkjmc-cli -- --socket "$socket" moderation ban \
        "$smoke_uuid" SmokeBanned --reason smoke-ban >/dev/null
    paper_env=(env LKJMC_INSTANCE_ID=paper LKJMC_DAEMON_HTTP_URL="http://127.0.0.1:$daemon_http_port" \
        LKJMC_DAEMON_HTTP_TOKEN="$daemon_token")
    velocity_env=(env LKJMC_DAEMON_HTTP_URL="http://127.0.0.1:$daemon_http_port" \
        LKJMC_DAEMON_HTTP_TOKEN="$daemon_token")
fi

cat >"$work/paper/eula.txt" <<'EOF'
eula=true
EOF
cat >"$work/paper/server.properties" <<EOF
online-mode=false
server-port=$paper_port
EOF
cat >"$work/velocity/velocity.toml" <<EOF
config-version = "2.7"
bind = "127.0.0.1:$velocity_port"
motd = "lkjmc smoke"
show-max-players = 20
login-ratelimit = 0
online-mode = false
force-key-authentication = false
player-info-forwarding-mode = "none"
forwarding-secret = "minecraft-smoke"
ping-passthrough = "disabled"
[servers]
paper = "127.0.0.1:$paper_port"
hub = "127.0.0.1:$paper_port"
try = ["paper"]
[forced-hosts]
EOF
( cd "$work/paper" && "${paper_env[@]}" java -Xms128M -Xmx512M -jar "$work/paper.jar" nogui ) >"$work/paper.log" 2>&1 &
paper_pid=$!
wait_for_log "$work/paper.log" 'lkjmc Paper plugin enabled' "$paper_pid" paper

( cd "$work/velocity" && "${velocity_env[@]}" java -Xms128M -Xmx512M -jar "$work/velocity.jar" ) >"$work/velocity.log" 2>&1 &
velocity_pid=$!
wait_for_log "$work/velocity.log" 'lkjmc Velocity plugin enabled' "$velocity_pid" velocity

if [ "${LKJMC_MINECRAFT_PLAYER_SMOKE:-0}" = "1" ]; then
    protocol=${LKJMC_SMOKE_PROTOCOL:-$(scripts/minecraft_login_probe.py status 127.0.0.1 "$velocity_port")}
    echo "minecraft protocol $protocol"
    allowed_uuid=$(scripts/minecraft_login_probe.py offline-uuid SmokeAllowed)
    if ! scripts/minecraft_login_probe.py accept 127.0.0.1 "$velocity_port" "$protocol" \
        SmokeAllowed "$allowed_uuid"; then
        cat "$work/velocity.log"
        cat "$work/daemon.log"
        exit 1
    fi
    sleep 5
    if ! scripts/minecraft_login_probe.py deny 127.0.0.1 "$velocity_port" "$protocol" \
        SmokeBanned "$smoke_uuid" smoke-ban; then
        cat "$work/velocity.log"
        cat "$work/daemon.log"
        exit 1
    fi
fi

printf '%s\n' 'ok minecraft smoke'
