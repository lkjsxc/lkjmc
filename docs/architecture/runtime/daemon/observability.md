# Daemon observability

## Purpose

Define the bounded product event, metric, health, and correlation contract without
claiming an independent observer.

## Status

implemented

## Event envelope

Every admitted command and typed adapter diagnostic uses one closed envelope:

- a server-assigned UUID `eventId` unique to that execution plus UTC `timestamp`,
  enumerated `severity`, `component`, and `eventKind`;
- nonempty `requestId`, `operationId`, and `correlationId` when that boundary has
  them; `requestId` is correlation data and is never an event or operation key;
- authenticated `actorKind`, bounded `actorName`, and enumerated `surface`;
- enumerated `outcome`, optional bounded `errorClass`, and a bounded attribute
  map with allowlisted keys and scalar values.

The daemon is the local event source. A JVM event identifies that local source
and its bounded server ID. No event is called independent, external, or
attested unless a future owner contract proves that provenance. Command HTTP,
web, runtime, network, sync, and JVM clients preserve supplied IDs and generate
a UUID only for an absent ID at their originating boundary.

## Durability and diagnostics

PostgreSQL is the sole durable event and operation store. State-changing
transactions write their operation terminal outcome and every execution event in
the same transaction as durable state where that store boundary supports it.
Reusing a client request ID cannot collapse distinct command outcomes. An exact
replay may retain an explicit operation ID, but every replay attempt has a new
server event ID. Events for rejected requests and non-database observations are
separate local diagnostic facts. Query filters are request, operation, or
correlation ID, with a maximum 500 events, a bounded age window, and database
retention maintenance.

The process writes the same envelope as one JSON object per diagnostic line.
Serialization or diagnostic persistence failure never changes a successful
state mutation into a fabricated outcome; it is reported as a local diagnostic
failure. Sensitive or unbounded event fields are dropped or replaced, and the
bounded retained event carries `attributes.redacted=true`; raw secrets never
become event attributes.

## Metrics

The in-process registry has fixed names for request admission, database,
runtime, sync, JVM, and support-bundle work. Labels are closed enums such as
component, operation class, outcome, and latency bucket. Player, instance,
request, operation, correlation, token, session, and arbitrary error strings
are forbidden labels. Export is available only on Unix transport or an
authenticated loopback TCP request, and output has a fixed series cap.

## Health

`/health/live` reports only that the process event loop is alive; its build
object identifies the reporting bytes and does not broaden that health claim.
Readiness is a separate non-success-capable check covering the PostgreSQL
connection and migration head, open admission, maintenance worker state,
required runtime capabilities,
and sync retention. A listening TCP socket alone is never readiness. The status
command exposes these dimensions rather than collapsing them to one healthy
boolean.

## Shutdown and limits

Event queues are bounded and fail visibly on overflow. JVM adapters enqueue
typed diagnostics off scheduler and event callbacks; no callback waits on HTTP,
PostgreSQL, filesystem, or process work. Shutdown closes admission, cancels
producers, and drains with a fixed deadline.

## Source and proof

The envelope and validation live under `crates/lkjmc-core/src/observability/`.
PostgreSQL operations, events, bounded queries, and retention are owned by
`crates/lkjmc-store/src/observability/`, `migrations/050-observability.sql`, and
`migrations/051-observability-attempt-identity.sql`.
Daemon routes, readiness, metrics, correlation, and support collection live
under `crates/lkjmc-daemon/src/observability/` and `support/bundle/`; JVM local
emitters have bounded off-callback queues.

Deterministic probes are `correlation-pass`, `fault-diagnostics-pass`,
`metrics-bounded`, `support-bundle-pass`, `secret-canary-pass`, and
`overhead-budget`. Correlation repeats the real PostgreSQL/daemon HTTP path 30
times and records that all observations are local-source facts. It does not
satisfy the research `B-O` independent-attested-observer gate.
