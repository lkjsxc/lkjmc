#!/bin/sh
set -eu

if [ -z "${LKJMC_STORE_TEST_DATABASE_URL:-}" ]; then
    printf '%s\n' 'skip db-test-isolation: LKJMC_STORE_TEST_DATABASE_URL is unset'
    exit 0
fi

cargo test -p lkjmc-daemon --bin lkjmc-daemon --no-run >/dev/null
daemon=$(find target/debug/deps -maxdepth 1 -type f -executable -name 'lkjmc_daemon-*' -print -quit)
test -n "$daemon"

run_daemon() {
    "$daemon" "$1" --test-threads=4
}

for attempt in 1 2; do
    printf 'db-test-isolation attempt=%s\n' "$attempt"
    run_daemon deadline_route_tests &
    deadline=$!
    run_daemon timeout_outcome_pass &
    timeout=$!
    run_daemon status_commands_share_bounded_pool &
    pool=$!
    status=0
    wait "$deadline" || status=1
    wait "$timeout" || status=1
    wait "$pool" || status=1
    [ "$status" -eq 0 ]
    cargo test -p lkjmc-store --tests -- --test-threads=4
done
