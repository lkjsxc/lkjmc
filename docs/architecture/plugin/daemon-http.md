# Daemon HTTP

## Purpose

This document defines how Java plugins call the daemon.

## Contract

- Plugins read `LKJMC_DAEMON_HTTP_URL` and `LKJMC_DAEMON_HTTP_TOKEN`.
- If either value is absent or blank, daemon-backed features fail clearly or use
  documented local-only behavior.
- Network calls use `CompletableFuture` and never block scheduler threads.
- Scheduler callbacks that touch Minecraft APIs return through the appropriate
  Paper/Folia or Velocity scheduler bridge.
- The bearer token is sent to the loopback daemon and is never logged.

## Source owners

- Java common client: `platforms/jvm/common/src/main/java/com/lkjmc/common/daemon/HttpDaemonClient.java`.
- Paper composition root: `platforms/jvm/paper/src/main/java/com/lkjmc/paper/LkjmcPaperPlugin.java`.
- Velocity composition root: `platforms/jvm/velocity/src/main/java/com/lkjmc/velocity/VelocityLifecycle.java`.
- Daemon HTTP server: `crates/lkjmc-daemon/src/http_api.rs`.

## Current boundary

The Java client currently decodes response bodies into a raw string map. The
transport hardening task will replace that with typed JSON helpers.
