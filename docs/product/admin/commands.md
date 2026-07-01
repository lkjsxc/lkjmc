# Admin commands

## Purpose

This document owns privileged command families shared by CLI, daemon adapters,
and `/lkjmc` where Minecraft can safely expose them.

## Command families

- Health: `status` and `doctor`.
- Server lifecycle: `instance.list`, `instance.create`, `instance.start`,
  `instance.stop`, `instance.restart`, and `instance.delete`.
- Config: `config.reload` plus adapter-local restart warnings.
- Security: `security.daemon-token.status`, `security.daemon-token.rotate`, and
  role catalog reads.
- Grants: `admin.role.list`, `admin.principal.inspect`,
  `admin.grant.create`, `admin.grant.revoke`, and `admin.audit.tail`.
- Economy: `economy.catalog.seed-defaults` and `shop.item.upsert`.
- Announcements and moderation: announcement, report, warning, note, ban, mute,
  and claim admin commands.

## Server lifecycle contract

A server create request from any product surface must either write a startable
instance or fail before success with diagnostics for missing jar assets, EULA
acceptance, port conflict, duplicate id, template mismatch, or launch metadata.
Start validates launch readiness, treats already-running as current state, and
must not leave desired state running after a failed launch.

## Actor requirements

Each request supplies actor kind, actor id or name, platform permission evidence,
principal kind, principal id, principal display name when known, reason text when
needed, and safe metadata. The daemon combines platform evidence with durable
grants and audits privileged attempts.

## Audit fields

Audit rows record actor, subject, action, target kind, target id, result, reason,
correlation id when available, and redacted metadata. Tokens, bearer headers,
forwarding secrets, and generated secrets must never appear in audit metadata.
