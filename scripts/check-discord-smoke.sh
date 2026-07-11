#!/usr/bin/env bash
set -euo pipefail

if [[ "${LKJMC_DISCORD_SMOKE:-}" != "1" ]]; then
  echo "skipped discord smoke: set LKJMC_DISCORD_SMOKE=1"
  exit 0
fi
: "${LKJMC_DISCORD_CONFIG:?set LKJMC_DISCORD_CONFIG to a JSON config path}"
args=("$LKJMC_DISCORD_CONFIG" --check-config)
if [[ "${LKJMC_DISCORD_REGISTER_SMOKE:-}" == "1" ]]; then
  args+=(--register-commands)
fi
cargo run -p lkjmc-discord -- "${args[@]}"
