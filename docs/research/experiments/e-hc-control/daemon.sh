#!/bin/sh
set -eu

owner=$1
operation=${2:?operation required}
expected=${3:?expected result required}

for _ in $(seq 1 30); do
    if pg_isready -h postgres -U lab -d lab >/dev/null 2>&1; then
        break
    fi
    sleep 1
done
pg_isready -h postgres -U lab -d lab >/dev/null

query() {
    psql -X -v ON_ERROR_STOP=1 -h postgres -U lab -d lab -Atqc "$1"
}

case "$operation" in
    acquire)
        result=$(query "SELECT coalesce(lab.acquire_lease('$owner')::text, 'REJECT')")
        ;;
    write-stale)
        result=$(query "SELECT lab.fenced_write('$owner', 1, 'stale')")
        ;;
    write-fresh)
        result=$(query "SELECT lab.fenced_write('$owner', 2, 'fresh')")
        ;;
    *)
        printf '%s\n' "unknown operation: $operation" >&2
        exit 64
        ;;
esac

printf '%s %s=%s\n' "$owner" "$operation" "$result"
[ "$result" = "$expected" ]
