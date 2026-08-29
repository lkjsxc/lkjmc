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
run ./scripts/check-bootstrap-docs.py
run ./scripts/check-asset-docs.py
run ./scripts/check-command-docs.py
run ./scripts/check-permissions.py
run ./scripts/check-locales.py
run ./scripts/check-menus.py
run ./scripts/check-jvm-containment.py
run ./scripts/check-config-schema.py
run ./scripts/check-config-examples.py
run python3 tests/lab/test_lab_harness.py
run python3 tests/test_release_identity.py
run python3 tests/test_release_archive.py
run ./scripts/check-installer.sh
run cargo fmt --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
printf '%s\n' 'ok verify-fast skips=db-backed/live-smokes/gradle-shadowJar'
