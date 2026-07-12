#!/bin/sh
set -eu
umask 077

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
artifacts=${E_SECURITY_ARTIFACT_ROOT:-"$root/tmp/e-security-20260711"}
container="lkjmc-e-security-$$"
password=$(tr -d '-' </proc/sys/kernel/random/uuid)
mkdir -p "$artifacts"
cleanup() { docker rm -f "$container" >/dev/null 2>&1 || true; }
trap cleanup EXIT HUP INT TERM

printf '%s\n' '$ docker run isolated postgres:16-alpine' >"$artifacts/commands.log"
docker run --detach --rm --name "$container" \
    -e POSTGRES_DB=e_security -e POSTGRES_USER=e_security \
    -e POSTGRES_PASSWORD="$password" -p 127.0.0.1::5432 postgres:16-alpine \
    >"$artifacts/container-id" 2>>"$artifacts/commands.log"
port_line=$(docker port "$container" 5432/tcp)
port=${port_line##*:}
attempt=0
while ! docker exec "$container" pg_isready -U e_security -d e_security \
    >>"$artifacts/postgres-readiness.log" 2>&1
do
    attempt=$((attempt + 1))
    [ "$attempt" -lt 60 ] || exit 1
    sleep 1
done

cd "$root"
printf '%s\n' '$ cargo test -p lkjmc-daemon e_security -- --nocapture' >>"$artifacts/commands.log"
cargo test -p lkjmc-daemon e_security -- --nocapture >"$artifacts/local-test.log" 2>&1
printf '%s\n' '$ cargo test -p lkjmc-daemon e_security_unix_peer_different_uid_docker -- --ignored --nocapture' >>"$artifacts/commands.log"
cargo test -p lkjmc-daemon e_security_unix_peer_different_uid_docker -- --ignored --nocapture \
    >"$artifacts/docker-peer-test.log" 2>&1
printf '%s\n' '$ LKJMC_STORE_TEST_DATABASE_URL=<redacted> cargo test -p lkjmc-daemon e_security_credential_candidates -- --ignored --nocapture' >>"$artifacts/commands.log"
LKJMC_STORE_TEST_DATABASE_URL="postgres://e_security:${password}@127.0.0.1:${port}/e_security" \
    cargo test -p lkjmc-daemon e_security_credential_candidates -- --ignored --nocapture \
    >"$artifacts/database-candidates.log" 2>&1
printf '%s\n' '$ LKJMC_STORE_TEST_DATABASE_URL=<redacted> cargo test -p lkjmc-daemon e_security_reactor_no_block -- --ignored --nocapture' >>"$artifacts/commands.log"
LKJMC_STORE_TEST_DATABASE_URL="postgres://e_security:${password}@127.0.0.1:${port}/e_security" \
    cargo test -p lkjmc-daemon e_security_reactor_no_block -- --ignored --nocapture \
    >"$artifacts/reactor-test.log" 2>&1
printf '%s\n' 'result: passed' >>"$artifacts/commands.log"
