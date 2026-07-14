#!/bin/sh
# Sourced by install.sh after its paths, service user, and fail() are defined.
group_name_for_gid() {
    entry=$(getent group "$1" 2>/dev/null || true)
    [ -n "$entry" ] || return 1
    name=${entry%%:*}
    [ -n "$name" ] && [ "$name" != "$entry" ] || return 1
    printf '%s\n' "$name"
}
resolve_service_identity() {
    entry=$(getent passwd "$SERVICE_USER" 2>/dev/null || true)
    [ -n "$entry" ] || fail "service user is absent from the account database: $SERVICE_USER"
    old_ifs=$IFS; IFS=:
    set -- $entry
    IFS=$old_ifs
    SERVICE_UID=$3; SERVICE_GID=$4
    case "$SERVICE_UID:$SERVICE_GID" in
        *[!0-9:]*|:|*:|:*:*) fail "invalid service UID/GID for $SERVICE_USER" ;;
    esac
    SERVICE_GROUP=$(group_name_for_gid "$SERVICE_GID") ||
        fail "service GID $SERVICE_GID has no system group; create that group before installing"
}
ensure_user() {
    if ! id -u "$SERVICE_USER" >/dev/null 2>&1; then
        useradd --system --home "$DATA_ROOT" --shell /usr/sbin/nologin "$SERVICE_USER"
    fi
    resolve_service_identity
    checkout_gid=$(stat -c %g .)
    [ "$checkout_gid" = 0 ] || [ "$checkout_gid" = "$SERVICE_GID" ] || {
        if checkout_group=$(group_name_for_gid "$checkout_gid"); then
            usermod -a -G "$checkout_group" "$SERVICE_USER"
        elif ! runuser -u "$SERVICE_USER" -- test -r Cargo.toml ||
            ! runuser -u "$SERVICE_USER" -- test -x .; then
            fail "checkout GID $checkout_gid has no system group; create it or grant $SERVICE_USER read/traverse access"
        fi
    }
}
stop_supervised_daemon() {
    [ -f "$RUN_ROOT/daemon.pid" ] || return 0
    pid=$(cat "$RUN_ROOT/daemon.pid")
    case "$pid" in ''|*[!0-9]*) fail "invalid daemon PID file: $RUN_ROOT/daemon.pid" ;; esac
    if ! kill -0 "$pid" 2>/dev/null; then
        rm -f "$SOCKET_PATH"
        return 0
    fi
    proc_uid=$(awk '/^Uid:/ {print $2; exit}' "/proc/$pid/status" 2>/dev/null || true)
    proc_name=$(cat "/proc/$pid/comm" 2>/dev/null || true)
    proc_arg0=$(tr '\000' '\n' <"/proc/$pid/cmdline" 2>/dev/null | head -1)
    [ "$proc_uid" = "$SERVICE_UID" ] && [ "$proc_name" = lkjmc-daemon ] &&
        [ "$proc_arg0" = "$INSTALL_ROOT/bin/lkjmc-daemon" ] ||
        fail "refusing to stop PID $pid: it is not the owned lkjmc daemon"
    kill "$pid"
    i=0
    while kill -0 "$pid" 2>/dev/null && [ "$i" -lt 50 ]; do
        sleep 0.1; i=$((i + 1))
    done
    if kill -0 "$pid" 2>/dev/null; then kill -KILL "$pid"; fi
    rm -f "$SOCKET_PATH"
}
start_without_systemd() {
    stop_supervised_daemon
    pid=$(su "$SERVICE_USER" -s /bin/sh -c "cd '$(pwd)' && nohup '$INSTALL_ROOT/bin/lkjmc-daemon' --config '$CONFIG_ROOT/lkjmc.json' --http-token-file '$HTTP_TOKEN_FILE' >'$LOG_ROOT/daemon.log' 2>&1 & echo \$!")
    case "$pid" in ''|*[!0-9]*) fail 'daemon did not return a valid PID' ;; esac
    printf '%s\n' "$pid" >"$RUN_ROOT/daemon.pid"
    chown "$SERVICE_UID:$SERVICE_GID" "$RUN_ROOT/daemon.pid"
}
