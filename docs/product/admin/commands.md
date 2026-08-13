# Admin commands

## Purpose

This document owns privileged command families exposed through CLI, authenticated
web, and daemon command transport. Java `/lkjmc` exposure is withdrawn.


## Status

implemented

## Command families

`status` is an admitted local observation and `admin.role.list` is an admitted
static role observation. `config.reload` returns non-success
`config.restart_required`. Every server lifecycle, security, grant, economy,
moderation, audit, and other catalog command is
`denied-unproved`: it returns non-success `command.effect_denied` before a
handler, database write, process, network, filesystem, plugin, proxy, transfer,
or observer effect. The catalog remains registered for closed-schema checking;
it is not a support claim.

## Lifecycle boundary

No admin surface can start, stop, restart, delete, download, or reconcile a
server. Missing post-launch external completion evidence blocks those operations
rather than allowing a durable intent row, a queue, or a synthetic result to
stand in for completion. The complete boundary is the
[command lifecycle](../../architecture/runtime/daemon/command-lifecycle.md).

## Actor requirements

Each request supplies actor kind, actor id or name, platform permission evidence,
principal kind, principal id, principal display name when known, reason text when
needed, and safe metadata. The daemon combines platform evidence with durable
grants and audits privileged attempts.

## Audit fields

Audit rows record actor, subject, action, target kind, target id, result, reason,
correlation id when available, and redacted metadata. Tokens, bearer headers,
forwarding secrets, and generated secrets must never appear in audit metadata.
