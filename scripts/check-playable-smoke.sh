#!/usr/bin/env bash
set -euo pipefail

if [ "${LKJMC_PLAYABLE_SMOKE:-0}" != "1" ]; then
    printf '%s\n' 'ok check-playable-smoke skipped'
    exit 0
fi
[ "${LKJMC_ACCEPT_MINECRAFT_EULA:-0}" = "1" ] || {
    printf '%s\n' 'error: set LKJMC_ACCEPT_MINECRAFT_EULA=1 for playable smoke' >&2
    exit 1
}
command -v docker >/dev/null 2>&1 || {
    printf '%s\n' 'error: docker is required for playable smoke' >&2
    exit 1
}
command -v java >/dev/null 2>&1 || {
    printf '%s\n' 'error: java is required for playable smoke' >&2
    exit 1
}

export LKJMC_COMPOSE_EXIT_AFTER_BOOTSTRAP=0
export LKJMC_PLAYABLE_ONLINE_MODE=${LKJMC_PLAYABLE_ONLINE_MODE:-false}
export LKJMC_PLAYABLE_JAVA_BIND_HOST=${LKJMC_PLAYABLE_JAVA_BIND_HOST:-0.0.0.0}
export LKJMC_PLAYABLE_HTTP_TOKEN=${LKJMC_PLAYABLE_HTTP_TOKEN:-LkJmC-Smoke-AbC123+/=}
export LKJMC_COMPOSE_PROJECT_NAME=${LKJMC_COMPOSE_PROJECT_NAME:-lkjmc-playable-smoke-$$}

compose() { docker compose --project-name "$LKJMC_COMPOSE_PROJECT_NAME" --profile playable "$@"; }
redact() {
    sed -E \
        -e 's#(postgres://[^:]+):[^@]+@#\1:<redacted>@#g' \
        -e 's#(Bearer )[A-Za-z0-9+/_=.:-]+#\1<redacted>#g'
}
dump_logs() { compose logs --no-color playable postgres 2>/dev/null | redact || true; }
cleanup() { compose down -v >/dev/null 2>&1 || true; }
on_exit() {
    code=$?
    if [ "$code" -ne 0 ]; then dump_logs; fi
    cleanup
    exit "$code"
}
trap on_exit EXIT INT TERM

wait_for_java() {
    port=${LKJMC_PLAYABLE_JAVA_PORT:-25565}
    for _ in $(seq 1 1800); do
        if scripts/minecraft_login_probe.py status 127.0.0.1 "$port" >/dev/null 2>&1; then
            return 0
        fi
        sleep 1
    done
    printf '%s\n' 'error: playable Java entry did not become reachable' >&2
    return 1
}

run_protocol_smoke() {
    port=${LKJMC_PLAYABLE_JAVA_PORT:-25565}
    work=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-command-menu-smoke.XXXXXX")
    mkdir -p "$work/src/main/java/com/lkjmc/smoke"
    cp tests/smoke/command_menu/*.java "$work/src/main/java/com/lkjmc/smoke/"
    cat >"$work/settings.gradle.kts" <<'EOF'
pluginManagement { repositories { gradlePluginPortal(); mavenCentral() } }
dependencyResolutionManagement { repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS); repositories { mavenCentral(); maven("https://repo.opencollab.dev/maven-releases") } }
rootProject.name = "lkjmc-command-menu-smoke"
EOF
    cat >"$work/build.gradle.kts" <<'EOF'
plugins { application }
java { toolchain.languageVersion.set(JavaLanguageVersion.of(21)) }
application { mainClass.set("com.lkjmc.smoke.MinecraftCommandMenuSmoke") }
dependencies {
    implementation("org.geysermc.mcprotocollib:protocol:1.21.7-1")
    implementation("net.kyori:adventure-text-serializer-plain:4.17.0")
}
EOF
    ./gradlew --no-daemon -q -p "$work" run --args="127.0.0.1 $port"
    rm -rf "$work"
}

cleanup
compose up -d --build --force-recreate playable
wait_for_java
run_protocol_smoke
printf '%s\n' 'ok check-playable-smoke'
