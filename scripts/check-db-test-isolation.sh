#!/bin/sh
set -eu

if [ -z "${LKJMC_STORE_TEST_DATABASE_URL:-}" ]; then
    printf '%s\n' 'skip db-test-isolation: LKJMC_STORE_TEST_DATABASE_URL is unset'
    exit 0
fi

artifacts=$(mktemp)
cleanup() { rm -f "$artifacts"; }
trap cleanup EXIT HUP INT TERM
cargo test -p lkjmc-daemon --bin lkjmc-daemon --no-run \
    --message-format=json >"$artifacts"
daemon=$(python3 - "$artifacts" "crates/lkjmc-daemon/Cargo.toml" <<'PY'
import json
import sys
from pathlib import Path

metadata = Path(sys.argv[1])
manifest = Path(sys.argv[2]).resolve()
eligible = []
for number, line in enumerate(metadata.read_text(encoding="utf-8").splitlines(), 1):
    try:
        artifact = json.loads(line)
    except json.JSONDecodeError as error:
        raise SystemExit(f"invalid Cargo JSON at line {number}: {error}")
    target = artifact.get("target", {})
    profile = artifact.get("profile", {})
    artifact_manifest = Path(artifact.get("manifest_path", "/missing")).resolve()
    if (artifact.get("reason") == "compiler-artifact"
            and artifact_manifest == manifest
            and target.get("name") == "lkjmc-daemon"
            and target.get("kind") == ["bin"]
            and profile.get("test") is True
            and isinstance(artifact.get("executable"), str)):
        eligible.append(artifact["executable"])
if len(eligible) != 1:
    raise SystemExit(
        f"expected exactly one daemon test harness in Cargo metadata; found {len(eligible)}"
    )
print(eligible[0])
PY
)
if [ ! -f "$daemon" ] || [ ! -x "$daemon" ]; then
    printf 'failed db-test-isolation: metadata executable is unavailable: %s\n' \
        "$daemon" >&2
    exit 1
fi

filters='deadline_route_tests timeout_outcome_pass status_commands_share_bounded_pool'
for filter in $filters; do
    if ! "$daemon" "$filter" --list --format terse | grep -q ': test$'; then
        printf 'failed db-test-isolation: no tests matched filter: %s\n' "$filter" >&2
        exit 1
    fi
done

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
