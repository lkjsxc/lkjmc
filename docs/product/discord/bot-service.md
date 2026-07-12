# Bot service

## Purpose

This document owns the initial `lkjmc-discord` service behavior.


## Status

partial

Missing: server-verified role evidence, durable replay detection, bounded rate
limits, and server-side confirmation for every Discord action.

## Slash commands

No `/lkjmc` command is registered. Setting `registerCommands=true` sends an
empty guild command list to withdraw prior registrations; it does not register a
replacement action. Status, server listing, wake, reports, linking, moderation,
announcements, grants, revokes, audit, and token rotation are all withdrawn.

## Target additions

After daemon reward claiming and account-link storage exist, Discord should add
linked-player achievement summaries and reward-claim commands. These target
commands must not register until the daemon command and link checks are real.

## Runtime rules

The service loads JSON config, reads its Discord token from a file or
environment variable name, redacts secret diagnostics, and can replace guild
command metadata with an empty list. `interactionBind` is refused before token
read, listener bind, or REST work. No interaction HTTP request, component id,
mapped role, or request-body value reaches an authorization boundary.

## Account linking

Minecraft can issue and revoke durable link records, but Discord cannot complete,
remove, or use a link while its action surface is withdrawn. Challenges, tokens,
and generated secrets are never logged.

## Current implementation path

The service can register an empty command list through Discord's REST API. It
starts no interaction endpoint and does not implement role enforcement, replay
storage, rate limiting, confirmation, or daemon delegation. The guarded external
lane needs a test bot token, application id, and guild to prove registration
withdrawal; without those prerequisites it is skipped or blocked, never passed.
