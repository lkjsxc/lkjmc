#!/bin/sh
set -eu
INSTALL_ROOT=${LKJMC_INSTALL_ROOT:-/opt/lkjmc}
CONFIG_ROOT=${LKJMC_CONFIG_ROOT:-/etc/lkjmc}
DATA_ROOT=${LKJMC_DATA_ROOT:-/var/lib/lkjmc}
LOG_ROOT=${LKJMC_LOG_ROOT:-/var/log/lkjmc}
RUN_ROOT=${LKJMC_RUN_ROOT:-/run/lkjmc}
DB_NAME=${LKJMC_DB_NAME:-lkjmc}
DB_USER=${LKJMC_DB_USER:-lkjmc}
SERVICE_USER=${LKJMC_SERVICE_USER:-lkjmc}
SOCKET_PATH=$RUN_ROOT/daemon.sock
DB_SECRET_FILE=$CONFIG_ROOT/database.secret
HTTP_TOKEN_FILE=$CONFIG_ROOT/daemon-http.token
FORWARDING_SECRET_FILE=$CONFIG_ROOT/forwarding.secret
ENV_FILE=$CONFIG_ROOT/daemon.env
PLAYABLE=0; ACCEPT_EULA=0
BEDROCK=auto; JAVA_BIND_HOST=0.0.0.0; JAVA_PORT=25565; JAVA_PUBLIC_HOST=
BEDROCK_PORT=19132
NO_START=0
info() { printf '%s\n' "$*"; }
fail() { printf '%s\n' "error: $*" >&2; exit 1; }
while [ "$#" -gt 0 ]; do
    case "$1" in
        --playable) PLAYABLE=1 ;;
        --accept-minecraft-eula) ACCEPT_EULA=1 ;;
        --bedrock) shift; BEDROCK=${1:-}; [ -n "$BEDROCK" ] || fail 'missing --bedrock value' ;;
        --java-bind-host) shift; JAVA_BIND_HOST=${1:-}; [ -n "$JAVA_BIND_HOST" ] || fail 'missing --java-bind-host value' ;;
        --java-port) shift; JAVA_PORT=${1:-}; [ -n "$JAVA_PORT" ] || fail 'missing --java-port value' ;;
        --java-public-host) shift; JAVA_PUBLIC_HOST=${1:-}; [ -n "$JAVA_PUBLIC_HOST" ] || fail 'missing --java-public-host value' ;;
        --bedrock-port) shift; BEDROCK_PORT=${1:-}; [ -n "$BEDROCK_PORT" ] || fail 'missing --bedrock-port value' ;;
        --no-start) NO_START=1 ;;
        *) fail "unknown flag: $1" ;;
    esac
    shift
done
[ "$(id -u)" -eq 0 ] || fail 'run installer as root'
[ -f Cargo.toml ] || fail 'run from an lkjmc checkout or curl into one'
[ "$PLAYABLE" = 0 ] || [ "$ACCEPT_EULA" = 1 ] || fail 'pass --accept-minecraft-eula to start a playable Minecraft server'
require_apt() {
    command -v apt-get >/dev/null 2>&1 || fail 'apt-get is required on this host'
    apt-get update
    apt-get install -y --no-install-recommends build-essential ca-certificates curl jq \
        openssl postgresql openjdk-21-jdk-headless unzip tar pkg-config libssl-dev
}
ensure_rust() { if ! command -v cargo >/dev/null 2>&1 || ! cargo -V | awk '{split($2,v,"."); exit !(v[1]>1 || (v[1]==1 && v[2]>=78))}'; then curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable >/dev/null; export PATH="$HOME/.cargo/bin:$PATH"; fi; }
has_systemd() { [ -d /run/systemd/system ] && command -v systemctl >/dev/null 2>&1; }
start_postgres() {
    if has_systemd; then systemctl enable --now postgresql >/dev/null
    elif command -v service >/dev/null 2>&1; then service postgresql start >/dev/null || true; fi
}
ensure_user() {
    if ! id -u "$SERVICE_USER" >/dev/null 2>&1; then
        useradd --system --home "$DATA_ROOT" --shell /usr/sbin/nologin "$SERVICE_USER"
    fi
    group=$(stat -c %G .); [ "$group" = root ] || usermod -a -G "$group" "$SERVICE_USER"
}
ensure_roots() {
    install -d -m 0755 "$INSTALL_ROOT" "$INSTALL_ROOT/bin" "$INSTALL_ROOT/jars" \
        "$INSTALL_ROOT/assets" "$CONFIG_ROOT" "$CONFIG_ROOT/templates" "$DATA_ROOT" \
        "$DATA_ROOT/instances" "$LOG_ROOT" "$LOG_ROOT/instances" "$RUN_ROOT"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_ROOT" "$LOG_ROOT" "$RUN_ROOT" "$INSTALL_ROOT/jars" "$INSTALL_ROOT/assets"
    chmod 0755 "$CONFIG_ROOT"
}
ensure_secret_file() {
    path=$1
    if [ ! -f "$path" ]; then (umask 077; openssl rand -base64 32 >"$path"); fi
    chown "$SERVICE_USER:$SERVICE_USER" "$path"; chmod 0600 "$path"
}
pg_as_postgres() { su postgres -c "$*"; }
ensure_database() {
    password=$(cat "$DB_SECRET_FILE")
    escaped=$(printf '%s' "$password" | sed "s/'/''/g")
    pg_as_postgres "psql -v ON_ERROR_STOP=1" <<SQL
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '$DB_USER') THEN
        CREATE ROLE $DB_USER LOGIN PASSWORD '$escaped';
    ELSE
        ALTER ROLE $DB_USER WITH LOGIN PASSWORD '$escaped';
    END IF;
