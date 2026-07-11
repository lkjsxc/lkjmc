#!/bin/sh
set -eu
log=$(mktemp)
cleanup() { rm -f "$log"; }
trap cleanup EXIT
run() {
    if ! "$@" >"$log" 2>&1; then
        printf 'failed: %s\n' "$*" >&2
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
run ./scripts/check-menus.py
run ./scripts/check-config-schema.py
run ./scripts/check-config-examples.py
run python3 tests/lab/test_lab_harness.py
run cargo fmt --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run ./scripts/check-daemon-cli.sh
run ./scripts/check-process-runtime.sh
run ./scripts/check-jar-registry.sh
run ./scripts/check-claim-smoke.sh
run ./scripts/check-installer.sh
run ./scripts/check-plugin-assets.sh
run ./scripts/check-web-smoke.sh
run ./gradlew --no-daemon test shadowJar
printf '%s\n' 'ok verify-full skips=live-smokes'
