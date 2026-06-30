# Discord security

## Purpose

This document owns Discord principal, role, token, and audit rules.

## Configuration

User-edited configuration is JSON and contains token source paths or environment
keys, daemon endpoint, guild allowlist, channel allowlist, role-to-admin mappings,
command enablement flags, public status rendering options, and an audit actor
name.

## Principals

- Discord user actor: `discord-user:<snowflake>`.
- Discord role actor: `discord-role:<snowflake>`.
- Linked Minecraft principal: `minecraft-player:<uuid>`.

Discord role mappings are visibility and authorization evidence. The daemon is
still final truth for privileged actions.

## Safety rules

- Never log bot tokens, daemon bearer tokens, generated link codes, or bearer
  headers.
- Rate-limit dangerous commands per actor and guild.
- Require reason and confirmation for grants, revokes, bans, mutes, and token
  rotation.
- Audit privileged success and denial with redacted metadata.
- Missing credentials produce a clean startup error, not a fake ready bot.
