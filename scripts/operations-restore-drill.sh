#!/bin/sh
set -eu
base=postgres://lkjmc:lkjmc-dev@postgres:5432
source_url=$base/lkjmc
out=${LKJMC_RESTORE_EVIDENCE_DIR:-/evidence/restore}
mkdir -p "$out"; chmod 0700 "$out"
evidence_owner=$(stat -c %u:%g "$(dirname "$out")")
export LKJMC_DATABASE_URL=$source_url
cargo build --locked -p lkjmc-cli -p lkjmc-daemon
target/debug/lkjmc db migrate >/dev/null
cleanup() {
  for db in aops_restore_1 aops_restore_2 aops_corrupt aops_version aops_partial; do
    PGPASSWORD=lkjmc-dev dropdb -h postgres -U lkjmc --if-exists "$db" >/dev/null 2>&1 || true
  done
}
finish() {
  status=$?
  trap - EXIT HUP INT TERM
  cleanup
  scripts/private-artifact-handoff.py --owner "$evidence_owner" "$out" || status=1
  exit "$status"
}
trap finish EXIT
trap 'exit 1' HUP INT TERM
boot() {
  db=$1; socket=/tmp/$db.sock; url=$base/$db; log=$out/$db-daemon.log
  LKJMC_DATABASE_URL=$url target/debug/lkjmc-daemon --socket "$socket" --http none \
    --database-url "$url" >"$log" 2>&1 & pid=$!
  stop() { kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; rm -f "$socket"; }
  trap stop INT TERM
  i=0; while [ "$i" -lt 200 ] && [ ! -S "$socket" ]; do sleep .05; i=$((i+1)); done
  [ -S "$socket" ] || { stop; return 1; }
  target/debug/lkjmc --socket "$socket" status --json >/dev/null
  target/debug/lkjmc --socket "$socket" db status >/dev/null
  doctor=$out/$db-doctor.log
  if target/debug/lkjmc --socket "$socket" doctor >"$doctor" 2>&1; then
    echo 'denied doctor unexpectedly succeeded' >&2; stop; return 1
  fi
  grep -q 'command.effect_denied' "$doctor"
  PGPASSWORD=lkjmc-dev psql -h postgres -U lkjmc -d "$db" -X -qAt \
    -c 'select count(*) from schema_migrations' | grep -Eq '^[1-9][0-9]*$'
  stop; trap 'exit 1' HUP INT TERM
  [ ! -S "$socket" ]
}
for rep in 1 2; do
  dump=$out/backup-$rep.dump
  LKJMC_DATABASE_URL=$source_url scripts/backup-postgres.sh "$dump"
  db=aops_restore_$rep
  PGPASSWORD=lkjmc-dev dropdb -h postgres -U lkjmc --if-exists "$db"
  PGPASSWORD=lkjmc-dev createdb -h postgres -U lkjmc "$db"
  LKJMC_DATABASE_URL=$base/$db LKJMC_CLI=target/debug/lkjmc scripts/restore-postgres.sh "$dump"
  boot "$db"
  PGPASSWORD=lkjmc-dev dropdb -h postgres -U lkjmc "$db"
done
negative() {
  kind=$1; db=aops_$kind; dir=$out/$kind; mkdir "$dir"
  for suffix in '' .manifest .metadata.json .sha256; do cp "$out/backup-2.dump$suffix" "$dir/test.dump$suffix"; done
  (cd "$dir" && sed -i 's/backup-2.dump/test.dump/g' test.dump.sha256)
  case "$kind" in
    corrupt) printf x >>"$dir/test.dump";;
    version) python3 - "$dir/test.dump.metadata.json" <<'PY'
import json,pathlib,sys
p=pathlib.Path(sys.argv[1]); d=json.loads(p.read_text()); d['schemaVersion']=999
p.write_text(json.dumps(d,indent=2,sort_keys=True)+'\n')
PY
      (cd "$dir" && sha256sum test.dump test.dump.manifest test.dump.metadata.json >test.dump.sha256);;
    partial) rm "$dir/test.dump.manifest";;
  esac
  PGPASSWORD=lkjmc-dev dropdb -h postgres -U lkjmc --if-exists "$db"
  PGPASSWORD=lkjmc-dev createdb -h postgres -U lkjmc "$db"
  if LKJMC_DATABASE_URL=$base/$db LKJMC_CLI=target/debug/lkjmc scripts/restore-postgres.sh "$dir/test.dump" >"$dir/result.log" 2>&1; then
    echo "$kind restore unexpectedly succeeded" >&2; return 1
  fi
  count=$(PGPASSWORD=lkjmc-dev psql -h postgres -U lkjmc -d "$db" -X -qAt \
    -c "select count(*) from pg_class c join pg_namespace n on n.oid=c.relnamespace where n.nspname='public' and c.relkind='r'")
  [ "$count" = 0 ]; PGPASSWORD=lkjmc-dev dropdb -h postgres -U lkjmc "$db"
}
negative corrupt; negative version; negative partial
cleanup; cleanup
remaining=$(PGPASSWORD=lkjmc-dev psql -h postgres -U lkjmc -d lkjmc -X -qAt \
  -c "select count(*) from pg_database where datname like 'aops\\_%' escape '\\'")
[ "$remaining" = 0 ]
printf '%s\n' 'ok restore-drill repeats=2 negatives=corrupt,version,partial cleanup=twice handoff=complete'
