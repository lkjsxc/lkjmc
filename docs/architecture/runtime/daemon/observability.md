# Daemon observability

## Purpose

Define the bounded product event, metric, health, and correlation contract without
claiming an independent observer.

## Status

planned

## Event envelope

Every admitted command and typed adapter diagnostic uses one closed envelope:

- UTC `timestamp`, enumerated `severity`, `component`, and `eventKind`;
- nonempty `requestId`, `operationId`, and `correlationId` when that boundary has
  them, preserved rather than re-derived;
- authenticated `actorKind`, bounded `actorName`, and enumerated `surface`;
- enumerated `outcome`, optional bounded `errorClass`, and a bounded attribute
  map with allowlisted keys and scalar values.

The daemon is the local event source. A JVM or Discord event identifies that
local source and its bounded server ID. No event is called independent,
external, or attested unless a future owner contract proves that provenance.
Command HTTP, web, runtime, network, sync, Discord, and JVM clients preserve
supplied IDs and generate a UUID only for an absent ID at their originating
boundary.

## Durability and diagnostics

PostgreSQL is the sole durable event and operation store. State-changing
transactions write their operation terminal outcome and event in the same
transaction as durable state where that store boundary supports it. Events for
rejected requests and non-database observations are separate local diagnostic
facts. Query filters are request, operation, or correlation ID, with a maximum
500 events, a bounded age window, and database retention maintenance.

The process writes the same envelope as one JSON object per diagnostic line.
Serialization or diagnostic persistence failure never changes a successful
state mutation into a fabricated outcome; it is reported as a local diagnostic
failure. Secret values and unbounded payloads are never event attributes.

## Metrics

The in-process registry has fixed names for request admission, database,
runtime, sync, JVM, and support-bundle work. Labels are closed enums such as
component, operation class, outcome, and latency bucket. Player, instance,
request, operation, correlation, token, session, and arbitrary error strings
are forbidden labels. Export is available only on Unix transport or an
authenticated loopback TCP request, and output has a fixed series cap.

## Health

`/health/live` reports only that the process event loop is alive. Readiness is a
separate non-success-capable check covering PostgreSQL connection and migration
head, open admission, maintenance worker state, required runtime capabilities,
and sync retention. A listening TCP socket alone is never readiness. The status
command exposes these dimensions rather than collapsing them to one healthy
boolean.

## Shutdown and limits

Event queues are bounded and fail visibly on overflow. JVM and Discord adapters
enqueue typed diagnostics off scheduler and event callbacks; no callback waits
on HTTP, PostgreSQL, filesystem, or process work. Shutdown closes admission,
cancels producers, and drains with a fixed deadline.

## Proof boundary

Deterministic probes are `correlation-pass`, `fault-diagnostics-pass`,
`metrics-bounded`, `support-bundle-pass`, `secret-canary-pass`, and
`overhead-budget`. Correlation repeats the real PostgreSQL/daemon HTTP path 30
times and records that all observations are local-source facts. It does not
satisfy the research `B-O` independent-attested-observer gate.
