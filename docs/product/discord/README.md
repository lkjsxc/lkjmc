# Discord

## Purpose

This area owns the community and operator Discord adapter product contract.


## Status

implemented

## Table of contents

- [Bot service](bot-service.md)
- [Linking](linking.md)
- [Security](security.md)

## Contract

Discord is an adapter around the daemon, not a source of product truth. Missing
credentials disable startup with an explicit diagnostic. A command is advertised
as live only when the bot can authenticate to Discord, call the daemon with a
configured token, authorize the principal, and return real data or a real audited
mutation result.

## Outcome, journey, and evidence boundary

A linked or authorized Discord user invokes a supported slash command and
receives deferred, ephemeral real data or a daemon-authorized mutation result.
Missing credentials, a missing link, invalid signatures, or daemon failure yield
an explicit failure rather than a ready bot or successful action. Local tests
cover request handling and delegation; Discord availability requires the guarded
live smoke with real credentials and endpoint access.
