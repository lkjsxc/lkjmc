# Player sync

## Purpose

This area owns durable profile data and the withdrawn Java synchronization
boundary.

## Status

implemented

## Table of contents

- [Player profile](player-profile.md)
- [Revisioned transport](revisioned-transport.md)
- [Transfer safety](transfer-safety.md)

## Contract

The daemon and store own durable snapshots, revisions, and a bounded read-only
change feed. Java common may cache revisioned domain views through the shared
coordinator; platform modules own lifecycle and presentation adapters only.

Paper/Folia profile application and Velocity transfer handling remain withdrawn
pending trusted identity/session attestation. Process-only servers and read-only
transport do not claim player sync.

## Evidence boundary

PostgreSQL, daemon HTTP, and Java 21 harnesses prove revisioned transport,
repair, bounds, and shutdown. They do not prove a Java save, load, application,
session, transfer, or arrival path.
