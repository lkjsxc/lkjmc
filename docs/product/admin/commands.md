# Admin commands

## Purpose

This document owns privileged command families exposed through CLI, authenticated
web, and daemon command transport. Java `/lkjmc` exposure is withdrawn.


## Status

implemented

## Command families

`status` is an admitted PostgreSQL observation and `admin.role.list` is an
admitted static role observation. Bootstrap plan, status, and doctor observe the
configured network; bootstrap apply is the sole admitted process/filesystem
operation and requires a kernel-authenticated local Unix peer plus explicit EULA
acceptance. `config.reload` returns non-success `config.restart_required`.
Every other server lifecycle, security, grant, economy, moderation, audit, and
catalog command is `denied-unproved`: it returns non-success
`command.effect_denied` before a handler or effect. Registration alone is not a
support claim.

## Lifecycle boundary

Only local bootstrap apply can reconcile, start, stop, and readiness-check the
closed configured network. It records durable intent and attempts, verifies
immutable assets, fences process identity, and reports success only after ready
logs and listeners are observed. Individual generic lifecycle and download
commands remain denied. The complete boundary is the
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
