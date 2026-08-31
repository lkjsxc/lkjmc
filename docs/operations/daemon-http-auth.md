# Daemon HTTP auth operations

## Purpose

This runbook owns the managed daemon HTTP bearer token lifecycle and incident
response. Daemon-capable Java adapters are withdrawn pending trusted
identity/session attestation.


## Status

implemented

## Token lifecycle

- Managed installs prefer `LKJMC_DAEMON_HTTP_TOKEN_FILE` over direct token
  environment variables so secrets do not appear in command lines.
- The daemon reads `--http-token-file` at startup and trims transport whitespace
  such as a trailing newline from the file content.
- Bootstrap and temporary-instance rendering do not render the TCP root token-file
  path into Java child environments. No Java daemon adapter is shipped until
  trusted identity/session attestation exists.
- Header names and the `Bearer` scheme are case-insensitive transport syntax;
  bearer credential bytes are case-sensitive and must be compared exactly.
- If no token is configured, the daemon HTTP endpoint denies requests by
  default. Do not run managed plugins against an unprotected endpoint.
- The single configured token is a CLI-shaped operator credential. JSON actors,
  principals, and `platformPermission` remain request data, not proof. Plugin
  and proxy traffic must use a database-backed scoped credential with a matching
  adapter actor; requests with a forged surface or subject are denied.

## Auth incident symptoms

Treat these as one incident class until proven otherwise:

- A daemon CLI or loopback probe reports token rejection.
- Reinstalling regenerates the token but the same auth failure returns.
- An operator attempted to use a withdrawn Java daemon adapter.

## First response

1. Do not print token contents in chat, logs, shell history, or handoff notes.
2. Check that the daemon has its configured root token file and that its mode is
   owner-limited.
3. Check daemon logs for secret-safe auth failures and HTTP reason phrases.
4. Restart affected consumers after any manual token-file change.
5. If auth still fails, run the daemon HTTP auth tests before blaming a
   withdrawn adapter.

## Rotation contract

`lkjmc security token plan|rotate|status|verify` and matching daemon commands
rotate the HTTP bearer token without printing secret bytes. Rotation stages old
and new verifiers, atomically writes the configured file, and makes a real
loopback HTTP probe proving new-token acceptance before old access is retired.
It restores the prior verifier and file together on a failed write or probe.
If restoring the old file fails, it clears both root verifiers so neither old nor
staged new token is accepted, then audits fingerprints only. It does not claim a
consumer reload. A Java token-file read is a construction snapshot, never a
per-request reread: any future Java consumer must restart or reconstruct after
rotation.

## Current rotation status

Automated rotation exists only with an active loopback listener and configured
old token, because both transport probes are required. Java daemon adapters are
withdrawn pending trusted identity/session attestation; no Java client rereads a
token file. A future consumer must restart or reconstruct from its construction
snapshot. Scoped credential creation permits only `lkjmc.admin.status`,
`lkjmc.admin.reload`, `lkjmc.admin.instance.list`,
`lkjmc.admin.instance.create`, `lkjmc.admin.instance.start`,
`lkjmc.admin.instance.stop`, `lkjmc.admin.instance.restart`,
`lkjmc.admin.instance.delete`, `lkjmc.admin.economy`, and
`lkjmc.admin.admin`; every requested scope must be on this list. It also
requires a bounded expiry and absolute owner-limited output file. Storage keeps
its hash and returns credential id, requested expiry, and fingerprint, never a
credential byte, path, principal, or scope. The owner provides the file path in
the request and retains it to recover an explicitly reported file orphan or
uncertain commit.

## Verification

A token incident is not closed until a daemon loopback probe proves new-token
acceptance and old-token retirement without printing token bytes. The local-safe
Paper menu/docs UI and Velocity MOTD/tab-list do not consume daemon credentials.
