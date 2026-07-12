# Command lifecycle

## Purpose

Define the fail-closed daemon command boundary. It accepts only locally
provable observations and PostgreSQL-only desired-state writes; it does not
adopt an executor, journal, actor, lease, broker, reconciliation history, or
external-effect completion claim.

## Status

planned

## Classification

Every registered command has one checked `effect` class in its command shard.
The dispatcher validates that class after authorization and closed-body
validation, then decides before it invokes a handler.

| Class | Commands | Result |
| --- | --- | --- |
| `local-observation` | `admin.role.list`, `status` | Run as a bounded local observation. |
| `postgresql-read` | `player.settings.get` | Run one bounded PostgreSQL read. |
| `postgresql-desired-set` | `player.settings.set`, `player.settings.hud` | Commit one atomic desired-state write, then report that row only. |
| `restart-required` | `config.reload` | Return non-success `config.restart_required`; no config is read or applied. |
| `denied-unproved` | Every other registration | Return non-success `command.effect_denied`; no handler runs. |

The checked registry contains 137 commands: 2 local observations, 1 database
read, 2 desired-state writes, 1 restart-required request, and 131 denials.
The shard checker rejects an unclassified class or a deadline/idempotency value
that disagrees with its class.

## Admission and deadline

The transport admits at most eight requests into blocking workers and keeps no
application queue. A ninth request returns non-success `command.queue_full`
before a worker or handler starts. A permit remains held until its worker exits,
even if the requester disconnects or the response deadline expires.

Only admitted PostgreSQL work may run after the reactor boundary. Pool checkout,
lock, and statement limits are shorter than the eight-second command deadline.
A statement timeout is normalized to `command.deadline_exceeded`; this says no
completion result is available, not that an external effect was cancelled.
There are no admitted filesystem, network, process, plugin, proxy, transfer, or
observer effects.

Cancellation is therefore bounded and truthful: rejection and pre-admission
cancellation start no work; a running admitted SQL statement ends through its
PostgreSQL timeout or normal completion. A dropped client never turns an
unknown result into success.

## Duplicate writes

`player.settings.set` and `player.settings.hud` use a single PostgreSQL
statement to upsert the identity and named desired setting. Repeating the same
body leaves one settings row with the same declared value. This is a
row-specific desired-state property, not request replay, an external
idempotency promise, or operation history.

## Configuration and shutdown

All main-config fields are restart-required. `config.reload` is deliberately
non-success because listener, token, runtime, roots, and pool changes do not
all reload atomically. Shutdown stops admission and listener acceptance; the
transport waits only for already admitted local/database workers. It never
starts or reports an external completion during shutdown.

## Verification

`scripts/check-command-lifecycle.py` runs: `effect-classes-enforced`,
`queues-bounded`, `timeout-outcome-pass`, `duplicate-mutations-pass`,
`config-apply-truthful`, `shutdown-pass`, `reactor-clean`, and
`command-load-budget`. PostgreSQL probes require
`LKJMC_STORE_TEST_DATABASE_URL`; the Compose verify profile supplies it.
