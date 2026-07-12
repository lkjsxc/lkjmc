# Command lifecycle

## Purpose

Define the fail-closed daemon command boundary. It accepts only locally
provable observations and PostgreSQL-only desired-state writes; it does not
adopt an executor, journal, actor, lease, broker, reconciliation history, or
external-effect completion claim.

## Status

implemented

## Classification

Every registered command has one checked `effect` class in its command shard.
The dispatcher validates that class after authorization and closed-body
validation, then decides before it invokes a handler.

| Class | Commands | Result |
| --- | --- | --- |
| `local-observation` | `admin.role.list` | Run a bounded local observation. |
| `postgresql-read` | `player.settings.get`, `status` | Run bounded PostgreSQL reads only. |
| `postgresql-desired-set` | `player.settings.set`, `player.settings.hud` | Commit one atomic desired-state write, then report that row only. |
| `restart-required` | `config.reload` | Return non-success `config.restart_required`; no config is read or applied. |
| `denied-unproved` | Every other registration | Return non-success `command.effect_denied`; no handler runs. |

The checked registry contains 137 commands: 1 local observation, 2 database
reads, 2 desired-state writes, 1 restart-required request, and 131 denials.
The shard checker rejects an unclassified class or a deadline/idempotency value
that disagrees with its class.

## Admission and deadline

One shared lease admits at most eight supported requests and keeps no application
queue. It is acquired before TCP credential authentication, Unix-peer denial
audit, body decoding, command dispatch, or any `/web` route work. A ninth request
returns non-success `command.queue_full` before a worker, audit, auth lookup, or
handler starts. Every blocking request action reuses that lease; it must not
acquire a second permit or spawn detached work. The lease remains held until all
of its blocking work exits, including after a client disconnect or deadline reply.

The eight-second deadline is the monotonic instant captured at admission and
bounds the entire response, including authentication and web rendering. Every
blocking action registers its handle before it can await; the registration owns
that handle until normal completion or shutdown joins it. Timeout and caller
cancellation linearize as a deadline response plus retained registration: they
cannot detach work or release the lease. A result observed strictly before the
instant may reply; at the instant or later the reply is
`command.deadline_exceeded` and does not claim an effect completed.

Before each request database connection, its worker derives connect, lock, and
statement limits from the remaining budget and passes lock and statement limits
to PostgreSQL at backend startup. A later operation therefore receives only what
the auth, audit, and earlier work left. PostgreSQL `QUERY_CANCELED` and
`LOCK_NOT_AVAILABLE` SQLSTATEs normalize structurally to
`command.deadline_exceeded`; messages are not classified by text. `status`
makes its four counts in one aggregate statement. Backend cancellation and
scheduler latency (with less than one millisecond rounding) are the residual
physical cleanup bound; no registered worker or SQL is intentionally detached.
A database timeout never produces a successful status body. There are no
admitted filesystem, network, process, plugin, proxy, transfer, or observer
effects.

Rejection and pre-admission cancellation start no work. A dropped client retains
its admitted lease until its registered worker has exited and been joined.

## Duplicate writes

`player.settings.set` and `player.settings.hud` use a single PostgreSQL
statement to upsert the identity and named desired setting. Repeating the same
body leaves one settings row with the same declared value. This is a
row-specific desired-state property, not request replay, an external
idempotency promise, or operation history.

## Configuration and shutdown

All main-config fields are restart-required. `config.reload` is deliberately
non-success because listener, token, runtime, roots, and pool changes do not
all reload atomically. Shutdown first closes shared admission, then stops
listener acceptance, and joins every registered blocking worker, including
authentication, denial audit, and web work. It never starts or reports an
external completion during shutdown.

## Verification

`scripts/check-command-lifecycle.py` runs: `effect-classes-enforced`,
`queues-bounded`, `timeout-outcome-pass`, `duplicate-mutations-pass`,
`config-apply-truthful`, `shutdown-pass`, `reactor-clean`, and
`command-load-budget`. Its worker probes cover outer cancellation, timeout
cleanup, shutdown joining, and auth-budgeted PostgreSQL cancellation; its
structural check rejects dropped request handles and fixed request SQL limits.
The load probe saturates command, TCP-auth, and web entry paths. PostgreSQL probes require
`LKJMC_STORE_TEST_DATABASE_URL`; the Compose verify profile supplies it.
