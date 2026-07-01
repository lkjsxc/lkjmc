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
metadata when configured to do so, verifies signed interaction HTTP requests,
acknowledges daemon-backed interactions with a deferred ephemeral response, and
sends follow-up responses after daemon work completes.

## Current implementation path

The service can register the `/lkjmc` slash-command tree through Discord's REST
API, serve a signed interaction HTTP endpoint, map Discord users and roles into
daemon principal evidence, delegate supported commands to daemon HTTP, and keep
link-required commands explicit instead of faking success. Live slash-command
smokes require a test bot token, application id, public key, guild, endpoint,
and daemon HTTP token.
