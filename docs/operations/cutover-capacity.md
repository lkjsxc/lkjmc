# Cutover and capacity

## Purpose

Define evidence-backed cutover, rollback, and capacity boundaries.

## Status

implemented

## Cutover rehearsal

External prerequisites are an approved change window, private source and target
PostgreSQL URLs, enough storage for two copies, the pinned release manifest, and
an operator able to stop all writers. Before production use, run:

```sh
LKJMC_DATABASE_URL="$SOURCE_URL" scripts/backup-postgres.sh cutover.dump
LKJMC_DATABASE_URL="$FRESH_URL" scripts/restore-postgres.sh cutover.dump
lkjmc --socket "$LAB_SOCKET" db status
lkjmc --socket "$LAB_SOCKET" doctor
```

Retain the backup metadata/checksums, exact exits, daemon readiness response,
query result, start/end LSN, elapsed downtime, config fingerprints without
secret values, and cleanup. Stop source writers, take the final backup, validate
the fresh target, switch one private config reference, restart, and observe
before admitting writes. DNS, load-balancer, firewall, cloud database, and player
routing changes are external and require their own authorized evidence.

## Rollback

Rollback is triggered by failed checksum/version/migration, daemon readiness,
status query, error budget, or post-switch observation. Stop writers, restore the
old private config reference, restart the prior pinned artifact, verify status
and a read query, and retain the failed target for investigation. Do not reverse
migrations in place, restore over the only database, or delete either side until
the incident record closes.

## Capacity evidence

A supported envelope requires a named commit and manifest, PostgreSQL and host
versions, CPU/memory/storage limits, workload seed, request mix, concurrency,
duration, queue depth, latency/error samples, database connections and locks,
filesystem usage, and cleanup. Use the real daemon, PostgreSQL, process and
network boundaries; do not substitute parser loops or fixture-only traffic.

No production capacity number is currently published. The operations fault lab
is correctness evidence only. Before changing that statement, retain at least
30 steady samples, saturation and recovery observations, one injected database
and process loss, and a rerun with no hidden retries. A missing workload driver,
external client, or authorized endpoint is an explicit prerequisite, not zero
load and not a pass.
