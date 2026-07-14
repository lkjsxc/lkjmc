#!/bin/sh
set -eu
url=${LKJMC_DATABASE_URL:-}; in=${1:-}
[ -n "$url" ] || { echo 'LKJMC_DATABASE_URL is required' >&2; exit 1; }
[ -n "$in" ] || { echo 'usage: scripts/restore-postgres.sh IN.dump' >&2; exit 1; }
for suffix in '' .manifest .metadata.json .sha256; do
  [ -f "$in$suffix" ] || { echo "backup member missing: $in$suffix" >&2; exit 1; }
done
command -v psql >/dev/null && command -v pg_restore >/dev/null || {
  echo 'psql and pg_restore are required' >&2; exit 1;
}
dir=$(cd "$(dirname "$in")" && pwd); base=$(basename "$in")
(cd "$dir" && sha256sum --check --strict "$base.sha256") >/dev/null
tmp=$(mktemp -d); cleanup() { rm -rf "$tmp"; }; trap cleanup EXIT HUP INT TERM
actual_manifest=$tmp/manifest; actual_schema=$tmp/schema.sql; restored_marker=$tmp/restored.json
pg_restore --list "$in" >"$actual_manifest"
pg_restore --schema-only --file="$actual_schema" "$in"
recorded=$(ACTUAL=$actual_manifest SCHEMA=$actual_schema DUMP=$in META=$in.metadata.json python3 - <<'PY'
import hashlib,json,os,pathlib,re
p=lambda n:pathlib.Path(os.environ[n])
h=lambda raw:hashlib.sha256(raw).hexdigest()
try: data=json.loads(p('META').read_text(encoding='utf-8'))
except (OSError,TypeError,ValueError) as e: raise SystemExit(f'invalid backup metadata: {e}')
fields={'schemaVersion','sourceCommit','postgresServerVersion','lsn','lsnSha256',
 'schemaSha256','migrationMarker','migrationSha256','dumpSha256','manifestSha256'}
if not isinstance(data,dict) or set(data)!=fields: raise SystemExit('metadata fields differ')
if data['schemaVersion']!=1: raise SystemExit('unsupported backup metadata version')
if isinstance(data['postgresServerVersion'],bool) or not isinstance(data['postgresServerVersion'],int): raise SystemExit('invalid PostgreSQL version')
for key in ('lsnSha256','schemaSha256','migrationSha256','dumpSha256','manifestSha256'):
 if not isinstance(data[key],str) or not re.fullmatch(r'[0-9a-f]{64}',data[key]): raise SystemExit(f'invalid {key}')
lsn=data['lsn']
if not isinstance(lsn,str) or not re.fullmatch(r'[0-9A-F]+/[0-9A-F]+',lsn): raise SystemExit('invalid WAL LSN')
if h(lsn.encode())!=data['lsnSha256']: raise SystemExit('LSN checksum mismatch')
marker=data['migrationMarker']
if not isinstance(marker,str) or '\n' in marker: raise SystemExit('migration marker is not compact single-line JSON')
try: rows=json.loads(marker)
except (TypeError,ValueError) as e: raise SystemExit(f'invalid migration marker: {e}')
canonical=json.dumps(rows,separators=(',',':'),sort_keys=True,ensure_ascii=False)
if marker!=canonical or h(marker.encode())!=data['migrationSha256']: raise SystemExit('migration marker mismatch')
if not isinstance(rows,list): raise SystemExit('migration marker is not an array')
last=-1
for row in rows:
 if not isinstance(row,dict) or set(row)!={'version','name','checksum'}: raise SystemExit('migration marker fields differ')
 version=row['version']
 if isinstance(version,bool) or not isinstance(version,int) or version<=last: raise SystemExit('migration versions are not ordered unique integers')
 if not isinstance(row['name'],str) or not row['name']: raise SystemExit('invalid migration name')
 if not isinstance(row['checksum'],str) or not re.fullmatch(r'(?:[0-9a-f]{64})?',row['checksum']): raise SystemExit('invalid migration checksum')
 last=version
checks={'DUMP':'dumpSha256','ACTUAL':'manifestSha256'}
for source,key in checks.items():
 if h(p(source).read_bytes())!=data[key]: raise SystemExit(f'{key} mismatch')
schema=p('SCHEMA').read_bytes().splitlines(keepends=True)
schema=b''.join(line for line in schema if not line.startswith((b'\\restrict ',b'\\unrestrict ')))
if h(schema)!=data['schemaSha256']: raise SystemExit('schemaSha256 mismatch')
print(data['postgresServerVersion'])
PY
)
server=$(psql "$url" -X --quiet --no-align --tuples-only -v ON_ERROR_STOP=1 \
  -c "select current_setting('server_version_num')")
tool_major() { "$1" --version | python3 -c "import re,sys; m=re.search(r'([0-9]+)(?:\\.[0-9]+)',sys.stdin.read()); print(m.group(1) if m else 'invalid')"; }
target_major=$((server / 10000)); recorded_major=$((recorded / 10000))
[ "$target_major" = "$recorded_major" ] && [ "$(tool_major psql)" = "$target_major" ] && \
  [ "$(tool_major pg_restore)" = "$target_major" ] || {
  echo "PostgreSQL major mismatch: backup=$recorded target=$server tools=$(tool_major psql)/$(tool_major pg_restore)" >&2; exit 1;
}
relations=$(psql "$url" -X --quiet --no-align --tuples-only -v ON_ERROR_STOP=1 -c \
  "select count(*) from pg_class c join pg_namespace n on n.oid=c.relnamespace where n.nspname not in ('pg_catalog','information_schema') and n.nspname !~ '^pg_toast' and c.relkind in ('r','p','v','m','S','f')")
[ "$relations" = 0 ] || { echo 'restore target is not a fresh database' >&2; exit 1; }
pg_restore --exit-on-error --no-owner --dbname "$url" "$in"
psql "$url" -X --quiet --no-align --tuples-only -v ON_ERROR_STOP=1 -c \
  "select coalesce(jsonb_agg(jsonb_build_object('version',version,'name',name,'checksum',coalesce(checksum,'')) order by version),'[]'::jsonb)::text from schema_migrations" >"$restored_marker"
META=$in.metadata.json RESTORED=$restored_marker python3 - <<'PY'
import json,os,pathlib
want=json.loads(pathlib.Path(os.environ['META']).read_text(encoding='utf-8'))['migrationMarker']
try: rows=json.loads(pathlib.Path(os.environ['RESTORED']).read_text(encoding='utf-8'))
except (OSError,TypeError,ValueError) as e: raise SystemExit(f'invalid restored migration marker: {e}')
got=json.dumps(rows,separators=(',',':'),sort_keys=True,ensure_ascii=False)
if got!=want: raise SystemExit('restored migration marker differs')
PY
cli=${LKJMC_CLI:-}
if [ -z "$cli" ]; then
  if [ -x target/release/lkjmc ]; then cli=target/release/lkjmc
  elif command -v lkjmc >/dev/null; then cli=$(command -v lkjmc)
  else echo 'LKJMC_CLI or a built lkjmc is required to migrate restored data' >&2; exit 1; fi
fi
LKJMC_DATABASE_URL=$url "$cli" db migrate >/dev/null
psql "$url" -X --quiet --no-align --tuples-only -v ON_ERROR_STOP=1 \
  -c 'select count(*) from schema_migrations' >/dev/null
printf 'restore applied and migrated: %s\n' "$in"
