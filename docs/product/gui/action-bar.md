# Action bar

## Purpose

This document records withdrawal of Java Action Bar status frames.

## Status

implemented

## Current boundary

Action Bar render loops, daemon snapshots, `/hud`, passive status, and priority
result frames are withdrawn pending trusted identity/session attestation. The
local-safe Paper docs UI does not fetch status, cache daemon data, or send an
Action Bar message.

## Verification

Daemon/store status data remains separate evidence. Java containment inspection
proves no Action Bar daemon adapter is packaged.
