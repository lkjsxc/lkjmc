#!/bin/sh
set -eu
url=${LKJMC_DATABASE_URL:-}; out=${1:-}
[ -n "$url" ] || { echo 'LKJMC_DATABASE_URL is required' >&2; exit 1; }
[ -n "$out" ] || { echo 'usage: scripts/backup-postgres.sh OUT.dump' >&2; exit 1; }
command -v psql >/dev/null && command -v pg_dump >/dev/null && command -v pg_restore >/dev/null || {
  echo 'psql, pg_dump, and pg_restore are required' >&2; exit 1;
}
umask 077
dir=$(dirname "$out"); base=$(basename "$out"); mkdir -p "$dir"
tmp=$(mktemp -d "$dir/.backup.XXXXXX"); in=$tmp/in; result=$tmp/result
mkfifo "$in" "$result"
dump=$tmp/$base; manifest=$tmp/$base.manifest; metadata=$tmp/$base.metadata.json
marker=$tmp/migrations.json; schema=$tmp/schema.sql
cleanup() {
  code=$?
  [ -z "${sql_pid:-}" ] || { printf 'ROLLBACK;\n\\q\n' >&3 2>/dev/null || true; wait "$sql_pid" 2>/dev/null || true; }
  exec 3>&- 4<&- 2>/dev/null || true
  rm -rf "$tmp"; exit "$code"
}
trap cleanup EXIT HUP INT TERM
psql "$url" -X --quiet --no-align --tuples-only -v ON_ERROR_STOP=1 <"$in" >"$result" & sql_pid=$!
exec 3>"$in"; exec 4<"$result"
printf '%s\n' 'BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;' >&3
printf '%s\n' 'SELECT pg_export_snapshot();' >&3; IFS= read -r snapshot <&4
printf '%s\n' 'SELECT pg_current_wal_lsn();' >&3; IFS= read -r lsn <&4
printf '%s\n' "SELECT current_setting('server_version_num');" >&3; IFS= read -r server <&4
printf '%s\n' "SELECT coalesce(jsonb_agg(jsonb_build_object('version',version,'name',name,'checksum',coalesce(checksum,'')) ORDER BY version),'[]'::jsonb)::text FROM schema_migrations;" >&3
IFS= read -r migrations <&4
[ -n "$snapshot" ] && [ -n "$lsn" ] && [ -n "$server" ] && [ -n "$migrations" ] || {
  echo 'snapshot marker missing' >&2; exit 1;
}
printf '%s\n' "$migrations" >"$marker"
server_major=$((server / 10000))
tool_major() { "$1" --version | python3 -c "import re,sys; m=re.search(r'([0-9]+)(?:\\.[0-9]+)',sys.stdin.read()); print(m.group(1) if m else 'invalid')"; }
[ "$(tool_major psql)" = "$server_major" ] && [ "$(tool_major pg_dump)" = "$server_major" ] || {
  echo "PostgreSQL backup tool major differs from server $server_major" >&2; exit 1;
}
pg_dump --format=custom --no-owner --snapshot="$snapshot" --file="$dump" "$url"
printf 'COMMIT;\n\\q\n' >&3; wait "$sql_pid"; sql_pid=
pg_restore --list "$dump" >"$manifest"
pg_restore --schema-only --file="$schema" "$dump"
commit=${LKJMC_SOURCE_COMMIT:-$(git rev-parse HEAD 2>/dev/null || printf unknown)}
DUMP=$dump MANIFEST=$manifest META=$metadata COMMIT=$commit LSN=$lsn \
SERVER=$server MARKER=$marker SCHEMA=$schema python3 - <<'PY'
import hashlib,json,os,pathlib,re
p=lambda n:pathlib.Path(os.environ[n])
h=lambda raw:hashlib.sha256(raw).hexdigest()
def file_hash(n): return h(p(n).read_bytes())
def schema_hash():
 raw=p('SCHEMA').read_bytes().splitlines(keepends=True)
 return h(b''.join(line for line in raw if not line.startswith((b'\\restrict ',b'\\unrestrict '))))
raw=p('MARKER').read_text(encoding='utf-8')
if '\n' in raw.rstrip('\n') or not raw.endswith('\n'): raise SystemExit('migration marker is not one line')
try: rows=json.loads(raw)
except (TypeError,ValueError) as e: raise SystemExit(f'invalid migration marker: {e}')
if not isinstance(rows,list): raise SystemExit('migration marker is not an array')
last=-1
for row in rows:
 if not isinstance(row,dict) or set(row)!={'version','name','checksum'}: raise SystemExit('migration marker fields differ')
 version=row['version']
 if isinstance(version,bool) or not isinstance(version,int) or version<=last: raise SystemExit('migration versions are not ordered unique integers')
 if not isinstance(row['name'],str) or not row['name']: raise SystemExit('invalid migration name')
 if not isinstance(row['checksum'],str) or not re.fullmatch(r'(?:[0-9a-f]{64})?',row['checksum']): raise SystemExit('invalid migration checksum')
 last=version
canonical=json.dumps(rows,separators=(',',':'),sort_keys=True,ensure_ascii=False)
lsn=os.environ['LSN']
if not re.fullmatch(r'[0-9A-F]+/[0-9A-F]+',lsn): raise SystemExit('invalid WAL LSN')
data={'schemaVersion':1,'sourceCommit':os.environ['COMMIT'],
 'postgresServerVersion':int(os.environ['SERVER']),'lsn':lsn,'lsnSha256':h(lsn.encode()),
 'schemaSha256':schema_hash(),'migrationMarker':canonical,
 'migrationSha256':h(canonical.encode()),'dumpSha256':file_hash('DUMP'),
 'manifestSha256':file_hash('MANIFEST')}
p('META').write_text(json.dumps(data,indent=2,sort_keys=True)+'\n',encoding='utf-8')
PY
checks=$tmp/$base.sha256
(cd "$tmp" && sha256sum "$base" "$base.manifest" "$base.metadata.json" >"$base.sha256")
for suffix in '' .manifest .metadata.json .sha256; do
  [ ! -e "$out$suffix" ] || { echo "refusing existing output: $out$suffix" >&2; exit 1; }
done
mv "$dump" "$out"; mv "$manifest" "$out.manifest"
mv "$metadata" "$out.metadata.json"; mv "$checks" "$out.sha256"
trap - EXIT HUP INT TERM; rm -rf "$tmp"
printf 'backup written: %s\n' "$out"
