# HUD setting

## Purpose

This document records withdrawal of the Java HUD and Action Bar setting.

## Status

implemented

## Current boundary

`/hud`, durable HUD settings, Action Bar snapshots, and Java status rendering are
withdrawn pending trusted identity/session attestation. The local documentation
plugin has no HUD preference and does not call `player.settings.*`.

## Verification

Java containment inspection proves no HUD command, setting reader, or daemon
status adapter is packaged.
