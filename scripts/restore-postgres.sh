#!/bin/sh
set -eu
if [ "${LKJMC_DATABASE_URL:-}" = "" ]; then
  echo "LKJMC_DATABASE_URL is required" >&2
  exit 1
fi
in=${1:-}
if [ "$in" = "" ]; then
  echo "usage: scripts/restore-postgres.sh IN.dump" >&2
  exit 1
fi
pg_restore --clean --if-exists --no-owner --dbname "$LKJMC_DATABASE_URL" "$in"
printf 'restore applied: %s\n' "$in"
