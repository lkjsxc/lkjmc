# Paper and Folia plugin

## Purpose

This document defines the Paper/Folia adapter boundary for menus and JVM sync.

## Status

implemented

## Responsibilities

Paper registers `/menu` and `/docs`, maintains the hard-locked slot-8 entrypoint,
and owns one common JVM runtime. One menu listener and renderer serve all 62
routes and the curated docs browser. The adapter subscribes to typed menu,
permission, claim, and settings snapshots needed by sessions.

Paper/Folia ownership hops are explicit. A menu response uses the player entity
scheduler, not the global scheduler, and revalidates adapter, generation,
locale, route, session, and request on that ownership stage before any Bukkit
inventory or chat effect. Global scheduling is reserved for global-safe work.
Callbacks never wait on HTTP, database, filesystem, process, download, or
worker futures.

## Authority

Current permission and claim snapshots are hints, not final authorization.
Mutation requires an exact current capability plus trusted session attestation.
The menu task adds no daemon mutation port, so an admitted mutation is reported
unsupported and never as success.

## Verification

The disposable protocol-like inventory harness drives production adapter code
for open, click, navigation, close, stale response, outage, locale, repeated
clicks, disconnect, and shutdown. Delayed completions are tested after close,
different-route reopen, locale change, disconnect, and disable; none may mutate
UI or chat. It is deterministic integration evidence, not a live Paper server,
Folia server, or Minecraft client. External proof remains a guarded lane.
