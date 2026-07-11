# Identity view

## Purpose

This view traces authenticated subjects, player identity, and authorization.

## Status

implemented

## Identity flow

A transport authenticates a subject before daemon authorization. The command
envelope actor is context, not proof. A root daemon token produces a root
subject; scoped subjects carry verified permissions. Otherwise authorization
looks up durable grants using `principalKind` and `principalId`.

Minecraft join records the UUID/name identity and replaces the active session
for its server. The UUID, not a display name, is the durable player key.
Discord and browser adapters still delegate authorization to daemon commands.
Denied authorization appends a redacted audit event when the store is available.

## Exact non-atomic boundaries

- Authentication in the HTTP transport and durable grant lookup are separate;
  revocation between them can change the authorization result.
- A future attested platform join and a daemon `player.session.join` write would
  be separate systems; Java adapters are currently withdrawn and leave no
  fabricated session.
- An audit insert follows a denied lookup and is best-effort; denial does not
  become authorization if the audit write fails.

## Source trace

- `crates/lkjmc-daemon/src/authz.rs`
- `crates/lkjmc-daemon/src/transport/auth.rs`
- `crates/lkjmc-daemon/src/commands/player_session.rs`
- `crates/lkjmc-store/src/admin.rs`
- `crates/lkjmc-store/src/player.rs`
