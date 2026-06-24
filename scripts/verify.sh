#!/bin/sh
set -eu
log=$(mktemp)
cleanup() {
    rm -f "$log"
}
trap cleanup EXIT
run() {
    if ! "$@" >"$log" 2>&1; then
        cat "$log"
        return 1
    fi
}
run ./scripts/check-lines.py
run ./scripts/check-docs.py
run cargo fmt --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run ./scripts/check-daemon-cli.sh
run ./scripts/check-process-runtime.sh
run ./scripts/check-jar-registry.sh
run ./scripts/check-installer.sh
run ./gradlew --no-daemon test
printf '%s\n' 'ok verify'
