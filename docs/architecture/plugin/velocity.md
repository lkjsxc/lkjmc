# Velocity plugin

## Purpose

This document defines the target proxy behavior.

## Responsibilities

- Initialize after Velocity initialization.
- Check daemon and database connectivity.
- Observe desired server registry.
- Register dynamic servers.
- Provide `/hub` and functional `/lkjmc` admin commands.
- Render MOTD and tab list.
- Coordinate profile-safe transfers.
- Route to fallback servers when targets are unavailable.

## Current status

The Velocity module builds a real Velocity plugin jar with an annotated
composition root. On proxy initialization it registers `/lkjmc status`, `/lkjmc
server list`, server lifecycle commands, `/lkjmc send <player> <server>`, and
`/hub`, plus MOTD and tab-list listeners. `/lkjmc status` reports proxy player count. `/lkjmc server list`
lists registered Velocity servers. `/lkjmc server start|stop|restart|create`
and `/lkjmc server delete <id> confirm` call the daemon HTTP API when
`LKJMC_DAEMON_HTTP_URL` and `LKJMC_DAEMON_HTTP_TOKEN` are configured. Startup
also calls daemon `instance.list` and registers returned localhost server ports.
`/hub` connects players to a registered `hub` server or returns a failure
message, and `/lkjmc send` moves an online player to a registered target server
only after a source Paper snapshot acknowledgement.
The MOTD listener renders a fixed `lkjmc network` description, and
post-login tab header/footer shows the current proxy player count. `/lkjmc reload` refreshes daemon-backed server
registration when daemon HTTP is configured. `/lkjmc restart warn <seconds>`
broadcasts a warning and schedules a follow-up warning without pretending to
restart the proxy. Transfer sync coordination uses the `lkjmc:profile` plugin
message channel and denies transfers if the source server does not save in time.
