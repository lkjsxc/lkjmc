#!/bin/sh
set -eu
log=$(mktemp)
ran=""
skipped=""
cleanup() { rm -f "$log"; }
trap cleanup EXIT
record() {
    case "$1" in
        ran) ran="${ran}${ran:+,}$2" ;;
        skipped) skipped="${skipped}${skipped:+,}$2" ;;
    esac
}
record_many() {
    field=$1
    items=$2
    [ "$items" = none ] && return
    old_ifs=$IFS
    IFS=,
    for item in $items; do record "$field" "$item"; done
    IFS=$old_ifs
}
run() {
    if ! "$@" >"$log" 2>&1; then
        printf 'failed: %s\n' "$*" >&2
        cat "$log"
        return 1
    fi
}
value() { eval "printf '%s' \"\${$1:-}\""; }
run_safe_ops() {
    if ! ./scripts/check-safe-ops.py --all >"$log" 2>&1; then
        printf '%s\n' 'failed: ./scripts/check-safe-ops.py --all' >&2
        cat "$log"
        return 1
    fi
    summary=$(tail -n 1 "$log")
    case "$summary" in
        'ok check-safe-ops ran='*' skipped='*) ;;
        *) printf '%s\n' 'failed: invalid check-safe-ops summary' >&2; return 1 ;;
    esac
    outcomes=${summary#ok check-safe-ops ran=}
    record_many ran "${outcomes%% skipped=*}"
    record_many skipped "${outcomes#* skipped=}"
}
run_when_set() {
    name=$1
    guard=$2
    shift 2
    if [ -n "$(value "$guard")" ]; then
        run "$@"
        record ran "$name"
    else
        record skipped "$name:$guard"
    fi
}
run_when_one() {
    name=$1
    guard=$2
    shift 2
    if [ "$(value "$guard")" = "1" ]; then
        run "$@"
        record ran "$name"
    else
        record skipped "$name:$guard"
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
run ./scripts/check-security-probes.py
record ran security-probes
run_safe_ops
run ./scripts/check-daemon-cli.sh
run ./scripts/check-process-runtime.sh
run_when_set jar-registry LKJMC_STORE_TEST_DATABASE_URL ./scripts/check-jar-registry.sh
if [ -n "$(value LKJMC_STORE_TEST_DATABASE_URL)" ]; then
    if [ "$(value LKJMC_JAR_LIVE_SMOKE)" = "1" ]; then record ran jar-live; else record skipped 'jar-live:LKJMC_JAR_LIVE_SMOKE'; fi
else
    record skipped 'jar-live:LKJMC_STORE_TEST_DATABASE_URL'
fi
run_when_one claim LKJMC_CLAIM_SMOKE ./scripts/check-claim-smoke.sh
run_when_one web LKJMC_WEB_SMOKE ./scripts/check-web-smoke.sh
run_when_one installer LKJMC_INSTALLER_SMOKE ./scripts/check-installer.sh
run_when_one plugin-assets LKJMC_PLUGIN_LIVE_SMOKE ./scripts/check-plugin-assets.sh
run ./gradlew --no-daemon --no-build-cache test shadowJar
run ./scripts/check-jvm-containment.py --artifacts
printf 'ok verify-full ran=%s skipped=%s\n' "${ran:-none}" "${skipped:-none}"