END
\$\$;
SQL
    if ! pg_as_postgres "psql -tAc \"select 1 from pg_database where datname='$DB_NAME'\"" | grep -q 1; then
        pg_as_postgres "createdb -O $DB_USER $DB_NAME"
    fi
}
write_config() {
    if [ -n "$JAVA_PUBLIC_HOST" ]; then JAVA_PUBLIC_JSON=",\"publicHosts\":[\"$JAVA_PUBLIC_HOST\"],\"preferredPublicHost\":\"$JAVA_PUBLIC_HOST\""; else JAVA_PUBLIC_JSON=",\"publicHosts\":[]"; fi
    cat >"$CONFIG_ROOT/lkjmc.json" <<JSON
{
  "installRoot": "$INSTALL_ROOT",
  "configRoot": "$CONFIG_ROOT",
  "dataRoot": "$DATA_ROOT",
  "logRoot": "$LOG_ROOT",
  "socketPath": "$SOCKET_PATH",
  "database": {"host":"127.0.0.1","port":5432,"database":"$DB_NAME","user":"$DB_USER","secretFile":"$DB_SECRET_FILE"},
  "network": {
    "revision":1,
    "instances":[{"id":"hub","owner":"lkjmc-daemon","kind":"folia","desiredState":"running","listener":"hub-java","memoryMb":2048,"assetIds":["folia-server","lkjmc-paper"]},{"id":"proxy","owner":"lkjmc-daemon","kind":"velocity","desiredState":"running","listener":"proxy-java","memoryMb":512,"assetIds":["velocity-server","lkjmc-velocity"]}],
    "routes":[{"id":"default","listener":"proxy-java","target":"hub","fallbacks":[]}],
    "listeners":[{"id":"hub-java","protocol":"java-tcp","bindHost":"127.0.0.1","port":25566,"publicHosts":[]},{"id":"proxy-java","protocol":"java-tcp","bindHost":"$JAVA_BIND_HOST","port":$JAVA_PORT$JAVA_PUBLIC_JSON}],
    "auth":{"onlineMode":true},
    "forwarding":{"mode":"modern","secretFile":"$FORWARDING_SECRET_FILE"},
    "assets":[{"id":"folia-server","kind":"server","path":"$INSTALL_ROOT/assets/folia.jar","sha256":"1111111111111111111111111111111111111111111111111111111111111111","required":true},{"id":"lkjmc-paper","kind":"plugin","path":"$INSTALL_ROOT/assets/lkjmc-paper.jar","sha256":"2222222222222222222222222222222222222222222222222222222222222222","required":true},{"id":"lkjmc-velocity","kind":"plugin","path":"$INSTALL_ROOT/assets/lkjmc-velocity.jar","sha256":"3333333333333333333333333333333333333333333333333333333333333333","required":true},{"id":"velocity-server","kind":"server","path":"$INSTALL_ROOT/assets/velocity.jar","sha256":"4444444444444444444444444444444444444444444444444444444444444444","required":true}],
    "capabilities":{"runtime":"local-process","mountedConfig":true,"mountedSecrets":true,"mountedAssets":true}
  },
  "jars": {"root":"$INSTALL_ROOT/jars","defaultChannel":"stable","userAgent":"lkjmc (+https://github.com/lkjsxc/lkjmc)"},
  "daemonHttp": {"enabled":true,"address":"127.0.0.1:8765","tokenFile":"$HTTP_TOKEN_FILE"},
  "assets": {"root":"$INSTALL_ROOT/assets","serverChannel":"stable","pluginChannel":"stable","userAgent":"lkjmc (+https://github.com/lkjsxc/lkjmc)","downloadTimeoutSeconds":120},
  "plugins": {
    "lkjmc":{"enabled":true},
    "viaversion":{"mode":"auto","installOn":"backend"},
    "viabackwards":{"mode":"auto","installOn":"backend"},
    "geyser":{"mode":"auto","installOn":"proxy"},
    "floodgate":{"mode":"auto","installOn":"proxy","backendApi":false}
  },
  "runtime": {"adapter":"local-process","defaultJavaMemoryMb":2048,"proxyJavaMemoryMb":512,"stopTimeoutSeconds":30,"portRangeStart":25566,"portRangeEnd":25665}
}
JSON
    chmod 0644 "$CONFIG_ROOT/lkjmc.json"
}
build_install() {
    cargo build --release -p lkjmc-daemon -p lkjmc-cli
    install -m 0755 target/release/lkjmc-daemon "$INSTALL_ROOT/bin/lkjmc-daemon"
    install -m 0755 target/release/lkjmc "$INSTALL_ROOT/bin/lkjmc"
    if [ -x ./gradlew ]; then ./gradlew --no-daemon test shadowJar; fi
}
database_url() {
    password=$(cat "$DB_SECRET_FILE")
    printf 'postgres://%s:%s@127.0.0.1:5432/%s' "$DB_USER" "$password" "$DB_NAME"
}
migrate_database() { LKJMC_DATABASE_URL=$(database_url) "$INSTALL_ROOT/bin/lkjmc" db migrate >/dev/null; }
write_env_file() {
    (umask 077; printf 'LKJMC_DATABASE_URL=%s\n' "$(database_url)" >"$ENV_FILE")
    chmod 0600 "$ENV_FILE"
}
write_service() {
    cat >/etc/systemd/system/lkjmc-daemon.service <<UNIT
[Unit]
Description=lkjmc daemon
After=network.target postgresql.service
[Service]
User=$SERVICE_USER
Group=$SERVICE_USER
EnvironmentFile=$ENV_FILE
WorkingDirectory=$(pwd)
ExecStart=$INSTALL_ROOT/bin/lkjmc-daemon --config $CONFIG_ROOT/lkjmc.json --http-token-file $HTTP_TOKEN_FILE
Restart=on-failure
RuntimeDirectory=lkjmc
[Install]
WantedBy=multi-user.target
UNIT
    systemctl daemon-reload
    systemctl enable lkjmc-daemon >/dev/null; systemctl restart lkjmc-daemon >/dev/null
}
start_without_systemd() {
    su "$SERVICE_USER" -s /bin/sh -c "cd '$(pwd)' && nohup '$INSTALL_ROOT/bin/lkjmc-daemon' --config '$CONFIG_ROOT/lkjmc.json' --http-token-file '$HTTP_TOKEN_FILE' >'$LOG_ROOT/daemon.log' 2>&1 & echo \$! >'$RUN_ROOT/daemon.pid'"
}
start_daemon() { if has_systemd; then write_service; else start_without_systemd; fi; }
wait_socket() {
    i=0
    while [ "$i" -lt 60 ]; do [ -S "$SOCKET_PATH" ] && return 0; i=$((i + 1)); sleep 1; done
    fail "daemon socket did not appear: $SOCKET_PATH"
}
run_playable() {
    wait_socket
    runuser -u "$SERVICE_USER" -- "$INSTALL_ROOT/bin/lkjmc" bootstrap apply --profile playable --accept-minecraft-eula --bedrock "$BEDROCK" >/dev/null
    status=$(runuser -u "$SERVICE_USER" -- "$INSTALL_ROOT/bin/lkjmc" --json bootstrap status)
    proxy=$(printf '%s' "$status" | jq -r '.instances[]? | select(.id=="proxy") | .observedState // "unknown"')
    hub=$(printf '%s' "$status" | jq -r '.instances[]? | select(.id=="hub") | .observedState // "unknown"')
    info 'ok install lkjmc playable'
    JAVA_DISPLAY=${JAVA_PUBLIC_HOST:-127.0.0.1}
    info "java: $JAVA_DISPLAY:$JAVA_PORT"
    info "bedrock: see bootstrap status"
    info "proxy: ${proxy:-unknown}"
    info "hub: ${hub:-unknown}"
    info "status: $INSTALL_ROOT/bin/lkjmc bootstrap status"
    info "logs: $INSTALL_ROOT/bin/lkjmc instance logs proxy --lines 100"
}
require_apt
ensure_rust
start_postgres
ensure_user
ensure_roots
ensure_secret_file "$DB_SECRET_FILE"
ensure_secret_file "$HTTP_TOKEN_FILE"
ensure_secret_file "$FORWARDING_SECRET_FILE"
ensure_database
write_config
build_install
migrate_database
write_env_file
if [ "$NO_START" = 0 ]; then
    start_daemon
    if [ "$PLAYABLE" = 1 ]; then run_playable; else runuser -u "$SERVICE_USER" -- "$INSTALL_ROOT/bin/lkjmc" doctor >/dev/null; fi
else
    info 'ok install lkjmc no-start'
fi
