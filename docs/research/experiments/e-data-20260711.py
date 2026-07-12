#!/usr/bin/env python3
"""Run bounded E-DATA research with current and private-schema evidence."""
import argparse
import json
import os
import sys
import tempfile
import uuid
from pathlib import Path

from e_data_20260711_support import (
    ROOT, SQL, archive, candidate_sql, cli, compose, digest, finish_sql, free_port,
    invoke, logged, probe, psql, query, record, restore, socket_command, start_daemon,
    start_sql, stop_daemon, wait_lock, wait_postgres,
)

LOCK = 7_526_470
PLAYER = "00000000-0000-4000-8000-000000000001"
PROBES = ("migration-lock-timeout", "current-profile-transfer-slice",
          "database-delay-current-profile", "typed-profile-integrity",
          "transfer-item-variants", "crash-rollback", "delay-reclaim",
          "stale-writer", "compensation", "revision-feed-invalidation",
          "private-schema-cutover-restore", "workflow-journal-plugin-ack")
SUPPORT = Path(__file__).with_name("e_data_20260711_support.py")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    raw = args.output or Path(tempfile.mkdtemp(prefix="lkjmc-e-data-"))
    if raw.exists() and any(raw.iterdir()):
        raise SystemExit("output must be new or empty")
    raw.mkdir(parents=True, exist_ok=True)
    result = {"base": invoke(["git", "rev-parse", "HEAD"], cwd=ROOT)[1].strip(), "seed": 20260711,
        "harnessSha256": digest(Path(__file__).read_bytes()), "supportSha256": digest(SUPPORT.read_bytes()),
        "candidateSqlSha256": digest(SQL.read_bytes()), "commands": [],
        "probes": {name: "BLOCKED" for name in PROBES}, "blocked": {}}
    schema = "edata_" + uuid.uuid4().hex[:12]
    project, port = "lkjmc_e_data_" + uuid.uuid4().hex[:10], free_port()
    compose_file = raw / "compose.yml"
    compose_file.write_text("services:\n  postgres:\n    image: postgres:16-alpine\n    environment:\n      POSTGRES_DB: lkjmc\n      POSTGRES_USER: lkjmc\n      POSTGRES_PASSWORD: lkjmc-research\n    ports:\n      - '127.0.0.1:%s:5432'\n" % port)
    base = (project, compose_file)
    env = os.environ | {"LKJMC_DATABASE_URL": f"postgres://lkjmc:lkjmc-research@127.0.0.1:{port}/lkjmc"}
    daemon = None
    try:
        code, _ = logged(result, raw, "build-research-binaries", ["cargo", "build", "--locked", "-q", "-p", "lkjmc-cli", "-p", "lkjmc-daemon"], cwd=ROOT, timeout=1800)
        if code:
            raise RuntimeError("research binaries did not build")
        code, _ = logged(result, raw, "compose-up", compose(base, "up", "-d"), cwd=ROOT)
        ready, output = wait_postgres(base)
        record(result, raw, "postgres-ready", ["pg_isready"], ready, output)
        if code or ready:
            raise RuntimeError("disposable PostgreSQL was unavailable")
        holder, holder_command = start_sql(base, f"select pg_advisory_lock({LOCK}); select pg_sleep(6); select pg_advisory_unlock({LOCK});")
        if not wait_lock(base, f"select not pg_try_advisory_lock({LOCK});"):
            raise RuntimeError("migration lock was not held")
        code, output = cli(result, raw, "migrate-lock-timeout", env, "--json", "db", "migrate", classification="expected-controlled-lock-timeout")
        finish_sql(result, raw, "migration-lock-holder", holder, holder_command, "controlled-cause")
        probe(result, "migration-lock-timeout", int(code == 0), output, "canceling statement due to lock timeout")
        code, _ = cli(result, raw, "migrate-after-lock-release", env, "--json", "db", "migrate", classification="post-conflict-success-not-resolution")
        if code:
            raise RuntimeError("current migration path failed after lock release")
        daemon, daemon_socket, daemon_command = start_daemon(result, raw, env)
        payload = raw / "profile.bin"
        payload.write_bytes(b"e-data-profile-v1")
        code, _ = cli(result, raw, "current-profile-snapshot", env, "--json", "--socket", str(daemon_socket), "player", "snapshot", PLAYER, "Research", "cli-restore", "--payload", str(payload))
        if code:
            raise RuntimeError("current profile snapshot failed")
        code, snapshot_id = query(result, raw, base, "current-profile-id", f"select id from player_profile_snapshots where player_uuid='{PLAYER}' and scope='profile' order by revision desc limit 1;")
        if code or not snapshot_id.strip():
            raise RuntimeError("current profile id was unavailable")
        code, _ = cli(result, raw, "current-profile-restore", env, "--json", "--socket", str(daemon_socket), "player", "restore", PLAYER, "--snapshot", snapshot_id.strip())
        if code:
            raise RuntimeError("current profile restore failed")
        code, output = socket_command(result, raw, "current-transfer-audit", daemon_socket, "player.transfer.saved", {"playerUuid": PLAYER})
        probe(result, "current-profile-transfer-slice", code, output, '"ok":true')
        current = f"select 'current-profile-transfer-slice=PASS' where (select count(*) from player_profile_snapshots where player_uuid='{PLAYER}' and scope='profile')=2 and (select max(revision) from player_profile_snapshots where player_uuid='{PLAYER}')=2 and exists(select 1 from audit_events where action='player.transfer.saved' and target_id='{PLAYER}' and result='succeeded');"
        code, output = query(result, raw, base, "current-profile-transfer-check", current)
        probe(result, "current-profile-transfer-slice", code, output, "current-profile-transfer-slice=PASS")
        delay, delay_command = start_sql(base, "begin; lock table player_profile_snapshots in access exclusive mode; select pg_sleep(6); commit;")
        if not wait_lock(base, "select exists(select 1 from pg_locks where relation='player_profile_snapshots'::regclass and mode='AccessExclusiveLock');"):
            raise RuntimeError("profile delay lock was not held")
        code, output = cli(result, raw, "database-delay-profile", env, "--json", "--socket", str(daemon_socket), "player", "snapshot", PLAYER, "Research", "cli-restore", "--payload", str(payload), classification="expected-current-lock-timeout")
        finish_sql(result, raw, "database-delay-holder", delay, delay_command, "controlled-cause")
        check, check_output = query(result, raw, base, "database-delay-check", f"select 'database-delay-current-profile=PASS' where (select count(*) from player_profile_snapshots where player_uuid='{PLAYER}' and scope='profile')=2;")
        probe(result, "database-delay-current-profile", int(not (code != 0 and check == 0)), output + check_output, "database-delay-current-profile=PASS")
        code, _ = query(result, raw, base, "candidate-schema", candidate_sql(schema))
        if code:
            raise RuntimeError("candidate schema failed")
        typed = """insert into %s.opaque_profiles values ('30000000-0000-4000-8000-000000000001',decode('00ff','hex')); insert into %s.typed_profiles values ('30000000-0000-4000-8000-000000000001',1,'{"formatVersion":1,"items":[],"selectedSlot":0}',%s.profile_sha('{"formatVersion":1,"items":[],"selectedSlot":0}'::jsonb)); do $$ begin begin insert into %s.typed_profiles values ('30000000-0000-4000-8000-000000000002',1,'{"formatVersion":2,"items":[],"selectedSlot":0}',%s.profile_sha('{"formatVersion":2,"items":[],"selectedSlot":0}'::jsonb)); raise exception 'bad shape accepted'; exception when check_violation then null; end; begin insert into %s.typed_profiles values ('30000000-0000-4000-8000-000000000003',1,'{"formatVersion":1,"items":[],"selectedSlot":0}','0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef'); raise exception 'bad checksum accepted'; exception when check_violation then null; end; end $$; select 'typed-profile-integrity=PASS';""" % ((schema,) * 6)
        code, output = query(result, raw, base, "typed-profile", typed)
        probe(result, "typed-profile-integrity", code, output, "typed-profile-integrity=PASS")
        flow = """insert into %s.weak_deliveries values ('40000000-0000-4000-8000-000000000001','pending'),('40000000-0000-4000-8000-000000000001','pending'); insert into %s.fenced_deliveries (correlation,state) values ('40000000-0000-4000-8000-000000000002','pending'); update %s.fenced_deliveries set state='claimed',holder='one',fence=1,claimed_at=now()-interval '1 minute' where fence=0; update %s.fenced_deliveries set holder='two',fence=2,claimed_at=now() where state='claimed' and claimed_at < now()-interval '1 second'; update %s.fenced_deliveries set state='acknowledged',acknowledged_at=now() where fence=2; with stale as (update %s.fenced_deliveries set state='acknowledged' where fence=1 returning correlation) select 'transfer-item-variants=PASS' where (select count(*) from %s.weak_deliveries)=2 and not exists(select 1 from stale); select 'delay-reclaim=PASS' where (select fence from %s.fenced_deliveries)=2; select 'stale-writer=PASS' where (select state from %s.fenced_deliveries)='acknowledged';""" % ((schema,) * 9)
        code, output = query(result, raw, base, "delivery-variants", flow)
        for name in ("transfer-item-variants", "delay-reclaim", "stale-writer"):
            probe(result, name, code, output, name + "=PASS")
        crash = "begin; insert into %s.fenced_deliveries values ('40000000-0000-4000-8000-000000000003','pending'); rollback; select 'crash-rollback=PASS' where not exists(select 1 from %s.fenced_deliveries where correlation='40000000-0000-4000-8000-000000000003');" % (schema, schema)
        code, output = query(result, raw, base, "crash-rollback", crash)
        probe(result, "crash-rollback", code, output, "crash-rollback=PASS")
        economy = """insert into points_accounts (player_uuid,balance) values ('%s',100); insert into points_ledger (id,player_uuid,delta,reason) values ('10000000-0000-4000-8000-000000000001','%s',100,'research.seed'); update points_accounts set balance=balance-25 where player_uuid='%s'; insert into points_ledger (id,player_uuid,delta,reason,correlation_id) values ('10000000-0000-4000-8000-000000000002','%s',-25,'research.spend','20000000-0000-4000-8000-000000000001'); update points_accounts set balance=balance+25 where player_uuid='%s'; insert into points_ledger (id,player_uuid,delta,reason,correlation_id) values ('10000000-0000-4000-8000-000000000003','%s',25,'research.refund','20000000-0000-4000-8000-000000000002'); select 'compensation=PASS' where (select balance from points_accounts where player_uuid='%s')=(select sum(delta) from points_ledger where player_uuid='%s');""" % ((PLAYER,) * 8)
        code, output = query(result, raw, base, "compensation", economy)
        probe(result, "compensation", code, output, "compensation=PASS")
        feed = """insert into %s.profile_heads values ('%s',0); with next as (update %s.profile_heads set revision=revision+1 where player_uuid='%s' returning revision) insert into %s.profile_events select '%s',revision,'{"formatVersion":1,"items":[],"selectedSlot":0}' from next; select 'revision-feed-invalidation=PASS' where (select count(*) from %s.profile_events where player_uuid='%s')=1;""" % (schema, PLAYER, schema, PLAYER, schema, PLAYER, schema, PLAYER)
        code, output = query(result, raw, base, "revision-feed", feed)
        probe(result, "revision-feed-invalidation", code, output, "revision-feed-invalidation=PASS")
        result["blocked"]["workflow-journal-plugin-ack"] = "No current transfer workflow joins a snapshot, journal correlation, and real plugin acknowledgement; player.transfer.saved only writes an audit event and Java adapters are withdrawn."
        stop_daemon(result, raw, daemon, daemon_command)
        daemon = None
        if archive(result, raw, base, schema)[0]:
            raise RuntimeError("private cutover dump failed")
        reset_env = env | {"LKJMC_TEST_RESET_DATABASE": "1"}
        if cli(result, raw, "reset-public-before-cutover", reset_env, "db", "reset-test")[0]:
            raise RuntimeError("public reset failed")
        if query(result, raw, base, "drop-public-private-before-restore", f"drop schema public cascade; create schema public; drop schema {schema} cascade;")[0]:
            raise RuntimeError("cutover drop failed")
        if restore(result, raw, base)[0]:
            raise RuntimeError("private cutover restore failed")
        if cli(result, raw, "status-after-cutover", env, "--json", "db", "status")[0]:
            raise RuntimeError("status after restore failed")
        check = f"select 'private-schema-cutover-restore=PASS' where (select count(*) from schema_migrations)=42 and (select count(*) from player_profile_snapshots where player_uuid='{PLAYER}')=2 and (select count(*) from {schema}.typed_profiles)=1 and (select state from {schema}.fenced_deliveries)='acknowledged' and (select balance from points_accounts where player_uuid='{PLAYER}')=(select sum(delta) from points_ledger where player_uuid='{PLAYER}');"
        code, output = query(result, raw, base, "private-cutover-check", check)
        probe(result, "private-schema-cutover-restore", code, output, "private-schema-cutover-restore=PASS")
    except Exception as error:
        result["error"] = str(error)
    finally:
        if daemon is not None:
            stop_daemon(result, raw, daemon, daemon_command)
        logged(result, raw, "compose-down", compose(base, "down", "--volumes", "--remove-orphans"), cwd=ROOT)
    (raw / "result.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"output": str(raw), "probes": result["probes"], "error": result.get("error")}, sort_keys=True))
    return int(bool(result.get("error")) or any(value == "FAIL" for value in result["probes"].values()))


if __name__ == "__main__":
    sys.exit(main())
