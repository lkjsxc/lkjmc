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
cleanup() {
    for pid in "$paper_pid" "$velocity_pid"; do
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

./gradlew --no-daemon shadowJar >/dev/null
mkdir -p "$work/paper/plugins" "$work/velocity/plugins"
cp platforms/jvm/paper/build/libs/paper-*-all.jar "$work/paper/plugins/lkjmc-paper.jar"
cp platforms/jvm/velocity/build/libs/velocity-*-all.jar "$work/velocity/plugins/lkjmc-velocity.jar"

paper_url=${LKJMC_PAPER_JAR_URL:-$(resolve_url paper "${LKJMC_PAPER_VERSION:-}")}
velocity_url=${LKJMC_VELOCITY_JAR_URL:-$(resolve_url velocity "${LKJMC_VELOCITY_VERSION:-}")}
download "$paper_url" "$work/paper.jar"
download "$velocity_url" "$work/velocity.jar"

cat >"$work/paper/eula.txt" <<'EOF'
eula=true
EOF
cat >"$work/paper/server.properties" <<EOF
online-mode=false
server-port=${LKJMC_SMOKE_PAPER_PORT:-25566}
EOF
( cd "$work/paper" && java -Xms128M -Xmx512M -jar "$work/paper.jar" nogui ) >"$work/paper.log" 2>&1 &
paper_pid=$!
wait_for_log "$work/paper.log" 'lkjmc Paper plugin enabled' "$paper_pid" paper

( cd "$work/velocity" && java -Xms128M -Xmx512M -jar "$work/velocity.jar" ) >"$work/velocity.log" 2>&1 &
velocity_pid=$!
wait_for_log "$work/velocity.log" 'lkjmc Velocity plugin enabled' "$velocity_pid" velocity

printf '%s\n' 'ok minecraft smoke'
