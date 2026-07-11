# Daemon HTTP auth operations

## Purpose

This runbook owns the managed daemon HTTP bearer token lifecycle and incident
response for Minecraft plugins.


## Status

implemented

## Token lifecycle

- Managed installs prefer `LKJMC_DAEMON_HTTP_TOKEN_FILE` over direct token
  environment variables so secrets do not appear in command lines.
- The daemon reads `--http-token-file` at startup and trims transport whitespace
  such as a trailing newline from the file content.
- Java child processes must receive the same token-file path through their
  rendered environment and must be able to read that file.
- Header names and the `Bearer` scheme are case-insensitive transport syntax;
  bearer credential bytes are case-sensitive and must be compared exactly.
- If no token is configured, the daemon HTTP endpoint denies requests by
  default. Do not run managed plugins against an unprotected endpoint.
- The single configured token is a CLI-shaped operator credential. JSON actors,
  principals, and `platformPermission` remain request data, not proof. Plugin,
  proxy, and Discord traffic must use a database-backed scoped credential with a
  matching adapter actor; requests with a forged surface or subject are denied.

## Auth incident symptoms

Treat these as one incident class until proven otherwise:

- Inventory menu rows show `daemon.auth_failed` or token rejection.
- `/lkjmc doctor` reports daemon HTTP auth failure from a plugin.
- Multiple dynamic menus fail while static menu chrome still renders.
- Reinstalling regenerates the token but the same auth failure returns.

## First response

1. Do not print token contents in chat, logs, shell history, or handoff notes.
2. Check that daemon, proxy, and backend processes were started by the same
   managed runtime and reference the same token-file path.
3. Check that the token file exists and is readable by the child Java processes.
4. Check daemon logs for secret-safe auth failures and HTTP reason phrases.
5. Restart processes after any manual token-file change because clients can cache
   token contents.
6. If auth still fails, run the daemon HTTP auth tests before blaming menu code.

## Rotation contract

`lkjmc security token plan|rotate|status|verify` and matching daemon commands
rotate the HTTP bearer token without printing secret bytes. Rotation atomically
writes the configured file, hot-swaps the verifier, then makes real loopback
HTTP probes for new-token acceptance and old-token rejection. It restores the
prior verifier and file on a probe failure and audits fingerprints only. It does
not claim consumer restart or reload; token-file consumers reread on their next
request.

## Current rotation status

Automated rotation exists only with an active loopback listener and configured
old token, because both transport probes are required. Java clients reread a
token file before requests; direct token environment consumers still need a
restart. Scoped credential creation accepts only known surface scopes, a bounded
expiry, and an absolute owner-limited output file. Storage keeps its hash and
returns path, expiry, and fingerprint, never the credential bytes.

## Verification

A token incident is not closed until a playable smoke proves:

- `/lkjmc doctor` reports daemon HTTP auth healthy or gives a real diagnostic.
- `/lkjmc status` and `/lkjmc server list` reach the daemon or report a real
  dependency failure.
- `/menu` can load the server list and at least one daemon-backed player menu.
