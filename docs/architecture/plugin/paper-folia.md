# Paper and Folia plugin

## Purpose

This document defines the Paper/Folia adapter boundary for menus, JVM sync, and readiness heartbeat.

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

After platform installation finishes, the common runtime starts one dedicated heartbeat reporter. It reads no Bukkit entity or region state and performs its bounded filesystem/loopback HTTP work only on its own daemon thread. Every ten seconds it sends an empty request under a three-second deadline using the instance-bound Paper credential. A committed heartbeat means this plugin lifecycle reached installation; stale data after 30 seconds fails readiness closed.

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
UI or chat. Heartbeat tests cover exact loopback targeting, empty-body bearer requests, retry after outage, secret redaction, and lifecycle shutdown. It is deterministic integration evidence until the exact plugin reports fresh heartbeat from both live Folia backends; it is never Minecraft-client proof.
