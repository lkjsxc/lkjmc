#!/usr/bin/env bash
set -euo pipefail

if [[ "${LKJMC_DISCORD_SMOKE:-}" != "1" ]]; then
  echo "skipped discord smoke: set LKJMC_DISCORD_SMOKE=1"
  exit 0
fi
: "${LKJMC_DISCORD_CONFIG:?set LKJMC_DISCORD_CONFIG to a JSON config path}"
cargo run -p lkjmc-discord -- "$LKJMC_DISCORD_CONFIG" --daemon-status
