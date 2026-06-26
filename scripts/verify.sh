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
run ./scripts/check-bootstrap-docs.py
run ./scripts/check-asset-docs.py
run ./scripts/check-command-docs.py
run ./scripts/check-permissions.py
run ./scripts/check-locales.py
run cargo fmt --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run ./scripts/check-daemon-cli.sh
run ./scripts/check-process-runtime.sh
run ./scripts/check-jar-registry.sh
run ./scripts/check-claim-smoke.sh
run ./scripts/check-installer.sh
run ./scripts/check-minecraft-smoke.sh
run ./scripts/check-minecraft-claim-smoke.sh
run ./gradlew --no-daemon test shadowJar
printf '%s\n' 'ok verify'
