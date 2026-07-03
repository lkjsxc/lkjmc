#!/usr/bin/env bash
set -euo pipefail

if [ "${LKJMC_MINECRAFT_CLAIM_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok minecraft claim smoke skipped'
    exit 0
fi

command -v java >/dev/null 2>&1 || { echo 'java is required' >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo 'python3 is required' >&2; exit 1; }
[ -n "${LKJMC_STORE_TEST_DATABASE_URL:-}" ] || {
    echo 'LKJMC_STORE_TEST_DATABASE_URL is required for minecraft claim smoke' >&2
    exit 1
}

work=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-mc-claim.XXXXXX")
paper_pid=''
daemon_pid=''
cleanup() {
    for pid in "$paper_pid" "$daemon_pid"; do
        [ -n "$pid" ] && kill "$pid" 2>/dev/null || true
        [ -n "$pid" ] && wait "$pid" 2>/dev/null || true
    done
    rm -rf "$work" 2>/dev/null || true
}
trap cleanup EXIT

resolve_paper_url() {
    python3 - "$1" <<'PY'
import json, sys, urllib.request
version = sys.argv[1]
base = "https://api.papermc.io/v2/projects/paper"
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
    python3 - "$1" "$2" <<'PY'
import shutil, sys, urllib.request
with urllib.request.urlopen(sys.argv[1], timeout=120) as response, open(sys.argv[2], "wb") as out:
    shutil.copyfileobj(response, out)
PY
}

pick_port() {
    python3 - <<'PY'
import socket
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}

wait_for_log() {
    log=$1
    pattern=$2
    pid=$3
    name=$4
    for _ in $(seq 1 300); do
        grep -q "$pattern" "$log" 2>/dev/null && return 0
        kill -0 "$pid" 2>/dev/null || { cat "$log"; echo "$name exited" >&2; exit 1; }
        sleep 0.5
    done
    cat "$log"
    echo "$name did not report $pattern" >&2
    exit 1
}

wait_for_socket() {
    socket=$1
    for _ in $(seq 1 1200); do
        [ -S "$socket" ] && return 0
        sleep 0.1
    done
    cat "$work/daemon.log"
    echo 'daemon socket did not appear' >&2
    exit 1
}

wait_for_http() {
    port=$1
    token=$2
    for _ in $(seq 1 100); do
        python3 - "$port" "$token" >/dev/null 2>&1 <<'PY' && return 0
import json, sys, urllib.request, uuid
body = json.dumps({"requestId": str(uuid.uuid4()), "actor": {"kind": "cli", "name": "smoke"}, "command": "doctor", "body": {}}).encode()
req = urllib.request.Request(f"http://127.0.0.1:{sys.argv[1]}", data=body, headers={"Authorization": f"Bearer {sys.argv[2]}", "Content-Type": "application/json"})
with urllib.request.urlopen(req, timeout=1) as response:
    raise SystemExit(0 if json.load(response).get("ok") else 1)
PY
        sleep 0.1
    done
    cat "$work/daemon.log"
    echo 'daemon HTTP did not become ready' >&2
    exit 1
}

run_protocol_smoke() {
    protocol_work="$work/protocol-smoke"
    mkdir -p "$protocol_work/src/main/java/com/lkjmc/smoke"
    cp tests/smoke/claim_protocol/MinecraftClaimProtocolSmoke.java \
        "$protocol_work/src/main/java/com/lkjmc/smoke/"
    cat >"$protocol_work/settings.gradle.kts" <<'EOF'
pluginManagement { repositories { gradlePluginPortal(); mavenCentral() } }
dependencyResolutionManagement { repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS); repositories { mavenCentral(); maven("https://repo.opencollab.dev/maven-releases") } }
rootProject.name = "lkjmc-protocol-smoke"
EOF
    cat >"$protocol_work/build.gradle.kts" <<'EOF'
plugins { application }
java { toolchain.languageVersion.set(JavaLanguageVersion.of(21)) }
application { mainClass.set("com.lkjmc.smoke.MinecraftClaimProtocolSmoke") }
dependencies {
    implementation("org.geysermc.mcprotocollib:protocol:1.21.7-1")
    implementation("net.kyori:adventure-text-serializer-plain:4.17.0")
}
EOF
    ./gradlew --no-daemon -q -p "$protocol_work" run --args="127.0.0.1 $paper_port"
}

