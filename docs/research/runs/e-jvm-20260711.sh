#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd -P)
tmp=$(CDPATH= cd -- "${TMPDIR:-/tmp}" && pwd -P)
project="$root/tests/research/e-jvm-20260711"
usage() {
    printf '%s\n' "usage: $0 run [--raw-dir DIR] | replay --raw-dir DIR | cleanup --raw-dir DIR" >&2
    exit 2
}
valid_root() {
    case "$1" in "$tmp"/lkjmc-e-jvm-*) ;; *) return 1 ;; esac
    [ "${1%/*}" = "$tmp" ] && [ ! -L "$1" ]
}
raw_option() {
    [ "${1:-}" = "--raw-dir" ] && [ -n "${2:-}" ] || usage
    printf '%s\n' "$2"
}
run() {
    if [ "$#" -eq 0 ]; then
        raw=$(mktemp -d "$tmp/lkjmc-e-jvm-XXXXXXXX")
    elif [ "$#" -eq 2 ]; then
        raw=$(raw_option "$@")
        valid_root "$raw" && [ ! -e "$raw" ] || { printf '%s\n' 'unsafe raw root' >&2; exit 2; }
        mkdir -- "$raw"
    else
        usage
    fi
    printf '%s\n' "./gradlew --no-daemon --no-build-cache -p $project test run --args $raw/result.json" \
        >"$raw/command.txt"
    java -version >"$raw/java-version.log" 2>&1
    if "$root/gradlew" --no-daemon --no-build-cache -p "$project" test run \
            --args "$raw/result.json" >"$raw/gradle.log" 2>&1; then
        result=PASS
    else
        result=FAIL
    fi
    [ -f "$raw/result.json" ] || printf '{"result":"%s"}\n' "$result" >"$raw/result.json"
    printf '{"result":"%s"}\n' "$result" >"$raw/outcome.json"
    sha256sum "$raw/command.txt" "$raw/java-version.log" "$raw/gradle.log" \
        "$raw/outcome.json" "$raw/result.json" >"$raw/index.sha256"
    printf 'E-JVM result=%s raw=%s replay=%s replay --raw-dir %s\n' \
        "$result" "$raw" "$0" "$raw"
    [ "$result" = PASS ]
}
replay() {
    raw=$(raw_option "$@")
    valid_root "$raw" && [ -f "$raw/index.sha256" ] || { printf '%s\n' 'unsafe or missing raw root' >&2; exit 2; }
    (cd / && sha256sum -c "$raw/index.sha256")
    printf 'E-JVM replay=PASS raw=%s\n' "$raw"
}
cleanup() {
    raw=$(raw_option "$@")
    valid_root "$raw" && [ -d "$raw" ] || { printf '%s\n' 'unsafe or missing raw root' >&2; exit 2; }
    rm -rf -- "$raw"
    printf 'E-JVM cleanup=PASS raw=%s\n' "$raw"
}
[ "$#" -ge 1 ] || usage
command=$1
shift
case "$command" in
    run) run "$@" ;;
    replay) replay "$@" ;;
    cleanup) cleanup "$@" ;;
    *) usage ;;
esac
