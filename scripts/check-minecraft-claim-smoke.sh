#!/usr/bin/env bash
set -euo pipefail

if [ "${LKJMC_MINECRAFT_CLAIM_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok minecraft claim smoke skipped'
    exit 0
fi

printf '%s\n' 'blocked: Java claim adapter smoke is withdrawn pending trusted identity/session attestation' >&2
exit 1
