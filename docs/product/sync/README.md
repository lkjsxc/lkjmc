# Player sync

## Purpose

This area owns plugin-enabled player profile synchronization.


## Status

implemented

## Table of contents

- [Player profile](player-profile.md)
- [Transfer safety](transfer-safety.md)

## Contract

Full profile sync applies only to plugin-enabled Paper/Folia servers. Process-only
servers are managed but do not claim profile sync.

## Outcome, journey, and evidence boundary

For plugin-enabled transfers, the source saves a leased snapshot before the
proxy connects the player and the target applies the acknowledged revision.
Missing acknowledgement denies the transfer; uncertain saves create recovery
records instead of silent overwrite. Store and adapter tests support this
protocol, but do not prove recovery of a real crash or sync on process-only
servers.
