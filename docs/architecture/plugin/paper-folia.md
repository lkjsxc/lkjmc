# Paper and Folia plugin

## Purpose

This document defines the local-safe Paper and Folia plugin contract.

## Status

implemented

## Shipped responsibilities

- Register only `/menu` and `/docs`.
- Render bundled documentation and local page navigation.
- Maintain the hard-locked local slot-8 documentation token.
- Own one Java-common read-only sync coordinator for plugin lifecycle.
- Expose immutable revisioned views without applying them to players.

Scheduler callbacks never wait on HTTP, database, filesystem, download, or
process work. Paper owns no poll loop; subscription and cache work are common.

## Withdrawn responsibilities

Daemon-backed commands, profile application/save/load, live claim enforcement,
moderation, heartbeats, dynamic menu actions, and transfer bridges remain
withdrawn pending trusted identity/session attestation.

## Verification

Paper tests prove one coordinator lifecycle, submit-return scheduler behavior,
and clean disable in addition to local resources. Containment inspects source,
metadata, and built jars for duplicate pollers and withdrawn mutation,
application, command, or transfer classes.
