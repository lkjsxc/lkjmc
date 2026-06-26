# Daemon HTTP

## Purpose

This document defines how JVM plugins call the daemon HTTP endpoint.

## Current contract

- Plugins read `LKJMC_DAEMON_HTTP_URL` plus either `LKJMC_DAEMON_HTTP_TOKEN` or
  `LKJMC_DAEMON_HTTP_TOKEN_FILE`.
- If the URL and a usable token source are absent or blank, daemon-backed
  features fail clearly or use documented local-only behavior.
- Network calls use `CompletableFuture` and never block scheduler threads.
- Scheduler callbacks that touch Minecraft APIs return through the appropriate
  Paper/Folia or Velocity scheduler bridge.
- The bearer token is sent to the loopback daemon and is never logged.

## Playable target

Managed instances should receive `LKJMC_INSTANCE_ID`,
`LKJMC_DAEMON_HTTP_URL=http://127.0.0.1:8765`, and
`LKJMC_DAEMON_HTTP_TOKEN_FILE=/etc/lkjmc/daemon-http.token`. Token-file mode is
implemented in Java common and is preferred for managed runtime.

## Source owners

- Java common HTTP client: `platforms/jvm/common/.../HttpDaemonClient.java`.
- Paper lifecycle wiring: `platforms/jvm/paper/.../LkjmcPaperPlugin.java`.
- Velocity lifecycle wiring: `platforms/jvm/velocity/.../VelocityLifecycle.java`.
- Daemon HTTP server: `crates/lkjmc-daemon/src/http_api.rs`.

## Current implementation

The daemon exposes a loopback HTTP command endpoint protected by a bearer token.
Plugin startup supports direct token and token-file environment variables.
