# Inventory sync

## Purpose

This document owns local documentation-token inventory repair.

## Status

implemented

## Repair triggers

The Paper runtime repairs the local token after join, respawn, pickup, and
cancelled token movement. It removes duplicate tokens outside hotbar slot `8`
and restores the local `NETHER_STAR` token to that slot.

## Safety rules

Repair uses only current Bukkit inventory state. It reads no daemon setting,
token file, database, filesystem, network, or process. Token copy and metadata
are local plugin values.

## Failure behavior

The token always opens bundled documentation. It does not report daemon failure,
change a player preference, or invoke a hidden mutation.
