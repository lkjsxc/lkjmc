#!/usr/bin/env bash
set -euo pipefail

if [ "${1:-}" != --output ] || [ -z "${2:-}" ] || [ "$#" -ne 2 ]; then
    printf '%s\n' "usage: $0 --output IGNORED_DIRECTORY" >&2
    exit 64
fi
if ! command -v docker >/dev/null 2>&1; then
    printf '%s\n' 'BLOCKED: docker is not installed' >&2
    exit 2
fi

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
output=$2
project="lkjmc_e_hc_control_$$"
log="$output/compose.log"
kube="$output/kubernetes.txt"
mkdir -p "$output"
: >"$log"

compose() { docker compose -f "$root/compose.yml" -p "$project" "$@"; }
die() { printf '%s\n' "FAILED: $*" >&2; exit 1; }
cleanup() { compose down --volumes --remove-orphans >>"$log" 2>&1 || true; }
trap cleanup EXIT

run_compose() {
    printf '+ docker compose %s\n' "$*" >>"$log"
    compose "$@" >>"$log" 2>&1 || die "compose $*"
}
pg() {
    pg_result=$(compose exec -T postgres psql -X -v ON_ERROR_STOP=1 -U lab -d lab \
        -Atqc "$1" 2>>"$log") || die 'PostgreSQL command failed'
    printf '%s\n' "$pg_result" >>"$log"
}
pg_expect() {
    expected=$1
    pg "$2"
    [ "$pg_result" = "$expected" ] || die "PostgreSQL expected $expected, got $pg_result"
}
pg_exec() {
    compose exec -T postgres psql -X -v ON_ERROR_STOP=1 -U lab -d lab \
        -qc "$1" >>"$log" 2>&1 || die 'PostgreSQL statement failed'
}
redis() {
    redis_result=$(compose exec -T redis redis-cli --raw "$@" 2>>"$log") \
        || die 'Redis command failed'
    printf '%s\n' "$redis_result" >>"$log"
}
redis_expect() {
    expected=$1
    shift
    redis "$@"
    [ "$redis_result" = "$expected" ] || die "Redis expected $expected, got $redis_result"
}
redis_first_expect() {
    expected=$1
    shift
    redis "$@"
    first=${redis_result%%$'\n'*}
    [ "$first" = "$expected" ] || die "Redis expected first $expected, got $first"
}
run_daemon() {
    printf '+ daemon %s\n' "$*" >>"$log"
    compose run --rm --no-deps "$@" >>"$log" 2>&1 || die "daemon $*"
}

run_compose version
run_compose up -d postgres redis
ready=0
for _ in $(seq 1 30); do
    if compose exec -T postgres psql -X -v ON_ERROR_STOP=1 -U lab -d lab \
        -Atqc 'SELECT 1' >>"$log" 2>&1; then
        ready=1
        break
    fi
    sleep 1
done
[ "$ready" = 1 ] || die 'PostgreSQL did not become healthy'
compose exec -T postgres psql -X -v ON_ERROR_STOP=1 -U lab -d lab <"$root/lab.sql" \
    >>"$log" 2>&1 || die 'lab schema setup'

for event in \
    'event-alpha-1 instance-alpha requested' \
    'event-alpha-2 instance-alpha starting' \
    'event-alpha-3 instance-alpha running' \
    'event-alpha-4 instance-alpha running' \
    'event-beta-1 instance-beta requested' \
    'event-beta-2 instance-beta starting' \
    'event-beta-3 instance-beta stopping' \
    'event-beta-4 instance-beta stopped'; do
    read -r event_id stream state <<<"$event"
    pg_expect t "SELECT lab.append_event('$event_id', '$stream', jsonb_build_object('state', '$state'));"
done
pg_expect f "SELECT lab.append_event('event-alpha-3', 'instance-alpha', jsonb_build_object('state', 'duplicate'));"
pg_exec 'SELECT lab.rebuild_projection();'
pg_expect 8 'SELECT count(*) FROM lab.event_log;'
pg_expect $'instance-alpha:running:4\ninstance-beta:stopped:4' \
    "SELECT stream || ':' || state || ':' || revision FROM lab.projection ORDER BY stream;"

run_daemon daemon-a acquire 1
run_daemon daemon-b acquire REJECT
sleep 11
run_daemon daemon-b acquire 2
run_daemon daemon-a write-stale f
run_daemon daemon-b write-fresh t
pg_expect fresh:2 "SELECT state || ':' || generation FROM lab.fenced_instance;"

pg_exec "INSERT INTO lab.outbox (event_id, channel, payload)
    SELECT 'simple-' || value, 'simple', jsonb_build_object('sequence', value)
    FROM generate_series(1, 12) AS value;"
pg_expect 12 "WITH claimed AS (
    UPDATE lab.outbox SET simple_claimed = true WHERE channel = 'simple' RETURNING 1
) SELECT count(*) FROM claimed;"
pg_expect 12 "SELECT count(*) FROM lab.outbox WHERE channel = 'simple' AND simple_claimed;"
pg_exec "INSERT INTO lab.outbox (event_id, channel, payload)
    SELECT 'broker-' || value, 'broker', jsonb_build_object('sequence', value)
    FROM generate_series(1, 12) AS value;"
redis_expect OK XGROUP CREATE lkjmc-control-feed lkjmc-control-consumers 0 MKSTREAM
{
    for number in $(seq 1 12); do
        printf 'XADD lkjmc-control-feed %s-0 event-id broker-%s payload observed\n' "$number" "$number"
    done
} | compose exec -T redis redis-cli --pipe >>"$log" 2>&1 || die 'Redis bridge publish'
redis_expect 12 XLEN lkjmc-control-feed
redis XREADGROUP GROUP lkjmc-control-consumers probe COUNT 12 STREAMS lkjmc-control-feed '>'
redis_first_expect 12 XPENDING lkjmc-control-feed lkjmc-control-consumers
ids=()
for number in $(seq 1 12); do ids+=("$number-0"); done
redis_expect 12 XACK lkjmc-control-feed lkjmc-control-consumers "${ids[@]}"
redis_first_expect 0 XPENDING lkjmc-control-feed lkjmc-control-consumers
pg_expect 12 "WITH published AS (
    UPDATE lab.outbox SET broker_published = true WHERE channel = 'broker' RETURNING 1
) SELECT count(*) FROM published;"
pg_exec "INSERT INTO lab.outbox (event_id, channel, payload)
    VALUES ('broker-gap', 'broker', jsonb_build_object('fault', 'before-publish'));"
pg_expect 1 "SELECT count(*) FROM lab.outbox
    WHERE event_id = 'broker-gap' AND NOT broker_published;"
pg_expect 1 "WITH recovered AS (
    UPDATE lab.outbox SET simple_claimed = true WHERE event_id = 'broker-gap' RETURNING 1
) SELECT count(*) FROM recovered;"

: >"$kube"
"$root/kubernetes.sh" "$kube" >>"$log" 2>&1
{
    printf 'E-HC-CONTROL Compose result: PASS\n'
    printf 'event rows=8; duplicate rejected; projections=2\n'
    printf 'lease generations=1,2; stale write=false; fresh write=true\n'
    printf 'simple outbox claimed=12; Redis stream/read/ack=12; gap recovered=1\n'
    printf 'Kubernetes attempt is recorded in kubernetes.txt\n'
} >"$output/result.txt"
sha256sum "$log" "$kube" "$output/result.txt" >"$output/sha256.txt"
printf '%s\n' "PASS: raw evidence retained at $output"
