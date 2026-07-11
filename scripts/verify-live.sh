#!/bin/sh
set -eu
ran=""
skipped=""
run_smoke() {
    name=$1
    guard=$2
    shift 2
    value=$(eval "printf '%s' \"\${$guard:-}\"")
    if [ "$value" = "1" ]; then
        "$@"
        ran="${ran}${ran:+,}$name"
    else
        skipped="${skipped}${skipped:+,}$name:$guard"
    fi
}
run_smoke bedrock LKJMC_BEDROCK_SMOKE ./scripts/check-bedrock-smoke.sh
run_smoke discord LKJMC_DISCORD_SMOKE ./scripts/check-discord-smoke.sh
run_smoke kubernetes LKJMC_KUBERNETES_SMOKE ./scripts/check-kubernetes-smoke.sh
printf 'ok verify-live ran=%s skipped=%s\n' "${ran:-none}" "${skipped:-none}"
