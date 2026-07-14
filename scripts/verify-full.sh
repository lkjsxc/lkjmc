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
run_data_workflows() {
    static="all-multiwrites-classified profile-format-safe-complete old-workflows-absent"
    database="transfer-crash-matrix delivery-crash-matrix adventure-crash-matrix fencing-pass schema-cutover-pass"
    if [ -n "$(value LKJMC_STORE_TEST_DATABASE_URL)" ]; then
        run ./scripts/check-data-workflows.py --all
        for probe in $static $database; do record ran "data-workflows/$probe"; done
        return
    fi
    run ./scripts/check-data-workflows.py --all --allow-database-skip
    cat "$log"
    for probe in $static; do record ran "data-workflows/$probe"; done
    for probe in $database; do
        record skipped "data-workflows/$probe:LKJMC_STORE_TEST_DATABASE_URL"
    done
}
run_sync_adoption() {
    probes="all-snapshots-revisioned freshness-bound-pass reconnect-storm-pass request-budget-pass auth-invalidation-pass typed-domains-pass shutdown-clean duplicate-pollers-absent"
    if [ -n "$(value LKJMC_STORE_TEST_DATABASE_URL)" ]; then
        run ./scripts/check-sync-adoption.py --all
        for probe in $probes; do record ran "sync-adoption/$probe"; done
        return
    fi
    for probe in $probes; do
        record skipped "sync-adoption/$probe:LKJMC_STORE_TEST_DATABASE_URL"
    done
}
run_command_lifecycle() {
    if [ -n "$(value LKJMC_STORE_TEST_DATABASE_URL)" ]; then
        run ./scripts/check-command-lifecycle.py --all
        return
    fi
    run ./scripts/check-command-lifecycle.py --all --allow-database-skip
    cat "$log"
    for probe in timeout-outcome-pass duplicate-mutations-pass auth-budget-sql \
        credential-cache-deadline tcp-db-deadline web-db-deadline; do
        record skipped "command-lifecycle/$probe:LKJMC_STORE_TEST_DATABASE_URL"
    done
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
run python3 tests/test_command_lifecycle_checker.py
run python3 tests/test_db_test_isolation.py
run python3 tests/test_data_workflow_checker.py
run python3 tests/test_runtime_adoption_checker.py
run python3 tests/test_sync_adoption_checker.py
run cargo fmt --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run_data_workflows
run ./scripts/check-runtime-adoption.py --all
for probe in runtime-global-mutex-absent cross-instance-hang-pass same-instance-race-pass \
    reconcile-idempotent effect-crash-recovery adapter-capability-pass runtime-load-budget; do
    record ran "runtime-adoption/$probe"
done
run_sync_adoption
run_when_set db-test-isolation LKJMC_STORE_TEST_DATABASE_URL ./scripts/check-db-test-isolation.sh
run_command_lifecycle
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
