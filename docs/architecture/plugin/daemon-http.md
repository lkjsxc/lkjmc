# Daemon HTTP

## Purpose

This document defines how JVM plugins call the daemon HTTP endpoint.

## Current contract

- Plugins read `LKJMC_DAEMON_HTTP_URL` and `LKJMC_DAEMON_HTTP_TOKEN`.
- If either value is absent or blank, daemon-backed features fail clearly or use
  documented local-only behavior.
- Network calls use `CompletableFuture` and never block scheduler threads.
- Scheduler callbacks that touch Minecraft APIs return through the appropriate
  Paper/Folia or Velocity scheduler bridge.
- The bearer token is sent to the loopback daemon and is never logged.

## Playable target

Managed instances should receive `LKJMC_INSTANCE_ID`,
`LKJMC_DAEMON_HTTP_URL=http://127.0.0.1:8765`, and
`LKJMC_DAEMON_HTTP_TOKEN_FILE=/etc/lkjmc/daemon-http.token`. Java common config
should read either a direct token environment variable or a token-file
environment variable, with token-file preferred for managed runtime.

## Source owners

- Java common HTTP client: `platforms/jvm/common/.../HttpDaemonClient.java`.
- Paper lifecycle wiring: `platforms/jvm/paper/.../LkjmcPaperPlugin.java`.
- Velocity lifecycle wiring: `platforms/jvm/velocity/.../VelocityLifecycle.java`.
- Daemon HTTP server: `crates/lkjmc-daemon/src/http_api.rs`.

## Current implementation

The daemon exposes a loopback HTTP command endpoint protected by a bearer token.
Current plugin startup requires direct URL and token environment variables.
