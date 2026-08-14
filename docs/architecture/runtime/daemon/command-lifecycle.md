# Command lifecycle

## Purpose

Define the fail-closed daemon command boundary. It accepts bounded observations,
journalled PostgreSQL desired-state writes, and one local-only desired-network
apply backed by the existing fenced runtime and durable attempt history. It does
not claim external exactly-once behavior.

## Status

implemented

## Classification

Every registered command has one checked `effect` class in its command shard.
The dispatcher validates that class after authorization and closed-body
validation, then decides before it invokes a handler.

| Class | Commands | Result |
| --- | --- | --- |
| `local-observation` | `admin.role.list` | Run a bounded local observation. |
| `runtime-observation` | `bootstrap.plan`, `bootstrap.status`, `bootstrap.doctor` | Inspect configured files, listeners, fenced runtime identity, and durable network state. |
| `network-apply` | `bootstrap.apply` | From a local Unix peer only, durably reconcile the closed configured network and wait for readiness. |
| `postgresql-read` | `player.settings.get`, `status` | Run bounded PostgreSQL reads only. |
| `postgresql-desired-set` | `player.settings.set`, `player.settings.hud` | Commit one atomic desired-state write, then report that row only. |
| `restart-required` | `config.reload` | Return non-success `config.restart_required`; no config is read or applied. |
| `denied-unproved` | Every other registration | Return non-success `command.effect_denied`; no handler runs. |

The checked registry contains 134 commands: 1 local observation, 3 runtime
observations, 1 network apply, 2 database reads, 2 desired-state writes, 1
restart-required request, and 124 denials.
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
bounds authentication, body decoding, ordinary command responses, and web
rendering. After a valid local Unix request is decoded as `bootstrap.apply`, its
dispatch alone receives a 20-minute monotonic budget under the same admission
lease; remote/TCP subjects and every other command retain eight seconds. The
Unix route timeout is longer than the apply budget. Readiness uses the remaining
apply budget and reserves time for durable terminal bookkeeping. The worker
tracker records a pending worker before `spawn_blocking` is submitted,
attaches its returned `JoinHandle` to that record, and releases the worker only
after that attachment. A record is removed only after exactly one await observes
its handle; a join failure remains an observed worker failure, never a detached
panic. Timeout and caller cancellation linearize as a deadline response plus
retained registration: they cannot detach work or release the lease while work
runs. A result observed strictly before the instant may reply; at the instant or later
the reply is `command.deadline_exceeded` and does not claim an effect completed.
Once a command envelope has been decoded, worker and outer-admission deadline
responses retain its validated `requestId`; correlation fallbacks are used only
when no validated identifier is technically known.

Pool connections start with the eight-second request ceiling. Before each
request database operation, its worker derives pool checkout, lock, and statement
limits from the remaining budget, recalculates after checkout, and applies lock
and statement limits before the handler query. The setup statement inherits a
prior bounded ceiling; the handler statement receives only what auth, audit, and
earlier work left. PostgreSQL `QUERY_CANCELED` and `LOCK_NOT_AVAILABLE` SQLSTATEs
remain `StoreError` deadline signals through credential lookup and transport, so
they normalize structurally to `command.deadline_exceeded`, never `auth.denied`;
messages are not classified by text. A command-handler deadline retains the
HTTP 200 command-envelope contract with a non-success structured code. TCP
authentication and web route deadlines use HTTP 408 with that same code. `status`
makes its four counts in one aggregate statement. No registered worker or SQL is intentionally detached. A database
timeout never produces a successful status body. The only admitted filesystem,
listener, and process effects are the explicit local network apply; plugin
heartbeat, player transfer, and generic lifecycle effects remain unavailable.

Rejection and pre-admission cancellation start no work. A dropped client retains
its admitted lease until its registered worker exits; that worker remains
registered until its handle has been joined.

## Desired-state operation journal

Every admitted `player.settings.set` and `player.settings.hud` mutation that
reaches its PostgreSQL write boundary enters one transaction, acquires a
transaction-scoped advisory lock for the client `requestId`, and inserts its
`requested` command row. The desired row and exactly one `succeeded` journal
update commit in that transaction; failure to update the journal rolls back the
desired row. A database failure is recorded as `failed`; a statement or request
deadline is recorded as `cancelled` when PostgreSQL remains reachable. The
worker does not intentionally finish with `requested`. If terminalization
cannot reach PostgreSQL, a later identical replay marks an interrupted durable
row failed rather than treating it as running or successful.

A mutation panic unwinds through PostgreSQL transaction drop, which synchronously
rolls back the insert and mutation and releases the transaction-scoped lock
before the pooled session can be reused. There is no manually released session
lock. The daemon observes the worker `Join` failure and returns non-success with
the original correlation; an identical retry on another connection may then
produce and replay an honest terminal outcome without waiting on a leaked lock.

A same-actor replay with identical command and JSON body returns the stored
terminal response without applying the mutation again. Reuse with a different
actor, command, or body fails closed as `request.id_conflict`. The store exposes
lookup by request ID, so a caller that timed out can correlate the eventual
durable result. Failure before journal admission has no durable outcome and no
desired-row mutation. These guarantees cover only the named PostgreSQL
transaction; they do not claim external exactly-once behavior.

## Configuration and shutdown

All main-config fields are restart-required. `config.reload` is deliberately
non-success because listener, token, runtime, roots, and pool changes do not
all reload atomically. Shutdown first closes shared admission, then stops
listener acceptance, and joins every registered blocking worker, including
authentication, denial audit, and web work. It never starts or reports an
external completion during shutdown. The deployed systemd unit owns restart
recovery by running a local bootstrap reapply after the new daemon socket is
available; a failed reapply makes service startup fail rather than leaving a
false ready unit.

## Verification

`scripts/check-command-lifecycle.py` runs: `effect-classes-enforced`,
`queues-bounded`, `timeout-outcome-pass`, `duplicate-mutations-pass`,
`config-apply-truthful`, `shutdown-pass`, `reactor-clean`, and
`command-load-budget`. Its worker probes cover pre-registration, completed-handle
observation, outer cancellation, timeout cleanup, and shutdown joining. Its
real-PostgreSQL mutation probes require the original request ID to reach a
queryable terminal timeout outcome, verify stable and conflicting replay, and
panic after journal insertion before retrying on another pooled connection to
reject leaked locks or `requested` rows. Credential and TCP/web route
probes reject SQLSTATE deadline laundering into authentication denial or
plaintext web timeouts. Its structural
check rejects dropped request handles, untracked Discord interaction listeners,
and fixed request SQL limits. The load probe saturates command, TCP-auth, and web
entry paths. PostgreSQL probes require a real `LKJMC_STORE_TEST_DATABASE_URL`. A named probe
or ordinary `--all` run fails nonzero when that prerequisite is absent. Only an
intentional aggregate host run may use `--all --allow-database-skip`; that mode
prints every skipped database probe ID. `verify-full.sh` opts into that mode
explicitly only when its host environment omits the URL. The Compose verify
profile supplies PostgreSQL and runs without skip permission.
