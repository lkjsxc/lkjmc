# E-OBS observability candidates

## Purpose

Define the bounded E-OBS comparison before its harness exists. This research does
not add product instrumentation, commands, metrics, history, or support bundles.

## Catalog and baseline

Catalog candidates are `CP-OPS-JOURNAL`, `DW-RUNTIME-FLOW`, `OP-FAULT-LAB`,
`OP-SUPPORT-BUNDLE`, `OP-CAPACITY`, `SE-SUPPORT-REDACT`, and
`SE-AUDIT-DENIAL`. The baseline is the real local daemon HTTP lifecycle path:
`instance.create`, `instance.start`, `status`, and `instance.stop` against a
fresh disposable PostgreSQL database and a local child process. Current audit
history has action and target fields but no request correlation identifier.

## Hypothesis and variants

An external research observer can compare the same real path with: (1) stable
structured events, (2) those events plus fixed-label counters and latency
buckets, and (3) those records plus existing `audit.tail`, status, log excerpts,
and a redacted support-bundle manifest. The third variant should reduce
operator diagnosis time without retaining a secret. This is a harness-only
prototype, not a product adapter or operation journal.

## Invariants

The harness uses only an owned temporary root, a disposable Docker PostgreSQL
container, loopback HTTP, and a local child owned by the daemon. It must not
edit controller state, use a non-loopback database, add a daemon command, print
or commit secret values, or describe a withdrawn Java adapter as connected.

## Workload and measurements

Each variant runs warm-up plus five lifecycle repetitions with a fixed seed.
The harness records median and nearest-rank p95 workflow latency, observer
overhead relative to an unobserved baseline, event-field and metric-series
cardinality, fault diagnosis time, retained artifact bytes, and exact
secret-canary scan outcome. It preserves the client request ID, daemon response
ID, and observer event ID as distinct fields from their respective sources; it
never derives an event ID from a response ID. Current daemon HTTP has no
independently emitted observer event, so `correlation-end-to-end` is
`BLOCKED`/unsupported. Missing, blank, or mismatched copied IDs fail correlation; all three IDs
must be nonempty before any external-provenance correlation can pass. A
matching synthetic record cannot turn that blocked fact into `PASS`.

## Faults and evidence

The harness attempts PostgreSQL table-lock delay, client timeout followed by
actual daemon continuation, immediate local-child failure, rejected bearer
authentication, and a Velocity-actor disconnect/denial. It retains capped, sanitized artifacts under its owned temporary root and emits
a replay command. A disposable known-positive canary artifact must be detected
then removed before scanning every retained file, including the index, checksum,
and scan record.
A run and decision will record exact commands, exits, deviations, and blocked
facts. Base commit: `d20e5e532db9d3a5577f567dd6a5a24fdc51eea1`.
