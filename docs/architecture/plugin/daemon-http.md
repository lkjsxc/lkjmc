# Daemon HTTP

## Purpose

This document defines the containment boundary for daemon HTTP credentials and
Java plugins.

## Status

implemented

## Current boundary

The daemon HTTP endpoint remains a loopback, bearer-protected operator and
service boundary. CLI, web, and daemon-owned components use their documented
credentials. Paper/Folia and Velocity daemon adapters are withdrawn pending
trusted identity/session attestation and no shipped plugin calls this endpoint.

A Java token-file read, if a future constructor performs one, is a construction
snapshot. It is not reread per request or on rotation. Rotation therefore
requires an explicit consumer restart or reconstruction; it never silently
refreshes a Java credential.

## Local-safe plugins

Paper retains local `/menu`, `/docs`, hotbar token, and bundled docs UI. Velocity
retains MOTD and tab-list presentation. Neither receives a daemon HTTP URL or
token, constructs a daemon client, or sends a daemon command.

## Reintroduction rule

No adapter may be re-enabled from configuration, a token file, a fake command,
or a cached grant. A future proposal needs trusted authenticated player identity
and session attestation, scoped authorization, nonblocking design, source
registration tests, and built-jar inspection before this boundary changes.

## Verification

Containment verification rejects daemon-client sources and daemon credentials in
plugin artifacts, as well as withdrawn Paper/Velocity command, registry, and
bridge classes. Daemon HTTP transport tests remain daemon evidence, not Java
plugin evidence.