./gradlew --no-daemon shadowJar >/dev/null
mkdir -p "$work/paper/plugins"
cp platforms/jvm/paper/build/libs/paper-*-all.jar "$work/paper/plugins/lkjmc-paper.jar"
protocol_smoke=${LKJMC_MINECRAFT_CLAIM_PROTOCOL_SMOKE:-0}
paper_version=${LKJMC_PAPER_VERSION:-}
if [ "$protocol_smoke" = "1" ] && [ -z "${LKJMC_PAPER_JAR_URL:-}" ]; then
    paper_version=${paper_version:-1.21.7}
fi
paper_url=${LKJMC_PAPER_JAR_URL:-$(resolve_paper_url "$paper_version")}
download "$paper_url" "$work/paper.jar"
LKJMC_TEST_RESET_DATABASE=1 LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL \
    cargo run -p lkjmc-cli -- db reset-test >/dev/null
LKJMC_DATABASE_URL=$LKJMC_STORE_TEST_DATABASE_URL cargo run -p lkjmc-cli -- db migrate >/dev/null
socket=$(mktemp -u "$work/lkjmc-daemon.XXXXXX.sock")
daemon_port=${LKJMC_SMOKE_DAEMON_PORT:-$(pick_port)}
daemon_token=${LKJMC_SMOKE_DAEMON_TOKEN:-minecraft-claim-smoke}
cargo run -p lkjmc-daemon -- --socket "$socket" --http "127.0.0.1:$daemon_port" \
    --http-token "$daemon_token" --database-url "$LKJMC_STORE_TEST_DATABASE_URL" \
    --log-root "$work/logs" --data-root "$work/data" >"$work/daemon.log" 2>&1 &
daemon_pid=$!
wait_for_socket "$socket"
wait_for_http "$daemon_port" "$daemon_token"
paper_port=${LKJMC_SMOKE_PAPER_PORT:-$(pick_port)}
cat >"$work/paper/eula.txt" <<'EOF'
eula=true
EOF
if [ "$protocol_smoke" = "1" ]; then
    owner_uuid=$(scripts/minecraft_login_probe.py offline-uuid ClaimOwner)
    printf '[{"uuid":"%s","name":"ClaimOwner","level":4,"bypassesPlayerLimit":false}]\n' \
        "$owner_uuid" >"$work/paper/ops.json"
fi
cat >"$work/paper/server.properties" <<EOF
online-mode=false
server-port=$paper_port
spawn-protection=0
gamemode=creative
force-gamemode=true
EOF
paper_env=(env LKJMC_INSTANCE_ID=paper LKJMC_PAPER_CLAIM_SMOKE=1 \
    LKJMC_DAEMON_HTTP_URL="http://127.0.0.1:$daemon_port" \
    LKJMC_DAEMON_HTTP_TOKEN="$daemon_token")
if [ "$protocol_smoke" = "1" ]; then
    paper_env+=(LKJMC_CLAIM_PROTOCOL_SMOKE=1)
fi
( cd "$work/paper" && "${paper_env[@]}" java -Xms128M -Xmx512M -jar "$work/paper.jar" nogui ) >"$work/paper.log" 2>&1 &
paper_pid=$!
wait_for_log "$work/paper.log" 'lkjmc Paper plugin enabled' "$paper_pid" paper
wait_for_log "$work/paper.log" 'lkjmc claim smoke passed' "$paper_pid" paper
if [ "$protocol_smoke" = "1" ]; then
    run_protocol_smoke || { cat "$work/paper.log"; cat "$work/daemon.log"; exit 1; }
    wait_for_log "$work/paper.log" 'lkjmc claim protocol denied break' "$paper_pid" paper
    wait_for_log "$work/paper.log" 'lkjmc claim protocol denied place' "$paper_pid" paper
    printf '%s\n' 'ok minecraft claim protocol smoke'
else
    printf '%s\n' 'ok minecraft claim smoke'
fi
