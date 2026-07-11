# Daemon HTTP

## Purpose

This document defines how JVM plugins call the daemon HTTP endpoint.


## Status

implemented

## Current contract

- Plugins read `LKJMC_DAEMON_HTTP_URL` plus either `LKJMC_DAEMON_HTTP_TOKEN` or
  `LKJMC_DAEMON_HTTP_TOKEN_FILE`.
- If the URL and a usable token source are absent or blank, daemon-backed
  features fail clearly with typed diagnostics or use documented local-only
  behavior.
- Network calls use `CompletableFuture` and never block scheduler threads.
- Scheduler callbacks that touch Minecraft APIs return through the appropriate
  Paper/Folia or Velocity scheduler bridge.
- The bearer token is sent to the loopback daemon and is never logged.
- The HTTP header name and `Bearer` scheme are matched case-insensitively, but
  the credential bytes are not case-folded or normalized.
- Token-file contents may have transport whitespace trimmed at read time; token
  characters inside the credential remain exact.

## Playable target

Managed instances should receive `LKJMC_INSTANCE_ID`,
`LKJMC_DAEMON_HTTP_URL=http://127.0.0.1:8765`, and
`LKJMC_DAEMON_HTTP_TOKEN_FILE=/etc/lkjmc/daemon-http.token`. Token-file mode is
implemented in Java common and is preferred for managed runtime. Playable smoke
must prove a JVM plugin reads that file, sends a bearer token whose credential
bytes are preserved, and receives a successful daemon response.

## Source owners

- Java common HTTP client: `platforms/jvm/common/.../HttpDaemonClient.java`.
- Paper lifecycle wiring: `platforms/jvm/paper/.../LkjmcPaperPlugin.java`.
- Velocity lifecycle wiring: `platforms/jvm/velocity/.../VelocityLifecycle.java`.
- Daemon HTTP server: `crates/lkjmc-daemon/src/transport/`.

## Test-only fault boundary

JVM acknowledgement, HTTP-deadline, and credential-lookup controls exist only in
JVM test sources. They model completion around an asynchronous daemon call
without a scheduler or plugin registration. Production keeps `HttpDaemonClient`
and its real token source; no command, environment variable, or configuration
can enable a fault control.

## Field status

The playable command/menu smoke covers managed token-file auth with a mixed-case
daemon token through JVM plugins. A future auth incident must be triaged with the
runbook rather than reopening this surface without fresh evidence.

## Current implementation

The daemon exposes a loopback HTTP command endpoint protected by a bearer token.
Plugin startup supports direct token and token-file environment variables. Menu
loaders and `/lkjmc doctor` classify missing URL, missing token, unreadable token
file, HTTP failure, auth failure, daemon command failure, database failure, and
schema mismatch without printing secrets.
