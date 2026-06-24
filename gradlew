#!/bin/sh
set -eu
VERSION=8.10.2
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
DIST_NAME="gradle-${VERSION}-bin"
GRADLE_USER_HOME=${GRADLE_USER_HOME:-"$HOME/.gradle"}

find_gradle() {
    if command -v gradle >/dev/null 2>&1; then
        command -v gradle
        return 0
    fi
    if [ -d "$GRADLE_USER_HOME/wrapper/dists/$DIST_NAME" ]; then
        found=$(find "$GRADLE_USER_HOME/wrapper/dists/$DIST_NAME" \
            -path "*/gradle-${VERSION}/bin/gradle" -type f | head -n 1)
        if [ -n "$found" ]; then
            printf '%s\n' "$found"
            return 0
        fi
    fi
    return 1
}

bootstrap_gradle() {
    cache="$ROOT/.gradle-bootstrap"
    zip="$cache/$DIST_NAME.zip"
    mkdir -p "$cache"
    if [ ! -f "$zip" ]; then
        curl -fsSL "https://services.gradle.org/distributions/$DIST_NAME.zip" -o "$zip"
    fi
    if [ ! -x "$cache/gradle-${VERSION}/bin/gradle" ]; then
        unzip -q "$zip" -d "$cache"
    fi
    printf '%s\n' "$cache/gradle-${VERSION}/bin/gradle"
}

if gradle_bin=$(find_gradle); then
    exec "$gradle_bin" "$@"
fi

gradle_bin=$(bootstrap_gradle)
exec "$gradle_bin" "$@"
