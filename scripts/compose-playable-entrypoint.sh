#!/bin/sh
set -eu
CONFIG_ROOT=${LKJMC_CONFIG_ROOT:-/etc/lkjmc}
DATA_ROOT=${LKJMC_DATA_ROOT:-/var/lib/lkjmc}
LOG_ROOT=${LKJMC_LOG_ROOT:-/var/log/lkjmc}
INSTALL_ROOT=${LKJMC_INSTALL_ROOT:-/opt/lkjmc}
RUN_ROOT=${LKJMC_RUN_ROOT:-/run/lkjmc}
SOCKET_PATH=$RUN_ROOT/daemon.sock
HTTP_TOKEN_FILE=$CONFIG_ROOT/daemon-http.token
FORWARDING_SECRET_FILE=$CONFIG_ROOT/forwarding.secret
DB_SECRET_FILE=$CONFIG_ROOT/database.secret
BEDROCK=${LKJMC_PLAYABLE_BEDROCK:-auto}
ONLINE_MODE=${LKJMC_PLAYABLE_ONLINE_MODE:-true}
JAVA_BIND_HOST=${LKJMC_PLAYABLE_JAVA_BIND_HOST:-127.0.0.1}
JAVA_PORT=${LKJMC_PLAYABLE_JAVA_PORT:-25565}
PUBLIC_HOST=${LKJMC_PLAYABLE_PUBLIC_HOST:-}
BEDROCK_PORT=${LKJMC_PLAYABLE_BEDROCK_PORT:-19132}
DATABASE_URL=${LKJMC_DATABASE_URL:-postgres://lkjmc:lkjmc-dev@postgres:5432/lkjmc}

secret_file() {
    path=$1
    if [ ! -f "$path" ]; then umask 077; head -c 32 /dev/urandom | base64 >"$path"; fi
    chmod 0600 "$path"
}
write_config() {
    if [ -n "$PUBLIC_HOST" ]; then JAVA_PUBLIC_JSON=",\"publicHosts\":[\"$PUBLIC_HOST\"],\"preferredPublicHost\":\"$PUBLIC_HOST\""; else JAVA_PUBLIC_JSON=",\"publicHosts\":[]"; fi
    cat >"$CONFIG_ROOT/lkjmc.json" <<JSON
{
  "installRoot":"$INSTALL_ROOT","configRoot":"$CONFIG_ROOT","dataRoot":"$DATA_ROOT","logRoot":"$LOG_ROOT","socketPath":"$SOCKET_PATH",
  "database":{"host":"postgres","port":5432,"database":"lkjmc","user":"lkjmc","secretFile":"$DB_SECRET_FILE"},
  "network":{"revision":1,"instances":[{"id":"hub","owner":"lkjmc-daemon","kind":"folia","desiredState":"running","listener":"hub-java","memoryMb":1024,"assetIds":["folia-server","lkjmc-paper"]},{"id":"proxy","owner":"lkjmc-daemon","kind":"velocity","desiredState":"running","listener":"proxy-java","memoryMb":512,"assetIds":["velocity-server","lkjmc-velocity"]}],"routes":[{"id":"default","listener":"proxy-java","target":"hub","fallbacks":[]}],"listeners":[{"id":"hub-java","protocol":"java-tcp","bindHost":"127.0.0.1","port":25566,"publicHosts":[]},{"id":"proxy-java","protocol":"java-tcp","bindHost":"$JAVA_BIND_HOST","port":$JAVA_PORT$JAVA_PUBLIC_JSON}],"auth":{"onlineMode":$ONLINE_MODE},"forwarding":{"mode":"modern","secretFile":"$FORWARDING_SECRET_FILE"},"assets":[{"id":"folia-server","kind":"server","path":"$INSTALL_ROOT/assets/folia.jar","sha256":"1111111111111111111111111111111111111111111111111111111111111111","required":true},{"id":"lkjmc-paper","kind":"plugin","path":"$INSTALL_ROOT/assets/lkjmc-paper.jar","sha256":"2222222222222222222222222222222222222222222222222222222222222222","required":true},{"id":"lkjmc-velocity","kind":"plugin","path":"$INSTALL_ROOT/assets/lkjmc-velocity.jar","sha256":"3333333333333333333333333333333333333333333333333333333333333333","required":true},{"id":"velocity-server","kind":"server","path":"$INSTALL_ROOT/assets/velocity.jar","sha256":"4444444444444444444444444444444444444444444444444444444444444444","required":true}],"capabilities":{"runtime":"local-process","mountedConfig":true,"mountedSecrets":true,"mountedAssets":true}},
  "jars":{"root":"$INSTALL_ROOT/jars","defaultChannel":"stable","userAgent":"lkjmc (+https://github.com/lkjsxc/lkjmc)"},
  "daemonHttp":{"enabled":true,"address":"127.0.0.1:8765","tokenFile":"$HTTP_TOKEN_FILE"},
  "assets":{"root":"$INSTALL_ROOT/assets","serverChannel":"stable","pluginChannel":"stable","userAgent":"lkjmc (+https://github.com/lkjsxc/lkjmc)","downloadTimeoutSeconds":120},
  "plugins":{"lkjmc":{"enabled":true},"viaversion":{"mode":"auto","installOn":"backend"},"viabackwards":{"mode":"auto","installOn":"backend"},"geyser":{"mode":"auto","installOn":"proxy"},"floodgate":{"mode":"auto","installOn":"proxy","backendApi":false}},
  "runtime":{"adapter":"local-process","defaultJavaMemoryMb":1024,"proxyJavaMemoryMb":512,"stopTimeoutSeconds":30,"portRangeStart":25566,"portRangeEnd":25665}
}
JSON
}
wait_socket() {
    i=0
    while [ "$i" -lt 60 ]; do [ -S "$SOCKET_PATH" ] && return 0; i=$((i + 1)); sleep 1; done
    printf '%s\n' 'error: daemon socket did not appear' >&2
    exit 1
}

[ "${LKJMC_ACCEPT_MINECRAFT_EULA:-0}" = "1" ] || {
    printf '%s\n' 'error: set LKJMC_ACCEPT_MINECRAFT_EULA=1 to start a playable Minecraft server' >&2
    exit 1
}
install -d -m 0755 "$CONFIG_ROOT" "$DATA_ROOT/instances" "$LOG_ROOT/instances" "$INSTALL_ROOT/bin" "$INSTALL_ROOT/assets" "$INSTALL_ROOT/jars" "$RUN_ROOT"
printf '%s\n' 'lkjmc-dev' >"$DB_SECRET_FILE"
chmod 0600 "$DB_SECRET_FILE"
secret_file "$HTTP_TOKEN_FILE"
if [ -n "${LKJMC_PLAYABLE_HTTP_TOKEN:-}" ]; then
    umask 077
    printf '%s\n' "$LKJMC_PLAYABLE_HTTP_TOKEN" >"$HTTP_TOKEN_FILE"
    chmod 0600 "$HTTP_TOKEN_FILE"
fi
secret_file "$FORWARDING_SECRET_FILE"
write_config
cargo build --release -p lkjmc-daemon -p lkjmc-cli
install -m 0755 target/release/lkjmc-daemon "$INSTALL_ROOT/bin/lkjmc-daemon"
install -m 0755 target/release/lkjmc "$INSTALL_ROOT/bin/lkjmc"
./gradlew --no-daemon test shadowJar
LKJMC_DATABASE_URL=$DATABASE_URL "$INSTALL_ROOT/bin/lkjmc" db migrate
LKJMC_DATABASE_URL=$DATABASE_URL "$INSTALL_ROOT/bin/lkjmc-daemon" --config "$CONFIG_ROOT/lkjmc.json" --database-url "$DATABASE_URL" --http-token-file "$HTTP_TOKEN_FILE" >"$LOG_ROOT/daemon.log" 2>&1 &
wait_socket
LKJMC_DATABASE_URL=$DATABASE_URL "$INSTALL_ROOT/bin/lkjmc" shop seed-defaults
SMOKE_UUID=$(python3 - <<'PY'
import hashlib, uuid
value = bytearray(hashlib.md5(b'OfflinePlayer:LkjmcSmoke').digest())
value[6] = (value[6] & 15) | 48
value[8] = (value[8] & 63) | 128
print(uuid.UUID(bytes=bytes(value)))
PY
)
LKJMC_DATABASE_URL=$DATABASE_URL "$INSTALL_ROOT/bin/lkjmc" admin grant "minecraft-player:$SMOKE_UUID" owner --reason playable-smoke
LKJMC_DATABASE_URL=$DATABASE_URL "$INSTALL_ROOT/bin/lkjmc" bootstrap apply --profile playable --accept-minecraft-eula --bedrock "$BEDROCK"
LKJMC_DATABASE_URL=$DATABASE_URL "$INSTALL_ROOT/bin/lkjmc" bootstrap status
printf 'java: %s:%s\n' "${PUBLIC_HOST:-127.0.0.1}" "$JAVA_PORT"
if [ "${LKJMC_COMPOSE_EXIT_AFTER_BOOTSTRAP:-0}" = "1" ]; then exit 0; fi
wait
