# Discord

## Purpose

This area owns the community and operator Discord adapter product contract.

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
