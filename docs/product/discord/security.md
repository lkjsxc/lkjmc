# Discord security

## Purpose

This document owns Discord principal, role, token, and audit rules.


## Status

partial

Missing: a trusted role source, durable replay storage, rate-limit state, and
server-side confirmation records.

## Configuration

User-edited configuration is JSON and contains Discord token source paths or
environment keys, guild allowlist, command-withdrawal registration intent,
Discord application id, interaction public key, and interaction bind address.

## Principals

Discord request-body user ids, role ids, mapped roles, and principal fields are
untrusted input. The service does not create a Discord actor or delegate to the
daemon while commands are withdrawn. Durable link records remain daemon-owned,
but no Discord interaction can mutate or use them.

## Safety rules

- Verify Discord Ed25519 interaction signatures before answering a ping.
- Never log bot tokens, generated link codes, link-code hashes paired with player
  identity, or bearer headers.
- Non-ping interactions deny before role mapping, replay handling, rate limits,
  confirmation, or daemon dispatch.
- Every former action, including grants, revokes, bans, mutes, announcements,
  link changes, wake, token rotation, audit, status, and reports, is withdrawn.
- Link codes expire after ten minutes and are stored as hashes only.
- Missing credentials produce a clean startup error, not a fake ready bot.
