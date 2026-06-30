# Bot service

## Purpose

This document owns the initial `lkjmc-discord` service behavior.

## Slash commands

- `/lkjmc status`: safe daemon status summary.
- `/lkjmc servers`: managed server list.
- `/lkjmc wake server:<id>`: wake-and-join request when permitted.
- `/lkjmc announce message:<text>`: permitted announcement mutation.
- `/lkjmc reports`: open reports for moderators.
- `/lkjmc link`: begin Discord-to-Minecraft account linking.
- `/lkjmc admin inspect user:<target>`: inspect effective admin grants.
- `/lkjmc admin grant` and `/lkjmc admin revoke`: reason-bearing confirmed
  mutations after daemon authorization.
- `/lkjmc audit tail`: privileged audit view.

## Runtime rules

The service loads JSON config, reads Discord and daemon tokens from files or
environment variable names, redacts secret diagnostics, registers command
metadata when configured to do so, acknowledges interactions quickly, and sends
follow-up responses after daemon work completes.

## Current implementation path

The first service slice validates config, verifies token sources without logging
them, builds daemon requests with Discord principal metadata, and provides safe
startup diagnostics. Live slash-command smokes require a test bot token, guild,
and daemon HTTP endpoint.
