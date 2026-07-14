# Action bar

## Purpose

This document records withdrawal of Java Action Bar status frames.

## Status

implemented

## Current boundary

Java common may cache the revisioned `settings/<player UUID>` view containing
settings plus action-bar source data. Action Bar render loops, `/hud`, player
application, and result frames remain withdrawn pending trusted identity/session
attestation. The local-safe Paper UI does not send an Action Bar message.

## Verification

PostgreSQL/HTTP/JVM tests prove revisioned transport and freshness only. Java
containment proves no Action Bar renderer, command, or player-application adapter
is packaged.
