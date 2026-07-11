#!/usr/bin/env bash
set -euo pipefail

if [ "${LKJMC_PLAYABLE_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok playable smoke skipped'
    exit 0
fi

printf '%s\n' 'blocked: no bounded local-safe Java protocol smoke is currently shipped' >&2
exit 1
