#!/bin/sh
set -eu
if [ "${LKJMC_PLAYABLE_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok check-playable-smoke skipped'
    exit 0
fi
[ "${LKJMC_ACCEPT_MINECRAFT_EULA:-0}" = "1" ] || {
    printf '%s\n' 'error: set LKJMC_ACCEPT_MINECRAFT_EULA=1 for playable smoke' >&2
    exit 1
}
command -v docker >/dev/null 2>&1 || {
    printf '%s\n' 'error: docker is required for playable smoke' >&2
    exit 1
}
if command -v timeout >/dev/null 2>&1; then
    timeout 1200 docker compose -f docker-compose.yml -f docker-compose.playable.yml up --build --abort-on-container-exit playable
else
    docker compose -f docker-compose.yml -f docker-compose.playable.yml up --build --abort-on-container-exit playable
fi
