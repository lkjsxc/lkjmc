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
DATABASE_URL=${LKJMC_DATABASE_URL:-postgres://lkjmc:lkjmc-dev@postgres:5432/lkjmc}

secret_file() {
    path=$1
    if [ ! -f "$path" ]; then umask 077; head -c 32 /dev/urandom | base64 >"$path"; fi
    chmod 0600 "$path"
}
write_config() {
    cat >"$CONFIG_ROOT/lkjmc.json" <<JSON
{
  "installRoot":"$INSTALL_ROOT","configRoot":"$CONFIG_ROOT","dataRoot":"$DATA_ROOT","logRoot":"$LOG_ROOT","socketPath":"$SOCKET_PATH",
  "database":{"host":"postgres","port":5432,"database":"lkjmc","user":"lkjmc","secretFile":"$DB_SECRET_FILE"},
  "network":{"name":"lkjmc-local","defaultLocale":"en","fallbackServer":"hub","onlineMode":true,"velocityForwarding":"modern","forwardingSecretFile":"$FORWARDING_SECRET_FILE","javaEntry":{"host":"0.0.0.0","port":25565},"bedrockEntry":{"mode":"$BEDROCK","host":"0.0.0.0","port":19132}},
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
secret_file "$FORWARDING_SECRET_FILE"
write_config
cargo build --release -p lkjmc-daemon -p lkjmc-cli
install -m 0755 target/release/lkjmc-daemon "$INSTALL_ROOT/bin/lkjmc-daemon"
install -m 0755 target/release/lkjmc "$INSTALL_ROOT/bin/lkjmc"
./gradlew --no-daemon test shadowJar
LKJMC_DATABASE_URL=$DATABASE_URL "$INSTALL_ROOT/bin/lkjmc" db migrate
LKJMC_DATABASE_URL=$DATABASE_URL "$INSTALL_ROOT/bin/lkjmc-daemon" --config "$CONFIG_ROOT/lkjmc.json" --database-url "$DATABASE_URL" --http-token-file "$HTTP_TOKEN_FILE" >"$LOG_ROOT/daemon.log" 2>&1 &
wait_socket
LKJMC_DATABASE_URL=$DATABASE_URL "$INSTALL_ROOT/bin/lkjmc" bootstrap apply --profile playable --accept-minecraft-eula --bedrock "$BEDROCK"
LKJMC_DATABASE_URL=$DATABASE_URL "$INSTALL_ROOT/bin/lkjmc" bootstrap status
if [ "${LKJMC_COMPOSE_EXIT_AFTER_BOOTSTRAP:-0}" = "1" ]; then exit 0; fi
wait
