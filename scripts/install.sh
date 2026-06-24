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
SECRET_FILE=$CONFIG_ROOT/database.secret
ENV_FILE=$CONFIG_ROOT/daemon.env

info() { printf '%s\n' "$*"; }
fail() { printf '%s\n' "error: $*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || fail 'run installer as root'
[ -f Cargo.toml ] || fail 'run from an lkjmc checkout or curl into one'

require_apt() {
    command -v apt-get >/dev/null 2>&1 || fail 'apt-get is required on this host'
    apt-get update
    apt-get install -y --no-install-recommends \
        build-essential ca-certificates curl jq openssl postgresql \
        openjdk-21-jdk-headless unzip tar pkg-config libssl-dev cargo rustc
}

has_systemd() {
    [ -d /run/systemd/system ] && command -v systemctl >/dev/null 2>&1
}

start_postgres() {
    if has_systemd; then
        systemctl enable --now postgresql >/dev/null
    elif command -v service >/dev/null 2>&1; then
        service postgresql start >/dev/null || true
    fi
}

ensure_user() {
    if ! id -u "$SERVICE_USER" >/dev/null 2>&1; then
        useradd --system --home "$DATA_ROOT" --shell /usr/sbin/nologin "$SERVICE_USER"
    fi
}

ensure_roots() {
    install -d -m 0755 "$INSTALL_ROOT" "$INSTALL_ROOT/bin" "$INSTALL_ROOT/jars" \
        "$INSTALL_ROOT/plugins" "$CONFIG_ROOT" "$CONFIG_ROOT/instances" \
        "$CONFIG_ROOT/templates" "$DATA_ROOT" "$DATA_ROOT/instances" \
        "$LOG_ROOT" "$LOG_ROOT/instances" "$RUN_ROOT"
    chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_ROOT" "$LOG_ROOT" "$RUN_ROOT"
    chmod 0755 "$CONFIG_ROOT"
}

ensure_secret() {
    if [ ! -f "$SECRET_FILE" ]; then
        umask 077
        openssl rand -base64 32 >"$SECRET_FILE"
    fi
    chmod 0600 "$SECRET_FILE"
}

pg_as_postgres() {
    su postgres -c "$*"
}

ensure_database() {
    password=$(cat "$SECRET_FILE")
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
    cat >"$CONFIG_ROOT/lkjmc.json" <<JSON
{
  "installRoot": "$INSTALL_ROOT",
  "configRoot": "$CONFIG_ROOT",
  "dataRoot": "$DATA_ROOT",
  "logRoot": "$LOG_ROOT",
  "socketPath": "$SOCKET_PATH",
  "database": {
    "host": "127.0.0.1",
    "port": 5432,
    "database": "$DB_NAME",
    "user": "$DB_USER",
    "secretFile": "$SECRET_FILE"
  },
  "network": {
    "defaultLocale": "en",
    "fallbackServer": "hub",
    "onlineMode": true,
    "velocityForwarding": "modern"
  },
  "jars": {
    "root": "$INSTALL_ROOT/jars",
    "defaultChannel": "stable",
    "userAgent": "lkjmc (+https://github.com/lkjsxc/lkjmc)"
  },
  "runtime": {
    "adapter": "local-process",
    "defaultJavaMemoryMb": 2048,
    "stopTimeoutSeconds": 30
  }
}
JSON
    chmod 0644 "$CONFIG_ROOT/lkjmc.json"
}

build_install() {
    cargo build --release -p lkjmc-daemon -p lkjmc-cli
    install -m 0755 target/release/lkjmc-daemon "$INSTALL_ROOT/bin/lkjmc-daemon"
    install -m 0755 target/release/lkjmc "$INSTALL_ROOT/bin/lkjmc"
    if [ -x ./gradlew ]; then
        ./gradlew --no-daemon jar >/dev/null
    fi
}

database_url() {
    password=$(cat "$SECRET_FILE")
    printf 'postgres://%s:%s@127.0.0.1:5432/%s' "$DB_USER" "$password" "$DB_NAME"
}

migrate_database() {
    LKJMC_DATABASE_URL=$(database_url) "$INSTALL_ROOT/bin/lkjmc" db migrate >/dev/null
}

write_env_file() {
    umask 077
    printf 'LKJMC_DATABASE_URL=%s\n' "$(database_url)" >"$ENV_FILE"
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
ExecStart=$INSTALL_ROOT/bin/lkjmc-daemon --config $CONFIG_ROOT/lkjmc.json
Restart=on-failure
RuntimeDirectory=lkjmc

[Install]
WantedBy=multi-user.target
UNIT
    systemctl daemon-reload
    systemctl enable --now lkjmc-daemon >/dev/null
}

start_without_systemd() {
    su "$SERVICE_USER" -s /bin/sh -c "nohup '$INSTALL_ROOT/bin/lkjmc-daemon' --config '$CONFIG_ROOT/lkjmc.json' >'$LOG_ROOT/daemon.log' 2>&1 & echo \$! >'$RUN_ROOT/daemon.pid'"
}

run_doctor() {
    for _ in $(seq 1 50); do
        [ -S "$SOCKET_PATH" ] && break
        sleep 0.2
    done
    "$INSTALL_ROOT/bin/lkjmc" --socket "$SOCKET_PATH" doctor
}

require_apt
start_postgres
ensure_user
ensure_roots
ensure_secret
ensure_database
write_config
build_install
migrate_database
if has_systemd; then
    write_env_file
    write_service
else
    start_without_systemd
fi
run_doctor
info 'ok install lkjmc'
info "next: $INSTALL_ROOT/bin/lkjmc --socket $SOCKET_PATH status"
