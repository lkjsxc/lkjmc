# Bot service

## Purpose

This document owns the initial `lkjmc-discord` service behavior.


## Status

implemented

## Slash commands

- `/lkjmc status`: safe daemon status summary.
- `/lkjmc servers`: managed server list with state and player counts.
- `/lkjmc wake server:<id>`: wake-and-join request for a linked Discord user.
- `/lkjmc reports`: open reports for a caller with a mapped moderator role.
- `/lkjmc link code:<code>`: complete Discord-to-Minecraft account linking.
- `/lkjmc unlink`: revoke the caller's Discord-to-Minecraft account link.

`announce`, `admin inspect`, grant, revoke, and audit commands are withdrawn.
They do not register until signed, server-side confirmation, replay prevention,
and bounded per-user and guild rate evidence are available for those mutations.

## Target additions

After daemon reward claiming and account-link storage exist, Discord should add
linked-player achievement summaries and reward-claim commands. These target
commands must not register until the daemon command and link checks are real.

## Runtime rules

The service loads JSON config, such as `config/discord.json.example`, reads
Discord and daemon tokens from files or environment variable names, redacts
secret diagnostics, registers command
metadata when configured to do so, verifies signed interaction HTTP requests,
acknowledges daemon-backed interactions with a deferred ephemeral response, and
sends follow-up responses after daemon work completes. Destructive operator
actions use ephemeral confirmations backed by signed or server-side state; client
component ids alone are never trusted.

## Account linking

Durable link state is stored as Discord user id, Minecraft UUID, verification
state, created time, verified time, revoked time, and metadata. Minecraft issues
a short-lived one-time code through `player.link.begin`; Discord completes it
through `discord.link.complete`. Link-required commands report the missing link
instead of faking success. Challenges, tokens, and generated secrets are never
logged.

## Current implementation path

The service can register the bounded `/lkjmc` slash-command tree through
Discord's REST API, serve a signed interaction HTTP endpoint, reject replayed or
rate-exceeded interactions, map a signed Discord user and mapped role into daemon
principal evidence, and delegate supported commands through a Discord-scoped
credential. Future commands must register only after backing daemon operations
exist. Live slash-command smokes require a test bot token, application id,
public key, guild, endpoint, and scoped daemon token.
