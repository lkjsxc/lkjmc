#!/bin/sh
set -eu
if [ "${LKJMC_DATABASE_URL:-}" = "" ]; then
  echo "LKJMC_DATABASE_URL is required" >&2
  exit 1
fi
out=${1:-}
if [ "$out" = "" ]; then
  echo "usage: scripts/backup-postgres.sh OUT.dump" >&2
  exit 1
fi
umask 077
pg_dump --format=custom --no-owner --file "$out" "$LKJMC_DATABASE_URL"
printf 'backup written: %s\n' "$out"
