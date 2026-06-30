# Daemon HTTP auth operations

## Purpose

This runbook owns the managed daemon HTTP bearer token lifecycle and incident
response for Minecraft plugins.

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

`lkjmc security token plan|rotate|status|verify` and the matching daemon
commands rotate the HTTP bearer token without printing secret bytes. Apply writes
the configured token file atomically with restrictive permissions, hot-swaps the
daemon verifier, restarts or reloads managed JVM consumers, verifies the new
token, verifies old-token rejection, and writes safe audit rows.

## Current rotation status

Automated rotation exists for token-file managed installs. Java clients reread
the token file before daemon requests, so managed consumers reload the rotated
file without printing or embedding new token bytes. Direct token environment
consumers still require a process restart.

## Verification

A token incident is not closed until a playable smoke proves:

- `/lkjmc doctor` reports daemon HTTP auth healthy or gives a real diagnostic.
- `/lkjmc status` and `/lkjmc server list` reach the daemon or report a real
  dependency failure.
- `/menu` can load the server list and at least one daemon-backed player menu.
