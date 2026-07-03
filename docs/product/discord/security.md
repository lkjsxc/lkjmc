# Discord security

## Purpose

This document owns Discord principal, role, token, and audit rules.


## Status

implemented

## Configuration

User-edited configuration is JSON and contains token source paths or environment
keys, daemon endpoint, guild allowlist, channel allowlist, role-to-admin
mappings, command registration flags, Discord application id, interaction public
key, interaction bind address, public status rendering options, and an audit
actor name.

## Principals

- Discord user actor: `discord-user:<snowflake>`.
- Discord role actor: `discord-role:<snowflake>`.
- Linked Minecraft principal: `minecraft-player:<uuid>`.

Discord role mappings are visibility and authorization evidence. Durable account
links map a Discord user id to one Minecraft UUID only after verification. The
daemon is still final truth for privileged actions.

## Safety rules

- Verify Discord Ed25519 interaction signatures before command handling.
- Never log bot tokens, daemon bearer tokens, generated link codes, link-code
  hashes paired with player identity, or bearer headers.
- Rate-limit dangerous commands per actor and guild.
- Require reason and confirmation for grants, revokes, bans, mutes, and token
  rotation.
- Audit privileged success and denial with redacted metadata.
- Link codes expire after ten minutes, are stored as hashes only, and are
  consumed once.
- Missing credentials produce a clean startup error, not a fake ready bot.
