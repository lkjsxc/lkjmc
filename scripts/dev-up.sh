#!/bin/sh
set -eu
accept=0
for value in "$@"; do
    [ "$value" = "--accept-minecraft-eula" ] && accept=1
    [ "$value" = "--bedrock-enabled" ] && export LKJMC_PLAYABLE_BEDROCK=enabled
    [ "$value" = "--bedrock-disabled" ] && export LKJMC_PLAYABLE_BEDROCK=disabled
done
[ "$accept" = "1" ] || {
    printf '%s\n' 'error: pass --accept-minecraft-eula to start a playable Minecraft server' >&2
    exit 1
}
export LKJMC_ACCEPT_MINECRAFT_EULA=1
exec docker compose --profile playable up --build playable
