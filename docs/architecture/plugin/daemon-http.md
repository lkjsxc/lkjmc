# Daemon HTTP

## Purpose

This document defines the containment boundary for daemon HTTP credentials and
Java plugins.

## Status

implemented

## Current boundary

The daemon HTTP endpoint remains loopback-only and bearer protected. CLI and web
use command routes. Java plugins may use only the read-only `/sync/snapshot` and
`/sync/feed` routes through the shared common coordinator and an
`lkjmc.sync.read` credential. Sync requests share daemon admission, authentication,
response-size, and database deadlines.

The credential is an explicit coordinator construction generation. It is never
read from a token file per request. Replacement cancels in-flight work and
clears cache; daemon credential-revision mismatch forces the same repair. Cursor
persistence is caller-owned through an opaque checkpoint and never performs file
I/O from a scheduler callback.

## Local-safe plugins

Paper retains `/menu`, `/docs`, hotbar, and bundled docs UI. Velocity retains
MOTD and tab-list presentation. Either may expose a revisioned read-only view,
but neither sends a daemon command or treats cached data as authorization.

## Withdrawn boundary

No configuration, token, cached grant, or sync payload re-enables mutations,
claim enforcement, dynamic menu actions, profile application, player save/load,
or transfer. Those paths still require trusted identity/session attestation.

## Verification

The real Java 21 HTTP harness covers loss, reorder, cursor reload, restart,
credential change, outage, bounds, cancellation, nonblocking lifecycle
submission, and off-scheduler shutdown await. Containment rejects withdrawn
command, registry, mutation, transfer, and player-application bridges in source
and built jars.
