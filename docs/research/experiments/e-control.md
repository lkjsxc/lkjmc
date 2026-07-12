# E-CONTROL execution comparison

## Purpose

This research-only hypothesis compares disposable execution adapters. It does
not change daemon behavior, register a controller, or select an implementation.

## Baseline and candidates

At `d20e5e532db9d3a5577f567dd6a5a24fdc51eea1`, HTTP dispatch uses
`spawn_blocking` and PostgreSQL access is synchronous through r2d2. Candidates
are baseline synchronous work, bounded keyed workers, concurrent bounded async
workers, async work plus a bounded effect-child pool, bounded journaling, and
sharded keyed async actors with journaling.

## Slice and invariants

A real disposable PostgreSQL slice writes operation, journal, effect-attempt,
and effect-completion rows and runs `true` as the child effect. It warms 40
requests then concurrently admits 216 requests: 200 distinct IDs and 16
interleaved duplicate IDs. The harness requires exactly 200 operation, effect
attempt, and effect rows, 16 suppressed duplicates, and 200 journal rows for a
journal candidate. It verifies same-key serialization and overlapping distinct
keys; actor messages are key-sharded and explicitly test both properties.

## Active admission proof

Each bounded candidate has a capacity-eight admission point. The bounded sync
pool, async ingress pool, mixed ingress and effect pools, and actor mailbox are
separately instantiated from their actual candidate implementation with their
consumer held behind a controlled gate. Each fills eight accepted jobs, makes a
ninth nonblocking `try_send`/`submit` reject, releases the gate, and drains real
PostgreSQL plus `true` work. This proves that component's local rejection and
drain behavior, not end-to-end production overload behavior. The mixed pool's
normal `submit` is nonblocking; only waiting for an admitted result is async.

## Faults and proofs

Each candidate sends a real PostgreSQL cancellation request after a 2 ms
deadline while `pg_sleep` precedes a write. It passes only when PostgreSQL
reports cancellation and the later count proves the write absent. Concurrent
duplicate requests must have one durable effect attempt per ID. A journal fault
uses the real durable intent and effect claim, starts `sleep`, then terminates
and reaps that launched child before an effect completion is durable. It records
one attempt and zero completions and deliberately does not retry it. This is a
controlled post-launch interruption, not proof of parent-process crash recovery
or exactly-once external effects.

## Disposable boundary

Both harnesses require `LKJMC_LAB_POSTGRES_DISPOSABLE=1`, a PostgreSQL URL with
an `lkjmc_lab_` database, and a loopback host. Host `postgres` is accepted only
with `LKJMC_E_CONTROL_COMPOSE=1`. The Rust harness waits for `SELECT 1`; the
config probe retries the real CLI migration until the database is healthy.
Neither opens a database or child before its safety gate.

## Limits

A post-launch crash before durable completion leaves external-effect ambiguity;
this harness intentionally leaves that row unresolved rather than asserting a
recovery. Actors are four key-sharded mailboxes, not a global actor or a
distributed fence. `kill -0` and `sleep` make child cleanup a Linux/Unix
laboratory proof. Config reload validates a changed runtime value by rejecting
zero, but status exposes no runtime memory value; it does not prove downstream
runtime application.

## Rerun

The independently runnable Compose command and the current blocked disposition
are in the [run](../runs/e-control-20260711.md) and
[decision](../decisions/e-control-20260711.md). Missing Docker or a database is
blocked evidence, never a candidate pass.
