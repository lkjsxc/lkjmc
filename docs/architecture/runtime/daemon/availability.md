# Daemon availability

## Purpose

This document defines daemon availability for daemon-owned, CLI, and web
surfaces, and the Java containment boundary.

## Status

implemented

## Current boundary

The daemon classifies missing configuration, invalid credentials, authentication
failure, HTTP failure, command failure, database failure, and schema mismatch
for consumers that are authorized to use daemon HTTP.

Paper/Folia and Velocity are not daemon consumers. Their daemon access component,
token-file reader, stale dynamic data cache, grant snapshot, and command mapping
are withdrawn pending trusted identity/session attestation. The local-safe
Paper menu/docs UI and Velocity MOTD/tab-list have no daemon-unavailable state.

## Future rule

A future Java consumer needs trusted authenticated player identity and session
attestation before it can construct an authorized client. Its token-file value
would be a construction snapshot, so token rotation requires restart or
reconstruction rather than a request-time reread.

## Verification

Daemon transport tests cover daemon classifications. Java source and artifact
inspection prove the withdrawn consumers are absent; local UI tests cover only
local content failures.
